#![feature(thread_is_running)]

use std::fs::File;
use std::io::Read;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{io, thread};
use std::time::Duration;
use better_term::{Color, Style};
use log::{error, info, Level, LevelFilter};
use crate::client::handle_connection;
use crate::config::read_config;
use crate::database::Database;

pub mod client;
pub mod database;
pub mod item;
pub mod player;
mod config;

/***
 * Todo(eric):
 *  - Add merchants
 *  - Handle exploit where users can disconnect while in encounter and end encounter
 *  - Password recovery?
***/

// versions
pub const ACCEPTED_CLIENT_VERSION: &str = "0.1.0";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

// info for the client
pub const MOTD: &str = "Welcome to SnD! We are still in ALPHA, so expect some bugs!";
pub const KEEPALIVE_INTERVAL: u64 = 20; // time in seconds to send the keepalive packet

// How long the main loop should wait between checking for incoming connections to save cpu resources
const MAIN_LOOP_WAIT_DELAY_MS: u64 = 20;
const LOG_LEVEL_FILTER_AT: LevelFilter = LevelFilter::Trace;
const LOG_TARGET: &str = "main";

pub fn read_config_raw(file: &mut File) -> String {
    let mut config_content = String::new();
    file.read_to_string(&mut config_content)
        .expect("Failed to read config file. Please make sure that the server has permission to edit files.");
    config_content
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let style = match record.level() {
                Level::Error => Style::reset().fg(Color::Red).bold(),
                Level::Warn => Style::reset().fg(Color::Yellow),
                Level::Info => Style::reset().fg(Color::Cyan),
                Level::Debug => Style::reset().fg(Color::White),
                Level::Trace => Style::reset().fg(Color::BrightBlack),
            };

            let time = chrono::Local::now();

            out.finish(format_args!(
                "{bc}[{ic}{}{bc}][{ic}{}{bc}][{ic}{}{bc}] {}{}{bc}: {ic}{}",
                time.format("%Y-%m-%d"),
                time.format("%H:%M:%S"),
                record.target(),
                style,
                record.level(),
                message,
                bc = Style::reset().fg(Color::BrightBlack),
                ic = Style::reset().fg(Color::White),
            ))
        })
        .level(LOG_LEVEL_FILTER_AT)
        .chain(std::io::stdout())
        //.chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    println!("{}", MOTD);
    // setup the logger using the fern crate
    if let Err(e) = setup_logger() {
        eprintln!("Failed to initialize the logging system: {}", e);
        return;
    }

    info!(target:LOG_TARGET, "Reading configuration file...");
    // handle configuration
    let cdir_r = std::env::current_dir();
    if let Err(e) = cdir_r {
        error!(target:LOG_TARGET, "Failed to create config file: no access! raw error: {}", e);
        return;
    }
    let cdir = cdir_r.unwrap();
    let current_dir_r = cdir.as_path().to_str();
    if current_dir_r.is_none() {
        error!(target:LOG_TARGET, "Could not access the config file!");
        return;
    }
    let current_dir = current_dir_r.unwrap();
    let config_path = format!("{}/config/config.toml", current_dir);
    let raw_path = Path::new(&config_path);
    let config = read_config(raw_path, format!("\
    [server]\
    \n# ip: the ip to listen on\
    \n# surround with '[' and ']' for Ipv6 addresses\
    \n# defaults to 0.0.0.0 and will listen on your machines current IP\
    \nip = \"0.0.0.0\"\
    \n# port: the port to listen on\
    \n# defaults to 2277\
    \nport = \"2277\""));

    // set default values for the config
    let mut ip = format!("0.0.0.0");
    let mut port = format!("2277");

    // if the configuration values are set, override defaults
    if let Some(server_conf) = config.server {
        if let Some(cfg_ip) = server_conf.ip {
            ip = cfg_ip;
        }
        if let Some(cfg_port) = server_conf.port {
            port = cfg_port;
        }
    }

    info!(target:LOG_TARGET, "Read config with the listening IP {} and the port {}", ip.clone(), port.clone());
    info!(target:LOG_TARGET, "Starting TCP Listener...");

    let full_ip = format!("{}:{}", ip, port);

    // start listening for connections
    let listener_result = TcpListener::bind(full_ip.clone());
    if listener_result.is_err() {
        error!(target:LOG_TARGET, "Failed to bind listener to ip: {}", listener_result.unwrap_err());
        return;
    }
    let listener = listener_result.unwrap();
    // set the listener to non-blocking mode to enable safely exiting the server
    if let Err(e) = listener.set_nonblocking(true) {
        error!(target:LOG_TARGET, "Failed to set the connection listener to non-blocking mode; safely exiting would not be possible.\n  Error: {}", e);
        return;
    }

    // create the database instance for the clients to use
    info!(target:LOG_TARGET, "Connecting to the database...");
    let db = Arc::new(Mutex::new(Database::new("snd")));
    info!(target:LOG_TARGET, "Connected to the database!");

    // create a flag for threads to access to let them know if the program is shutting down
    let terminate = Arc::new(AtomicBool::new(false));

    // safely exit when ctrl+c is called
    let ctrlc_tarc = Arc::clone(&terminate);
    let cc_handler = ctrlc::set_handler(move || {
        ctrlc_tarc.store(true, Ordering::SeqCst);
    });
    if let Err(e) = cc_handler {
        error!(target:LOG_TARGET, "Failed to set exit handler; no safe way to exit: {}", e);
        return;
    }

    // store the join handlers for closing later
    // todo(eric): I dont think this drops handlers that are no longer active
    let mut handlers = Vec::new();

    info!(target:LOG_TARGET, "Started listening at {}", full_ip);
    info!(target:LOG_TARGET, "Accpting client version {}", ACCEPTED_CLIENT_VERSION);

    // listen for incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                // create a new reference to the database for the client to access
                // save memory with only one database access and provide thread safety
                let db_arc = Arc::clone(&db);
                // create a reference to the terminate flag
                let tarc = Arc::clone(&terminate);

                // spawn a new thread with the client handler
                handlers.push(thread::spawn(move || {
                    handle_connection(s, db_arc, tarc);
                }));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // handle if the program needs to exit
                if terminate.load(Ordering::SeqCst) {
                    info!(target:LOG_TARGET, "Safely shutting down server...");
                    break;
                }

                // handle handlers no longer in use
                handlers.retain(|h| { h.is_running() });

                // save CPU resources with a sleep call
                thread::sleep(Duration::from_millis(MAIN_LOOP_WAIT_DELAY_MS));
                continue;
            }
            Err(e) => {
                error!(target:LOG_TARGET, "Encountered an IO error when polling for connections: {}", e);
                // safely exit
                break;
            }
        }
    }

    // store that the program is terminating and the clients should be disconnected
    terminate.store(true, Ordering::SeqCst);

    info!(target:LOG_TARGET, "Shutting down all active connections...");
    // ensure all threads are closed before shutting down the server
    for h in handlers {
        if let Err(_) = h.join() {
            // warn!(target:LOG_TARGET, "A thread was unavailable when shutting down!")
        }
    }

    // stop the listener
    drop(listener);

    info!(target:LOG_TARGET, "Server shut down!");

}

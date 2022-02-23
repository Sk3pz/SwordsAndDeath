use std::fs::File;
use std::io::Read;
use std::net::TcpListener;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::client::handle_connection;
use crate::config::read_config;

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

pub const ACCEPTED_CLIENT_VERSION: &str = "0.1.0";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MOTD: &str = "Welcome to SnD! We are still in ALPHA, so expect some bugs!";
pub const KEEPALIVE_INTERVAL: u64 = 20; // time in seconds to send the keepalive packet

pub fn systime() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Fatal error occurred: System time moved backwards! Are you a time traveler?")
}

pub fn to_epoch(time: SystemTime) -> Duration {
    time.duration_since(UNIX_EPOCH)
        .expect("Fatal error occurred: System time moved backwards! Are you a time traveler?")
}

pub fn read_config_raw(file: &mut File) -> String {
    let mut config_content = String::new();
    file.read_to_string(&mut config_content)
        .expect("Failed to read config file. Please make sure that the server has permission to edit files.");
    config_content
}

fn main() {

    // handle configuration
    let cdir = std::env::current_dir().expect("Error in attempting to get config file: no access.");
    let current_dir = cdir.as_path().to_str().expect("Error in attempting to get config file: no access.");
    let config_path = format!("{}/config/config.toml", current_dir);
    let raw_path = Path::new(&config_path);
    let config = read_config(raw_path, format!("[server]\
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

    // start listening for connections
    let listener_result = TcpListener::bind(format!("{}:{}", ip, port));
    if listener_result.is_err() {
        eprintln!("Failed to bind listener to ip: {}", listener_result.unwrap_err());
        return;
    }
    let listener = listener_result.unwrap();

    // listen for incoming connections
    for stream in listener.incoming() {
        let stream_result = stream;
        if stream_result.is_err() {
            eprintln!("Failed to accept incoming connection: {}", stream_result.unwrap_err());
            continue;
        }
        let stream = stream_result.unwrap();

        // todo(eric): log connection

        // spawn a new thread with the client handler
        // todo(eric): better handle connections, maybe through a thread pool?
        thread::spawn(move || {
            handle_connection(stream);
        });
    }

}

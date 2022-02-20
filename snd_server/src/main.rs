use std::fs::File;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::config::read_config;

pub mod client;
mod config;

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
    file.read_to_string(&mut config_content).expect("Failed to read config file. Please make sure that the server has permission to edit files.");
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

    let mut ip = format!("0.0.0.0");
    let mut port = format!("2277");

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
        // todo(eric): error here
        return;
    }

    let listener = listener_result.unwrap();

}

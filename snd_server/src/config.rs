use std::fs::OpenOptions;
use std::fs;
use std::path::Path;
use std::io::Write;
use serde::Deserialize;
use crate::read_config_raw;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Option<Server>
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub ip: Option<String>,
    pub port: Option<String>
}

pub fn read_config(path: &Path, default: String) -> Config {
    let dir = path.parent().expect("Failed to get parent location of config file. Invalid permissions?");
    if !dir.exists() {
        match fs::create_dir_all(dir) {
            Ok(_) => (),
            Err(e) => panic!("Failed to create parent directories: {}", e)
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path.clone())
        .expect("An error occurred in opening the config file.");

    let mut data = read_config_raw(&mut file);

    if data.is_empty() {
        match file.write_all(default.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                panic!("Failed to write defaults to config file: {}", e);
            }
        }
        data = default;
    }

    toml::from_str(data.as_str()).expect("Could not read config: Please make sure it is valid and has all keys defined, according to the server-config-example.toml")
}
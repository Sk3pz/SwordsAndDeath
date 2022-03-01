use std::io;
use std::io::{stdin, stdout, Write};
use std::net::TcpStream;
use better_term::{Color, flush_styles};
use regex::Regex;
use snd_network_lib::client_event::write_client_disconnect;
use snd_network_lib::entry_point_io::{write_entry_login_attempt, write_entry_point_ver};
use snd_network_lib::entry_response::read_entry_response;
use snd_network_lib::login_data::LoginData;

fn read_input<S: Into<String>>(prompt: S) -> String {
    print!("{}", prompt.into());
    let r = stdout().flush();
    if r.is_err() {
        panic!("Error flusing output: {}", r.unwrap_err());
    }
    let mut buffer = String::new();
    let r2 = stdin().read_line(&mut buffer);
    if r2.is_err() {
        panic!("Error in reading input: {}", r.unwrap_err());
    }
    buffer.replace("\n", "").replace("\r", "")
}

pub fn prompt<S: Into<String>>(prompt: S) -> bool {
    let p = prompt.into();
    loop {
        let input = read_input(format!("{} (Y or N): ", p));
        match input.to_ascii_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                println!("{}Warning: The input can only be Y or N!", Color::Yellow);
                flush_styles();
            }
        }
    }
}

fn get_ip() -> String {
    let ip_pattern =
        Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$")
            .expect("Failed to init regex");
    let port_pattern =
        Regex::new(r"^((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$")
            .expect("Failed to init regex");

    let mut ip = read_input(format!("Input the ip of the server: "));
    let mut port = read_input(format!("Input the port of the server: "));

    while !ip_pattern.is_match(ip.as_str()) {
        eprintln!("Invalid ip! please enter a valid ip!");
        ip = read_input(format!("Input the ip of the server: "));
    }

    while !port_pattern.is_match(port.as_str()) {
        eprintln!("Invalid entry for the connection port. Please enter the port ranging from 0 to 65535");
        port = read_input(format!("Input the port of the server: "));
    }

    format!("{}:{}", ip, port)
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    //let ip = get_ip();
    let ip = format!("127.0.0.1:2277");

    // ping loop
    loop {
        // get the connection
        let mut ping_stream = TcpStream::connect(ip.clone());
        if ping_stream.is_err() {
            eprintln!("Failed to connect to server.");
            continue;
        }

        // validate the connection
        let ps = ping_stream.unwrap();

        // write version to server for ping
        if let Err(e) = write_entry_point_ver(&ps, VERSION.to_string()) {
            eprintln!("Failed to write ping to server to check version. Error: {}", e);
            return;
        }

        // read response
        let (_, version, error) = read_entry_response(&ps);

        if let Some(err) = error {
            eprintln!("{}", err);
            return;
        }

        if version.is_none() {
            eprintln!("Unknown issue occurred getting version from the server.");
            return;
        }

        // if it was an error, print the message and see if the user wants to continue or exit
        if error.is_some() {
            eprintln!("Error from the server: {}", error.unwrap());
        }

        drop(ps);
        break;
    }

    // get login data
    // todo(eric): have a system for inputting login information
    // todo(eric): have a system to cache login data
    let username = "test_username123".to_string();
    let passwd = "password123".to_string();
    let signup = false;

    // connect to server and send login data
    let stream_res = TcpStream::connect(ip);
    if let Err(e) = stream_res {
        eprintln!("Failed to connect to server to login! Error: {}", e);
        return;
    }
    let stream = stream_res.unwrap();

    write_entry_login_attempt(&stream, LoginData { username, passwd, signup,
        client_ver: VERSION.to_string() });


    let (motd, _, error) = read_entry_response(&stream);

    if error.is_some() {
        eprintln!("Error from server: {}", error.unwrap());
        return;
    }

    if motd.is_none() {
        eprintln!("unexpected error: invalid response from server");
        return;
    }

    println!("{}", motd.unwrap());

    write_client_disconnect(&stream);
}

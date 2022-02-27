use std::io::{stdin, stdout, Write};
use std::net::TcpStream;
use regex::Regex;

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

fn get_ip() -> String {
    let ip_pattern =
        Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)(\.(?!$)|$)){4}$")
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

fn main() {
    let mut stream = TcpStream::connect(get_ip());
}

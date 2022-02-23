use std::io::{stdin, stdout, Write};
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

fn main() {
    let ip_pattern = Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)(\.(?!$)|$)){4}$");
}

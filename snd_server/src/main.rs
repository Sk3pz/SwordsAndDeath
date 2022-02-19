use std::net::{TcpListener, TcpStream};

fn handle_connection(stream: TcpStream) {
    // handle an incoming request
}

fn main() {
    // handle configuration

    // start listening for connections
    let listener_result = TcpListener::bind("");
    if listener_result.is_err() {
        // todo(eric): error here
        return;
    }

    let listener = listener_result.unwrap();

}

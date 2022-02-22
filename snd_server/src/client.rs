use std::net::TcpStream;
use crate::database::Database;

pub fn handle_connection(stream: TcpStream) {
    // create a database instance
    // this is concurrently safe because items are only accessed by their owners,
    // and only one instance of a specific player can be connected at a time, meaning
    // there should be no two threads accessing the same data in the database.
    let database = Database::new("snd.sqlite");

    // handle an incoming request

    // todo(eric): login system here

    // todo(eric): main game loop here
    loop {



        break;
    }
}
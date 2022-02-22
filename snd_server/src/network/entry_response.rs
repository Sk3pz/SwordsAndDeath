use std::net::TcpStream;
use capnp::message::Builder;
use capnp::serialize;
use crate::packet_capnp::entry_response;

pub fn write_valid_entry_response(mut stream: &TcpStream, motd: String) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<entry_response::Builder>();
        er.set_motd(motd.as_str());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_invalid_entry_response<S: Into<String>>(mut stream: &TcpStream, err: S) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<entry_response::Builder>();
        er.set_error(err.into().as_str());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_ping_entry_response(mut stream: &TcpStream, client_valid: bool, version: String) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<entry_response::Builder>();
        if client_valid {
            er.set_version(version.as_str());
        } else {
            er.set_error(format!("Invalid version: {}", version).as_str());
        }
    }
    serialize::write_message(&mut stream, &message)
}

/// returns vmotd, version, error
pub fn read_entry_response(mut stream: &TcpStream) -> (Option<String>, Option<String>, Option<String>) {
    let message_reader_result = serialize::read_message(&mut stream, ::capnp::message::ReaderOptions::new());
    if message_reader_result.is_err() {
        return (None, None, Some(String::from("Could not connect to server.")));
    }
    let message_reader = message_reader_result.unwrap();
    let er_raw = message_reader.get_root::<entry_response::Reader>();
    if er_raw.is_err() {
        return (None, None, Some(String::from("Could not connect to server.")));
    }
    let er = er_raw.unwrap();

    return match er.which() {
        Ok(entry_response::Version(v)) => {
            (None, Some(v.unwrap().to_string()), None)
        }
        Ok(entry_response::Motd(motd)) => {
            (Some(motd.unwrap().to_string()), None, None)
        }
        Ok(entry_response::Error(err)) => {
            (None, None, Some(err.unwrap().to_string()))
        }
        Err(::capnp::NotInSchema(_)) => {
            (None, None, Some(String::from("Invalid EntryResponse - no data found!")))
        }
    }
}
use std::net::TcpStream;
use capnp::message::Builder;
use capnp::serialize;
use crate::login_data::LoginData;
use crate::packet_capnp::entry_point;

pub fn write_entry_point_ver(mut stream: &TcpStream, version: String) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut ep = message.init_root::<entry_point::Builder>();
        ep.set_version(version.as_str());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_entry_login_attempt(mut stream: &TcpStream, login_data: LoginData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let ep = message.init_root::<entry_point::Builder>();
        let mut login = ep.init_login_attempt();
        login.set_username(login_data.username.as_str());
        login.set_password(login_data.passwd.as_str());
        login.set_signup(login_data.signup);
    }
    serialize::write_message(&mut stream, &message)
}

/// Returns LoginData, version, error
pub fn read_entry_point(mut stream: &TcpStream) -> (Option<LoginData>, Option<String>, Option<String>) {
    let msg_reader_raw = serialize::read_message(&mut stream, ::capnp::message::ReaderOptions::new());
    if msg_reader_raw.is_err() {
        return (None, None, Some(format!("Client disconnected while expecting message")));
    }
    let message_reader = msg_reader_raw.unwrap();
    let ep_raw = message_reader.get_root::<entry_point::Reader>();
    if ep_raw.is_err() {
        return (None, None, Some(format!("Client disconnected while expecting message")));
    }
    let ep = ep_raw.unwrap();

    return match ep.which() {
        Ok(entry_point::LoginAttempt(login_data)) => {
            let raw_ld = login_data.unwrap();
            let ld = LoginData {
                username: raw_ld.get_username().unwrap().to_string(),
                passwd: raw_ld.get_password().unwrap().to_string(),
                signup: raw_ld.get_signup(),
                client_ver: raw_ld.get_client_ver().unwrap().to_string(),
            };
            (Some(ld), None, None)
        }
        Ok(entry_point::Version(ver)) => {
            (None, Some(ver.unwrap().to_string()), None)
        }
        Err(::capnp::NotInSchema(_)) => {
            // todo(eric): error
            (None, None, Some(String::from("Invalid EntryPoint - no version or login data found!")))
        }
    }
}
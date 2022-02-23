use std::net::TcpStream;
use capnp::message::Builder;
use capnp::serialize;
use crate::error_data::ErrorData;
use crate::packet_capnp::c_event;
use crate::systime;

#[derive(Clone, Debug)]
pub enum ClientEvent {
    Disconnect,
    KeepAlive(u64),
    Step,
    OpenInv,
    DropItem(String),
    InspectItem(String),
    Attack,
    TryFlee,
    Error(ErrorData),
}

pub fn write_client_disconnect(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_disconnect(true);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_client_keepalive(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_keepalive(systime().as_secs());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_client_step(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_step(true);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_client_open_inv(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_open_inv(true);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_client_drop_item(mut stream: &TcpStream, item: String) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_drop_itm(item.as_str());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_client_inspect_item(mut stream: &TcpStream, item: String) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_inspect_itm(item.as_str());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_client_error(mut stream: &TcpStream, error: ErrorData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<c_event::Builder>();
        let mut error_reader = er.init_error();
        error_reader.set_error(error.msg.as_str());
        error_reader.set_disconnect(error.disconnect);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn read_client_event(mut stream: &TcpStream) -> ClientEvent {
    let message_reader_result =
        serialize::read_message(&mut stream, ::capnp::message::ReaderOptions::new());
    if message_reader_result.is_err() {
        return ClientEvent::Error(ErrorData { msg: format!("Failed to read packet from client!"), disconnect: true });
    }
    let message_reader = message_reader_result.unwrap();

    let er_raw = message_reader.get_root::<c_event::Reader>();
    if er_raw.is_err() {
        return ClientEvent::Error(ErrorData { msg: format!("Failed to read packet from client!"), disconnect: true });
    }
    let er = er_raw.unwrap();

    let which = er.which();

    if let Err(err) = which {
        return ClientEvent::Error(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true });
    }

    let w = which.unwrap();

    match w {
        c_event::Disconnect(_) => ClientEvent::Disconnect,
        c_event::Keepalive(a) => ClientEvent::KeepAlive(a),
        c_event::Step(_) => ClientEvent::Step,
        c_event::OpenInv(_) => ClientEvent::OpenInv,
        c_event::DropItm(name) => ClientEvent::DropItem(name.unwrap().to_string()),
        c_event::InspectItm(name) => ClientEvent::InspectItem(name.unwrap().to_string()),
        c_event::Attack(_) => ClientEvent::Attack,
        c_event::TryFlee(_) => ClientEvent::TryFlee,
        c_event::Error(err_reader) => {
            let err = err_reader.unwrap();
            ClientEvent::Error(ErrorData { msg: err.get_error().unwrap().to_string(), disconnect: err.get_disconnect() })
        }
    }
}
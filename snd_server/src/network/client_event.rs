use std::net::TcpStream;
use capnp::message::Builder;
use capnp::serialize;
use crate::network::error_data::ErrorData;
use crate::packet_capnp::c_event;
use crate::systime;

#[derive(Clone, Debug)]
pub struct ClientEvent {
    pub disconnect: Option<bool>,
    pub keepalive: Option<u64>,
    pub step: Option<bool>,
    pub open_inv: Option<bool>,
    pub drop_itm: Option<String>,
    pub inspect_itm: Option<String>,
    pub attck: Option<bool>,
    pub try_flee: Option<bool>,
    pub error: Option<ErrorData>,
}

impl ClientEvent {
    pub fn new(disconnect: Option<bool>, keepalive: Option<u64>,
               step: Option<bool>, open_inv: Option<bool>, drop_itm: Option<String>, inspect_itm: Option<String>,
               attck: Option<bool>, try_flee: Option<bool>, error: Option<ErrorData>) -> Self {
        Self {
            disconnect, keepalive, step, open_inv, drop_itm, inspect_itm, attck, try_flee, error
        }
    }
}

pub fn write_client_disconnect(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<c_event::Builder>();
        er.set_disconnect(true);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_keepalive(mut stream: &TcpStream) -> ::capnp::Result<()> {
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
    let message_reader_result = serialize::read_message(&mut stream, ::capnp::message::ReaderOptions::new());
    if message_reader_result.is_err() {
        return ClientEvent::new(None, None, None, None,
                                None, None, None, None,
                                Some(ErrorData { msg: format!("Failed to read packet from client!"), disconnect: true }));
    }
    let message_reader = message_reader_result.unwrap();

    let er_raw = message_reader.get_root::<c_event::Reader>();
    if er_raw.is_err() {
        return ClientEvent::new(None, None, None, None,
                                None, None, None, None,
                                Some(ErrorData { msg: format!("Failed to read packet from client!"), disconnect: true }));
    }
    let er = er_raw.unwrap();

    let which = er.which();

    if let Err(err) = which {
        return ClientEvent::new(None, None, None, None,
                                None, None, None, None,
                                Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
    }

    let w = which.unwrap();

    match w {
        c_event::Disconnect(b) => ClientEvent::new(Some(b), None, None, None,
                                                          None, None, None, None, None),
        c_event::Keepalive(a) => ClientEvent::new(None, Some(a), None, None,
                                                         None, None, None, None, None),
        c_event::Step(_) => ClientEvent::new(None, None, Some(true), None,
                                                    None, None, None, None, None),
        c_event::OpenInv(_) => ClientEvent::new(None, None, None, Some(true),
                                                None, None, None, None, None),
        c_event::DropItm(name) => ClientEvent::new(None, None, None, None,
                                                   Some(name.unwrap().to_string()), None, None, None, None),
        c_event::InspectItm(name) => ClientEvent::new(None, None, None, None,
                                                      None, Some(name.unwrap().to_string()), None, None, None),
        c_event::Attack(_) => ClientEvent::new(None, None, None, None,
                                               None, None, Some(true), None, None),
        c_event::TryFlee(_) => ClientEvent::new(None, None, None, None,
                                               None, None, None, Some(true), None),
        c_event::Error(err_reader) => {
            let err = err_reader.unwrap();
            ClientEvent::new(None, None, None, None,
                             None, None, None, None,
                             Some(ErrorData {
                                        msg: err.get_error().unwrap().to_string(),
                                        disconnect: err.get_disconnect()
                }))
        }
    }
}
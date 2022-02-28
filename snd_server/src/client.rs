use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use log::{error, info, warn};
use uuid::Uuid;
use crate::{ACCEPTED_CLIENT_VERSION, KEEPALIVE_INTERVAL, MOTD, SERVER_VERSION};
use crate::database::{Database, LoginFailReason};
use snd_network_lib::to_epoch;
use snd_network_lib::client_event::{ClientEvent, read_client_event};
use snd_network_lib::entry_point_io::read_entry_point;
use snd_network_lib::entry_response::{write_invalid_entry_response, write_ping_entry_response, write_valid_entry_response};
use snd_network_lib::error_data::ErrorData;
use snd_network_lib::item_data::ItemData;
use snd_network_lib::server_event::{write_server_disconnect, write_server_error, write_server_inventory, write_server_keepalive};
use crate::player::Player;

pub fn handle_connection(stream: TcpStream, db: Arc<Mutex<Database>>, tarc: Arc<AtomicBool>) {
    // ensure the stream is blocking as the listener was not
    if let Err(e) = stream.set_nonblocking(false) {
        error!("Failed to set a connected stream to blocking, can not handle this connection properly, dropping.");
        write_server_error(&stream, ErrorData {
            msg: format!("Failed to set stream to blocking, can not properly handle connection. error: {}", e),
            disconnect: true
        });
        return;
    }

    // handle an incoming request
    let ip_res = stream.peer_addr();
    let ip = if ip_res.is_err() {
        warn!("Failed to get IP from connection.");
        format!("<INVALID IP: {}>", ip_res.unwrap_err())
    } else {
        ip_res.unwrap().to_string()
    };

    // expect an entrypoint packet
    let (login, version, error) = read_entry_point(&stream);

    if let Some(err) = error {
        error!("Error trying to read entry point packet from {}: {}", ip, err);
        return;
    }

    if let Some(ver) = version {
        let valid = ver == ACCEPTED_CLIENT_VERSION;
        info!("Ping request from {} was {}", ip, match valid.clone() {
            true => "valid",
            false => "invalid"
        });
        let res = write_ping_entry_response(&stream, valid, SERVER_VERSION.to_string());
        if res.is_err() {
            error!("Failed to send ping entry response to {}", ip);
        }
        return;
    }

    info!("Accepted connection from '{}'", ip.clone());

    if login.is_none() {
        error!("Invalid packet from {}: No entry point data received in entry point packet.", ip);
        return;
    }

    // handle logging in and signup
    let login_data = login.unwrap();
    let mut uuid = Uuid::new_v4();
    let username;

    if login_data.signup {
        // validate signup data
        username = login_data.username.escape_debug().to_string();

        if !username.chars().all(|c| { c.is_alphanumeric() || c == '_' }) {
            if let Err(e) = write_invalid_entry_response(&stream, "Username must be only letters, numbers, and underscores") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if username.len() < 3 {
            if let Err(e) = write_invalid_entry_response(&stream, "Username is too short") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if username.len() > 16 {
            if let Err(e) = write_invalid_entry_response(&stream, "Username is too long") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }

        if db.lock().unwrap().player_exists(username.clone()) {
            if let Err(e) = write_invalid_entry_response(&stream, "Username already exists") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }

        let passwd = login_data.passwd.escape_debug().to_string();

        if !passwd.chars().all(|c| { c.is_ascii() && c != ' ' && c != '\'' }) {
            if let Err(e) =
            write_invalid_entry_response(&stream, "Invalid Password: Password must be plain ascii with no spaces or ''s") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if passwd.len() < 4 {
            if let Err(e) = write_invalid_entry_response(&stream, "Password is too short") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if passwd.len() > 32 {
            if let Err(e) = write_invalid_entry_response(&stream, "Password is too long") {
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }

        let player = Player {
            uuid: uuid.clone(), name: login_data.username.clone(),
        };

        if !db.lock().unwrap().new_player(&player, passwd) {
            if let Err(e) = write_invalid_entry_response(&stream, "Failed to enter data into the database"){
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }

    } else {
        // avoid sql injections
        username = login_data.username.escape_debug().to_string();
        let passwd = login_data.passwd.escape_debug().to_string();

        let attempt = db.lock().unwrap().validate_login(username.clone(), passwd);
        if let Err(err) = attempt {
            let res = match err {
                LoginFailReason::Unrecognized => write_invalid_entry_response(&stream, "Invalid User"),
                LoginFailReason::Unauthorized => write_invalid_entry_response(&stream, "Invalid Password"),
                LoginFailReason::AlreadyOnline => write_invalid_entry_response(&stream, "Already Online"),
            };
            if let Err(e) = res {
                error!("Failed to write invalid login data to {}: {}", ip, e);
            }
            return;
        }
        let set_uuid = db.lock().unwrap().uuid_from_username(username.clone());
        if set_uuid.is_none() {
            if let Err(e) = write_invalid_entry_response(&stream, "Failed to find user"){
                error!("Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        uuid = set_uuid.unwrap();
    };

    if let Err(e) = write_valid_entry_response(&stream, MOTD.to_string()) {
        error!("Failed to send entry response to {}: {}", ip, e);
        return;
    }

    let mut last_keepalive = SystemTime::now();
    let mut expecting_keepalive = false;
    let mut ping = 0;

    db.lock().unwrap().set_player_active(&uuid);

    // game loop
    loop {
        // check if the server is being shutdown
        if tarc.load(Ordering::SeqCst) {
            if let Err(e) = write_server_error(&stream, ErrorData { msg: format!("The server is shutting down!"), disconnect: true}) {
                error!("Failed to send shutdown message to {}: {}", ip, e);
                break;
            }
        }

        // check keepalive
        let now = SystemTime::now();
        let duration = now.duration_since(last_keepalive)
            .expect("Fatal error occurred: System time moved backwards! Are you a time traveler?")
            .as_secs();
        if duration >= KEEPALIVE_INTERVAL {
            if !expecting_keepalive { // if there is not a keepalive expected, send a request
                if let Err(e) = write_server_keepalive(&stream) {
                    error!("Failed to write keepalive request to {}: {}", ip, e);
                    break;
                }
                last_keepalive = SystemTime::now();
                expecting_keepalive = true;
            } else { // if there is a keepalive scheduled, disconnect the client
                // todo(eric): if any extra steps need to be taken to disconnect the client
                if let Err(e) = write_server_disconnect(&stream) {
                    error!("failed to send disconnect for no keepalive response to {}: {}", ip, e);
                }
                break;
            }
        }

        // expect a client event from the user
        let event = read_client_event(&stream);
        match event {
            ClientEvent::Disconnect => {
                // if the user sends that it disconnected, drop the connection properly
                break;
            }
            ClientEvent::KeepAlive(a) => {
                // for handling user disconnects and timeouts
                if !expecting_keepalive {
                    // Not expecting a keepalive, ignore
                    continue;
                }
                // calculate the ping
                ping = a - to_epoch(last_keepalive).as_secs() - KEEPALIVE_INTERVAL;
                // set flag
                expecting_keepalive = false;
            }
            ClientEvent::Step => {
                // todo(eric): game logic
            }
            ClientEvent::OpenInv => {
                // get the player's inventory from the database and send it to the client to display
                let inv = db.lock().unwrap().get_player_items(&uuid);
                if let Err(e) = write_server_inventory(&stream,
                                                       inv.unwrap_or(Vec::new())
                                                           .iter().map(|i| { i.as_data() }).collect::<Vec<ItemData>>()) {
                    error!("error sending inventory to {}: {}", ip, e);
                    break;
                }
            }
            ClientEvent::DropItem(item_name) => {
                // todo(eric): game logic
            }
            ClientEvent::InspectItem(item_name) => {
                // todo(eric): send item data
            }
            ClientEvent::Attack => {
                // todo(eric): game logic
            }
            ClientEvent::TryFlee => {
                // todo(eric): game logic
            }
            ClientEvent::Error(err) => {
                // todo(eric): implement better error handling?
                error!("{} encountered an error: {}", ip, err.msg);
                if err.disconnect {
                    break;
                }
            }
        }

    }

    // clean up stuff and properly disconnect the user
    db.lock().unwrap().set_player_inactive(&uuid);
}
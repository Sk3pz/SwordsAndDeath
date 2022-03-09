use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use log::{error, info, trace, warn};
use rand::{Rng, thread_rng};
use rand_distr::{Normal, Distribution};
use uuid::Uuid;
use crate::{ACCEPTED_CLIENT_VERSION, KEEPALIVE_INTERVAL, MOTD};
use crate::database::{Database, LoginFailReason, PlayerValueDB};
use snd_network_lib::to_epoch;
use snd_network_lib::client_event::{ClientEvent, read_client_event};
use snd_network_lib::entry_point_io::read_entry_point;
use snd_network_lib::entry_response::{write_invalid_entry_response, write_ping_entry_response, write_valid_entry_response};
use snd_network_lib::error_data::ErrorData;
use snd_network_lib::item_data::ItemData;
use snd_network_lib::player_data::PlayerData;
use snd_network_lib::server_event::{write_server_disconnect, write_server_error, write_server_event, write_server_find_item, write_server_gain_exp, write_server_inventory, write_server_item_view, write_server_keepalive, write_server_update};
use crate::item::{Item, ItemRarity, ItemType};
use crate::player::Player;

const LOG_TARGET: &str = "client_handler";

pub fn handle_connection(stream: TcpStream, db: Arc<Mutex<Database>>, tarc: Arc<AtomicBool>) {
    // ensure the stream is blocking as the listener was not
    if let Err(e) = stream.set_nonblocking(false) {
        error!(target:LOG_TARGET, "Failed to set a connected stream to blocking, can not handle this connection properly, dropping.");
        let _ = write_server_error(&stream, ErrorData {
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
        error!(target:LOG_TARGET, "Error trying to read entry point packet from {}: {}", ip, err);
        return;
    }

    if let Some(ver) = version {
        let valid = ver == ACCEPTED_CLIENT_VERSION;
        info!(target:LOG_TARGET, "Ping request from {} was {}", ip, match valid.clone() {
            true => "valid",
            false => "invalid"
        });
        let res = write_ping_entry_response(&stream, valid, ACCEPTED_CLIENT_VERSION.to_string());
        if res.is_err() {
            error!(target:LOG_TARGET, "Failed to send ping entry response to {}", ip);
        }
        return;
    }

    info!(target:LOG_TARGET, "Accepted connection from '{}'", ip.clone());

    if login.is_none() {
        error!(target:LOG_TARGET, "Invalid packet from {}: No entry point data received in entry point packet.", ip);
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
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if username.len() < 3 {
            if let Err(e) = write_invalid_entry_response(&stream, "Username is too short") {
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if username.len() > 16 {
            if let Err(e) = write_invalid_entry_response(&stream, "Username is too long") {
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }

        if db.lock().unwrap().player_exists(username.clone()) {
            if let Err(e) = write_invalid_entry_response(&stream, "Username already exists") {
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }

        let passwd = login_data.passwd.escape_debug().to_string();

        if !passwd.chars().all(|c| { c.is_ascii() && c != ' ' && c != '\'' }) {
            if let Err(e) =
            write_invalid_entry_response(&stream, "Invalid Password: Password must be plain ascii with no spaces or ''s") {
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if passwd.len() < 4 {
            if let Err(e) = write_invalid_entry_response(&stream, "Password is too short") {
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        if passwd.len() > 32 {
            if let Err(e) = write_invalid_entry_response(&stream, "Password is too long") {
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }

        let player = Player {
            uuid: uuid.clone(), name: login_data.username.clone(),
        };

        if !db.lock().unwrap().new_player(&player, passwd) {
            if let Err(e) = write_invalid_entry_response(&stream, "Failed to enter data into the database"){
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
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
                error!(target:LOG_TARGET, "Failed to write invalid login data to {}: {}", ip, e);
            }
            return;
        }
        let set_uuid = db.lock().unwrap().uuid_from_username(username.clone());
        if set_uuid.is_none() {
            if let Err(e) = write_invalid_entry_response(&stream, "Failed to find user"){
                error!(target:LOG_TARGET, "Failed to write error to {}: {}", ip, e);
            }
            return;
        }
        uuid = set_uuid.unwrap();
    };

    if let Err(e) = write_valid_entry_response(&stream, MOTD.to_string()) {
        error!(target:LOG_TARGET, "Failed to send entry response to {}: {}", ip, e);
        return;
    }

    info!(target:LOG_TARGET, "User {} logged in with the uuid {}", username, uuid);

    let mut last_keepalive = SystemTime::now();
    let mut expecting_keepalive = false;
    let mut ping = 0;

    db.lock().unwrap().set_player_active(&uuid);

    // game loop
    loop {
        // check if the server is being shutdown
        if tarc.load(Ordering::SeqCst) {
            if let Err(e) = write_server_error(&stream, ErrorData { msg: format!("The server is shutting down!"), disconnect: true}) {
                error!(target:LOG_TARGET, "Failed to send shutdown message to {}: {}", ip, e);
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
                    error!(target:LOG_TARGET, "Failed to write keepalive request to {}: {}", ip, e);
                    break;
                }
                last_keepalive = SystemTime::now();
                expecting_keepalive = true;
            } else { // if there is a keepalive scheduled, disconnect the client
                // todo(eric): if any extra steps need to be taken to disconnect the client
                if let Err(e) = write_server_disconnect(&stream) {
                    error!(target:LOG_TARGET, "failed to send disconnect for no keepalive response to {}: {}", ip, e);
                }
                break;
            }
        }

        // expect a client event from the user
        let event = read_client_event(&stream);
        match event {
            ClientEvent::Disconnect => {
                // if the user sends that it disconnected, drop the connection properly
                let _ = write_server_disconnect(&stream);
                break;
            }
            ClientEvent::KeepAlive(a) => {
                // for handling user disconnects and timeouts
                if !expecting_keepalive {
                    // Not expecting a keepalive, ignore
                    continue;
                }
                // calculate the ping
                ping = a - (to_epoch(last_keepalive).as_secs() - KEEPALIVE_INTERVAL);
                trace!(target:LOG_TARGET, "Connection with {} has ping {}", ip.clone(), ping.clone());
                // set flag
                expecting_keepalive = false;
            }
            ClientEvent::RqstUpdate => {
                let pd = PlayerData {
                    level: db.lock().unwrap().get_player_level(&uuid).unwrap(),
                    exp: db.lock().unwrap().get_player_exp(&uuid).unwrap(),
                    health: db.lock().unwrap().get_player_health(&uuid).unwrap(),
                    steps: db.lock().unwrap().get_player_steps(&uuid).unwrap(),
                    region: db.lock().unwrap().get_player_region(&uuid).unwrap(),
                };

                if let Err(e) = write_server_update(&stream, pd) {
                    error!(target:LOG_TARGET, "Failed to write update to {} connected at ip {}: {}", username, ip, e);
                    break;
                }
            }
            ClientEvent::Step => {
                // increment the player's total step count
                if !db.lock().unwrap().inc_player_steps(&uuid) {
                    warn!(target:LOG_TARGET, "Player {} took a step but the database failed to write steps", username);
                }

                // randomly select between gaining exp, finding an item, or having an encounter
                // todo(eric): add finding items and encounters

                let rng = thread_rng().gen_range(0..100);

                match rng {
                    // 60% - Gain EXP
                    _ if rng < 80 => {
                        // generate the amount of exp the player gets
                        let normal_res = Normal::new(5.0, 3.2);
                        if normal_res.is_err() {
                            error!(target:LOG_TARGET,
                        "Failed to create normal distribution for EXP generation for ip {}", ip);
                            break;
                        }
                        let rnd = normal_res.unwrap().sample(&mut thread_rng()) as u32;
                        let amt = rnd.min(10).max(2);
                        if let Err(e) = write_server_gain_exp(&stream, amt.clone()) {
                            error!(target:LOG_TARGET, "Failed to send exp gain to client: {}", e);
                            break;
                        }
                        // update the player's exp in the database
                        db.lock().unwrap().add_player_exp(&uuid, amt);
                        // check if the player needs to level up
                        db.lock().unwrap().check_levelup(&uuid);
                    }
                    // 10% - Find Item
                    _ if rng < 90 => {
                        let found_item = Item::new_rand(ItemType::rand(), &uuid,
                                                        db.lock().unwrap().get_player_level(&uuid).unwrap_or(0),
                                                        ItemRarity::new_rand());
                        db.lock().unwrap().new_item(&found_item);
                        if let Err(e) = write_server_find_item(&stream, found_item.as_data()) {
                            error!(target:LOG_TARGET, "error sending found item to {}: {}", ip, e);
                            break;
                        }
                    }
                    // 10% - Encounter enemy
                    _ if rng < 100 => {
                        // todo(eric): add encounters
                    }
                    _ => { unreachable!() }
                }
            }
            ClientEvent::OpenInv => {
                // get the player's inventory from the database and send it to the client to display
                let inv = db.lock().unwrap().get_player_items(&uuid);
                if let Err(e) = write_server_inventory(&stream,
                                                       inv.unwrap_or(Vec::new())
                                                           .iter().map(|i| { i.as_data() })
                                                           .collect::<Vec<ItemData>>()) {
                    error!(target:LOG_TARGET, "error sending inventory to {}: {}", ip, e);
                    break;
                }
            }
            ClientEvent::DropItem(item_name) => {
                // avoid sql injections :)
                let safe_name = item_name.escape_debug().to_string().replace("'", "");
                // ensure the item exists
                let item_uuid_op = db.lock().unwrap().item_uuid_from_name(safe_name.clone(), &uuid);
                if item_uuid_op.is_none() {
                    if let Err(e) = write_server_event(&stream, "The item you requested to drop does not exist!") {
                        error!(target:LOG_TARGET, "Failed to send event to {}: {}", ip, e);
                        break;
                    }
                    continue;
                }
                let item_uuid = item_uuid_op.unwrap();
                // ensure the player owns the item
                // the item is known to exist, so this shouldn't fail
                let owner_uuid = db.lock().unwrap().get_item_owner(&item_uuid).unwrap();
                if owner_uuid != uuid {
                    if let Err(e) = write_server_event(&stream,
                                                       "You can only drop items that are in your inventory!") {
                        error!(target:LOG_TARGET, "Failed to send event to {}: {}", ip, e);
                        break;
                    }
                    continue;
                }
                // get the item data
                let item = db.lock().unwrap().get_item(&item_uuid);
                if item.is_none() {
                    if let Err(e) = write_server_event(&stream, "The item you requested to drop does not exist!") {
                        error!(target:LOG_TARGET, "Failed to send event to {}: {}", ip, e);
                        break;
                    }
                    continue;
                }

                // delete the item
                let i = item.unwrap();
                if !db.lock().unwrap().drop_item(&i) {
                    error!(target:LOG_TARGET, "Failed to delete item '{}' from player {}", i.name.clone(), username);
                    let _ = write_server_error(&stream, ErrorData { msg: format!("Failed to delete the item!"), disconnect: false });
                    continue; // not fatal
                }

                if let Err(e) = write_server_event(&stream, format!("You dropped your '{}'", i.name)) {
                    error!("Failed to send event to {} with ip {}: {}", username, ip, e);
                    break;
                }
            }
            ClientEvent::InspectItem(item_name) => {
                // avoid sql injections :)
                let safe_name = item_name.escape_debug().to_string().replace("'", "");
                // ensure the item exists
                let item_uuid_op = db.lock().unwrap().item_uuid_from_name(safe_name.clone(), &uuid);
                if item_uuid_op.is_none() {
                    if let Err(e) = write_server_event(&stream, "The item you requested to view does not exist!") {
                        error!(target:LOG_TARGET, "Failed to send event to {}: {}", ip, e);
                        break;
                    }
                    continue;
                }
                let item_uuid = item_uuid_op.unwrap();
                // ensure the player owns the item
                // the item is known to exist, so this shouldn't fail
                let owner_uuid = db.lock().unwrap().get_item_owner(&item_uuid).unwrap();
                if owner_uuid != uuid {
                    if let Err(e) = write_server_event(&stream,
                                                       "You can currently only view items in your inventory!") {
                        error!(target:LOG_TARGET, "Failed to send event to {}: {}", ip, e);
                        break;
                    }
                    continue;
                }
                // get the item data
                let item = db.lock().unwrap().get_item(&item_uuid);
                if item.is_none() {
                    if let Err(e) = write_server_event(&stream, "The item you requested to view does not exist!") {
                        error!(target:LOG_TARGET, "Failed to send event to {}: {}", ip, e);
                        break;
                    }
                    continue;
                }
                if let Err(e) = write_server_item_view(&stream, item.unwrap().as_data()) {
                    error!(target:LOG_TARGET, "Failed to send item data of {} to {}: {}", safe_name, ip, e);
                    break;
                }
            }
            ClientEvent::Attack => {
                // todo(eric): game logic
            }
            ClientEvent::TryFlee => {
                // todo(eric): game logic
            }
            ClientEvent::Error(err) => {
                error!(target:LOG_TARGET, "{} encountered an error: {}", ip, err.msg);
                if err.disconnect {
                    break;
                }
            }
        }

    }

    // clean up stuff and properly disconnect the user
    db.lock().unwrap().set_player_inactive(&uuid);
}
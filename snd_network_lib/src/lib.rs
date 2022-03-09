use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod packet_capnp;

pub mod entry_point_io;
pub mod entry_response;

pub mod login_data;
pub mod item_data;
pub mod loot_data;
pub mod enemy_data;
pub mod error_data;
pub mod encounter_data;
pub mod player_data;

pub mod client_event;
pub mod server_event;

pub fn systime() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Fatal error occurred: System time moved backwards! Are you a time traveler?")
}

pub fn to_epoch(time: SystemTime) -> Duration {
    time.duration_since(UNIX_EPOCH)
        .expect("Fatal error occurred: System time moved backwards! Are you a time traveler?")
}
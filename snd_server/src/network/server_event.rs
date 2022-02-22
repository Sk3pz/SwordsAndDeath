use std::net::TcpStream;
use capnp::message::Builder;
use capnp::serialize;
use crate::network::enemy_data::EnemyData;
use crate::network::error_data::ErrorData;
use crate::network::item_data::ItemData;
use crate::{packet_capnp, systime};
use crate::network::encounter_data::EncounterData;
use crate::network::loot_data::LootData;
use crate::packet_capnp::{encounter, s_event};

#[derive(Clone, Debug)]
pub struct ServerEvent {
    pub disconnect: Option<bool>,
    pub keepalive: Option<u64>,
    pub gain_exp: Option<u32>,
    pub find_item: Option<ItemData>,
    pub encounter: Option<EncounterData>,
    pub inventory: Option<Vec<ItemData>>,
    pub item_view: Option<ItemData>,
    pub error: Option<ErrorData>,
}

impl ServerEvent {
    pub fn new(disconnect: Option<bool>, keepalive: Option<u64>, gain_exp: Option<u32>,
               find_item: Option<ItemData>, encounter: Option<EncounterData>,
               inventory: Option<Vec<ItemData>>, item_view: Option<ItemData>,
               error: Option<ErrorData>) -> Self {
        Self {
            disconnect, keepalive, gain_exp,
            find_item, encounter, inventory,
            item_view, error,
        }
    }
}

pub fn write_server_disconnect(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<s_event::Builder>();
        er.set_disconnect(true);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_keepalive(mut stream: &TcpStream) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let mut er = message.init_root::<s_event::Builder>();
        er.set_keepalive(systime().as_secs());
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_find_item(mut stream: &TcpStream, item_data: ItemData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut found_item_reader = er.init_find_item();
        found_item_reader.set_name(item_data.name.as_str());
        found_item_reader.set_itype(item_data.itype);
        found_item_reader.set_level(item_data.level);
        found_item_reader.set_rarity(item_data.rarity);
        found_item_reader.set_damage(item_data.damage.unwrap_or(0));
        found_item_reader.set_defense(item_data.defense.unwrap_or(0));
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_encounter_attack(mut stream: &TcpStream, enemy: EnemyData, damage: u32) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut encounter_builder = er.init_encounter();

        encounter_builder.set_attk(damage);

        let mut enemy_builder = encounter_builder.init_enemy();
        enemy_builder.set_name(enemy.name.as_str());
        enemy_builder.set_health(enemy.health);
        enemy_builder.set_level(enemy.level);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_encounter_flee(mut stream: &TcpStream, enemy: EnemyData, success: bool) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut encounter_builder = er.init_encounter();

        encounter_builder.set_flee(success);

        let mut enemy_builder = encounter_builder.init_enemy();
        enemy_builder.set_name(enemy.name.as_str());
        enemy_builder.set_health(enemy.health);
        enemy_builder.set_level(enemy.level);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_encounter_lost(mut stream: &TcpStream, enemy: EnemyData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut encounter_builder = er.init_encounter();

        encounter_builder.set_lost(true);

        let mut enemy_builder = encounter_builder.init_enemy();
        enemy_builder.set_name(enemy.name.as_str());
        enemy_builder.set_health(enemy.health);
        enemy_builder.set_level(enemy.level);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_encounter_win(mut stream: &TcpStream, enemy: EnemyData, loot: LootData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut encounter_builder = er.init_encounter();

        let mut win_builder = encounter_builder.reborrow().init_win();
        win_builder.set_exp(loot.exp);

        for x in 0..loot.items.len() {
            let item_data = loot.items.get(x).unwrap();
            let index = x as u32;
            let mut ib = win_builder.reborrow().get_items().unwrap().get(index);
            ib.reborrow().set_name(item_data.name.as_str());
            ib.reborrow().set_itype(item_data.itype);
            ib.reborrow().set_level(item_data.level);
            ib.reborrow().set_rarity(item_data.rarity);
            ib.reborrow().set_damage(item_data.damage.unwrap_or(0));
            ib.reborrow().set_defense(item_data.defense.unwrap_or(0));
            //item_builder.set_with_caveats(index, ib);
        }

        let mut enemy_builder = encounter_builder.reborrow().init_enemy();
        enemy_builder.set_name(enemy.name.as_str());
        enemy_builder.set_health(enemy.health);
        enemy_builder.set_level(enemy.level);
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_inventory(mut stream: &TcpStream, inventory: Vec<ItemData>) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut inv_builder = er.init_inventory(inventory.len() as u32);
        for x in 0..inventory.len() {
            let item_data = inventory.get(x).unwrap();
            let index = x as u32;
            inv_builder.reborrow().get(index).set_name(item_data.name.as_str());
            inv_builder.reborrow().get(index).set_itype(item_data.itype);
            inv_builder.reborrow().get(index).set_level(item_data.level);
            inv_builder.reborrow().get(index).set_rarity(item_data.rarity);
            inv_builder.reborrow().get(index).set_damage(item_data.damage.unwrap_or(0));
            inv_builder.reborrow().get(index).set_defense(item_data.defense.unwrap_or(0));
        }
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_item_view(mut stream: &TcpStream, item_data: ItemData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut item_view_reader = er.init_item_view();
        item_view_reader.set_name(item_data.name.as_str());
        item_view_reader.set_itype(item_data.itype);
        item_view_reader.set_level(item_data.level);
        item_view_reader.set_rarity(item_data.rarity);
        item_view_reader.set_damage(item_data.damage.unwrap_or(0));
        item_view_reader.set_defense(item_data.defense.unwrap_or(0));
    }
    serialize::write_message(&mut stream, &message)
}

pub fn write_server_error(mut stream: &TcpStream, error: ErrorData) -> ::capnp::Result<()> {
    let mut message = Builder::new_default();
    {
        let er = message.init_root::<s_event::Builder>();
        let mut error_reader = er.init_error();
        error_reader.set_error(error.msg.as_str());
        error_reader.set_disconnect(error.disconnect);
    }
    serialize::write_message(&mut stream, &message)
}


// a method for the client to expect messages from the server
pub fn read_server_event(mut stream: &TcpStream) -> ServerEvent {
    let message_reader_result = serialize::read_message(&mut stream, ::capnp::message::ReaderOptions::new());
    if message_reader_result.is_err() {
        return ServerEvent::new(None, None, None,
                                None, None, None,
                                None,
                                Some(ErrorData { msg: format!("Read invalid packet from server!"), disconnect: true }));
    }
    let message_reader = message_reader_result.unwrap();
    let er_raw = message_reader.get_root::<s_event::Reader>();
    if er_raw.is_err() {
        return ServerEvent::new(None, None, None,
                                None, None, None,
                                None,
                                Some(ErrorData { msg: format!("Read invalid packet from server!"), disconnect: true }));
    }
    let er = er_raw.unwrap();

    let which = er.which();

    if let Err(err) = which {
        return ServerEvent::new(None, None, None,
                                None, None, None,
                                None,
                                Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
    }

    match which.unwrap() {
        s_event::Disconnect(b) => ServerEvent::new(Some(b), None, None,
                                                          None, None, None,
                                                          None, None),
        s_event::Keepalive(v) => ServerEvent::new(None, Some(v), None,
                                                  None, None, None,
                                                  None, None),
        s_event::GainExp(v) => ServerEvent::new(None, None, Some(v),
                                                None, None, None,
                                                None, None),
        s_event::FindItem(id_reader) => {
            let raw_id = id_reader.unwrap();
            let id_r_which = raw_id.which();
            if let Err(err) = id_r_which {
                return ServerEvent::new(None, None, None,
                                        None, None, None,
                                        None,
                                        Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
            }
            let mut defense: Option<u32> = None;
            let mut damage: Option<u32> = None;
            let w = id_r_which.unwrap();
            match w {
                packet_capnp::item::Which::Damage(i) => damage = Some(i),
                packet_capnp::item::Which::Defense(i) => defense = Some(i),
            }
            let found = ItemData {
                name: raw_id.get_name().unwrap().to_string(),
                level: raw_id.get_level(),
                itype: raw_id.get_itype(),
                rarity: raw_id.get_rarity(),
                defense, damage
            };

            ServerEvent::new(None, None, None,
                             Some(found), None, None,
                             None, None)
        }
        s_event::Encounter(ed_reader) => {
            let emy = ed_reader.unwrap();
            let which = emy.which();
            if let Err(err) = which {
                return ServerEvent::new(None, None, None,
                                        None, None, None,
                                        None,
                                        Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
            }
            let enemy = emy.get_enemy().unwrap();
            let emydata = EnemyData {
                name: enemy.get_name().unwrap().to_string(),
                level: enemy.get_level(),
                health: enemy.get_health(),
            };
            let edata = match which.unwrap() {
                 encounter::Attk(damage) => EncounterData {
                     enemy: emydata,
                     attk: Some(damage),
                     flee: None,
                     win: None,
                     lost: None
                 },
                encounter::Flee(b) => EncounterData {
                    enemy: emydata,
                    attk: None,
                    flee: Some(b),
                    win: None,
                    lost: None
                },
                encounter::Lost(_) => EncounterData {
                    enemy: emydata,
                    attk: None,
                    flee: None,
                    win: None,
                    lost: Some(true)
                },
                encounter::Win(loot) => {
                    let win = loot.unwrap();
                    let mut items = Vec::new();
                    let loot_i = win.get_items().unwrap();
                    for i in loot_i {
                        let iwhich = i.which();
                        if let Err(err) = iwhich {
                            return ServerEvent::new(None, None, None,
                                                    None, None, None,
                                                    None,
                                                    Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
                        }
                        let w = iwhich.unwrap();
                        let mut damage: Option<u32> = None;
                        let mut defense: Option<u32> = None;

                        match w {
                            packet_capnp::item::Which::Damage(i) => damage = Some(i),
                            packet_capnp::item::Which::Defense(i) => defense = Some(i),
                        }
                        items.push(ItemData {
                            name: i.get_name().unwrap().to_string(),
                            level: i.get_level(),
                            itype: i.get_itype(),
                            rarity: i.get_rarity(),
                            damage, defense
                        });
                    }
                    let loot_data = LootData {
                        items,
                        exp: win.get_exp(),
                    };
                    EncounterData {
                        enemy: emydata,
                        attk: None,
                        flee: None,
                        win: Some(loot_data),
                        lost: None
                    }
                }
            };
            ServerEvent::new(None, None, None,
                             None,
                             Some(edata), None,
                             None, None)
        }
        s_event::Inventory(inv_reader) => {
            let inv = inv_reader.unwrap();
            let mut items = Vec::new();
            for item in inv.into_iter() {
                let iwhich = item.which();
                if let Err(err) = iwhich {
                    return ServerEvent::new(None, None, None,
                                            None, None, None,
                                            None,
                                            Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
                }
                let w = iwhich.unwrap();
                let mut damage: Option<u32> = None;
                let mut defense: Option<u32> = None;

                match w {
                    packet_capnp::item::Which::Damage(i) => damage = Some(i),
                    packet_capnp::item::Which::Defense(i) => defense = Some(i),
                }

                items.push(ItemData {
                    name: item.get_name().unwrap().to_string(),
                    level: item.get_level(),
                    itype: item.get_itype(),
                    rarity: item.get_rarity(),
                    damage, defense
                });
            }

            ServerEvent::new(None, None, None,
                             None, None, Some(items),
                             None, None)
        }
        s_event::ItemView(item_reader) => {
            let raw_id = item_reader.unwrap();
            let id_r_which = raw_id.which();
            if let Err(err) = id_r_which {
                return ServerEvent::new(None, None, None,
                                        None, None, None,
                                        None,
                                        Some(ErrorData { msg: format!("Read invalid Server Event packet! Error: {}", err), disconnect: true }));
            }
            let mut defense: Option<u32> = None;
            let mut damage: Option<u32> = None;
            let w = id_r_which.unwrap();
            match w {
                packet_capnp::item::Which::Damage(i) => damage = Some(i),
                packet_capnp::item::Which::Defense(i) => defense = Some(i),
            }
            let item = ItemData {
                name: raw_id.get_name().unwrap().to_string(),
                level: raw_id.get_level(),
                itype: raw_id.get_itype(),
                rarity: raw_id.get_rarity(),
                defense, damage
            };

            ServerEvent::new(None, None, None,
                             None, None, None,
                             Some(item), None)
        }
        s_event::Error(err_reader) => {
            let err = err_reader.unwrap();
            ServerEvent::new(None, None, None,
                             None, None, None,
                             None, Some(ErrorData {
                    msg: err.get_error().unwrap().to_string(),
                    disconnect: err.get_disconnect()
                }))
        }
    }
}
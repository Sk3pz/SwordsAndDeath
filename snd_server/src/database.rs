use std::process::exit;
use std::str::FromStr;
use sqlite::{Connection, State};
use uuid::Uuid;
use crate::item::{Item, ItemRarity, ItemType};
use crate::player::Player;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum LoginFailReason {
    Unauthorized, Unrecognized, AlreadyOnline,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum PlayerValueDB {
    UUID, Username, Password,
    Level, Exp, Steps, Health,
    CurrentRegion, Active,
}

impl ToString for PlayerValueDB {
    fn to_string(&self) -> String {
        match self {
            Self::UUID => "owner",
            Self::Username => "username",
            Self::Password => "password",
            Self::Level => "level",
            Self::Exp => "exp",
            Self::Steps => "steps",
            Self::Health => "health",
            Self::CurrentRegion => "current_region",
            Self::Active => "active",
        }.to_string()
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ItemValueDB {
    UUID, Owner, Name, Type,
    Level, Damage, Defense,
    SpecialAbility, Rarity,
}

impl ToString for ItemValueDB {
    fn to_string(&self) -> String {
        match self {
            Self::UUID => "uuid",
            Self::Name => "name",
            Self::Owner => "owner",
            Self::Type => "type",
            Self::Level => "level",
            Self::Damage => "damage",
            Self::Defense => "defense",
            Self::SpecialAbility => "special_ability",
            Self::Rarity => "rarity",
        }.to_string()
    }
}

pub struct Database {
    pub connection: Connection
}

impl Database {
    pub fn new<S: Into<String>>(database_name: S) -> Self {
        let dn = database_name.into();
        let name = if dn.ends_with(".sqlite") {
            dn
        } else {
            format!("{}.sqlite", dn)
        };
        let connection_result = sqlite::open(name);
        if connection_result.is_err() {
            eprintln!("Failed to connect to database!");
            exit(1);
        }
        let connection = connection_result.unwrap();
        Self {
            connection
        }
    }

    pub fn get_value<S: Into<String>>(&self, select: S, table: S, key: S, key_value: S) -> Option<String> {
        let mut statement = self.connection
            .prepare(format!("SELECT {} FROM {} WHERE {} IS '{}'",
                             select.into(), table.into(), key.into(), key_value.into()))
            .expect("Failed to prepare statement for database interaction.");
        let state = statement.next();
        if state.is_err() {
            return None;
        }

        if state.unwrap() == State::Row {
            if let Ok(val) = statement.read::<String>(0) {
                return Some(val);
            }
        }

        None
    }

    pub fn set_value<S: Into<String>>(&self, update: S, set: S, value: S, key: S, where_key_is: S) -> bool {
        let r = self.connection.execute(format!("\
        UPDATE {}\
        SET {}='{}',\
        WHERE {}='{}'", update.into(), set.into(), value.into(), key.into(), where_key_is.into()));

        r.is_ok()
    }

    pub fn get_u32<S: Into<String>>(&self, select: S, from: S, key: S, where_key_is: S) -> Option<u32> {
        let value = self.get_value(select, from, key, where_key_is);
        if value.is_some() {
            if let Ok(val) = value.unwrap().parse::<u32>() {
                return Some(val);
            }
        }
        None
    }

    pub fn get_player_value(&self, uuid: &Uuid, val: PlayerValueDB) -> Option<String> {
        self.get_value(val.to_string(), format!("players"), format!("uuid"), uuid.to_string())
    }

    pub fn uuid_from_username(&self, username: String) -> Option<Uuid> {
        let v = self.get_value("uuid", "players", "username", username.as_str());
        return if let Some(s) = v {
            Some(Uuid::from_str(s.as_str()).expect(format!("Invalid UUID in database at username {}", username).as_str()))
        } else { None }
    }

    pub fn validate_login(&self, username: String, password: String) -> Result<Uuid, LoginFailReason> {
        let attempt_uuid = self.uuid_from_username(username);
        if attempt_uuid.is_none() {
            return Err(LoginFailReason::Unrecognized);
        }
        let uuid = attempt_uuid.unwrap();

        if let Some(active) = self.get_player_value(&uuid, PlayerValueDB::Active) {
            if active.as_str() == "1" {
                return Err(LoginFailReason::AlreadyOnline);
            }
        }

        let found_pass = self.get_player_value(&uuid, PlayerValueDB::Password);
        if let Some(p) = found_pass {
            if p == password {
                return Ok(uuid);
            }
        } else {
            return Err(LoginFailReason::Unrecognized);
        }

        Err(LoginFailReason::Unauthorized)
    }

    pub fn player_exists(&self, username: String) -> bool {
        self.uuid_from_username(username).is_some()
    }

    pub fn get_player_items(&self, owner_uuid: &Uuid) -> Option<Vec<Item>> {
        let stmt = format!("SELECT * FROM items WHERE owner IS '{}'", owner_uuid.to_string());
        let mut items = Vec::new();

        let r = self.connection.iterate(stmt, |pairs| {
            let mut uuid = Uuid::new_v4();
            let mut item_type = ItemType::Sword;
            let mut rarity = ItemRarity::Common;
            let mut level = 0;
            let mut damage = 9999;
            let mut defense = 9999;
            let mut name = format!("Glitched Sword");
            for (col, val) in pairs {
                if val.is_some() {
                    let v = val.unwrap();
                    match *col {
                        "uuid" => uuid = Uuid::from_str(v)
                            .expect(format!("Failed to get uuid from item - invalid uuid: {}", v).as_str()),
                        "type" => item_type = ItemType::from(v.parse::<u32>()
                            .expect(format!("Invalid type value in database in item owned by {}: '{}' should be integer",
                                            uuid.to_string(), v).as_str())),
                        "name" => name = v.to_string(),
                        "level" => level = v.parse::<u32>()
                            .expect(format!("Invalid level value in database in item owned by {}: '{}' should be integer",
                                            uuid.to_string(), v).as_str()),
                        "damage" => damage = v.parse::<u32>()
                            .expect(format!("Invalid damage value in database in item owned by {}: '{}' should be integer",
                                            uuid.to_string(), v).as_str()),
                        "defense" => defense = v.parse::<u32>()
                            .expect(format!("Invalid defense value in database in item owned by {}: '{}' should be integer",
                                            uuid.to_string(), v).as_str()),
                        "rarity" => rarity = ItemRarity::from(v.parse::<u32>()
                            .expect(format!("Invalid rarity value in database in item owned by {}: '{}' should be integer",
                                            uuid.to_string(), v).as_str())),
                        _ => {}
                    }
                }
            }
            items.push(Item {
                uuid,
                owner: uuid.clone(),
                name, item_type, rarity,
                level, damage, defense,
            });
            true
        });

        return if r.is_ok() {
            Some(items)
        } else {
            None
        }
    }

    pub fn new_player(&self, player: &Player, password: String) -> bool {
        let r = self.connection.execute(
            format!("INSERT INTO players VALUES ('{}','{}','{}','{}','{}','{}','{}','{}','{}')",
                    player.uuid, player.name, password, 1, 0, 0, 100, "Plains of Arenlok", 0));

        r.is_ok()
    }

    pub fn set_player_active(&self, uuid: &Uuid) -> bool {
        self.set_value("players", "active", "1", "uuid", uuid.to_string().as_str())
    }

    pub fn set_player_inactive(&self, uuid: &Uuid) -> bool {
        self.set_value("players", "active", "0", "uuid", uuid.to_string().as_str())
    }

    pub fn is_player_active(&self, uuid: &Uuid) -> bool {
        self.get_player_value(uuid, PlayerValueDB::Active).unwrap_or("0".to_string()) == "1"
    }

    pub fn new_item(&self, item: &Item) -> bool {
        let r = self.connection.execute(
            format!("INSERT INTO items VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')",
                    item.owner.to_string(), item.item_type as u32, item.level, item.damage, item.defense, "NONE", item.name, item.uuid, item.rarity as u32));

        r.is_ok()
    }

    pub fn item_uuid_from_name(&self, name: String) -> Option<Uuid> {
        let v = self.get_value("uuid", "items", "name", name.as_str());
        return if let Some(s) = v {
            Some(Uuid::from_str(s.as_str()).expect(format!("Invalid UUID in database at item name {}", name).as_str()))
        } else { None }
    }

    pub fn update_item(&self, item: &Item) -> bool {
        let r = self.connection.execute(format!("\
        UPDATE items\
        SET owner='{}',\
            name='{}'\
            type='{}'\
            level='{}'\
            damage='{}'\
            defense='{}'\
            rarity='{}'\
        WHERE uuid='{}'", item.owner, item.name, item.item_type as u32, item.level, item.damage, item.defense, item.rarity as u32, item.uuid));

        r.is_ok()
    }

    pub fn set_item_value(&self, item: &Item, value: ItemValueDB) -> bool {
        self.set_value("items", value.to_string().as_str(),
                       item.get_value_from_ivdb(value).as_str(), "uuid", item.uuid.to_string().as_str())
    }
}
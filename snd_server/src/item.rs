use rand::{Rng, thread_rng};
use rand_distr::{Normal, Distribution};
use uuid::Uuid;
use crate::database::ItemValueDB;
use snd_network_lib::item_data::ItemData;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ItemType {
    Sword,
    Shield,
    Helmet, Chestplate, Leggings, Boots,
}

impl ItemType {
    pub fn rand() -> Self {
        Self::from(thread_rng().gen_range(0..=5))
    }
}

impl Into<u32> for ItemType {
    fn into(self) -> u32 {
        match self {
            Self::Sword      => 0,
            Self::Shield     => 1,
            Self::Helmet     => 2,
            Self::Chestplate => 3,
            Self::Leggings   => 4,
            Self::Boots      => 5,
        }
    }
}

impl From<u32> for ItemType {
    fn from(x: u32) -> Self {
        match x {
            1 => Self::Shield,
            2 => Self::Helmet,
            3 => Self::Chestplate,
            4 => Self::Leggings,
            5 => Self::Boots,
            _ => Self::Sword,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ItemRarity {
    Common, Rare, Epic, Legendary
}

impl ItemRarity {
    pub fn new_rand() -> Self {
        let rng = thread_rng().gen_range(0..100);
        match rng {
            _ if rng < 20 => Self::Rare,      // 30% chance
            _ if rng < 35 => Self::Epic,      // 15% chance
            100           => Self::Legendary, // 1% chance
            _             => Self::Common     // 55% chance
        }
    }

    pub fn get_multiplier(&self) -> u32 {
        match self {
            Self::Common => 1,
            Self::Rare => 2,
            Self::Epic => 5,
            Self::Legendary => 10,
        }
    }

    pub fn get_weight(&self, level: u32) -> u32 {
        self.get_multiplier() * (level / ((level / 2).max(1)))
    }
}

impl Into<u32> for ItemRarity {
    fn into(self) -> u32 {
        match self {
            Self::Common    => 0,
            Self::Rare      => 1,
            Self::Epic      => 2,
            Self::Legendary => 3,
        }
    }
}

impl From<u32> for ItemRarity {
    fn from(x: u32) -> Self {
        match x {
            1 => Self::Rare,
            2 => Self::Epic,
            3 => Self::Legendary,
            _ => Self::Common
        }
    }
}

#[derive(Clone)]
pub struct Item {
    pub uuid: Uuid,
    pub owner: Uuid,
    pub name: String,
    pub item_type: ItemType,
    pub rarity: ItemRarity,
    pub level: u32,
    pub damage: u32,
    pub defense: u32,
}

impl Item {

    pub fn new_rand(item_type: ItemType, owner: &Uuid, around_level: u32, rarity: ItemRarity) -> Self {
        let uuid = Uuid::new_v4();
        // generate a random item name (based on type and possibly level / rarity?)
        let name = format!("NO_NAME{}", uuid.to_string()); // todo(eric): Name Generator (NG)

        // generate the item's level
        let normal = Normal::new(around_level as f32, 5.5)
            .expect("Failed to create Normal Distribution for item generation.");
        let level = normal.sample(&mut thread_rng())
            .round().max(1.0) as u32;

        // set defaults
        let mut damage: u32 = 0;
        let mut defense: u32 = 0;

        // generate defense or damage value depending on item type
        let val_norm = Normal::new(
            rarity.get_weight(level.clone()) as f32, 2.2
        ).expect("Failed to create Normal Distribution for item weight generation.");
        let weighted_value = val_norm.sample(&mut thread_rng())
            .round().max(1.0) as u32;

        match item_type {
            ItemType::Sword => damage = weighted_value,
            _ => defense = weighted_value,
        }

        Self {
            uuid,
            owner: owner.clone(),
            name, item_type, rarity, level, damage, defense,
        }
    }

    pub fn get_value_from_ivdb(&self, ivdb: ItemValueDB) -> String {
        match ivdb {
            ItemValueDB::UUID => self.uuid.to_string(),
            ItemValueDB::Name => self.name.clone(),
            ItemValueDB::Owner => self.owner.to_string(),
            ItemValueDB::Type => (self.item_type.clone() as u32).to_string(),
            ItemValueDB::Level => self.level.to_string(),
            ItemValueDB::Damage => self.damage.to_string(),
            ItemValueDB::Defense => self.defense.to_string(),
            ItemValueDB::SpecialAbility => "NONE".to_string(),
            ItemValueDB::Rarity => (self.rarity.clone() as u32).to_string(),
        }
    }

    pub fn as_data(&self) -> ItemData {
        ItemData {
            name: self.name.clone(),
            level: self.level,
            itype: self.item_type as u32,
            rarity: self.rarity as u32,
            damage: Some(self.damage),
            defense: Some(self.defense)
        }
    }
}
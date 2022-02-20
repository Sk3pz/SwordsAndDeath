use uuid::Uuid;
use crate::database::ItemValueDB;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ItemType {
    Sword,
    Shield,
    Helmet, Chestplate, Leggings, Boots,
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

    pub fn new_rand(target_level: u32) -> Self {
        todo!()
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

}
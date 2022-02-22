use crate::network::enemy_data::EnemyData;
use crate::network::loot_data::LootData;

#[derive(Clone, Debug)]
pub struct EncounterData {
    pub enemy: EnemyData,
    pub attk: Option<u32>,
    pub flee: Option<bool>,
    pub win: Option<LootData>,
    pub lost: Option<bool>,
}
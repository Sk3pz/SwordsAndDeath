use crate::network::item_data::ItemData;

#[derive(Clone, Debug)]
pub struct LootData {
    pub items: Vec<ItemData>,
    pub exp: u32,
}
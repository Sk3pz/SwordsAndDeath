
#[derive(Clone, Debug)]
pub struct ItemData {
    pub name: String,
    pub level: u32,
    pub itype: u32,
    pub rarity: u32,
    pub damage: Option<u32>,
    pub defense: Option<u32>,
}
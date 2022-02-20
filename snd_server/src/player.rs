use uuid::Uuid;

#[derive(Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct LoginData {
    pub username: String,
    pub passwd: String,
    pub signup: bool,
    pub client_ver: String,
}
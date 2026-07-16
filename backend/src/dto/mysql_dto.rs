use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChangeRootPw {
    pub password: String,
}

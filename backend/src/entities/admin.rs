use serde::Deserialize;

#[derive(Clone)]
pub struct Admin {
    pub login: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

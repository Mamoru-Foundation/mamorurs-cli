use serde::{Deserialize, Serialize};

pub mod get_token;
pub mod jwtverifier;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub expires_in: Option<usize>,
    pub token_type: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub iat: usize,
    pub exp: usize,
    pub azp: String,
    pub scope: String,
}

use crate::guards::emailer::SmtpConfig;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use jsonwebtoken::{DecodingKey, EncodingKey};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RetronomiconConfig {
    pub base_url: String,
    pub root_team: Vec<String>,
    pub root_team_id: i32,

    pub smtp: SmtpConfig,
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn from_base64(secret: &str) -> Self {
        let secret = STANDARD.decode(secret).expect("Invalid base64 JWT secret");
        let encoding = EncodingKey::from_secret(&secret);
        let decoding = DecodingKey::from_secret(&secret);
        Self { encoding, decoding }
    }
}

pub struct DbPepper(pub Vec<u8>);

impl DbPepper {
    pub fn from_base64(secret: &str) -> Self {
        let secret = STANDARD.decode(secret).expect("Invalid base64 pepper");
        Self(secret)
    }
}

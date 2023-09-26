pub mod error;
pub mod params;

pub mod teams;
pub mod user;

/// The expected response of an end point that does not return anything.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ok;

/// The auth token response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthTokenResponse {
    pub token: String,
}

pub use error::JsonError;

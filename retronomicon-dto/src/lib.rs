pub mod encodings;
pub mod error;
pub mod params;
pub mod types;

pub mod artifact;
pub mod cores;
pub mod games;
pub mod platforms;
pub mod systems;
pub mod tags;
pub mod teams;
pub mod user;

pub mod client;
pub use client::routes;

/// The expected response of an end point that does not return anything.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Ok;

/// A JWT authentication token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct AuthTokenResponse {
    /// The token itself.
    pub token: String,
}

pub use error::JsonError;

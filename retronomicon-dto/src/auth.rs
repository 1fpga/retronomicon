use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SignupRequest<'a> {
    /// An optional username. If provided, it must be unique and will
    /// be validated before creating the user and sending the email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<&'a str>,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SignupResponse {
    pub id: i32,
    pub email: String,
}

/// A login request with an email and password.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct LoginRequest<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

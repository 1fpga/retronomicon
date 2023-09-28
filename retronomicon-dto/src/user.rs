use crate::teams::TeamRef;
use crate::types::UserTeamRole;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// A user ID can be either an User ID (as an integer) or a username string.
#[derive(Debug, Hash, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum UserIdOrUsername<'v> {
    Id(i32),
    Username(&'v str),
}

impl<'v> UserIdOrUsername<'v> {
    pub fn as_id(&self) -> Option<i32> {
        match self {
            UserIdOrUsername::Id(id) => Some(*id),
            _ => None,
        }
    }
    pub fn as_username(&self) -> Option<&str> {
        match self {
            UserIdOrUsername::Username(name) => Some(name),
            _ => None,
        }
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::request::FromParam<'v> for UserIdOrUsername<'v> {
    type Error = std::convert::Infallible;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        match param.parse::<i32>() {
            Ok(id) => Ok(UserIdOrUsername::Id(id)),
            Err(_) => Ok(UserIdOrUsername::Username(param)),
        }
    }
}

/// Parameters for updating a user.
#[derive(Default, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct UserUpdate<'a> {
    pub username: Option<&'a str>,
    pub display_name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub links: Option<BTreeMap<&'a str, &'a str>>,
    pub metadata: Option<BTreeMap<&'a str, Value>>,
    pub add_links: Option<BTreeMap<&'a str, &'a str>>,
    pub remove_links: Option<Vec<&'a str>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct UserRef {
    pub id: i32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct UserDetailsInner {
    pub id: i32,
    pub username: String,
    pub description: String,
    pub links: Value,
    pub metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct UserTeamRef {
    #[serde(flatten)]
    pub team: TeamRef,
    pub role: UserTeamRole,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct UserDetails {
    #[serde(flatten)]
    pub user: UserDetailsInner,

    pub teams: Vec<UserTeamRef>,
}

/// A User information.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct User {
    /// The user id.
    pub id: i32,
    /// The username. If missing, the user has no username and must
    /// set its username before being able to make changes.
    pub username: Option<String>,

    /// The user's avatar.
    #[serde(skip_serializing)]
    pub avatar_url: Option<String>,

    /// The user's display name.
    #[serde(skip_serializing)]
    pub display_name: Option<String>,
}

pub type Me = UserDetails;

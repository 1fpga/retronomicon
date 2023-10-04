use crate::teams::TeamRef;
use crate::types::UserTeamRole;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// A valid username (not empty, not too long, no special characters).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Username<'v>(&'v str);

impl<'v> Username<'v> {
    pub fn new(username: &'v str) -> Result<Self, &'static str> {
        if username.len() < 2 {
            return Err("Username cannot be less than 2 characters");
        }
        if username.len() > 32 {
            return Err("Username is too long");
        }

        // We know username isn't less than 2 characters.
        if !username.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Err("Username must start with a letter");
        }

        // Validate against the regex `^[a-z_]([a-z0-9_.-]*[a-z0-9_])?$`
        for ch in username[1..username.len() - 1].chars() {
            if !matches!(ch, 'a'..='z' | '0'..='9' | '_' | '.' | '-') {
                return Err("Username must contain only lowercase letters, numbers, underscores, dots and dashes");
            }
        }
        if !username.ends_with(|c| matches!(c, 'a'..='z' | '0'..='9' | '_')) {
            return Err("Username must end with a lowercase letter, number or underscore");
        }

        Ok(Self(username))
    }

    pub fn into_inner(self) -> &'v str {
        self.0
    }
}

impl<'v> TryInto<Username<'v>> for &'v str {
    type Error = &'static str;

    fn try_into(self) -> Result<Username<'v>, Self::Error> {
        Username::new(self)
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::request::FromParam<'v> for Username<'v> {
    type Error = &'static str;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        Username::new(param)
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::form::FromFormField<'v> for Username<'v> {
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        Self::new(field.value)
            .map_err(|_| rocket::form::Error::validation("Invalid username").into())
    }
}

/// A user ID can be either an User ID (as an integer) or a username string.
#[derive(Copy, Clone, Debug, Hash, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum UserIdOrUsername<'v> {
    Id(i32),
    #[serde(borrow)]
    Username(Username<'v>),
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
            UserIdOrUsername::Username(Username(name)) => Some(*name),
            _ => None,
        }
    }
}

impl From<i32> for UserIdOrUsername<'static> {
    fn from(value: i32) -> Self {
        Self::Id(value)
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::request::FromParam<'v> for UserIdOrUsername<'v> {
    type Error = &'static str;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        match param.parse::<i32>() {
            Ok(id) => Ok(UserIdOrUsername::Id(id)),
            Err(_) => Username::new(param).map(UserIdOrUsername::Username),
        }
    }
}

/// Response when asking for the availability of a username.
#[derive(Default, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct UserCheckResponse {
    pub username: String,
    pub available: bool,
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

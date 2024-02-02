use crate::teams::TeamRef;
use crate::types::UserTeamRole;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A valid username (not empty, not too long, no special characters).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Username<'v>(Cow<'v, str>);

impl<'v> Username<'v> {
    pub fn new(username: impl Into<Cow<'v, str>>) -> Result<Self, &'static str> {
        let username = username.into();
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
        if username[1..username.len() - 1]
            .chars()
            .any(|ch| !matches!(ch, 'a'..='z' | '0'..='9' | '_' | '.' | '-'))
        {
            return Err("Username must contain only lowercase letters, numbers, underscores, dots and dashes");
        }
        if !username.ends_with(|c| matches!(c, 'a'..='z' | '0'..='9' | '_')) {
            return Err("Username must end with a lowercase letter, number or underscore");
        }

        Ok(Self(username))
    }

    pub fn into_inner(self) -> Cow<'v, str> {
        self.0
    }
}

impl<'v> TryInto<Username<'v>> for &'v str {
    type Error = &'static str;

    fn try_into(self) -> Result<Username<'v>, Self::Error> {
        Username::new(self)
    }
}

impl FromStr for Username<'static> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Cow::Owned(s.to_owned())))
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
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum UserIdOrUsername<'v> {
    Id(i32),
    #[serde(borrow)]
    Username(Username<'v>),
}

impl<'v> Display for UserIdOrUsername<'v> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UserIdOrUsername::Id(id) => write!(f, "{}", *id),
            UserIdOrUsername::Username(Username(name)) => f.write_str(name),
        }
    }
}

impl FromStr for UserIdOrUsername<'static> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.parse::<i32>() {
            Ok(id) => Self::Id(id),
            Err(_) => Self::Username(Username::new(s.to_owned())?),
        })
    }
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
            UserIdOrUsername::Username(Username(name)) => Some(name.as_ref()),
            _ => None,
        }
    }
}

impl<'v> From<Username<'v>> for UserIdOrUsername<'v> {
    fn from(value: Username<'v>) -> Self {
        Self::Username(value)
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

#[cfg(feature = "rocket")]
impl<'v, T: rocket::http::uri::fmt::Part> rocket::http::uri::fmt::UriDisplay<T>
    for UserIdOrUsername<'v>
{
    fn fmt(&self, f: &mut rocket::http::uri::fmt::Formatter<'_, T>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_str(&self.to_string())
    }
}

#[cfg(feature = "rocket")]
impl<'v, T: rocket::http::uri::fmt::Part>
    rocket::http::uri::fmt::FromUriParam<T, UserIdOrUsername<'v>> for UserIdOrUsername<'v>
{
    type Target = UserIdOrUsername<'v>;

    fn from_uri_param(param: UserIdOrUsername<'v>) -> Self::Target {
        param
    }
}

#[cfg(feature = "rocket")]
impl<'v, T: rocket::http::uri::fmt::Part> rocket::http::uri::fmt::FromUriParam<T, &'v str>
    for UserIdOrUsername<'static>
{
    type Target = UserIdOrUsername<'static>;

    fn from_uri_param(param: &'v str) -> Self::Target {
        Self::from_str(param).expect("Invalid Username")
    }
}

#[cfg(feature = "rocket")]
impl<T: rocket::http::uri::fmt::Part> rocket::http::uri::fmt::FromUriParam<T, i32>
    for UserIdOrUsername<'static>
{
    type Target = UserIdOrUsername<'static>;

    fn from_uri_param(param: i32) -> Self::Target {
        UserIdOrUsername::Id(param)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
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

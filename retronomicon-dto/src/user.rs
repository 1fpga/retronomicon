use crate::teams::TeamRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserId<'v> {
    Id(i32),
    Username(&'v str),
}

impl<'v> UserId<'v> {
    pub fn as_id(&self) -> Option<i32> {
        match self {
            UserId::Id(id) => Some(*id),
            _ => None,
        }
    }
    pub fn as_username(&self) -> Option<&str> {
        match self {
            UserId::Username(name) => Some(name),
            _ => None,
        }
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::request::FromParam<'v> for UserId<'v> {
    type Error = std::convert::Infallible;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        match param.parse::<i32>() {
            Ok(id) => Ok(UserId::Id(id)),
            Err(_) => Ok(UserId::Username(param)),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserUpdate<'a> {
    pub username: Option<&'a str>,
    pub display_name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub links: Option<BTreeMap<&'a str, &'a str>>,
    pub metadata: Option<BTreeMap<&'a str, Value>>,

    #[serde(default)]
    pub add_links: BTreeMap<&'a str, &'a str>,
    #[serde(default)]
    pub remove_links: Vec<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRef {
    pub id: i32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailsInner {
    pub id: i32,
    pub username: String,
    pub description: String,
    pub links: Value,
    pub metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetails {
    #[serde(flatten)]
    pub user: UserDetailsInner,

    pub teams: Vec<TeamRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: Option<String>,

    #[serde(skip_serializing)]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub display_name: Option<String>,
}

pub type Me = UserDetails;

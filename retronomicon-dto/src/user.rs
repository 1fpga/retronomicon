use crate::details::GroupRef;
use rocket::serde::json::Value;
use rocket::serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

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
    pub links: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetails {
    #[serde(flatten)]
    pub user: UserDetailsInner,

    pub groups: Vec<GroupRef>,
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

use crate::user::UserRef;
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupRef {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDetails {
    #[serde(flatten)]
    pub group: GroupRef,

    pub description: String,
    pub links: Option<Value>,

    pub users: Vec<UserRef>,
}

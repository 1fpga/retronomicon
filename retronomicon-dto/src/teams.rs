use crate::user::{UserId, UserRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamRef {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamDetails {
    #[serde(flatten)]
    pub team: TeamRef,

    pub description: String,
    pub links: Option<Value>,

    pub users: Vec<UserRef>,
}

#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, strum::EnumString, strum::Display,
)]
pub enum UserTeamRole {
    Owner,
    Admin,
    #[default]
    Member,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInvite<'a> {
    #[serde(borrow)]
    pub user: UserId<'a>,

    #[serde(default)]
    pub role: UserTeamRole,
}

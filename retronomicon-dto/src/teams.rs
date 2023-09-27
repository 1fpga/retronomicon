use crate::user::{UserIdOrUsername, UserRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamRef {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Team {
    #[serde(flatten)]
    pub team: TeamRef,

    pub description: String,
    pub links: Value,
    pub metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamDetails {
    #[serde(flatten)]
    pub team: TeamRef,

    pub description: String,
    pub links: Value,
    pub metadata: Value,

    pub users: Vec<UserRef>,
}

#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, strum::EnumString, strum::Display,
)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum UserTeamRole {
    Owner,
    Admin,
    #[default]
    Member,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamInvite<'a> {
    #[serde(borrow)]
    pub user: UserIdOrUsername<'a>,

    #[serde(default)]
    pub role: UserTeamRole,
}

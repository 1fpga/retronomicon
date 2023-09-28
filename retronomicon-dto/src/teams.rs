use crate::types::UserTeamRole;
use crate::user::{UserIdOrUsername, UserRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
pub struct TeamUserRef {
    #[serde(flatten)]
    pub user: UserRef,
    pub role: UserTeamRole,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamDetails {
    #[serde(flatten)]
    pub team: TeamRef,

    pub description: String,
    pub links: BTreeMap<String, String>,
    pub metadata: BTreeMap<String, Value>,

    pub users: Vec<TeamUserRef>,
}

/// Arguments to create a team.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamCreateRequest<'a> {
    /// A slug for the team.
    pub slug: &'a str,
    /// A name for the team.
    pub name: &'a str,
    /// The description of the team.
    pub description: &'a str,

    /// Links to the team's various aspects.
    pub links: Option<BTreeMap<&'a str, &'a str>>,

    /// Generic metadata associated with the team.
    pub metadata: Option<BTreeMap<&'a str, Value>>,
}

/// Response when creating a team.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamCreateResponse {
    pub id: i32,
    pub slug: String,
}

/// Arguments to create a team.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamUpdateRequest<'a> {
    /// A slug for the team.
    pub slug: Option<&'a str>,
    /// A name for the team.
    pub name: Option<&'a str>,
    /// The description of the team.
    pub description: Option<&'a str>,

    /// Replace all links in the team.
    pub links: Option<BTreeMap<&'a str, &'a str>>,

    /// Replace all metadata associated with the team.
    pub metadata: Option<BTreeMap<&'a str, Value>>,

    /// Add new links to the list. If the `links` key is also passed,
    /// this is ignored.
    pub add_links: Option<BTreeMap<&'a str, &'a str>>,

    /// Remove links from the list.
    pub remove_links: Option<Vec<&'a str>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TeamInvite<'a> {
    #[serde(borrow)]
    pub user: UserIdOrUsername<'a>,

    #[serde(default)]
    pub role: UserTeamRole,
}

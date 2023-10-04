use crate::systems::SystemRef;
use crate::teams::TeamRef;
use crate::types::IdOrSlug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub mod releases;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreRef {
    pub id: i32,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreListItem {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub owner_team: TeamRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreCreateRequest<'v> {
    pub slug: &'v str,
    pub name: &'v str,
    pub description: &'v str,
    pub links: BTreeMap<&'v str, &'v str>,
    pub metadata: BTreeMap<&'v str, Value>,
    pub system: IdOrSlug<'v>,
    pub owner_team: IdOrSlug<'v>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreCreateResponse {
    pub id: i32,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreDetailsResponse {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub links: BTreeMap<String, String>,
    pub metadata: BTreeMap<String, Value>,
    pub system: SystemRef,
    pub owner_team: TeamRef,
}

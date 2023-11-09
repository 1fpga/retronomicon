use crate::teams::TeamRef;
use crate::types::IdOrSlug;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SystemRef {
    pub id: i32,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SystemListItem {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub manufacturer: String,
    pub owner_team: TeamRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SystemDetails {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub manufacturer: String,
    pub links: BTreeMap<String, String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
    pub owner_team: TeamRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SystemCreateRequest<'a> {
    pub slug: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub manufacturer: &'a str,
    pub links: Option<BTreeMap<&'a str, &'a str>>,
    pub metadata: Option<BTreeMap<&'a str, serde_json::Value>>,
    pub owner_team: IdOrSlug<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct SystemCreateResponse {
    pub id: i32,
    pub slug: String,
}

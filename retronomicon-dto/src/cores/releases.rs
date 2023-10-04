use crate::cores::CoreRef;
use crate::platforms::PlatformRef;
use crate::types::IdOrSlug;
use crate::user::UserRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseListItem {
    pub id: i32,
    pub version: String,

    /// Whether this release is a prerelease. Prereleases are not shown by default.
    pub prerelease: bool,

    /// Whether this release was yanked. Yanked releases are not shown by default.
    pub yanked: bool,

    /// Date the release was uploaded to the server, in seconds since UNIX EPOCH.
    pub date_released: i64,

    pub core: CoreRef,
    pub platform: PlatformRef,
    pub uploader: UserRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseCreateRequest<'v> {
    pub version: &'v str,
    pub notes: &'v str,
    pub date_released: Option<i64>,
    pub prerelease: bool,
    pub links: BTreeMap<&'v str, &'v str>,
    pub metadata: BTreeMap<&'v str, Value>,

    #[serde(borrow)]
    pub platform: IdOrSlug<'v>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseCreateResponse {
    pub id: i32,
}

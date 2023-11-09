use crate::cores::CoreRef;
use crate::platforms::PlatformRef;
use crate::types::IdOrSlug;
use crate::user::UserRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Parameters for filtering a list of core releases.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseFilterParams<'v> {
    /// Whether to include prereleases in the results. Defaults to false.
    pub prerelease: Option<bool>,

    /// Whether to include yanked releases in the results. Defaults to false.
    pub yanked: Option<bool>,

    /// Minimum date to include in the results, in seconds since UNIX EPOCH.
    /// Defaults to 0 (all releases).
    pub min_release_date: Option<i64>,

    /// Maximum date to include in the results, in seconds since UNIX EPOCH.
    /// Defaults to i64::MAX (all releases).
    pub max_release_date: Option<i64>,

    /// Filter releases by platform. By default, include all platforms.
    #[serde(borrow)]
    pub platform: Option<IdOrSlug<'v>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseRef {
    pub id: i32,
    pub version: String,

    /// Whether this release is a prerelease. Prereleases are not shown by default.
    pub prerelease: bool,

    /// Whether this release was yanked. Yanked releases are not shown by default.
    pub yanked: bool,

    /// Date the release was uploaded to the server, in seconds since UNIX EPOCH.
    pub date_released: i64,

    /// Which platform was this release made for.
    pub platform: PlatformRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseListItem {
    #[serde(flatten)]
    pub release: CoreReleaseRef,
    pub core: CoreRef,
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

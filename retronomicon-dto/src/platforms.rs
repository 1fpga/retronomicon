use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Platform {
    pub id: i32,
    pub slug: String,

    pub name: String,
    pub description: String,
    pub links: Value,
    pub metadata: Value,
}

/// Parameters for creating a new platform.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct PlatformCreateRequest<'v> {
    /// A slug for the platform. Must be unique to all platforms.
    pub slug: &'v str,

    /// The human-readable name of the platform.
    pub name: &'v str,

    /// A description of the platform.
    pub description: &'v str,

    /// Links to the platform's website, documentation, etc.
    pub links: Option<Value>,

    /// Metadata for the platform. No schema is enforced.
    pub metadata: Option<Value>,

    /// The team id who will own the platform. The user must be a member of the
    /// team.
    pub team_id: i32,
}

/// Parameters for creating a new platform.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct PlatformCreateResponse {
    /// The ID of the new platform created.
    pub id: i32,

    /// The slug of the platform.
    pub slug: String,
}

/// Parameters for updating a platform's information.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct PlatformUpdateRequest<'v> {
    /// A slug for the platform. Must be unique to all platforms.
    pub slug: Option<&'v str>,

    /// The human-readable name of the platform.
    pub name: Option<&'v str>,

    /// A description of the platform.
    pub description: Option<&'v str>,

    /// Links to the platform's website, documentation, etc.
    pub links: Option<Value>,

    /// Metadata for the platform. No schema is enforced.
    pub metadata: Option<Value>,

    /// The team id who will own the platform. The user must be a member of the
    /// team.
    pub team_id: Option<i32>,
}

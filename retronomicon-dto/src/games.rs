use crate::artifact::ArtifactRef;
use crate::encodings::HexString;
use crate::params::{PagingParams, RangeParams};
use crate::systems::SystemRef;
use crate::types::IdOrSlug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Parameters for filtering the list of games.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameListQueryParams<'v> {
    /// Filter games by system. By default, include all systems.
    #[serde(borrow)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<IdOrSlug<'v>>,

    /// Filter by year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<RangeParams<i32>>,

    /// Paging parameters.
    #[serde(flatten)]
    pub paging: PagingParams,

    /// Filter by name, exact substring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Parameters for filtering the list of games using checksums.
#[derive(Default, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameListBody {
    /// Filter by md5 checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<Vec<HexString>>,

    /// Filter by sha1 checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<Vec<HexString>>,

    /// Filter by sha256 checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<Vec<HexString>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameListItemResponse {
    /// The identifier for the game, this is unique for ALL games.
    pub id: i32,

    /// The name of the game. Does not include any `[]` tags.
    pub name: String,

    /// A short description of the game.
    pub short_description: String,

    /// The year the game was released.
    pub year: i32,

    /// The system this game is for.
    pub system_id: SystemRef,

    /// The identifier for the game, in this system. This is unique
    /// for all games in this system.
    pub system_unique_id: i32,

    /// The checksums and sizes of all artifacts the game.
    pub artifacts: Vec<ArtifactRef>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
pub struct GameCreateRequest<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub short_description: &'a str,
    pub year: i32,
    pub publisher: &'a str,
    pub developer: &'a str,
    pub links: BTreeMap<&'a str, &'a str>,
    pub system: IdOrSlug<'a>,
    pub system_unique_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameCreateResponse {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameDetails {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub short_description: String,
    pub year: i32,
    pub publisher: String,
    pub developer: String,
    pub links: Value,
    pub system_unique_id: i32,
    pub system: SystemRef,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameUpdateRequest<'a> {
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub short_description: Option<&'a str>,
    pub year: Option<i32>,
    pub publisher: Option<&'a str>,
    pub developer: Option<&'a str>,
    pub add_links: Option<BTreeMap<&'a str, &'a str>>,
    pub remove_links: Option<Vec<&'a str>>,
    pub system_unique_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameAddArtifactRequest<'a> {
    /// Its content type.
    pub mime_type: &'a str,

    /// Size of the file in bytes. Files cannot be larger than 20MB.
    pub size: i32,

    /// MD5 checksum of the file, in hexadecimal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<HexString>,

    /// SHA1 checksum of the file, in hexadecimal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<HexString>,

    /// SHA256 checksum of the file, in hexadecimal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<HexString>,
}

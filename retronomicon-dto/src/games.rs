use crate::params::{PagingParams, RangeParams};
use crate::systems::SystemRef;
use crate::types::IdOrSlug;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Parameters for filtering the list of games.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct GameListQueryParams<'v> {
    /// Filter games by system. By default, include all systems.
    #[serde(borrow)]
    pub system: Option<IdOrSlug<'v>>,

    /// Filter by year.
    pub year: Option<RangeParams<i32>>,

    /// Paging parameters.
    #[serde(flatten)]
    pub paging: PagingParams,

    /// Filter by name, exact substring.
    pub name: Option<String>,
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

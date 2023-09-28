use crate::teams::TeamRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreListItem {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub owner_team: TeamRef,
}

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Tag {
    pub id: i32,
    /// The slug of the tag.
    pub slug: String,
    /// An RGB color. The top 8 bits are ignored.
    pub color: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct TagCreate {
    pub slug: String,
    pub description: String,
    /// An RGB color. The top 8 bits are ignored.
    pub color: u32,
}

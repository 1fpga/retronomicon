#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct Image {
    /// The image's name.
    pub name: String,
    /// The image's content type.
    pub mime_type: String,
    /// The image's URL to download.
    pub url: String,
}

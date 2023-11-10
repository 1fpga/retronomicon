use serde::{Deserialize, Serialize};

/// Checksum of an artifact. There needs to be at least one checksum.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct ArtifactChecksum<'v> {
    /// Optional URL containing the data.
    pub download_url: Option<&'v str>,

    /// Size of the file in bytes. Files cannot be larger than 20MB.
    pub size: i32,

    /// MD5 checksum of the file, in hexadecimal.
    pub md5: Option<&'v str>,

    /// SHA256 checksum of the file, in hexadecimal.
    pub sha256: Option<&'v str>,
}

/// The content being uploaded. Either a file, or the checksums of a file
/// to be validated against the download.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum ArtifactData<'v> {
    /// Base64 encoded data of the file. Checksums will be generated automatically.
    /// Files cannot be larger than 20MB.
    Data(&'v str),

    /// Checksums of the data, with the data available somewhere else.
    Checksums(ArtifactChecksum<'v>),
}

/// The result of creating a new artifact.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct ArtifactCreateResponse {
    /// The ID of the artifact.
    pub id: i32,

    /// A URL to download it if there was an uploaded file.
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseArtifactListItem {
    pub id: i32,
    pub filename: String,
    pub mime_type: String,
    pub created_at: i64,
    pub md5: String,
    pub sha256: String,
    pub size: i32,
    pub download_url: Option<String>,
}

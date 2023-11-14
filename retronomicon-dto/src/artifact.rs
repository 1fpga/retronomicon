use crate::encodings::{Base64String, HexString};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct ArtifactRef {
    /// Optional URL to download this artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,

    pub size: Option<NonZeroU32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<HexString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<HexString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<HexString>,
}

/// Checksum of an artifact. There needs to be at least one checksum.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct ArtifactChecksum {
    /// Optional URL containing the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<Url>,

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

/// The content being uploaded. Either a file, or the checksums of a file
/// to be validated against the download.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum ArtifactData {
    /// Base64 encoded data of the file. Checksums will be generated automatically.
    /// Files cannot be larger than 20MB.
    Data(Base64String),

    /// Checksums of the data, with the data available somewhere else.
    Checksums(ArtifactChecksum),
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
    pub download_url: String,
    pub mime_type: String,
    pub created_at: i64,
    pub r#ref: ArtifactRef,
}

#[test]
fn artifact_data_1() {
    let data = ArtifactData::Data(b"data".into());
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#""ZGF0YQ""#);
    let data2: ArtifactData = serde_json::from_str(&json).unwrap();
    assert_eq!(data, data2);
}

#[test]
fn artifact_data_2() {
    let data = ArtifactData::Checksums(ArtifactChecksum {
        download_url: None,
        size: 123,
        md5: Some(b"abc".into()),
        sha1: Some(b"def".into()),
        sha256: None,
    });
    let json = serde_json::to_string(&data).unwrap();
    assert_eq!(json, r#"{"size":123,"md5":"616263","sha1":"646566"}"#);
    let data2: ArtifactData = serde_json::from_str(&json).unwrap();
    assert_eq!(data, data2);
}

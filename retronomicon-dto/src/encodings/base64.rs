use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Eq, PartialEq)]
pub struct Base64String(Vec<u8>);

#[cfg(feature = "openapi")]
impl schemars::JsonSchema for Base64String {
    fn schema_name() -> String {
        // Exclude the module path to make the name in generated schemas clearer.
        "Base64String".to_owned()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema_for_value!("0123").schema)
    }
}

impl From<Base64String> for Vec<u8> {
    fn from(value: Base64String) -> Self {
        value.0
    }
}

impl From<Vec<u8>> for Base64String {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for Base64String {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for Base64String {
    fn from(value: &[u8; N]) -> Self {
        Self(value.to_vec())
    }
}

impl Serialize for Base64String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded: String = general_purpose::STANDARD_NO_PAD.encode(&self.0);
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for Base64String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        general_purpose::STANDARD_NO_PAD
            .decode(String::deserialize(deserializer)?)
            .map(Base64String)
            .map_err(serde::de::Error::custom)
    }
}

impl Deref for Base64String {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Base64String {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

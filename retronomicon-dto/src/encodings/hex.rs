use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[repr(transparent)]
pub struct HexString(Vec<u8>);

#[cfg(feature = "openapi")]
impl schemars::JsonSchema for HexString {
    fn schema_name() -> String {
        // Exclude the module path to make the name in generated schemas clearer.
        "HexString".to_owned()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema_for_value!("01020304").schema)
    }
}

impl FromStr for HexString {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s)?))
    }
}

impl From<HexString> for Vec<u8> {
    fn from(value: HexString) -> Self {
        value.0
    }
}

impl From<Vec<u8>> for HexString {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for HexString {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for HexString {
    fn from(value: &[u8; N]) -> Self {
        Self(value.to_vec())
    }
}

impl Serialize for HexString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for HexString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        hex::decode(String::deserialize(deserializer)?)
            .map(HexString)
            .map_err(serde::de::Error::custom)
    }
}

impl Deref for HexString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HexString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl HexString {
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn to_string(&self) -> String {
        hex::encode(&self.0)
    }
}

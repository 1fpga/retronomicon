use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, EnumString, Display)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserTeamRole {
    Owner,
    Admin,
    #[default]
    Member,
}

/// Either an ID (integer) or a slug (string).
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum IdOrSlug<'v> {
    Id(i32),
    Slug(&'v str),
}

impl<'v> IdOrSlug<'v> {
    pub fn parse(value: &'v str) -> Self {
        match value.parse::<i32>() {
            Ok(id) => IdOrSlug::Id(id),
            Err(_) => IdOrSlug::Slug(value),
        }
    }

    pub fn as_id(&self) -> Option<i32> {
        match self {
            IdOrSlug::Id(id) => Some(*id),
            _ => None,
        }
    }
    pub fn as_slug(&self) -> Option<&str> {
        match self {
            IdOrSlug::Slug(name) => Some(name),
            _ => None,
        }
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::request::FromParam<'v> for IdOrSlug<'v> {
    type Error = std::convert::Infallible;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        Ok(Self::parse(param))
    }
}

impl From<i32> for IdOrSlug<'_> {
    fn from(id: i32) -> Self {
        IdOrSlug::Id(id)
    }
}

impl<'v> From<&'v str> for IdOrSlug<'v> {
    fn from(slug: &'v str) -> Self {
        IdOrSlug::Slug(slug)
    }
}

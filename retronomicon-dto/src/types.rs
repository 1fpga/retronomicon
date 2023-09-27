use serde::{Deserialize, Serialize};

/// Either an ID (integer) or a slug (string).
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum IdOrSlug<'v> {
    Id(i32),
    Slug(&'v str),
}

impl<'v> IdOrSlug<'v> {
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
        match param.parse::<i32>() {
            Ok(id) => Ok(IdOrSlug::Id(id)),
            Err(_) => Ok(IdOrSlug::Slug(param)),
        }
    }
}

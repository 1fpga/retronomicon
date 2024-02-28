use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, EnumString, Display,
)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub enum IdOrSlug<'v> {
    Id(i32),
    Slug(Cow<'v, str>),
}

impl FromStr for IdOrSlug<'static> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.parse::<i32>() {
            Ok(id) => IdOrSlug::Id(id),
            Err(_) => IdOrSlug::Slug(s.to_string().into()),
        })
    }
}

#[cfg(feature = "rocket")]
mod rocket_impls {
    use super::*;
    use rocket::http::uri::fmt::Formatter;
    use std::fmt::Write;

    impl<'v> rocket::form::FromFormField<'v> for IdOrSlug<'v> {
        fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
            Ok(IdOrSlug::parse(field.value))
        }
    }

    impl<'v, T: rocket::http::uri::fmt::Part> rocket::http::uri::fmt::UriDisplay<T> for IdOrSlug<'v> {
        fn fmt(&self, f: &mut Formatter<'_, T>) -> std::fmt::Result {
            f.write_str(&self.to_string())
        }
    }

    impl<'v, T: rocket::http::uri::fmt::Part> rocket::http::uri::fmt::FromUriParam<T, &'v str>
        for IdOrSlug<'v>
    {
        type Target = IdOrSlug<'v>;

        fn from_uri_param(param: &'v str) -> Self::Target {
            Self::parse(param)
        }
    }

    impl<T: rocket::http::uri::fmt::Part> rocket::http::uri::fmt::FromUriParam<T, i32>
        for IdOrSlug<'static>
    {
        type Target = IdOrSlug<'static>;

        fn from_uri_param(param: i32) -> Self::Target {
            IdOrSlug::Id(param)
        }
    }
}

impl<'v> std::fmt::Display for IdOrSlug<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrSlug::Id(id) => write!(f, "{id}"),
            IdOrSlug::Slug(slug) => f.write_str(slug),
        }
    }
}

impl<'v> IdOrSlug<'v> {
    pub fn parse(value: &'v str) -> Self {
        match value.parse::<i32>() {
            Ok(id) => IdOrSlug::Id(id),
            Err(_) => IdOrSlug::Slug(value.into()),
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

    pub fn reduce<R>(self, f_id: impl FnOnce(i32) -> R, f_slug: impl FnOnce(&str) -> R) -> R {
        match self {
            IdOrSlug::Id(id) => f_id(id),
            IdOrSlug::Slug(slug) => f_slug(slug.as_ref()),
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
        IdOrSlug::Slug(slug.into())
    }
}

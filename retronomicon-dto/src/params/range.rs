use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::Bound;
use std::str::FromStr;

/// Parameters for a range of integers.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct RangeParams<T> {
    pub from: Bound<T>,
    pub to: Bound<T>,
}

impl<T> Default for RangeParams<T> {
    fn default() -> Self {
        Self {
            from: Bound::Unbounded,
            to: Bound::Unbounded,
        }
    }
}

impl<T> From<RangeParams<T>> for (Bound<T>, Bound<T>) {
    fn from(value: RangeParams<T>) -> Self {
        (value.from, value.to)
    }
}

#[cfg(feature = "rocket")]
#[rocket::async_trait]
impl<'v, T> rocket::form::FromFormField<'v> for RangeParams<T>
where
    T: Send + Copy + FromStr,
{
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        Ok(Self::from_str(field.value).map_err(|_| field.unexpected())?)
    }
}

#[cfg(feature = "rocket")]
impl<'v, T> rocket::request::FromParam<'v> for RangeParams<T>
where
    T: Copy + FromStr,
{
    type Error = &'static str;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        Self::from_str(param)
    }
}

impl<T> FromStr for RangeParams<T>
where
    T: Copy + FromStr,
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const LEFT_ERR: &str = "Invalid left value";
        const RIGHT_ERR: &str = "Invalid right value";

        let (from, to) = if let Some((l, r)) = s.split_once("..=") {
            (
                Bound::Included(l.parse::<T>().map_err(|_| LEFT_ERR)?),
                Bound::Included(r.parse::<T>().map_err(|_| RIGHT_ERR)?),
            )
        } else if let Some((l, r)) = s.split_once("..") {
            (
                Bound::Included(l.parse::<T>().map_err(|_| LEFT_ERR)?),
                Bound::Excluded(r.parse::<T>().map_err(|_| RIGHT_ERR)?),
            )
        } else if let Some(ge) = s.strip_prefix(">=") {
            (
                Bound::Included(ge.parse::<T>().map_err(|_| LEFT_ERR)?),
                Bound::Unbounded,
            )
        } else if let Some(gt) = s.strip_prefix('>') {
            (
                Bound::Excluded(gt.parse::<T>().map_err(|_| LEFT_ERR)?),
                Bound::Unbounded,
            )
        } else if let Some(le) = s.strip_prefix("<=") {
            (
                Bound::Unbounded,
                Bound::Included(le.parse::<T>().map_err(|_| RIGHT_ERR)?),
            )
        } else if let Some(lt) = s.strip_prefix('<') {
            (
                Bound::Unbounded,
                Bound::Excluded(lt.parse::<T>().map_err(|_| RIGHT_ERR)?),
            )
        } else if let Ok(eq) = s.parse::<T>() {
            (Bound::Included(eq), Bound::Included(eq))
        } else {
            return Err("Invalid range value");
        };

        Ok(RangeParams { from, to })
    }
}

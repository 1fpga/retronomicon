use crate::types::IdOrSlug;
use serde::{Deserialize, Serialize};

pub const PAGE_DEFAULT: i64 = 0;
pub const PAGE_MIN: i64 = 0;
pub const PAGE_MAX: i64 = i64::MAX;
pub const LIMIT_DEFAULT: i64 = 20;
pub const LIMIT_MIN: i64 = 10;
pub const LIMIT_MAX: i64 = 100;

/// Parameters for filtering a list of core releases.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct CoreReleaseFilterParams<'v> {
    /// Whether to include prereleases in the results. Defaults to false.
    pub prerelease: Option<bool>,

    /// Whether to include yanked releases in the results. Defaults to false.
    pub yanked: Option<bool>,

    /// Minimum date to include in the results, in seconds since UNIX EPOCH.
    /// Defaults to 0 (all releases).
    pub min_release_date: Option<i64>,

    /// Maximum date to include in the results, in seconds since UNIX EPOCH.
    /// Defaults to i64::MAX (all releases).
    pub max_release_date: Option<i64>,

    /// Filter releases by platform. By default, include all platforms.
    #[serde(borrow)]
    pub platform: Option<IdOrSlug<'v>>,
}

/// Parameters for paging through a list of items.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
#[cfg_attr(feature = "openapi", derive(schemars::JsonSchema))]
pub struct PagingParams {
    /// The page index to retrieve. The first page is 0. This will
    /// multiply by the limit to get the actual item offset.
    /// Defaults to 0.
    pub page: Option<i64>,

    /// The maximum number of items to retrieve. Must be between 10
    /// and 100. Defaults to 20.
    pub limit: Option<i64>,
}

impl PagingParams {
    pub fn validate(&self) -> Result<(i64, i64), String> {
        let page = self.page.unwrap_or(PAGE_DEFAULT);
        let limit = self.limit.unwrap_or(LIMIT_DEFAULT);

        if page < PAGE_MIN {
            Err(format!("Page must be greater than or equal to {PAGE_MIN}"))
        } else if limit < LIMIT_MIN {
            Err(format!(
                "Limit must be greater than or equal to {LIMIT_MIN}"
            ))
        } else if limit > 100 {
            Err(format!("Limit must be less than or equal to {LIMIT_MAX}"))
        } else {
            Ok((page, limit))
        }
    }
}

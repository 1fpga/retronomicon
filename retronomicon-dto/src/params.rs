use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "rocket", derive(rocket::form::FromForm))]
pub struct PagingParams {
    #[cfg_attr(feature = "rocket", field(default = 0, validate = range(0..)))]
    pub page: i64,
    #[cfg_attr(feature = "rocket", field(default = 10, validate = range(10..=100)))]
    pub limit: i64,
}

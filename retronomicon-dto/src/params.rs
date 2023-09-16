use rocket::FromForm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct PagingParams {
    #[field(default = 0, validate = range(0..))]
    pub page: i64,
    #[field(default = 10, validate = range(10..=100))]
    pub limit: i64,
}

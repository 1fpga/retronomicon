use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonError {
    status: String,
    message: String,
}

#[cfg(feature = "rocket")]
impl From<(rocket::http::Status, String)> for JsonError {
    fn from((status, message): (rocket::http::Status, String)) -> Self {
        JsonError {
            status: status.to_string(),
            message,
        }
    }
}

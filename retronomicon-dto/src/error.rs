use rocket::http::Status;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonError {
    status: String,
    message: String,
}

impl From<(Status, String)> for JsonError {
    fn from((status, message): (Status, String)) -> Self {
        JsonError {
            status: status.to_string(),
            message,
        }
    }
}

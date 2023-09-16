use retronomicon_dto::JsonError;
use rocket::http::Status;
use rocket::serde::json::Json;

#[derive(Debug)]
pub enum Error {
    Request(String),
    Database(String),
    RecordNotFound,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::Request(message) => message.clone(),
            Error::Database(message) => message.clone(),
            Error::RecordNotFound => "Record not found".to_string(),
        }
    }
}

impl<'a> rocket::response::Responder<'a, 'a> for Error {
    fn respond_to(self, request: &'a rocket::request::Request<'_>) -> rocket::response::Result<'a> {
        let status = match self {
            Error::Request(_) => Status::BadRequest,
            Error::Database(_) => Status::InternalServerError,
            Error::RecordNotFound => Status::NotFound,
        };

        let json = JsonError::from((status, self.to_string()));

        rocket::response::Response::build_from(Json(json).respond_to(request)?)
            .status(status)
            .ok()
    }
}

impl<'a> From<rocket::form::Errors<'a>> for Error {
    fn from(value: rocket::form::Errors<'a>) -> Self {
        Error::Request(value.to_string())
    }
}

impl<'a> From<rocket::form::Error<'a>> for Error {
    fn from(value: rocket::form::Error<'a>) -> Self {
        Error::Request(value.to_string())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Error::RecordNotFound,
            value => Error::Database(value.to_string()),
        }
    }
}

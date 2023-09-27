use crate::db::Db;
use crate::models;
use retronomicon_dto as dto;
use rocket::get;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;

#[openapi(tag = "Platforms", ignore = "db")]
#[get("/platforms?<paging..>")]
pub async fn platforms(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::platform::Platform>>, (Status, String)> {
    use crate::schema::platforms::dsl::*;

    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    platforms
        .offset(page * limit)
        .limit(limit)
        .load::<models::Platform>(&mut db)
        .await
        .map(|p| Json(p.into_iter().map(Into::into).collect()))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

use crate::db::Db;
use crate::models;
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;

#[openapi(tag = "Cores", ignore = "db")]
#[get("/cores?<paging..>")]
pub async fn cores_list(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::cores::CoreListItem>>, (Status, String)> {
    use crate::schema::cores::dsl::*;

    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    let core_list = cores
        .offset(page * limit)
        .limit(limit)
        .load::<models::Core>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    todo!()
}

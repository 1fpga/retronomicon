use crate::db::Db;
use crate::models;
use retronomicon_dto::params::PagingParams;
use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use rocket_db_pools::diesel::prelude::*;

#[get("/platforms?<paging..>")]
async fn platforms(
    mut db: Db,
    paging: rocket::form::Result<'_, PagingParams>,
) -> Result<Json<Vec<models::Platform>>, crate::error::Error> {
    use crate::schema::platforms::dsl::*;

    let PagingParams { page, limit } = paging?;

    platforms
        .offset(page * limit)
        .limit(limit)
        .load::<models::Platform>(&mut db)
        .await
        .map(Json)
        .map_err(crate::error::Error::from)
}

pub fn routes() -> Vec<Route> {
    routes![platforms]
}

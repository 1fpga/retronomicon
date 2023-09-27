use crate::db::Db;
use crate::{guards, models, schema};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, put};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;

/// List tags.
#[openapi(tag = "Tags", ignore = "db")]
#[get("/tags?<paging..>")]
pub async fn tags(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::tags::Tag>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    schema::tags::table
        .offset(page * limit)
        .limit(limit)
        .load::<models::Tag>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
        .map(|u| Json(u.into_iter().map(Into::into).collect()))
}

/// Create or update a tag.
#[openapi(tag = "Tags", ignore = "db")]
#[put("/tags", data = "<tag>")]
pub async fn tags_create(
    mut db: Db,
    _user: guards::users::RootUserGuard,
    tag: Json<dto::tags::TagCreate>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let tag = tag.into_inner();

    diesel::insert_into(schema::tags::table)
        .values((
            schema::tags::slug.eq(&tag.slug),
            schema::tags::description.eq(&tag.description),
            schema::tags::color.eq(tag.color as i64),
        ))
        .on_conflict(schema::tags::slug)
        .do_update()
        .set((
            schema::tags::description.eq(&tag.description),
            schema::tags::color.eq(tag.color as i64),
        ))
        .execute(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
        .map(|_| Json(dto::Ok))
}

/// Get a tag information (including its description).
#[openapi(tag = "Tags", ignore = "db")]
#[delete("/tags/<tag_id>")]
pub async fn tags_delete(
    mut db: Db,
    _user: guards::users::RootUserGuard,
    tag_id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    if let Some(tag_id) = tag_id.as_id() {
        diesel::delete(schema::tags::table)
            .filter(schema::tags::id.eq(tag_id))
            .execute(&mut db)
            .await
    } else if let Some(slug) = tag_id.as_slug() {
        diesel::delete(schema::tags::table)
            .filter(schema::tags::slug.eq(slug))
            .execute(&mut db)
            .await
    } else {
        return Err((Status::BadRequest, "Invalid tag ID or slug".to_string()));
    }
    .map_err(|e| (Status::InternalServerError, e.to_string()))
    .map(|_| Json(dto::Ok))
}

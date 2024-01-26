use crate::guards;
use retronomicon_db::models;
use retronomicon_db::types::FetchModel;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post};
use rocket_okapi::openapi;

/// List tags.
#[openapi(tag = "Tags", ignore = "db")]
#[get("/tags?<paging..>")]
pub async fn tags(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::tags::Tag>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    models::Tag::list(&mut db, page, limit)
        .await
        .map(|u| Json(u.into_iter().map(Into::into).collect()))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

/// Create or update a tag.
#[openapi(tag = "Tags", ignore = "db")]
#[post("/tags", data = "<tag>")]
pub async fn tags_create(
    mut db: Db,
    _user: guards::users::RootUserGuard,
    tag: Json<dto::tags::TagCreate>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let tag = tag.into_inner();

    models::Tag::create(&mut db, tag.slug, tag.description, tag.color)
        .await
        .map(|_| Json(dto::Ok))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

/// Get a tag information (including its description).
#[openapi(tag = "Tags", ignore = "db")]
#[delete("/tags/<tag_id>")]
pub async fn tags_delete(
    mut db: Db,
    _user: guards::users::RootUserGuard,
    tag_id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let tag = models::Tag::from_id_or_slug(&mut db, tag_id).await?;

    tag.delete(&mut db)
        .await
        .map(|_| Json(dto::Ok))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

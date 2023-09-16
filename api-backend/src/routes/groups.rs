use crate::db::Db;
use crate::{models, schema};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use rocket_db_pools::diesel::prelude::*;

#[get("/groups/?<paging..>")]
async fn groups(
    mut db: Db,
    paging: rocket::form::Result<'_, dto::params::PagingParams>,
) -> Result<Json<Vec<models::Group>>, (Status, crate::error::Error)> {
    let dto::params::PagingParams { page, limit } =
        paging.map_err(|e| (Status::BadRequest, e.into()))?;

    schema::groups::table
        .offset(page * limit)
        .limit(limit)
        .load::<models::Group>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.into()))
        .map(Json)
}

async fn group_details_from_(
    mut db: Db,
    group: models::Group,
) -> Result<Json<dto::details::GroupDetails>, crate::error::Error> {
    let users = models::UserGroup::belonging_to(&group)
        .inner_join(schema::users::table)
        .select((schema::users::id, schema::users::username))
        .filter(schema::users::username.is_not_null())
        .load::<(i32, Option<String>)>(&mut db)
        .await?
        .into_iter()
        .filter_map(|(id, username)| username.map(|username| dto::user::UserRef { id, username }))
        .collect();

    Ok(Json(dto::details::GroupDetails {
        group: dto::details::GroupRef {
            id: group.id,
            slug: group.slug,
            name: group.name,
        },
        description: group.description,
        links: group.links,
        users,
    }))
}

#[get("/groups/<id>", rank = 1)]
async fn groups_details_id(
    mut db: Db,
    id: i32,
) -> Result<Json<dto::details::GroupDetails>, crate::error::Error> {
    let group = schema::groups::table
        .filter(schema::groups::id.eq(id))
        .first::<models::Group>(&mut db)
        .await?;

    group_details_from_(db, group).await
}

#[get("/groups/<slug>", rank = 2)]
async fn groups_details_slug(
    mut db: Db,
    slug: String,
) -> Result<Json<dto::details::GroupDetails>, crate::error::Error> {
    let group = schema::groups::table
        .filter(schema::groups::slug.eq(&slug))
        .first::<models::Group>(&mut db)
        .await?;

    group_details_from_(db, group).await
}

pub fn routes() -> Vec<Route> {
    routes![groups, groups_details_id, groups_details_slug]
}

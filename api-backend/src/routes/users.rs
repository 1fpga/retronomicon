use crate::db::Db;
use crate::error::Error;
use crate::{models, schema};
use retronomicon_dto as dto;
use retronomicon_dto::params::PagingParams;
use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use rocket_db_pools::diesel::prelude::*;

#[get("/users/?<paging..>")]
async fn users_(
    db: Db,
    paging: rocket::form::Result<'_, PagingParams>,
) -> Result<Json<Vec<dto::user::User>>, Error> {
    users(db, paging).await
}

#[get("/users?<paging..>")]
async fn users(
    mut db: Db,
    paging: rocket::form::Result<'_, PagingParams>,
) -> Result<Json<Vec<dto::user::User>>, Error> {
    let PagingParams { page, limit } = paging?;

    schema::users::table
        .offset(page * limit)
        .limit(limit)
        .load::<models::User>(&mut db)
        .await
        .map_err(|e| e.into())
        .map(|u| Json(u.into_iter().map(Into::into).collect()))
}

async fn user_details_from_(
    mut db: Db,
    user: models::User,
) -> Result<Json<dto::user::UserDetails>, Error> {
    let username = user.username.as_ref().ok_or(Error::RecordNotFound)?;
    if user.deleted {
        return Err(Error::RecordNotFound);
    }

    let groups = models::UserGroup::belonging_to(&user)
        .inner_join(schema::groups::table)
        .select((
            schema::groups::id,
            schema::groups::name,
            schema::groups::slug,
        ))
        .load(&mut db)
        .await?
        .into_iter()
        .map(|(id, name, slug)| dto::details::GroupRef { id, name, slug })
        .collect();

    Ok(Json(dto::user::UserDetails {
        groups,
        user: dto::user::UserDetailsInner {
            id: user.id,
            username: username.clone(),
            description: user.description,
            links: user.links,
            metadata: user.metadata,
        },
    }))
}

#[get("/users/<id>", rank = 1)]
async fn users_details_id(mut db: Db, id: i32) -> Result<Json<dto::user::UserDetails>, Error> {
    let user = schema::users::table
        .filter(schema::users::id.eq(id))
        .first::<models::User>(&mut db)
        .await?;

    user_details_from_(db, user).await
}

#[get("/users/<username>", rank = 2)]
async fn users_details_slug(
    mut db: Db,
    username: String,
) -> Result<Json<dto::user::UserDetails>, Error> {
    let user = schema::users::table
        .filter(schema::users::username.eq(&Some(username)))
        .first::<models::User>(&mut db)
        .await?;

    user_details_from_(db, user).await
}

pub fn routes() -> Vec<Route> {
    routes![users_, users, users_details_id, users_details_slug]
}

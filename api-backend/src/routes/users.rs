use crate::db::Db;
use crate::error::Error;
use crate::{guards, models, schema};
use retronomicon_dto as dto;
use retronomicon_dto::params::PagingParams;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::diesel::prelude::*;
use serde_json::json;

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

pub async fn user_details_from_(
    mut db: Db,
    user: models::User,
) -> Result<Json<dto::user::UserDetails>, Error> {
    let username = user.username.as_ref().ok_or(Error::RecordNotFound)?;
    if user.deleted {
        return Err(Error::RecordNotFound);
    }

    let teams = models::UserTeam::belonging_to(&user)
        .inner_join(schema::teams::table)
        .select((schema::teams::id, schema::teams::name, schema::teams::slug))
        .load(&mut db)
        .await?
        .into_iter()
        .map(|(id, name, slug)| dto::teams::TeamRef { id, name, slug })
        .collect();

    Ok(Json(dto::user::UserDetails {
        teams,
        user: dto::user::UserDetailsInner {
            id: user.id,
            username: username.clone(),
            description: user.description,
            links: user.links.unwrap_or_else(|| json!({})),
            metadata: user.metadata.unwrap_or_else(|| json!({})),
        },
    }))
}

#[get("/users/<id>")]
async fn users_id(
    mut db: Db,
    id: dto::user::UserId<'_>,
) -> Result<Json<dto::user::UserDetails>, Error> {
    let user = models::User::from_userid(&mut db, id).await?;

    user_details_from_(db, user).await
}

/// Only root users can update other users.
#[post(
    "/users/<id>/update",
    rank = 1,
    format = "application/json",
    data = "<form>"
)]
async fn users_id_update(
    mut db: Db,
    _root_user: guards::users::RootUserGuard,
    id: dto::user::UserId<'_>,
    form: Json<dto::user::UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, Error> {
    let user = models::User::from_userid(&mut db, id).await?;
    let user_guard = guards::users::UserGuard::from_model(user);

    user_guard.update(&mut db, form.into_inner()).await?;

    Ok(Json(dto::Ok))
}

pub fn routes() -> Vec<Route> {
    routes![users_, users, users_id, users_id_update]
}

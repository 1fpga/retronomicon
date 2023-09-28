use crate::db::Db;
use crate::{guards, models, schema};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;

/// Check availability of a username.
#[openapi(tag = "Users", ignore = "db")]
#[post("/users/check?<username>")]
pub async fn check_username(mut db: Db, username: String) -> Result<Json<bool>, Status> {
    let exists = schema::users::table
        .filter(schema::users::username.eq(&Some(username)))
        .first::<models::User>(&mut db)
        .await
        .is_ok();

    Ok(Json(exists))
}

/// List all users.
#[openapi(tag = "Users", ignore = "db")]
#[get("/users?<paging..>")]
pub async fn users(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::user::User>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    schema::users::table
        .offset(page * limit)
        .limit(limit)
        .load::<models::User>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
        .map(|u| Json(u.into_iter().map(Into::into).collect()))
}

pub async fn user_details_from_(
    mut db: Db,
    user: models::User,
) -> Result<Json<dto::user::UserDetails>, (Status, String)> {
    let username = user
        .username
        .as_ref()
        .ok_or((Status::NotFound, "User not found".to_string()))?;
    if user.deleted {
        return Err((Status::NotFound, "User not found".to_string()));
    }

    let teams = models::UserTeam::belonging_to(&user)
        .inner_join(schema::teams::table)
        .select((
            schema::teams::id,
            schema::teams::name,
            schema::teams::slug,
            schema::user_teams::role,
        ))
        .load::<(i32, String, String, models::UserTeamRole)>(&mut db)
        .await
        .optional()
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Team not found".to_string()))?
        .into_iter()
        .map(|(id, name, slug, role)| dto::user::UserTeamRef {
            team: dto::teams::TeamRef { id, name, slug },
            role: role.into(),
        })
        .collect();

    Ok(Json(dto::user::UserDetails {
        teams,
        user: dto::user::UserDetailsInner {
            id: user.id,
            username: username.clone(),
            description: user.description,
            links: user.links,
            metadata: user.metadata,
        },
    }))
}

#[openapi(tag = "Users", ignore = "db")]
#[get("/users/<id>")]
pub async fn users_id(
    mut db: Db,
    id: dto::user::UserIdOrUsername<'_>,
) -> Result<Json<dto::user::UserDetails>, (Status, String)> {
    let user = models::User::from_userid(&mut db, id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;

    user_details_from_(db, user).await
}

/// Only root users can update other users.
#[openapi(tag = "Users", ignore = "db")]
#[put("/users/<id>", rank = 1, format = "application/json", data = "<form>")]
pub async fn users_id_update(
    mut db: Db,
    _root_user: guards::users::RootUserGuard,
    id: dto::user::UserIdOrUsername<'_>,
    form: Json<dto::user::UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let user = models::User::from_userid(&mut db, id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;
    let user_guard = guards::users::UserGuard::from_model(user);

    user_guard
        .update(&mut db, form.into_inner())
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;

    Ok(Json(dto::Ok))
}

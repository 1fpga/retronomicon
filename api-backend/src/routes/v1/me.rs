use crate::db::Db;
use crate::guards::users::UserGuard;
use crate::routes::v1::users::user_details_from_;
use crate::{models, schema};
use retronomicon_dto as dto;
use rocket::http::{CookieJar, Status};
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;

#[openapi(tag = "Users", ignore = "db")]
#[put("/me/update", format = "application/json", data = "<form>")]
pub async fn me_update(
    mut db: Db,
    cookies: &CookieJar<'_>,
    mut user: UserGuard,
    form: Json<dto::user::UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, String> {
    if user.username.is_some() && form.username.is_some() {
        return Err("Username already set".to_string());
    }

    let username = form.username.clone();
    user.update(&mut db, form.into_inner())
        .await
        .map_err(|e| e.to_string())?;

    // At this point, because of the unique constraint on username, we know
    // that the username is set.
    user.username = username.map(Into::into);
    user.update_cookie(cookies);

    Ok(Json(dto::Ok))
}

#[openapi(tag = "Users", ignore = "db")]
#[get("/me")]
pub async fn me(
    mut db: Db,
    user: UserGuard,
) -> Result<Json<dto::user::UserDetails>, (Status, String)> {
    let user = schema::users::table
        .filter(schema::users::id.eq(user.id))
        .first::<models::User>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    user_details_from_(db, user).await
}

/// Create a JWT token for the current logged-in user.
#[openapi(tag = "Authentication")]
#[post("/me/token")]
pub async fn me_token(user: UserGuard) -> Result<Json<dto::AuthTokenResponse>, (Status, String)> {
    user.create_jwt()
        .map(|token| Json(dto::AuthTokenResponse { token }))
        .map_err(|e| (Status::Unauthorized, e.to_string()))
}

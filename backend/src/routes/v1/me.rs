use crate::guards::users::UserGuard;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::{CookieJar, Status};
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_okapi::openapi;

#[openapi(tag = "Users", ignore = "db")]
#[put("/me/update", format = "application/json", data = "<form>")]
pub async fn me_update(
    mut db: Db,
    cookies: &CookieJar<'_>,
    mut user: UserGuard,
    form: Json<dto::user::UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    if user.username.is_some() && form.username.is_some() {
        return Err((Status::Forbidden, "Username already set".to_string()));
    }
    if let Some(username) = form.username {
        dto::user::Username::new(username).map_err(|e| (Status::BadRequest, e.to_string()))?;
    }

    let username = form.username;
    user.update(&mut db, form.into_inner())
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    // At this point, because of the unique constraint on username, we know
    // that the username is set.
    user.username = username.map(Into::into);
    user.update_cookie(cookies);

    Ok(Json(dto::Ok))
}

#[openapi(tag = "Users", ignore = "db")]
#[get("/me")]
pub async fn me(db: Db, user: UserGuard) -> Result<Json<dto::user::UserDetails>, (Status, String)> {
    crate::routes::v1::users::users_details(db, user.id.into()).await
}

/// Create a JWT token for the current logged-in user.
#[openapi(tag = "Authentication")]
#[post("/me/token")]
pub async fn me_token(user: UserGuard) -> Result<Json<dto::AuthTokenResponse>, (Status, String)> {
    user.create_jwt()
        .map(|token| Json(dto::AuthTokenResponse { token }))
        .map_err(|e| (Status::Unauthorized, e.to_string()))
}

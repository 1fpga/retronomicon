use crate::db::Db;
use crate::error::Error;
use crate::guards::users::UserGuard;
use crate::routes::users::user_details_from_;
use crate::{models, schema};
use retronomicon_dto as dto;
use retronomicon_dto::user::UserDetails;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::diesel::prelude::*;

#[get("/me/check?<username>")]
async fn me_check(mut db: Db, username: String) -> Result<Json<bool>, Error> {
    let exists = schema::users::table
        .filter(schema::users::username.eq(&Some(username)))
        .first::<models::User>(&mut db)
        .await
        .is_ok();

    Ok(Json(exists))
}

#[post("/me/update", format = "application/json", data = "<form>")]
async fn me_update(
    mut db: Db,
    cookies: &CookieJar<'_>,
    mut user: UserGuard,
    form: Json<dto::user::UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, Error> {
    if user.username.is_some() && form.username.is_some() {
        Err(Error::Request("Username already set".into()))?
    }

    let username = form.username.clone();
    user.update(&mut db, form.into_inner()).await?;

    // At this point, because of the unique constraint on username, we know
    // that the username is set.
    user.username = username.map(Into::into);
    user.update_cookie(cookies);

    Ok(Json(dto::Ok))
}

#[get("/me")]
async fn me(mut db: Db, user: UserGuard) -> Result<Json<UserDetails>, Error> {
    let user = schema::users::table
        .filter(schema::users::id.eq(user.id))
        .first::<models::User>(&mut db)
        .await?;

    user_details_from_(db, user).await
}

pub fn routes() -> Vec<Route> {
    routes![me, me_check, me_update]
}

use crate::fairings::config::{DbPepper, RetronomiconConfig};
use crate::guards::emailer::EmailGuard;
use crate::guards::users::UserGuard;
use crate::routes::auth::{GitHubUserInfo, GoogleUserInfo, PatreonUserInfo};
use retronomicon_db::models::{User, UserPassword};
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, post, uri, State};
use rocket_oauth2::OAuth2;
use rocket_okapi::openapi;
use serde_json::json;

/// Create a user with a password. This cannot be used if the user already exists.
#[openapi(tag = "Authentication", ignore = "db", ignore = "emailer")]
#[post("/signup", format = "application/json", data = "<form>")]
pub async fn signup(
    mut db: Db,
    form: Json<dto::auth::SignupRequest<'_>>,
    pepper: &State<DbPepper>,
    config: &State<RetronomiconConfig>,
    emailer: EmailGuard,
) -> Result<Json<dto::auth::SignupResponse>, (Status, String)> {
    let form = form.into_inner();

    let user = User::create(
        &mut db,
        None,
        None,
        None,
        form.email,
        None,
        None,
        json!({}),
        json!({}),
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    // Create the user password.
    let user_password = UserPassword::create(&mut db, &user, Some(form.password), &pepper.0, true)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    if let Some(token) = user_password.validation_token {
        // Send an email.
        emailer.send_email_verification(
            &user.email,
            url::Url::parse(&format!(
                "{}{}",
                config.inner().base_url,
                uri!(
                    "/api",
                    crate::routes::auth::login_token_callback(&user.email, &token)
                ),
            ))
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .as_str(),
        )?;

        Ok(Json(dto::auth::SignupResponse { email: user.email }))
    } else {
        let _ = user_password.delete(&mut db).await;
        let _ = user.delete_row(&mut db).await;
        Err((
            Status::InternalServerError,
            "Validation token not set".to_string(),
        ))
    }
}

/// Login with an email and password.
#[openapi(tag = "Authentication", ignore = "db")]
#[post("/login", format = "application/json", data = "<form>")]
pub async fn login(
    mut db: Db,
    cookies: &CookieJar<'_>,
    pepper: &State<DbPepper>,
    form: Json<dto::auth::SignupRequest<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let form = form.into_inner();

    let user = User::from_email(&mut db, form.email, form.password, &pepper.inner().0)
        .await
        .map_err(|e| (Status::Unauthorized, e.to_string()))?;

    let guard = UserGuard::from_model(user);
    guard.update_cookie(cookies);

    Ok(Json(dto::Ok))
}

/// Login using GitHub with OAuth2. This will redirect the user to GitHub's login
/// page. If the user accepts the request, GitHub will redirect the user back to
/// the callback URL specified in the OAuth2 configuration.
///
/// This is not a REST endpoint, but a normal web page.
// NB: Here we are using the same struct as a type parameter to OAuth2 and
// TokenResponse as we use for the user's GitHub login details. For
// `TokenResponse` and `OAuth2` the actual type does not matter; only that they
// are matched up.
#[openapi(tag = "Authentication", ignore = "oauth2")]
#[get("/login/github")]
pub async fn github_login(oauth2: OAuth2<GitHubUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["user:read"]).unwrap()
}

/// Login using Patreon with OAuth2.
#[openapi(tag = "Authentication", ignore = "oauth2")]
#[get("/login/patreon")]
pub async fn patreon_login(oauth2: OAuth2<PatreonUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["identity[email]"]).unwrap()
}

/// Login using Google with OAuth2. This will redirect the user to GitHub's login
/// page. If the user accepts the request, GitHub will redirect the user back to
/// the callback URL specified in the OAuth2 configuration.
///
/// This is not a REST endpoint, but a normal web page.
#[openapi(tag = "Authentication", ignore = "oauth2")]
#[get("/login/google")]
pub async fn google_login(oauth2: OAuth2<GoogleUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["profile", "email", "openid"])
        .unwrap()
}

/// Logout the current user.
#[openapi(tag = "Authentication")]
#[post("/logout")]
pub async fn logout(
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
    user: UserGuard,
) -> Redirect {
    user.remove_cookie(cookies);
    let base_url = config.base_url.clone();
    Redirect::to(base_url)
}

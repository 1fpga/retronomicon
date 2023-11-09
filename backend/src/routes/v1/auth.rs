use crate::guards::users::UserGuard;
use crate::routes::auth::{GitHubUserInfo, GoogleUserInfo, PatreonUserInfo};
use crate::RetronomiconConfig;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_oauth2::OAuth2;
use rocket_okapi::openapi;

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

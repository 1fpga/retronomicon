use crate::fairings::config::RetronomiconConfig;
use crate::guards::users::UserGuard;
use retronomicon_db::{models, Db};
use rocket::http::hyper::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::{error, get, State};
use rocket_oauth2::TokenResponse;
use serde_json::Value;
use std::collections::BTreeMap;

async fn maybe_add_to_root(
    db: &mut Db,
    config: &RetronomiconConfig,
    model: &models::User,
) -> Result<(), (Status, String)> {
    if config.should_add_to_root(&model.email) {
        if let Err(error) = model
            .join_team(db, config.root_team_id, models::UserTeamRole::Owner)
            .await
        {
            error!("Failed to add user to root team: {:?}", error);
        }
    }

    Ok(())
}

async fn login_(
    mut db: Db,
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
    username: Option<String>,
    email: &str,
    auth_provider: &str,
) -> Result<Redirect, (Status, String)> {
    let (_created, model, user_guard) =
        UserGuard::login_from_auth(&mut db, username, email, auth_provider.to_string(), None)
            .await?;

    maybe_add_to_root(&mut db, config, &model).await?;
    user_guard.update_cookie(cookies);

    let base_url = config.base_url.clone();
    Ok(Redirect::to(base_url))
}

#[get("/auth/verify?<email>&<token>")]
pub async fn login_token_callback(
    mut db: Db,
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
    email: String,
    token: String,
) -> Result<Redirect, (Status, String)> {
    let (user, user_password) =
        models::UserPassword::from_validation_token(&mut db, &email, &token)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::NotFound, "Invalid token".to_string()))?;

    // At this point we know we have the right token, user and user_password entry.
    user_password
        .validated(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    maybe_add_to_root(&mut db, config, &user).await?;

    let user_guard = UserGuard::from_model(user);
    user_guard.update_cookie(cookies);

    let base_url = config.base_url.clone();
    Ok(Redirect::to(base_url))
}

/// User information to be retrieved from the GitHub API.
#[derive(serde::Deserialize)]
pub struct GitHubUserInfo {
    login: String,
    email: String,
}

#[get("/auth/github")]
pub async fn github_callback(
    db: Db,
    token: TokenResponse<GitHubUserInfo>,
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
) -> Result<Redirect, (Status, String)> {
    let json: Value = reqwest::Client::builder()
        .build()
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .get("https://api.github.com/user")
        .header(AUTHORIZATION, format!("token {}", token.access_token()))
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(USER_AGENT, "retronomicon-backend")
        .send()
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .json()
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    // Use the token to retrieve the user's GitHub account information.
    let user_info: GitHubUserInfo = serde_json::from_str(&json.to_string()).unwrap();

    login_(
        db,
        cookies,
        config,
        Some(user_info.login),
        &user_info.email,
        "github",
    )
    .await
}

/// User information to be retrieved from the Google People API.
#[derive(serde::Deserialize)]
pub struct GoogleUserInfo {
    #[allow(unused)]
    names: Vec<Value>,

    #[serde(default, rename = "emailAddresses")]
    email_addresses: Vec<Value>,
}

#[get("/auth/google")]
pub async fn google_callback(
    db: Db,
    token: TokenResponse<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
    frontend_config: &State<RetronomiconConfig>,
) -> Result<Redirect, (Status, String)> {
    let json: Value = reqwest::Client::builder()
        .build()
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .get("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .json()
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    // Use the token to retrieve the user's Google account information.
    let user_info: GoogleUserInfo = serde_json::from_str(&json.to_string()).unwrap();
    let email = user_info.email_addresses[0]
        .get("value")
        .and_then(|e| e.as_str());

    if let Some(email) = email {
        login_(db, cookies, frontend_config, None, email, "google").await
    } else {
        Err((
            Status::InternalServerError,
            "Failed to get email".to_string(),
        ))
    }
}

#[derive(serde::Deserialize)]
pub struct PatreonUserInfoData {
    attributes: BTreeMap<String, Value>,
}

/// User information to be retrieved from the Patreon OAuth API.
#[derive(serde::Deserialize)]
pub struct PatreonUserInfo {
    errors: Option<Vec<Value>>,
    data: Option<PatreonUserInfoData>,
}

#[get("/auth/patreon")]
pub async fn patreon_callback(
    db: Db,
    token: TokenResponse<PatreonUserInfo>,
    cookies: &CookieJar<'_>,
    frontend_config: &State<RetronomiconConfig>,
) -> Result<Redirect, (Status, String)> {
    let json: Value = reqwest::Client::builder()
        .build()
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .get("https://api.patreon.com/api/oauth2/v2/identity?fields%5Buser%5D=email")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .json()
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;
    let user_info: PatreonUserInfo = serde_json::from_str(&json.to_string()).unwrap();

    if let Some(err) = user_info.errors {
        return Err((
            Status::InternalServerError,
            format!("failed to get email: {:#?}", err),
        ));
    }
    let data = match user_info.data {
        Some(data) => data,
        None => {
            return Err((
                Status::InternalServerError,
                "Failed to get email".to_string(),
            ));
        }
    };
    let email = match data.attributes.get("email") {
        Some(email) => match email.as_str() {
            Some(email) => email,
            None => {
                return Err((
                    Status::InternalServerError,
                    "Invalid email type".to_string(),
                ));
            }
        },
        None => {
            return Err((Status::InternalServerError, "no email".to_string()));
        }
    };

    login_(db, cookies, frontend_config, None, email, "patreon").await
}

use crate::guards::users::UserGuard;
use crate::RetronomiconConfig;
use anyhow::{Context, Error};
use retronomicon_db::{models, Db};
use rocket::http::hyper::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use rocket::http::CookieJar;
use rocket::response::{Debug, Redirect};
use rocket::{get, State};
use rocket_oauth2::TokenResponse;
use serde_json::Value;
use std::collections::BTreeMap;
use tracing::error;
use wildmatch::WildMatch;

async fn login_(
    mut db: Db,
    cookies: &CookieJar<'_>,
    config: &State<RetronomiconConfig>,
    username: Option<String>,
    email: &str,
    auth_provider: &str,
) -> Result<Redirect, Debug<Error>> {
    let mut add_to_root = false;

    if config
        .root_team
        .iter()
        .any(|e| WildMatch::new(e).matches(email))
    {
        add_to_root = true;
    }

    if let Ok(env_email) = std::env::var("ROCKET_DEBUG_ROOT_ADDITIONAL_EMAIL") {
        if env_email == email {
            add_to_root = true;
        }
    }

    let (_created, model, user) =
        UserGuard::login_from_auth(&mut db, username, email, auth_provider.to_string(), None)
            .await
            .map_err(|(_, msg)| Debug(Error::msg(msg)))?;

    if add_to_root {
        if let Err(error) = model
            .join_team(&mut db, config.root_team_id, models::UserTeamRole::Owner)
            .await
        {
            error!(?error, "Failed to add user to root team");
        }
    }

    user.update_cookie(cookies);

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
) -> Result<Redirect, Debug<Error>> {
    let json: Value = reqwest::Client::builder()
        .build()
        .context("Failed to build reqwest client")?
        .get("https://api.github.com/user")
        .header(AUTHORIZATION, format!("token {}", token.access_token()))
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(USER_AGENT, "retronomicon-backend")
        .send()
        .await
        .context("Failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

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
) -> Result<Redirect, Debug<Error>> {
    let json: Value = reqwest::Client::builder()
        .build()
        .context("Failed to build reqwest client")?
        .get("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .context("Failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

    // Use the token to retrieve the user's Google account information.
    let user_info: GoogleUserInfo = serde_json::from_str(&json.to_string()).unwrap();
    let email = user_info.email_addresses[0]
        .get("value")
        .and_then(|e| e.as_str());

    if let Some(email) = email {
        login_(db, cookies, frontend_config, None, email, "google").await
    } else {
        Err(Debug(Error::msg("Failed to get email")))
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
) -> Result<Redirect, Debug<Error>> {
    let json: Value = reqwest::Client::builder()
        .build()
        .context("Failed to build reqwest client")?
        .get("https://api.patreon.com/api/oauth2/v2/identity?fields%5Buser%5D=email")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .context("Failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;
    let user_info: PatreonUserInfo = serde_json::from_str(&json.to_string()).unwrap();

    if let Some(err) = user_info.errors {
        return Err(Debug(Error::msg(format!(
            "failed to get email: {:#?}",
            err
        ))));
    }
    let data = match user_info.data {
        Some(data) => data,
        None => {
            return Err(Debug(Error::msg("Failed to get email")));
        }
    };
    let email = match data.attributes.get("email") {
        Some(email) => match email.as_str() {
            Some(email) => email,
            None => {
                return Err(Debug(Error::msg("Invalid email type")));
            }
        },
        None => {
            return Err(Debug(Error::msg("no email")));
        }
    };

    login_(db, cookies, frontend_config, None, email, "patreon").await
}

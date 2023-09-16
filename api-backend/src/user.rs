use crate::db::Db;
use crate::models;
use crate::schema::users::dsl;
use diesel::prelude::*;
use jsonwebtoken::DecodingKey;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::{IntoOutcome, Outcome};
use rocket::request;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};

/// A user that went through the signed up process and has a username.
#[derive(Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub auth_provider: String,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for AuthenticatedUser {
    type Error = String;

    async fn from_request(
        request: &'r request::Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        User::from_request(request).await.and_then(|user| {
            if let Some(user) = user.into() {
                Outcome::Success(user)
            } else {
                Outcome::Forward(Status::Unauthorized)
            }
        })
    }
}

/// A potentially non-fully signed up user for the website.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: Option<String>,
    pub auth_provider: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "exp")]
    pub expiry: Option<i64>,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for User {
    type Error = String;

    async fn from_request(
        request: &'r request::Request<'_>,
    ) -> request::Outcome<User, Self::Error> {
        // Check cookies.
        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");
        if let Some(cookie) = cookies.get_private("auth") {
            return serde_json::from_str(cookie.value())
                .map_err(|e| e.to_string())
                .into_outcome(Status::InternalServerError);
        }

        // Check JWT.
        fn is_valid(key: &str) -> Result<User, String> {
            User::decode_jwt(String::from(key)).map_err(|e| e.to_string())
        }

        match request.headers().get_one("authorization") {
            None => Outcome::Forward(Status::Unauthorized),
            Some(key) => is_valid(key)
                .map_err(|e| e.to_string())
                .into_outcome(Status::Unauthorized),
        }
    }
}

impl From<User> for Option<AuthenticatedUser> {
    fn from(value: User) -> Self {
        if let Some(username) = value.username {
            Some(AuthenticatedUser {
                id: value.id,
                email: value.email,
                username,
                auth_provider: value.auth_provider.unwrap(),
            })
        } else {
            None
        }
    }
}

impl<'a> From<&User> for Cookie<'a> {
    fn from(user: &User) -> Self {
        Cookie::build("auth", serde_json::to_string(user).unwrap())
            .same_site(rocket::http::SameSite::Lax)
            .finish()
    }
}

impl User {
    pub fn new(
        id: i32,
        email: String,
        username: Option<String>,
        auth_provider: Option<String>,
    ) -> Self {
        Self {
            id,
            email,
            username,
            auth_provider,
            expiry: None,
        }
    }

    pub fn set_expiry(&mut self, expiry: i64) {
        self.expiry = Some(expiry);
    }

    pub fn clear_expiry(&mut self) {
        self.expiry = None;
    }

    pub fn from_model(user: models::User) -> Self {
        Self::new(user.id, user.email, user.username, user.auth_provider)
    }

    pub async fn from_db(db: &mut Db, id: i32) -> Result<Self, diesel::result::Error> {
        dsl::users
            .filter(dsl::id.eq(id))
            .first::<models::User>(db)
            .await
            .map(Self::from_model)
    }

    pub async fn from_db_by_email(
        db: &mut Db,
        email: String,
    ) -> Result<Self, diesel::result::Error> {
        dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::User>(db)
            .await
            .map(Self::from_model)
    }

    /// Create a new user or select an existing one. This should only be used
    /// from an OAuth provider.
    pub async fn login_from_auth(
        db: &mut Db,
        username: Option<String>,
        email: String,
        auth_provider: String,
        avatar_url: Option<String>,
    ) -> Result<Self, diesel::result::Error> {
        db.transaction(|db| {
            async move {
                let maybe_user = dsl::users
                    .filter(dsl::email.eq(&email))
                    .filter(dsl::auth_provider.eq(&auth_provider))
                    .first::<models::User>(db)
                    .await
                    .optional()?;

                if let Some(u) = maybe_user {
                    return Ok(Self::from_model(u));
                }

                let user = diesel::insert_into(dsl::users)
                    .values((
                        dsl::username.eq(&username),
                        dsl::email.eq(&email),
                        dsl::auth_provider.eq(&auth_provider),
                        dsl::avatar_url.eq(avatar_url),
                        dsl::need_reset.eq(false),
                        dsl::deleted.eq(false),
                        dsl::description.eq(""),
                    ))
                    .on_conflict_do_nothing()
                    .get_result::<models::User>(db)
                    .await?;

                Ok(Self::from_model(user))
            }
            .scope_boxed()
        })
        .await
    }

    pub fn update_cookie(&self, cookies: &CookieJar<'_>) {
        // Set a private cookie with the user's name, and redirect to the home page.
        cookies.add_private(self.clone().into());
    }

    pub fn decode_jwt(token: String) -> Result<Self, jsonwebtoken::errors::Error> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
        let token = token.trim_start_matches("Bearer").trim();
        match jsonwebtoken::decode(
            &token,
            &DecodingKey::from_secret(&secret.as_bytes()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
        ) {
            Ok(token) => Ok(token.claims),
            Err(e) => Err(e),
        }
    }

    pub fn create_jwt(mut self) -> Result<String, jsonwebtoken::errors::Error> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(7))
            .expect("Invalid timestamp")
            .timestamp();
        self.set_expiry(expiration);

        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512),
            &self,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
    }

    pub fn remove_cookie(&self, cookies: &CookieJar) {
        cookies.remove_private(Cookie::named("auth"));
    }
}

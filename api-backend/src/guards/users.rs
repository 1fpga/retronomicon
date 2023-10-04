use crate::db::Db;
use crate::models;
use crate::schema;
use crate::schema::users;
use diesel::prelude::*;
use jsonwebtoken::DecodingKey;
use retronomicon_dto as dto;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::{IntoOutcome, Outcome};
use rocket::{request, Request};
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use rocket_okapi::OpenApiFromRequest;
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use tracing::warn;

/// A user that is part of the root team.
#[derive(Debug, Clone, Serialize, Deserialize, OpenApiFromRequest)]
pub struct RootUserGuard {
    pub id: i32,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for RootUserGuard {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut db = request
            .guard::<Db>()
            .await
            .expect("database connection guard");
        let user = request.guard::<UserGuard>().await.expect("user guard");
        schema::user_teams::table
            .filter(schema::user_teams::user_id.eq(user.id))
            .filter(schema::user_teams::team_id.eq(1))
            .first::<models::UserTeam>(&mut db)
            .await
            .map_err(|e| e.to_string())
            .into_outcome(Status::Unauthorized)
            .map(|_| RootUserGuard { id: user.id })
    }
}

/// A user that went through the signed up process and has a username.
#[derive(Debug, Clone, Serialize, Deserialize, OpenApiFromRequest)]
pub struct AuthenticatedUserGuard {
    inner: UserGuard,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for AuthenticatedUserGuard {
    type Error = String;

    async fn from_request(
        request: &'r request::Request<'_>,
    ) -> request::Outcome<Self, Self::Error> {
        UserGuard::from_request(request).await.and_then(|user| {
            if let Some(user) = user.into() {
                Outcome::Success(user)
            } else {
                Outcome::Forward(Status::Unauthorized)
            }
        })
    }
}

impl From<AuthenticatedUserGuard> for dto::user::UserIdOrUsername<'static> {
    fn from(user: AuthenticatedUserGuard) -> Self {
        Self::Id(user.id)
    }
}

impl Deref for AuthenticatedUserGuard {
    type Target = UserGuard;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AuthenticatedUserGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl TryFrom<UserGuard> for AuthenticatedUserGuard {
    type Error = &'static str;

    fn try_from(value: UserGuard) -> Result<Self, Self::Error> {
        if value.username.is_some() {
            Ok(Self { inner: value })
        } else {
            Err("User is not authenticated")
        }
    }
}

impl AuthenticatedUserGuard {
    pub fn into_inner(self) -> UserGuard {
        self.inner
    }

    pub fn username(&self) -> &str {
        // This is guaranteed to be Some because of the `into()` implementation.
        self.inner.username.as_ref().unwrap()
    }

    pub async fn into_model(self, db: &mut Db) -> Result<models::User, diesel::result::Error> {
        self.inner.into_model(db).await
    }
}

/// A potentially non-fully signed up user for the website.
#[derive(Clone, Debug, Serialize, Deserialize, OpenApiFromRequest)]
pub struct UserGuard {
    pub id: i32,
    pub username: Option<String>,

    pub exp: i64,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for UserGuard {
    type Error = String;

    async fn from_request(
        request: &'r request::Request<'_>,
    ) -> request::Outcome<UserGuard, Self::Error> {
        fn validate_exp(user: UserGuard) -> request::Outcome<UserGuard, String> {
            if chrono::Utc::now().timestamp() > user.exp {
                Outcome::Forward(Status::Unauthorized)
            } else {
                Outcome::Success(user)
            }
        }

        // Check cookies.
        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");
        if let Some(cookie) = cookies.get_private("auth") {
            return serde_json::from_str(cookie.value())
                .map_err(|e| e.to_string())
                .into_outcome(Status::Unauthorized)
                .and_then(validate_exp)
                .and_then(|user: UserGuard| {
                    user.update_cookie(cookies);
                    Outcome::Success(user)
                });
        }

        // Check JWT from the headers.
        request
            .headers()
            .get_one("Authorization")
            .ok_or("Unauthorized".to_string())
            .and_then(|key| UserGuard::decode_jwt(key).map_err(|e| e.to_string()))
            .into_outcome(Status::Unauthorized)
            .and_then(validate_exp)
    }
}

impl From<UserGuard> for Option<AuthenticatedUserGuard> {
    fn from(value: UserGuard) -> Self {
        AuthenticatedUserGuard::try_from(value).ok()
    }
}

impl<'a> From<&UserGuard> for Cookie<'a> {
    fn from(user: &UserGuard) -> Self {
        Cookie::build("auth", serde_json::to_string(user).unwrap())
            .same_site(rocket::http::SameSite::Lax)
            .finish()
    }
}

impl From<UserGuard> for Option<dto::user::UserRef> {
    fn from(user: UserGuard) -> Self {
        user.username.map(|username| dto::user::UserRef {
            id: user.id,
            username,
        })
    }
}

impl From<UserGuard> for dto::user::UserIdOrUsername<'static> {
    fn from(user: UserGuard) -> Self {
        Self::Id(user.id)
    }
}

impl UserGuard {
    pub fn new(id: i32, username: Option<String>, exp: i64) -> Result<Self, &'static str> {
        if exp < chrono::Utc::now().timestamp() {
            return Err("Invalid expiry");
        }
        if let Some(ref n) = username {
            dto::user::Username::new(n)?; // Validate username.
        }

        Ok(Self::new_unchecked(id, username, exp))
    }

    pub fn new_unchecked(id: i32, username: Option<String>, exp: i64) -> Self {
        Self { id, username, exp }
    }

    pub fn set_expiry(&mut self, expiry: i64) {
        self.exp = expiry;
    }

    pub async fn into_model(self, db: &mut Db) -> Result<models::User, diesel::result::Error> {
        use users::dsl;
        dsl::users
            .filter(dsl::id.eq(self.id))
            .first::<models::User>(db)
            .await
    }

    pub fn from_model(user: models::User) -> Self {
        Self::new_unchecked(user.id, user.username, default_expiration_())
    }

    pub async fn from_db(db: &mut Db, id: i32) -> Result<Self, diesel::result::Error> {
        use users::dsl;
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
        use users::dsl;
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
        email: &str,
        auth_provider: String,
        avatar_url: Option<String>,
    ) -> Result<(bool, models::User, Self), diesel::result::Error> {
        use users::dsl;
        db.transaction(|db| {
            async move {
                // Set username to None if it doesn't validate.
                let username = username.as_deref().and_then(|u| {
                    dto::user::Username::new(u)
                        .map(|u| u.into_inner())
                        .map_err(|e| {
                            warn!("login_from_auth: invalid username: {}", e);
                            e
                        })
                        .ok()
                });

                let maybe_user = dsl::users
                    .filter(dsl::email.eq(email))
                    .filter(dsl::auth_provider.eq(&auth_provider))
                    .first::<models::User>(db)
                    .await
                    .optional()?;

                if let Some(u) = maybe_user {
                    return Ok((false, u.clone(), Self::from_model(u)));
                }

                let user = models::User::create(
                    db,
                    username,
                    None,
                    avatar_url.as_deref(),
                    email,
                    Some(&auth_provider),
                    None,
                    json!({}),
                    json!({}),
                )
                .await?;

                Ok((true, user.clone(), Self::from_model(user)))
            }
            .scope_boxed()
        })
        .await
    }

    pub async fn update(
        &self,
        db: &mut Db,
        form: dto::user::UserUpdate<'_>,
    ) -> Result<(), diesel::result::Error> {
        #[derive(AsChangeset)]
        #[diesel(table_name = schema::users)]
        struct UserSignupChangeset<'a> {
            username: Option<&'a str>,
            display_name: Option<&'a str>,
            description: Option<&'a str>,
            links: Option<Value>,
            metadata: Option<Value>,
        }

        db.transaction(|db| {
            async move {
                let mut changeset = UserSignupChangeset {
                    username: form.username,
                    display_name: form.display_name,
                    description: form.description,
                    links: None,
                    metadata: None,
                };

                if let Some(links) = form.links.as_ref() {
                    changeset.links = Some(serde_json::to_value(links).unwrap());
                } else if form.add_links.is_some() || form.remove_links.is_some() {
                    let mut links = BTreeMap::new();
                    let user: models::User = schema::users::table.find(self.id).first(db).await?;

                    if let Value::Object(user_links) = user.links {
                        links.extend(user_links.into_iter());
                    }

                    if let Some(user_links) = form.add_links {
                        for (k, v) in user_links.into_iter() {
                            links.insert(k.to_string(), v.into());
                        }
                    }
                    if let Some(user_links) = form.remove_links {
                        for k in user_links.into_iter() {
                            links.remove(&k.to_string());
                        }
                    }

                    changeset.links = Some(serde_json::to_value(links).unwrap());
                }

                diesel::update(schema::users::table)
                    .filter(schema::users::id.eq(self.id))
                    .set(changeset)
                    .execute(db)
                    .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await
    }

    pub fn update_cookie(&self, cookies: &CookieJar<'_>) {
        // Set a private cookie with the user's name, and redirect to the home page.
        cookies.add_private(self.into());
    }

    pub fn decode_jwt(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
        let token = token.trim_start_matches("Bearer").trim();
        match jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
        ) {
            Ok(token) => Ok(token.claims),
            Err(e) => Err(e),
        }
    }

    pub fn create_jwt(mut self) -> Result<String, jsonwebtoken::errors::Error> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
        let expiration = default_expiration_();
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

fn default_expiration_() -> i64 {
    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("Invalid timestamp")
        .timestamp()
}

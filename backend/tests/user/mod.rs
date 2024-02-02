use anyhow::{anyhow, Error};
use backend::routes::v1;
use rand::distributions::Alphanumeric;
use rand::Rng;
use retronomicon_dto as dto;
use retronomicon_dto::types::IdOrSlug;
use retronomicon_dto::user::UserIdOrUsername;
use rocket::http::uri::Origin;
use rocket::http::{Cookie, Method, Status};
use rocket::local::asynchronous::Client;
use rocket::uri;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

static mut COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_id() -> usize {
    unsafe { COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) }
}

fn slugify(s: &str) -> String {
    format!(
        "slug-{}",
        s.to_lowercase()
            .chars()
            .map(|c| match c {
                'a'..='z' | '0'..='9' | '-' => c,
                ' ' => '-',
                _ => '-',
            })
            .collect::<String>()
    )
}

#[derive(Debug, Clone)]
pub enum User {
    NoAuth {
        client: Arc<Client>,
        cookie: Cookie<'static>,
        name: String,
        id: i32,
    },
    Auth {
        client: Arc<Client>,
        cookie: Cookie<'static>,
        name: String,
        id: i32,
    },
    Anonymous {
        client: Arc<Client>,
    },
}

impl User {
    fn is_authenticated(&self) -> bool {
        match self {
            User::Auth { .. } => true,
            _ => false,
        }
    }

    fn is_anonymous(&self) -> bool {
        match self {
            User::Anonymous { .. } => true,
            _ => false,
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            User::NoAuth { id, .. } | User::Auth { id, .. } => *id,
            User::Anonymous { .. } => -1,
        }
    }

    async fn req_<R: serde::de::DeserializeOwned>(
        client: &Client,
        method: Method,
        uri: Origin<'_>,
        cookie: &mut Cookie<'static>,
        body: &impl serde::Serialize,
    ) -> Result<R, Error> {
        let response = client
            .req(method, uri)
            .cookie(cookie.clone())
            .json(body)
            .dispatch()
            .await;

        if response.status() != Status::Ok {
            let status = response.status();
            let body = response.into_string().await;
            return Err(anyhow!(
                "Server returned status: {} body: {:?}",
                status,
                body
            ));
        }

        // Update the internal cookie.
        if let Some(c) = response.cookies().get("auth") {
            *cookie = c.clone();
        }

        let content = response
            .into_string()
            .await
            .ok_or_else(|| anyhow!("Could not deserialize from JSON: empty response."))?;
        serde_json::from_str(&content).map_err(|e| anyhow!(e))
    }

    async fn req<R: serde::de::DeserializeOwned>(
        &mut self,
        method: Method,
        uri: Origin<'_>,
        body: &impl serde::Serialize,
    ) -> Result<R, Error> {
        match self {
            User::NoAuth { client, cookie, .. } | User::Auth { client, cookie, .. } => {
                Self::req_(client, method, uri, cookie, body).await
            }
            User::Anonymous { client } => {
                Self::req_(client, method, uri, &mut Cookie::new("empty", ""), body).await
            }
        }
    }

    async fn get<R: serde::de::DeserializeOwned>(
        &mut self,
        uri: Origin<'_>,
        body: &impl serde::Serialize,
    ) -> Result<R, Error> {
        self.req(Method::Get, uri, body).await
    }

    async fn post<R: serde::de::DeserializeOwned>(
        &mut self,
        uri: Origin<'_>,
        body: &impl serde::Serialize,
    ) -> Result<R, Error> {
        self.req(Method::Post, uri, body).await
    }

    async fn put<R: serde::de::DeserializeOwned>(
        &mut self,
        uri: Origin<'_>,
        body: &impl serde::Serialize,
    ) -> Result<R, Error> {
        self.req(Method::Put, uri, body).await
    }

    async fn delete<R: serde::de::DeserializeOwned>(
        &mut self,
        uri: Origin<'_>,
        body: &impl serde::Serialize,
    ) -> Result<R, Error> {
        self.req(Method::Delete, uri, body).await
    }

    fn gen_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect::<String>()
    }

    fn create_email(domain: &str) -> String {
        format!("{}@{}", Self::gen_string(20), domain)
    }

    fn create_username(prefix: &str) -> String {
        format!("{}_{}", prefix, Self::gen_string(10)).to_lowercase()
    }

    fn create_slug(prefix: &str) -> String {
        slugify(&format!("{}-{}", prefix, Self::gen_string(10)))
    }

    pub async fn anonymous(client: Arc<Client>) -> Result<Self, Error> {
        Ok(Self::Anonymous { client })
    }

    pub async fn admin(client: Arc<Client>, name: &str) -> Result<Self, Error> {
        let mut result = Self::create(client, &format!("cucumber-admin-{name}")).await?;
        result.authenticate().await?;
        Ok(result)
    }

    pub async fn user(client: Arc<Client>, name: &str) -> Result<Self, Error> {
        Self::create(client, &format!("user-{name}")).await
    }

    async fn create(client: Arc<Client>, name: &str) -> Result<Self, Error> {
        let (cookie, id) = {
            let email = Self::create_email(name);
            let password = email.as_str();
            let user = client
                .post(uri!(v1::auth::signup()))
                .json(&dto::auth::SignupRequest {
                    email: &email,
                    password,
                })
                .dispatch()
                .await;

            if user.status() != Status::Ok {
                return Err(anyhow!(
                    "Failed to create user: {:?}",
                    user.into_string().await.expect("No response body")
                ));
            }

            let cookie = user.cookies().get("auth").expect("No auth cookie").clone();
            let id = user
                .into_json::<dto::auth::SignupResponse>()
                .await
                .expect("Failed to deserialize response")
                .id;

            (cookie, id)
        };

        Ok(Self::NoAuth {
            client,
            cookie,
            id,
            name: name.to_string(),
        })
    }

    pub async fn authenticate(&mut self) -> Result<(), Error> {
        match self {
            User::Anonymous { .. } => Err(anyhow!("Cannot authenticate anonymous user.")),
            User::Auth { .. } => Ok(()),
            User::NoAuth {
                client,
                cookie,
                id,
                name,
            } => {
                // `self` is already borrowed, so can't borrow twice.
                Self::req_::<dto::Ok>(
                    client,
                    Method::Put,
                    uri!(v1::me::me_update()),
                    cookie,
                    &dto::user::UserUpdate {
                        username: Some(&Self::create_username(name)),
                        ..Default::default()
                    },
                )
                .await?;

                *self = Self::Auth {
                    client: client.clone(),
                    cookie: cookie.clone(),
                    name: name.clone(),
                    id: *id,
                };
                Ok(())
            }
        }
    }

    pub async fn whoami(&mut self) -> Result<dto::user::UserDetails, Error> {
        self.get(uri!(v1::me::me()), &()).await
    }

    pub async fn create_team(
        &mut self,
        name: &str,
    ) -> Result<dto::teams::TeamCreateResponse, Error> {
        self.post(
            uri!(v1::teams::teams_create()),
            &dto::teams::TeamCreateRequest {
                name,
                slug: &Self::create_slug(name),
                description: "",
                links: None,
                metadata: None,
            },
        )
        .await
    }

    pub async fn invite_to_team(
        &mut self,
        team: i32,
        user: i32,
        role: dto::types::UserTeamRole,
    ) -> Result<(), Error> {
        self.post::<dto::Ok>(
            uri!(v1::teams::invite(team)),
            &dto::teams::TeamInvite {
                user: dto::user::UserIdOrUsername::Id(user),
                role,
            },
        )
        .await?;
        Ok(())
    }

    pub async fn accept_team_invitation(&mut self, team: i32) -> Result<(), Error> {
        self.post::<dto::Ok>(uri!(v1::teams::invite_accept(team)), &())
            .await?;
        Ok(())
    }

    pub async fn set_info(&mut self, info: dto::user::UserUpdate<'_>) -> Result<(), Error> {
        let username = info.username.map(Self::create_username);
        self.put::<dto::Ok>(
            uri!(v1::me::me_update()),
            &dto::user::UserUpdate {
                username: username.as_deref(),
                ..info
            },
        )
        .await?;

        Ok(())
    }

    pub async fn team_details(&mut self, team: i32) -> Result<dto::teams::TeamDetails, Error> {
        self.get(uri!(v1::teams::teams_details(team)), &()).await
    }

    pub async fn create_system(
        &mut self,
        team: i32,
        name: &str,
    ) -> Result<dto::systems::SystemCreateResponse, Error> {
        let owner_team = self.team_details(team).await?.team.into();
        let name = Self::create_username(name);
        self.post(
            uri!(v1::systems::systems_create()),
            &dto::systems::SystemCreateRequest {
                name: &name,
                slug: &Self::create_slug(&name),
                description: "",
                manufacturer: "cucumber-manufacturer",
                links: None,
                metadata: None,
                owner_team,
            },
        )
        .await
    }

    pub async fn create_game(
        &mut self,
        system_id: i32,
        name: &str,
    ) -> Result<dto::games::GameCreateResponse, Error> {
        let system = self
            .get::<dto::systems::SystemDetails>(uri!(v1::systems::systems_details(system_id)), &())
            .await?
            .id;
        let system = system.into();

        self.post(
            uri!(v1::games::games_create()),
            &dto::games::GameCreateRequest {
                name,
                description: "",
                short_description: "",
                year: 1234,
                publisher: "cucumber-publisher",
                developer: "cucumber-developer",
                links: BTreeMap::new(),
                system,
                system_unique_id: unique_id() as i32,
            },
        )
        .await
    }

    pub async fn get_game_by_id(&mut self, game_id: i32) -> Result<dto::games::GameDetails, Error> {
        self.get(uri!(v1::games::games_details(game_id as u32)), &())
            .await
    }

    pub async fn get_user_details(
        &mut self,
        user: Option<UserIdOrUsername<'_>>,
    ) -> Result<dto::user::UserDetails, Error> {
        match user {
            Some(user) => self.get(uri!(v1::users::users_details(user)), &()).await,
            None => self.whoami().await,
        }
    }
}

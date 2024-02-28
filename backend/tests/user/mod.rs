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
use std::io::Cursor;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

fn create_image(text: String) -> Vec<u8> {
    use embedded_graphics::framebuffer::{buffer_size, Framebuffer};
    use embedded_graphics::mono_font::{ascii, MonoTextStyle};
    use embedded_graphics::pixelcolor::raw::LittleEndian;
    use embedded_graphics::pixelcolor::Rgb888;
    use embedded_graphics::prelude::*;
    use embedded_graphics::text::{Alignment, Text};

    // Create an image.
    let mut display =
        Framebuffer::<Rgb888, _, LittleEndian, 320, 240, { buffer_size::<Rgb888>(320, 240) }>::new(
        );

    let character_style = MonoTextStyle::new(&ascii::FONT_10X20, Rgb888::new(255, 255, 255));
    Text::with_alignment(
        &text,
        display.bounding_box().center() + Point::new(0, 0),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .expect("Could not draw text");

    let image = image::RgbImage::from_raw(320, 240, display.data().to_vec())
        .expect("Failed to create image");
    let mut bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .expect("Failed to write image.");

    bytes
}

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

    fn gen_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect::<String>()
    }

    fn create_email(domain: &str) -> String {
        format!("{}@{}", Self::gen_string(10), domain)
    }

    fn create_username(prefix: &str) -> String {
        format!("{}_{}", prefix, Self::gen_string(5)).to_lowercase()
    }

    fn create_slug(prefix: &str) -> String {
        slugify(&format!("{}-{}", prefix, Self::gen_string(5)))
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
                    username: None,
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

    pub async fn get_game_images(
        &mut self,
        game_id: i32,
    ) -> Result<dto::Paginated<dto::images::Image>, Error> {
        self.get(
            uri!(v1::games::games_images(
                game_id as u32,
                dto::params::PagingParams::default()
            )),
            &(),
        )
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

    pub async fn upload_image(&mut self, game_id: i32, image_name: &str) -> Result<(), Error> {
        let bytes = create_image(format!("{game_id} / {image_name}.png"));
        let cookie = match self {
            User::NoAuth { cookie, .. } | User::Auth { cookie, .. } => cookie.clone(),
            User::Anonymous { .. } => Cookie::new("empty", ""),
        };
        let mut request = match self {
            User::NoAuth { client, .. } | User::Auth { client, .. } => client,
            User::Anonymous { client } => client,
        }
        .req(Method::Post, uri!(v1::games::games_images_upload(game_id)))
        .cookie(cookie);

        // Build the form manually. This is very cobbersome but Rocket doesn't provide a better
        // API just yet. See https://github.com/rwf2/Rocket/issues/1591.
        let form = [
            b"-----testboundary\r\n".to_vec(),
            format!(
                "Content-Disposition: form-data; name=\"image\"; filename=\"{image_name}.png\"\r\n"
            )
            .as_bytes()
            .to_vec(),
            b"Content-Type: image/png\r\n".to_vec(),
            b"\r\n".to_vec(),
            bytes,
            b"\r\n".to_vec(),
            b"-----testboundary--\r\n".to_vec(),
        ]
        .concat();
        request.add_header(rocket::http::Header::new(
            "Content-Type",
            "multipart/form-data; boundary=---testboundary",
        ));
        request.set_body(form);

        let response = request.dispatch().await;
        if response.status() != Status::Ok {
            return Err(anyhow!(
                "Server returned status: {} body: {:?}",
                response.status(),
                response.into_string().await
            ));
        }

        Ok(())
    }
}

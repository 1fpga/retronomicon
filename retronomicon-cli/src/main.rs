use anyhow::Error;
use clap::Parser;
use clap_verbosity_flag::Level as VerbosityLevel;
use clap_verbosity_flag::Verbosity;
use reqwest::{RequestBuilder, StatusCode};
use retronomicon_dto as dto;
use serde::{Deserialize, Serialize};
use tracing::{debug, Level};
use tracing_subscriber::fmt::Subscriber;
use url::Url;

#[derive(Debug, Parser)]
struct Opts {
    #[command(subcommand)]
    pub command: Command,

    /// Server to connect to.
    // In debug mode this is set to localhost:8000,  while in production this is set to
    // retronomicon.com.
    #[clap(long, env = "RETRONOMICON_SERVER")]
    #[cfg_attr(debug_assertions, clap(default_value = "http://localhost:8000/"))]
    #[cfg_attr(
        not(debug_assertions),
        clap(default_value = "https://retronomicon.com/")
    )]
    pub server: Url,

    /// A token to use for authentication.
    #[clap(long, env = "RETRONOMICON_TOKEN")]
    pub token: Option<String>,

    #[command(flatten)]
    pub verbose: Verbosity,
}

#[derive(Debug, Parser)]
enum Command {
    /// Returns the authentication information.
    Whoami,

    /// Update the user information.
    UserUpdate(UpdateUser),

    /// List users.
    UsersList(UsersList),

    /// Get a user's details.
    UserGet(UserGet),
}

#[derive(Debug, Parser)]
pub struct PagingParams {
    /// The page to download.
    #[clap(long)]
    page: Option<u32>,
    /// The maximum number of items to return.
    #[clap(long)]
    limit: Option<u32>,
}

impl PagingParams {
    pub fn to_query(&self) -> String {
        match (self.page, self.limit) {
            (None, None) => "".to_string(),
            (Some(page), None) => format!("page={page}"),
            (None, Some(limit)) => format!("limit={limit}"),
            (Some(page), Some(limit)) => format!("page={page}&limit={limit}"),
        }
    }
}

#[derive(Debug, Parser)]
pub struct UpdateUser {
    /// The user's name.
    #[clap(long)]
    pub username: Option<String>,

    /// The user's description.
    #[clap(long)]
    pub description: Option<String>,

    /// Add a link to the user's links. This is a key-value pair, separated by an equal sign.
    #[clap(long)]
    pub add_link: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct UsersList {
    #[clap(flatten)]
    paging: PagingParams,
}

#[derive(Debug, Parser)]
pub struct UserGet {
    /// The user's name or numerical id.
    id: String,
}

fn update_request<B: Serialize>(
    mut request: RequestBuilder,
    opts: &Opts,
    body: Option<B>,
) -> RequestBuilder {
    if let Some(body) = body {
        request = request.json(&body);
    }

    if let Some(token) = &opts.token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    request
}

async fn get<R>(path: &str, opts: &Opts) -> Result<R, Error>
where
    R: for<'de> Deserialize<'de>,
{
    let client = reqwest::Client::new();
    let request = update_request::<()>(
        client.request(reqwest::Method::GET, opts.server.join(path)?),
        opts,
        None,
    )
    .build()?;

    let response = client.execute(request).await?;

    match response.status() {
        StatusCode::OK => response.json().await.map_err(Into::into),
        code => Err(Error::msg(format!("Status code: {}", code))),
    }
}

async fn post<Q, R>(path: &str, opts: &Opts, request: Q) -> Result<R, Error>
where
    Q: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let client = reqwest::Client::new();
    let request = update_request(
        client.request(reqwest::Method::POST, opts.server.join(path)?),
        opts,
        Some(request),
    )
    .build()?;

    let response = client.execute(request).await?;

    match response.status() {
        StatusCode::OK => response.json().await.map_err(Into::into),
        code => Err(Error::msg(format!("Status code: {}", code))),
    }
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    debug!(?opts);

    // Initialize tracing.
    let subscriber = Subscriber::builder();
    let subscriber = match opts.verbose.log_level() {
        Some(VerbosityLevel::Error) => subscriber.with_max_level(Level::ERROR),
        Some(VerbosityLevel::Warn) => subscriber.with_max_level(Level::WARN),
        Some(VerbosityLevel::Info) => subscriber.with_max_level(Level::INFO),
        Some(VerbosityLevel::Debug) => subscriber.with_max_level(Level::DEBUG),
        None | Some(VerbosityLevel::Trace) => subscriber.with_max_level(Level::TRACE),
    };
    subscriber
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    match opts.command {
        Command::Whoami => {
            let response: dto::user::Me = get("/api/me", &opts).await.unwrap();
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Command::UserUpdate(ref user) => {
            let response: dto::Ok = post(
                "/api/me/update",
                &opts,
                dto::user::UserUpdate {
                    username: user.username.as_deref(),
                    display_name: None,
                    description: user.description.as_deref(),
                    links: None,
                    metadata: None,
                    add_links: user
                        .add_link
                        .iter()
                        .map(|x| x.split_once('=').unwrap())
                        .collect(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Command::UsersList(ref users) => {
            let query = format!("/api/users?{}", users.paging.to_query());

            let response: Vec<dto::user::UserRef> = get(&query, &opts).await.unwrap();
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Command::UserGet(ref user) => {
            let response: dto::user::UserDetails = get(&format!("/api/users/{}", user.id), &opts)
                .await
                .unwrap();
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
    }
}

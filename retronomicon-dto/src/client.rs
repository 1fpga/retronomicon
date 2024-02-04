#[cfg(feature = "client")]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid url")]
    InvalidUrl(#[from] url::ParseError),
    #[error("invalid token")]
    InvalidToken(#[from] reqwest::header::InvalidHeaderValue),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Server error: {0}\n{1}")]
    ServerError(reqwest::StatusCode, String),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

pub const DEFAULT_SERVER_URL: &str = "https://retronomicon.land/";

#[cfg(feature = "client")]
#[derive(Default, Debug, Clone)]
pub struct ClientConfig<'a> {
    pub url_base: Option<url::Url>,
    pub token: Option<&'a str>,
}

#[cfg(feature = "client")]
impl<'a> ClientConfig<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_url(mut self, url: impl AsRef<str>) -> Result<Self, Error> {
        self.url_base = Some(url::Url::parse(url.as_ref())?);
        Ok(self)
    }

    pub fn with_token(mut self, token: &'a str) -> Self {
        self.token = Some(token);
        self
    }
}

macro_rules! declare_client_impl {
    ($async_or_blocking: ident) => {
        declare_client! {
            $async_or_blocking;

            get users(
                ("users"),
                @query paging: &crate::params::PagingParams,
            ) -> Vec<crate::user::UserRef>;
            get users_details(
                ("users/{id}", id: &crate::user::UserIdOrUsername<'_>),
            ) -> crate::user::UserDetails;
            put users_update(
                ("users/{id}", id: &crate::user::UserIdOrUsername<'_>),
                @body body: &crate::user::UserUpdate<'_>,
            ) -> crate::Ok;
            put me_update(
                ("me"),
                @body body: &crate::user::UserUpdate<'_>,
            ) -> crate::Ok;

            get cores(
                ("cores"),
                @query paging: &crate::params::PagingParams,
            ) -> Vec<crate::cores::CoreListItem>;
            get cores_details(
                ("cores/{id}", id: &crate::types::IdOrSlug<'_>),
            ) -> crate::cores::CoreDetailsResponse;
            post cores_create(
                ("cores"),
                @body body: &crate::cores::CoreCreateRequest<'_>,
            ) -> crate::cores::CoreCreateResponse;

            get cores_releases(
                ("cores/{id}/releases", id: &crate::types::IdOrSlug<'_>),
                @query paging: &crate::params::PagingParams,
            ) -> Vec<crate::cores::releases::CoreReleaseListItem>;
            get cores_releases_artifacts(
                (
                    "cores/{core_id}/releases/{release_id}/artifacts",
                    core_id: &crate::types::IdOrSlug<'_>,
                    release_id: i32,
                ),
                @query paging: &crate::params::PagingParams,
            ) -> Vec<crate::artifact::CoreReleaseArtifactListItem>;
            post cores_releases_create(
                ("cores/{id}/releases", id: &crate::types::IdOrSlug<'_>),
                @body body: &crate::cores::releases::CoreReleaseCreateRequest<'_>,
            ) -> crate::cores::releases::CoreReleaseCreateResponse;
            post cores_releases_artifacts_upload(
                (
                    "cores/{core_id}/releases/{release_id}/artifacts",
                    core_id: &crate::types::IdOrSlug<'_>,
                    release_id: i32,
                ),
                @file file,
            ) -> Vec<crate::artifact::ArtifactCreateResponse>;

            get games(
                ("games"),
                @query paging: &crate::games::GameListQueryParams<'_>,
                @body filter: &crate::games::GameListBody,
            ) -> Vec<crate::games::GameListItemResponse>;
            get games_details(
                ("games/{id}", id: i32),
            ) -> crate::games::GameDetails;
            post games_create(
                ("games"),
                @body body: &crate::games::GameCreateRequest<'_>,
            ) -> crate::games::GameCreateResponse;
            put games_update(
                ("games/{id}", id: i32),
                @body body: &crate::games::GameUpdateRequest<'_>,
            ) -> crate::Ok;
            post games_add_artifact(
                ("games/{id}/artifacts", id: i32),
                @body body: &Vec<crate::games::GameAddArtifactRequest<'_>>,
            ) -> crate::Ok;
            get games_images(
                ("games/{id}/images", id: i32),
                @query paging: &crate::params::PagingParams,
            ) -> Vec<crate::images::Image>;
            post games_add_image(
                ("games/{id}/images", id: i32),
                @file file,
            ) -> Vec<crate::images::Image>;
        }
    };
}

macro_rules! declare_client {
    (
        url;

        $(
            $(#[$fattr:meta])*
            $method: ident $fname: ident(
                (
                    $url: literal $(,)?
                    $( $path_name: ident: $path_type: ty ),*
                    $(,)?
                ),
                $(@query $query_name: ident: $query_type: ty, )*
                $(@body $body_name: ident: $body_type: ty, )*
                $(@file $file_name: ident $(,)? )*
            ) -> $rtype: ty;
        )*
    ) => {
        $(
            $(#[$fattr:meta])*
            pub fn $fname(url: &Url, $( $path_name: $path_type, )*) -> Url {
                url.join(BASE).unwrap().join(&format!($url)).unwrap()
            }
        )*
    };

    (
        async;

        $(
            $(#[$fattr:meta])*
            $method: ident $fname: ident(
                (
                    $url: literal $(,)?
                    $( $path_name: ident: $path_type: ty ),*
                    $(,)?
                ),
                $(@query $query_name: ident: $query_type: ty, )*
                $(@body $body_name: ident: $body_type: ty, )*
                $(@file $file_name: ident $(,)? )*
            ) -> $rtype: ty;
        )*
    ) => {
        $(
            $(#[$fattr:meta])*
            pub async fn $fname(
                &self,
                $( $path_name: $path_type, )*
                $( $query_name: $query_type, )*
                $( $body_name: $body_type, )*
                $( $file_name: &std::path::Path, )*
            ) -> Result<$rtype, super::Error> {
                let request = self.1
                    . $method (crate::routes::v1:: $fname ( &self.0, $( $path_name, )* ))
                    $(.query( $query_name ))*
                    $(.json( $body_name ))*
                ;

                $(
                    let path = $file_name.to_path_buf();
                    let file_name = path
                        .file_name()
                        .map(|filename| filename.to_string_lossy().into_owned());
                    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    let mime = mime_guess::from_ext(ext).first_or_octet_stream();
                    let file = std::fs::read(path)?;
                    let field = reqwest::multipart::Part::bytes(file).mime_str(&mime.to_string()).unwrap();

                    let field = if let Some(file_name) = file_name {
                        field.file_name(file_name)
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "File name not found",
                        ).into());
                    };

                    let form = reqwest::multipart::Form::new().part(stringify!($file_name), field);
                    let request = request.multipart(form);
                )*

                let response = request
                    .send()
                    .await?;

                if response.status().is_success() {
                    Ok(response
                        .json()
                        .await?)
                } else {
                    let status = response.status();
                    let body = response.text().await?;
                    Err(crate::client::Error::ServerError(status, body))
                }
            }
        )*
    };

    (
        blocking;

        $(
            $(#[$fattr:meta])*
            $method: ident $fname: ident(
                (
                    $url: literal $(,)?
                    $( $path_name: ident: $path_type: ty ),*
                    $(,)?
                ),
                $(@query $query_name: ident: $query_type: ty, )*
                $(@body $body_name: ident: $body_type: ty, )*
                $(@file $file_name: ident $(,)? )*
            ) -> $rtype: ty;
        )*
    ) => {
        $(
            $(#[$fattr:meta])*
            pub fn $fname(
                &self,
                $( $path_name: $path_type, )*
                $( $query_name: $query_type, )*
                $( $body_name: $body_type, )*
                $( $file_name: &std::path::Path, )*
            ) -> Result<$rtype, super::Error> {
                let request = self.1
                    . $method (crate::routes::v1:: $fname ( &self.0, $( $path_name, )* ))
                    $(.query( $query_name ))*
                    $(.json( $body_name ))*
                ;

                $(
                    let path = $file_name.to_path_buf();
                    let file_name = path
                        .file_name()
                        .map(|filename| filename.to_string_lossy().into_owned());
                    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                    let mime = mime_guess::from_ext(ext).first_or_octet_stream();
                    let file = std::fs::read(path)?;
                    let field = reqwest::blocking::multipart::Part::bytes(file).mime_str(&mime.to_string()).unwrap();

                    let field = if let Some(file_name) = file_name {
                        field.file_name(file_name)
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "File name not found",
                        ).into());
                    };

                    let form = reqwest::blocking::multipart::Form::new().part(stringify!($file_name), field);
                    let request = request.multipart(form);
                )*

                let response = request
                    .send()?;

                if response.status().is_success() {
                    Ok(response.json()?)
                } else {
                    let status = response.status();
                    let body = response.text()?;
                    Err(crate::client::Error::ServerError(status, body))
                }
            }
        )*
    };
}

pub mod routes {
    pub mod v1 {
        use url::Url;

        pub const BASE: &str = "/api/v1/";

        declare_client_impl!(url);
    }
}

#[cfg(feature = "client")]
pub mod v1 {
    use crate::client::ClientConfig;
    use reqwest::header;
    use reqwest::{Client, Url};

    pub struct V1Client(Url, Client);

    #[cfg(feature = "blocking")]
    pub struct BlockingV1Client(Url, reqwest::blocking::Client);

    impl V1Client {
        fn client(auth_token: Option<&str>) -> Result<reqwest::Client, reqwest::Error> {
            if let Some(token) = auth_token {
                let mut headers = header::HeaderMap::new();
                let mut auth_value =
                    header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();
                auth_value.set_sensitive(true);
                headers.insert(header::AUTHORIZATION, auth_value);

                Client::builder().default_headers(headers)
            } else {
                Client::builder()
            }
            .cookie_store(true)
            .build()
        }

        pub fn new(ClientConfig { url_base, token }: ClientConfig) -> Result<Self, String> {
            let url = url_base.unwrap_or_else(|| Url::parse(super::DEFAULT_SERVER_URL).unwrap());
            Ok(Self(url, Self::client(token).map_err(|e| e.to_string())?))
        }

        declare_client_impl!(async);
    }

    #[cfg(feature = "blocking")]
    impl BlockingV1Client {
        fn client(auth_token: Option<&str>) -> Result<reqwest::blocking::Client, reqwest::Error> {
            let mut headers = header::HeaderMap::new();
            if let Some(token) = auth_token {
                let mut auth_value =
                    header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();
                auth_value.set_sensitive(true);
                headers.insert(header::AUTHORIZATION, auth_value);
            }

            let client = reqwest::blocking::Client::builder()
                .cookie_store(true)
                .default_headers(headers);

            client.build()
        }

        pub fn new(ClientConfig { url_base, token }: ClientConfig) -> Result<Self, String> {
            let url = url_base.unwrap_or_else(|| Url::parse(super::DEFAULT_SERVER_URL).unwrap());
            Ok(Self(url, Self::client(token).map_err(|e| e.to_string())?))
        }

        declare_client_impl!(blocking);
    }
}

#[cfg(feature = "blocking")]
pub use v1::BlockingV1Client;
#[cfg(feature = "client")]
pub use v1::V1Client;

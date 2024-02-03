use reqwest::Url;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use s3::creds::error::CredentialsError;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::Bucket;
use s3::Region;
use serde::Deserialize;
use std::str::Utf8Error;

#[derive(Clone, Deserialize)]
pub struct StorageConfig {
    region: String,
    /// The base URL for the S3 bucket. This is used to construct the URL for the uploaded file.
    /// If this is not set, and the *_url_base fields are not set, the server will panic when
    /// using any storage functionality.
    url_base: Option<String>,

    access_key: String,
    secret_key: String,

    cores_bucket: String,
    cores_url_base: Option<String>,

    games_bucket: String,
    games_url_base: Option<String>,
}

impl StorageConfig {
    pub fn credentials(&self) -> Result<Credentials, CredentialsError> {
        Credentials::new(
            Some(self.access_key.as_str()),
            Some(self.secret_key.as_str()),
            None,
            None,
            None,
        )
    }

    pub fn region(&self) -> Result<Region, Utf8Error> {
        Ok(Region::Custom {
            region: "eu-central-1".to_owned(),
            endpoint: self.region.clone(),
        })
    }

    pub fn url_base(&self) -> Result<&str, String> {
        self.url_base
            .as_deref()
            .ok_or_else(|| "No URL base set for storage".to_string())
    }

    pub fn cores_url_base(&self) -> &str {
        self.cores_url_base
            .as_deref()
            .unwrap_or_else(|| self.url_base().expect("No URL base set for S3 cores"))
    }

    pub fn games_url_base(&self) -> &str {
        self.games_url_base
            .as_deref()
            .unwrap_or_else(|| self.url_base().expect("No URL base set for S3 cores"))
    }
}

pub struct Storage {
    config: StorageConfig,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Storage {
    type Error = String;

    async fn from_request(
        request: &'a rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let config = match request
            .rocket()
            .figment()
            .extract_inner::<StorageConfig>("s3")
        {
            Ok(storage) => storage,
            Err(e) => return Outcome::Error((Status::InternalServerError, e.to_string())),
        };

        Outcome::Success(Self {
            config: config.clone(),
        })
    }
}

impl Storage {
    async fn bucket(&self, bucket_name: &str, public: bool) -> Result<Bucket, S3Error> {
        let credentials = self.config.credentials()?;
        let region: Region = self.config.region()?;
        let mut bucket =
            Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

        if public {
            bucket.add_header("x-amz-acl", "public-read");
        }

        Ok(bucket)
    }

    async fn upload(
        &self,
        bucket_name: &str,
        public: bool,
        filename: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<(), String> {
        let bucket = self
            .bucket(bucket_name, public)
            .await
            .map_err(|e| e.to_string())?;

        let response = match bucket
            .put_object_with_content_type(&filename, data, content_type)
            .await
        {
            Ok(response) => response,
            Err(e) => {
                rocket::error!("Failed to upload file to S3: {}", e);
                return Err(e.to_string());
            }
        };

        if response.status_code() != 200 {
            rocket::error!("Failed to upload file to S3: {}", response.status_code());
            return Err(format!(
                "Failed to upload file to S3: {}",
                response.status_code()
            ));
        }
        Ok(())
    }

    pub async fn upload_core(
        &self,
        filename: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        self.upload(
            self.config.cores_bucket.as_str(),
            true,
            filename,
            data,
            content_type,
        )
        .await?;
        let url = Url::parse(&format!("{}/{}", self.config.cores_url_base(), filename)).unwrap();

        Ok(url.to_string())
    }

    pub async fn upload_game_asset(
        &self,
        filename: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        self.upload(
            self.config.games_bucket.as_str(),
            true,
            filename,
            data,
            content_type,
        )
        .await?;
        let url = Url::parse(&format!("{}/{}", self.config.games_url_base(), filename)).unwrap();

        Ok(url.to_string())
    }

    pub fn path_for_game_image(&self, id: i32, filename: &str) -> String {
        format!("games/{}/images/{}", id, filename)
    }

    pub fn url_for_game_image(&self, id: i32, filename: &str) -> Result<String, String> {
        let url = Url::parse(&format!(
            "{}/{}",
            self.config.games_url_base(),
            self.path_for_game_image(id, filename)
        ))
        .map_err(|e| e.to_string())?;
        Ok(url.to_string())
    }
}

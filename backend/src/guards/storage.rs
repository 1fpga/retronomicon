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
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub cores_bucket: String,
    pub cores_url_base: String,
    pub images_bucket: String,
    pub images_url_base: String,
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
        self.region.parse()
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
            &self.config.cores_bucket,
            true,
            filename,
            data,
            content_type,
        )
        .await?;
        let url = Url::parse(&format!("{}/{}", self.config.cores_url_base, filename)).unwrap();

        Ok(url.to_string())
    }

    pub async fn upload_image(
        &self,
        filename: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        self.upload(
            &self.config.images_bucket,
            true,
            filename,
            data,
            content_type,
        )
        .await?;
        let url = Url::parse(&format!("{}/{}", self.config.images_url_base, filename)).unwrap();

        Ok(url.to_string())
    }
}

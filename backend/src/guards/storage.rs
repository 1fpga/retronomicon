use reqwest::Url;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::Bucket;
use s3::Region;

#[derive(Clone)]
pub struct StorageConfig {
    pub region: String,
    pub cores_bucket: String,
    pub cores_url_base: String,
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
        let config = match request.rocket().state::<StorageConfig>() {
            Some(storage) => storage,
            None => {
                return Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    "Storage not configured".to_string(),
                ))
            }
        };

        Outcome::Success(Self {
            config: config.clone(),
        })
    }
}

impl Storage {
    async fn bucket(&self, bucket_name: &str, public: bool) -> Result<Bucket, S3Error> {
        let credentials = Credentials::default()?;
        let region = Region::Custom {
            region: "eu-central-1".to_owned(),
            endpoint: self.config.region.clone(),
        };

        let mut bucket =
            Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

        if public {
            bucket.add_header("x-amz-acl", "public-read");
        }

        Ok(bucket)
    }

    pub async fn upload_core(
        &self,
        filename: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        let bucket = self
            .bucket(&self.config.cores_bucket, true)
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

        let url = Url::parse(&format!("{}/{}", self.config.cores_url_base, filename)).unwrap();

        Ok(url.to_string())
    }
}

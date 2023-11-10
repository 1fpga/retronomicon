use crate::guards::storage::Storage;
use reqwest::Url;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use s3::error::S3Error;
use s3::Bucket;

pub struct CoreBucketStorage {
    pub storage: Storage,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for CoreBucketStorage {
    type Error = String;

    async fn from_request(
        request: &'a rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let storage = match request.guard::<Storage>().await {
            Outcome::Success(storage) => storage,
            Outcome::Failure((status, message)) => {
                return Outcome::Failure((status, message));
            }
            Outcome::Forward(status) => {
                return Outcome::Forward(status);
            }
        };

        Outcome::Success(Self::new(storage))
    }
}

impl CoreBucketStorage {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn bucket(&self) -> Result<Bucket, S3Error> {
        self.storage.bucket("cores", true).await
    }

    pub async fn upload(
        &self,
        filename: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        let bucket = self.bucket().await.map_err(|e| e.to_string())?;

        let response = match bucket
            .put_object_with_content_type(filename, data, content_type)
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

        let url = Url::parse(&format!("{}/{}/{}", bucket.host(), bucket.name(), filename)).unwrap();

        Ok(url.to_string())
    }
}

impl From<Storage> for CoreBucketStorage {
    fn from(value: Storage) -> Self {
        Self::new(value)
    }
}

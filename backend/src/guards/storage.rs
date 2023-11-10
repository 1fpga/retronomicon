use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use s3::bucket_ops::CannedBucketAcl;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::Region;
use s3::{Bucket, BucketConfiguration};

pub mod buckets;
pub use buckets::*;

pub struct StorageState {
    pub region: String,
}

pub struct Storage {
    region: String,
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Storage {
    type Error = String;

    async fn from_request(
        request: &'a rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let state = match request.rocket().state::<StorageState>() {
            Some(storage) => storage,
            None => {
                return Outcome::Failure((
                    rocket::http::Status::InternalServerError,
                    "Storage not configured".to_string(),
                ))
            }
        };

        Outcome::Success(Self {
            region: state.region.clone(),
        })
    }
}

impl Storage {
    pub async fn bucket(&self, bucket_name: &str, public: bool) -> Result<Bucket, S3Error> {
        let credentials = Credentials::default()?;

        let region = Region::Custom {
            region: "eu-central-1".to_owned(),
            endpoint: self.region.clone(),
        };

        let mut bucket =
            Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

        if !bucket.exists().await? {
            let config = if public {
                BucketConfiguration::private()
            } else {
                BucketConfiguration::new(
                    Some(CannedBucketAcl::PublicRead),
                    false,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
            };
            rocket::info!("Creating S3 bucket: {}", bucket_name);
            bucket = Bucket::create_with_path_style(bucket_name, region, credentials, config)
                .await?
                .bucket;
        }

        Ok(bucket)
    }
}

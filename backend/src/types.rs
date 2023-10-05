use crate::db::Db;
use retronomicon_dto as dto;
use rocket::http::Status;

#[rocket::async_trait]
pub trait FromIdOrSlug {
    async fn from_id(db: &mut Db, id: i32) -> Result<Option<Self>, diesel::result::Error>
    where
        Self: Sized;
    async fn from_slug(db: &mut Db, slug: &str) -> Result<Option<Self>, diesel::result::Error>
    where
        Self: Sized;
}

#[rocket::async_trait]
pub trait FetchModel<T: FromIdOrSlug> {
    async fn from_id_or_slug(
        db: &mut Db,
        id: dto::types::IdOrSlug<'_>,
    ) -> Result<T, (Status, String)> {
        if let Some(id) = id.as_id() {
            Self::from_id(db, id).await
        } else if let Some(slug) = id.as_slug() {
            Self::from_slug(db, slug).await
        } else {
            Err((Status::BadRequest, "Invalid id or slug".to_string()))
        }
    }
    async fn from_id(db: &mut Db, id: i32) -> Result<T, (Status, String)>;
    async fn from_slug(db: &mut Db, slug: &str) -> Result<T, (Status, String)>;
}

#[rocket::async_trait]
impl<T: FromIdOrSlug> FetchModel<T> for T {
    async fn from_id(db: &mut Db, id: i32) -> Result<T, (Status, String)> {
        Ok(T::from_id(db, id)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::NotFound, "Not found".to_string()))?)
    }

    async fn from_slug(db: &mut Db, slug: &str) -> Result<T, (Status, String)> {
        Ok(T::from_slug(db, slug)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::NotFound, "Not found".to_string()))?)
    }
}

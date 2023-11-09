use crate::db::Db;
use crate::models::{Core, CoreRelease, Platform, System, User};
use crate::schema;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::upsert::on_constraint;
use diesel::{AsExpression, FromSqlRow};
use retronomicon_dto as dto;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde_json::Value as Json;
use sha2::Digest;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(core_release_id, artifact_id))]
#[diesel(belongs_to(models::CoreRelease))]
#[diesel(belongs_to(models::Artifact))]
#[diesel(table_name = schema::core_release_artifacts)]
pub struct CoreReleaseArtifact {
    pub core_release_id: i32,
    pub artifact_id: i32,
}

impl CoreReleaseArtifact {
    pub async fn create(
        db: &mut Db,
        core_release: &CoreRelease,
        artifact: &Artifact,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::core_release_artifacts::table)
            .values((
                schema::core_release_artifacts::core_release_id.eq(core_release.id),
                schema::core_release_artifacts::artifact_id.eq(artifact.id),
            ))
            .returning(schema::core_release_artifacts::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn list(
        db: &mut Db,
        core_release: &CoreRelease,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::core_release_artifacts::table
            .filter(schema::core_release_artifacts::core_release_id.eq(core_release.id))
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }

    pub async fn is_filename_unique_for_release(
        db: &mut Db,
        core_release: &CoreRelease,
        filename: &str,
    ) -> Result<bool, diesel::result::Error> {
        schema::core_release_artifacts::table
            .inner_join(schema::artifacts::table)
            .filter(schema::core_release_artifacts::core_release_id.eq(core_release.id))
            .filter(schema::artifacts::filename.eq(filename))
            .count()
            .get_result::<i64>(db)
            .await
            .map(|c| c == 0)
    }

    pub async fn is_filename_conform(
        _db: &mut Db,
        _core_release: &CoreRelease,
        filename: &str,
    ) -> Result<bool, diesel::result::Error> {
        Ok(filename
            .chars()
            .all(|c| c.is_alphanumeric() || "()[]{}<>-_+=!@#$%^&*~,. ".contains(c)))
    }
}

#[derive(Queryable, Debug, Selectable, Identifiable)]
#[diesel(table_name = schema::files)]
#[diesel(belongs_to(models::Artifact))]
pub struct File {
    pub id: i32,
    pub data: Vec<u8>,
}

#[derive(Queryable, Debug, Selectable, Identifiable)]
#[diesel(table_name = schema::artifacts)]
pub struct Artifact {
    pub id: i32,
    pub filename: String,
    pub mime_type: String,
    pub created_at: NaiveDateTime,
    pub md5: Vec<u8>,
    pub sha256: Vec<u8>,
    pub size: i32,
    pub download_url: Option<String>,
    pub sha1: Vec<u8>,
}

impl Artifact {
    pub async fn create_with_data(
        db: &mut Db,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> Result<Self, diesel::result::Error> {
        let md5 = md5::compute(data).to_vec();
        let sha1 = sha1::Sha1::digest(data).to_vec();
        let sha256 = sha2::Sha256::digest(data).to_vec();

        let artifact = diesel::insert_into(schema::artifacts::table)
            .values((
                schema::artifacts::created_at.eq(chrono::Utc::now().naive_utc()),
                schema::artifacts::filename.eq(filename),
                schema::artifacts::mime_type.eq(mime_type),
                schema::artifacts::md5.eq(md5),
                schema::artifacts::sha256.eq(sha256),
                schema::artifacts::size.eq(data.len() as i32),
                schema::artifacts::sha1.eq(sha1),
            ))
            .returning(schema::artifacts::all_columns)
            .get_result::<Self>(db)
            .await?;

        diesel::insert_into(schema::files::table)
            .values((
                schema::files::id.eq(artifact.id),
                schema::files::data.eq(data),
            ))
            .execute(db)
            .await?;
        Ok(artifact)
    }

    pub async fn create_with_checksum(
        db: &mut Db,
        filename: &str,
        mime_type: &str,
        md5: Option<&[u8]>,
        sha256: Option<&[u8]>,
        download_url: Option<&str>,
        size: i32,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::artifacts::table)
            .values((
                schema::artifacts::created_at.eq(chrono::Utc::now().naive_utc()),
                schema::artifacts::filename.eq(filename),
                schema::artifacts::mime_type.eq(mime_type),
                schema::artifacts::md5.eq(md5.unwrap_or(&[])),
                schema::artifacts::sha256.eq(sha256.unwrap_or(&[])),
            ))
            .returning(schema::artifacts::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn list(
        db: &mut Db,
        release: &CoreRelease,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::artifacts::table
            .inner_join(schema::core_release_artifacts::table)
            .inner_join(schema::files::table)
            .filter(schema::core_release_artifacts::core_release_id.eq(release.id))
            .select(schema::artifacts::all_columns)
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }

    pub async fn get_file(
        db: &mut Db,
        core_id: dto::types::IdOrSlug<'_>,
        release_id: u32,
        artifact_id: u32,
    ) -> Result<(Self, Option<File>), diesel::result::Error> {
        let mut query =
            schema::artifacts::table
                .inner_join(schema::files::table)
                .inner_join(schema::core_release_artifacts::table)
                .inner_join(schema::core_releases::table.on(
                    schema::core_releases::id.eq(schema::core_release_artifacts::core_release_id),
                ))
                .inner_join(
                    schema::cores::table.on(schema::cores::id.eq(schema::core_releases::core_id)),
                )
                .into_boxed();

        if let Some(id) = core_id.as_id() {
            query = query.filter(schema::cores::id.eq(id));
        } else if let Some(slug) = core_id.as_slug() {
            query = query.filter(schema::cores::slug.eq(slug));
        } else {
            return Err(diesel::result::Error::NotFound);
        }

        let artifact = query
            .filter(schema::core_releases::id.eq(release_id as i32))
            .filter(schema::artifacts::id.eq(artifact_id as i32))
            .select(schema::artifacts::all_columns)
            .first::<Self>(db)
            .await?;

        let file = schema::files::table
            .filter(schema::files::id.eq(artifact.id))
            .first::<File>(db)
            .await
            .optional()?;

        Ok((artifact, file))
    }
    pub async fn get_fileby_filename(
        db: &mut Db,
        core_id: dto::types::IdOrSlug<'_>,
        release_id: u32,
        filename: &str,
    ) -> Result<(Self, Option<File>), diesel::result::Error> {
        let artifact =
            schema::artifacts::table
                .inner_join(schema::files::table)
                .inner_join(schema::core_release_artifacts::table)
                .inner_join(schema::core_releases::table.on(
                    schema::core_releases::id.eq(schema::core_release_artifacts::core_release_id),
                ))
                .inner_join(
                    schema::cores::table.on(schema::cores::id.eq(schema::core_releases::core_id)),
                )
                .filter(schema::cores::id.eq(core_id.as_id().unwrap()))
                .filter(schema::core_releases::id.eq(release_id as i32))
                .filter(schema::artifacts::filename.eq(filename))
                .select(schema::artifacts::all_columns)
                .first::<Self>(db)
                .await?;

        let file = schema::files::table
            .filter(schema::files::id.eq(artifact.id))
            .first::<File>(db)
            .await
            .optional()?;

        Ok((artifact, file))
    }
}

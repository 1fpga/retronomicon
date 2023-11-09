use crate::db::Db;
use crate::models::{Core, Platform, System, User};
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

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = schema::core_releases)]
pub struct CoreRelease {
    pub id: i32,
    pub version: String,
    pub notes: String,
    pub date_released: NaiveDateTime,
    pub prerelease: bool,
    pub yanked: bool,
    pub links: Json,
    pub metadata: Json,
    pub uploader_id: i32,
    pub core_id: i32,
    pub platform_id: i32,
}

impl CoreRelease {
    pub fn into_ref(self, platform: Platform) -> dto::cores::releases::CoreReleaseRef {
        dto::cores::releases::CoreReleaseRef {
            id: self.id,
            version: self.version,
            prerelease: self.prerelease,
            yanked: self.yanked,
            date_released: self.date_released.timestamp(),
            platform: platform.into(),
        }
    }
}

impl CoreRelease {
    pub async fn from_id(db: &mut Db, id: i32) -> Result<Option<Self>, diesel::result::Error> {
        schema::core_releases::table
            .filter(schema::core_releases::id.eq(id))
            .first::<Self>(db)
            .await
            .optional()
    }

    pub async fn create(
        db: &mut Db,
        version: &str,
        notes: &str,
        date_released: NaiveDateTime,
        prerelease: bool,
        links: Json,
        metadata: Json,
        uploader_id: &User,
        core_id: &Core,
        platform_id: &Platform,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::core_releases::table)
            .values((
                schema::core_releases::version.eq(version),
                schema::core_releases::notes.eq(notes),
                schema::core_releases::date_released.eq(date_released),
                schema::core_releases::prerelease.eq(prerelease),
                schema::core_releases::yanked.eq(false),
                schema::core_releases::links.eq(links),
                schema::core_releases::metadata.eq(metadata),
                schema::core_releases::uploader_id.eq(uploader_id.id),
                schema::core_releases::core_id.eq(core_id.id),
                schema::core_releases::platform_id.eq(platform_id.id),
            ))
            .returning(schema::core_releases::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn list(
        db: &mut Db,
        core_id: dto::types::IdOrSlug<'_>,
        page: i64,
        limit: i64,
        _filter: dto::cores::releases::CoreReleaseFilterParams<'_>,
    ) -> Result<Vec<(Self, Platform, Core, User)>, diesel::result::Error> {
        let mut query = schema::core_releases::table
            .inner_join(schema::platforms::table)
            .inner_join(schema::cores::table)
            .inner_join(
                schema::users::table.on(schema::users::id.eq(schema::core_releases::uploader_id)),
            )
            .select((
                schema::core_releases::all_columns,
                schema::platforms::all_columns,
                schema::cores::all_columns,
                schema::users::all_columns,
            ))
            .into_boxed();

        if let dto::types::IdOrSlug::Id(id) = core_id {
            query = query.filter(schema::core_releases::core_id.eq(id));
        } else if let dto::types::IdOrSlug::Slug(slug) = core_id {
            query = query.filter(schema::cores::slug.eq(slug));
        } else {
            return Err(diesel::result::Error::NotFound);
        }

        query
            .offset(page * limit)
            .limit(limit)
            .load::<(Self, Platform, Core, User)>(db)
            .await
    }
}

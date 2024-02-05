use crate::db::Db;
use crate::models::{Artifact, CoreRelease, Game, System};
use crate::{models, schema};
use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::{AsExpression, FromSqlRow};
use retronomicon_dto::artifact::ArtifactRef;
use retronomicon_dto::types::IdOrSlug;
use rocket::http::Status;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use scoped_futures::ScopedFutureExt;
use serde_json::{Value as Json, Value};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::ops::Bound;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = schema::game_images)]
pub struct GameImage {
    pub id: i32,
    pub game_id: i32,
    pub image_name: String,
    pub width: i32,
    pub height: i32,
    pub mime_type: String,
    pub url: String,
}

impl GameImage {
    pub async fn is_filename_conform(
        _db: &mut Db,
        _game_id: i32,
        filename: &str,
    ) -> Result<bool, diesel::result::Error> {
        Ok(filename
            .chars()
            .all(|c| c.is_alphanumeric() || "()[]{}-_+=!@#$%^&*~,. ".contains(c)))
    }

    pub async fn create(
        db: &mut Db,
        game_id: i32,
        image_name: &str,
        width: i32,
        height: i32,
        mime_type: &str,
        url: &str,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::game_images::table)
            .values((
                schema::game_images::game_id.eq(game_id),
                schema::game_images::image_name.eq(image_name),
                schema::game_images::width.eq(width),
                schema::game_images::height.eq(height),
                schema::game_images::mime_type.eq(mime_type),
                schema::game_images::url.eq(url),
            ))
            .returning(schema::game_images::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn list(
        db: &mut Db,
        page: i64,
        limit: i64,
        game_id: i32,
    ) -> Result<Vec<(Self, Game)>, diesel::result::Error> {
        schema::game_images::table
            .inner_join(schema::games::table)
            .filter(schema::game_images::game_id.eq(game_id))
            .order(schema::game_images::image_name.asc())
            .offset(page * limit)
            .limit(limit)
            .load::<(Self, Game)>(db)
            .await
    }
}

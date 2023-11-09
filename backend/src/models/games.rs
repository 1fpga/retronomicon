use crate::db::Db;
use crate::models::System;
use crate::{models, schema};
use diesel::prelude::*;
use retronomicon_dto::types::IdOrSlug;
use rocket_db_pools::diesel::RunQueryDsl;
use serde_json::Value as Json;
use std::num::NonZeroUsize;
use std::ops::Bound;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = schema::games)]
pub struct Game {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub short_description: String,
    pub year: i32,
    pub publisher: String,
    pub developer: String,
    pub links: Json,
    pub system_id: i32,
    pub system_unique_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(game_id, artifact_id))]
#[diesel(belongs_to(Game))]
#[diesel(belongs_to(models::Artifact))]
#[diesel(table_name = schema::game_artifacts)]
pub struct GameArtifact {
    pub game_id: i32,
    pub artifact_id: i32,
}

impl Game {
    pub async fn create(
        db: &mut Db,
        name: &str,
        description: &str,
        short_description: &str,
        year: i32,
        publisher: &str,
        developer: &str,
        links: Json,
        system_id: i32,
        system_unique_id: i32,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::games::table)
            .values((
                schema::games::name.eq(name),
                schema::games::description.eq(description),
                schema::games::short_description.eq(short_description),
                schema::games::year.eq(year),
                schema::games::publisher.eq(publisher),
                schema::games::developer.eq(developer),
                schema::games::links.eq(links),
                schema::games::system_id.eq(system_id),
                schema::games::system_unique_id.eq(system_unique_id),
            ))
            .returning(schema::games::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn list(
        db: &mut Db,
        page: i64,
        limit: i64,
        system: Option<IdOrSlug<'_>>,
        year: (Bound<i32>, Bound<i32>),
        name: Option<&str>,
    ) -> Result<Vec<(Self, System)>, diesel::result::Error> {
        use schema::games::dsl;

        let mut query = schema::games::table
            .inner_join(schema::systems::table)
            .into_boxed();

        if let Some(system) = system {
            if let Some(system_id) = system.as_id() {
                query = query.filter(dsl::system_id.eq(system_id));
            } else if let Some(system_slug) = system.as_slug() {
                query = query.filter(schema::systems::dsl::slug.eq(system_slug.to_string()));
            }
        }

        query = match year {
            (Bound::Unbounded, Bound::Unbounded) => query,
            (Bound::Included(s), Bound::Unbounded) => query.filter(dsl::year.ge(s)),
            (Bound::Excluded(s), Bound::Unbounded) => query.filter(dsl::year.gt(s)),
            (Bound::Unbounded, Bound::Included(e)) => query.filter(dsl::year.le(e)),
            (Bound::Unbounded, Bound::Excluded(e)) => query.filter(dsl::year.lt(e)),
            (Bound::Included(s), Bound::Included(e)) => query.filter(dsl::year.between(s, e)),
            (Bound::Included(s), Bound::Excluded(e)) => query.filter(dsl::year.between(s, e - 1)),
            (Bound::Excluded(s), Bound::Included(e)) => query.filter(dsl::year.between(s + 1, e)),
            (Bound::Excluded(s), Bound::Excluded(e)) => {
                query.filter(dsl::year.between(s + 1, e - 1))
            }
        };

        if let Some(name) = name {
            query = query.filter(dsl::name.ilike(format!("%{}%", name)));
        }

        query
            .order_by(dsl::name.asc())
            .offset(page * limit)
            .limit(limit)
            .select((schema::games::all_columns, schema::systems::all_columns))
            .load(db)
            .await
    }

    pub async fn find_by_sha256(
        db: &mut Db,
        page: i64,
        limit: i64,
        sha256: Vec<Vec<u8>>,
        system: Option<IdOrSlug<'_>>,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use schema::games::dsl;

        let mut query = schema::games::table
            .inner_join(schema::systems::table)
            .inner_join(schema::game_artifacts::table)
            .inner_join(
                schema::artifacts::table
                    .on(schema::artifacts::id.eq(schema::game_artifacts::artifact_id)),
            )
            .filter(schema::artifacts::dsl::sha256.eq_any(sha256))
            .order_by(dsl::name.asc())
            .offset(page * limit)
            .limit(limit)
            .select(schema::games::all_columns)
            .into_boxed();

        if let Some(system) = system {
            if let Some(system_id) = system.as_id() {
                query = query.filter(dsl::system_id.eq(system_id));
            } else if let Some(system_slug) = system.as_slug() {
                query = query.filter(schema::systems::dsl::slug.eq(system_slug.to_string()));
            };
        }

        return query.load::<Self>(db).await;
    }
}

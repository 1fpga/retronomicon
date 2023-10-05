use crate::db::Db;
use crate::types::FromIdOrSlug;
use crate::{models, schema};
use diesel::prelude::*;
use diesel::{Identifiable, Queryable};
use retronomicon_dto as dto;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde_json::Value as Json;

mod releases;
pub use releases::*;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = schema::cores)]
pub struct Core {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub metadata: Json,
    pub links: Json,
    pub system_id: i32,
    pub owner_team_id: i32,
}

#[rocket::async_trait]
impl FromIdOrSlug for Core {
    async fn from_id(db: &mut Db, id: i32) -> Result<Option<Self>, diesel::result::Error>
    where
        Self: Sized,
    {
        schema::cores::table
            .filter(schema::cores::id.eq(id))
            .first::<Self>(db)
            .await
            .optional()
    }

    async fn from_slug(db: &mut Db, slug: &str) -> Result<Option<Self>, diesel::result::Error>
    where
        Self: Sized,
    {
        schema::cores::table
            .filter(schema::cores::slug.eq(slug))
            .first::<Self>(db)
            .await
            .optional()
    }
}

impl Core {
    pub async fn list(
        db: &mut Db,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::cores::table
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }

    pub async fn list_with_teams(
        db: &mut Db,
        page: i64,
        limit: i64,
    ) -> Result<Vec<(Self, models::Team)>, diesel::result::Error> {
        schema::cores::table
            .inner_join(schema::teams::table)
            .offset(page * limit)
            .limit(limit)
            .load::<(Self, models::Team)>(db)
            .await
    }

    pub async fn create(
        db: &mut Db,
        slug: &str,
        name: &str,
        description: &str,
        metadata: Json,
        links: Json,
        system: &models::System,
        owner_team: &models::Team,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::cores::table)
            .values((
                schema::cores::slug.eq(slug),
                schema::cores::name.eq(name),
                schema::cores::description.eq(description),
                schema::cores::metadata.eq(metadata),
                schema::cores::links.eq(links),
                schema::cores::system_id.eq(system.id),
                schema::cores::owner_team_id.eq(owner_team.id),
            ))
            .returning(schema::cores::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn get_with_owner_and_system(
        db: &mut Db,
        id: dto::types::IdOrSlug<'_>,
    ) -> Result<Option<(Self, models::Team, models::System)>, diesel::result::Error> {
        let mut query = schema::cores::table
            .inner_join(schema::teams::table)
            .inner_join(schema::systems::table)
            .into_boxed();

        if let Some(id) = id.as_id() {
            query = query.filter(schema::cores::id.eq(id));
        } else if let Some(slug) = id.as_slug() {
            query = query.filter(schema::cores::slug.eq(slug));
        } else {
            return Ok(None);
        }

        query
            .first::<(Self, models::Team, models::System)>(db)
            .await
            .optional()
    }
}

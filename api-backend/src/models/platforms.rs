use crate::db::Db;
use crate::types::FromIdOrSlug;
use crate::{models, schema};
use diesel::prelude::*;
use diesel::{Identifiable, Queryable};
use retronomicon_dto as dto;
use retronomicon_dto::types::IdOrSlug;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde_json::value::Value as Json;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = schema::platforms)]
pub struct Platform {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub links: Json,
    pub metadata: Json,
    pub owner_team_id: i32,
}

impl From<Platform> for dto::platforms::Platform {
    fn from(value: Platform) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
        }
    }
}

#[rocket::async_trait]
impl FromIdOrSlug for Platform {
    async fn from_id(db: &mut Db, id: i32) -> Result<Option<Self>, diesel::result::Error> {
        schema::platforms::table
            .filter(schema::platforms::id.eq(id))
            .first::<Self>(db)
            .await
            .optional()
    }

    async fn from_slug(db: &mut Db, slug: &str) -> Result<Option<Self>, diesel::result::Error> {
        schema::platforms::table
            .filter(schema::platforms::slug.eq(slug))
            .first::<Self>(db)
            .await
            .optional()
    }
}

impl Platform {
    pub async fn create(
        db: &mut Db,
        slug: &str,
        name: &str,
        description: &str,
        links: Json,
        metadata: Json,
        owner: &models::Team,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::platforms::table)
            .values((
                schema::platforms::slug.eq(slug),
                schema::platforms::name.eq(name),
                schema::platforms::description.eq(description),
                schema::platforms::links.eq(links),
                schema::platforms::metadata.eq(metadata),
                schema::platforms::owner_team_id.eq(owner.id),
            ))
            .returning(schema::platforms::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn list(
        db: &mut Db,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::platforms::table
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }

    pub async fn get_with_owner(
        db: &mut Db,
        id: IdOrSlug<'_>,
    ) -> Result<Option<(Self, models::Team)>, diesel::result::Error> {
        if let Some(id) = id.as_id() {
            schema::platforms::table
                .filter(schema::platforms::id.eq(id))
                .inner_join(schema::teams::table)
                .select((schema::platforms::all_columns, schema::teams::all_columns))
                .first::<(Self, models::Team)>(db)
                .await
                .optional()
        } else if let Some(slug) = id.as_slug() {
            schema::platforms::table
                .filter(schema::platforms::slug.eq(slug))
                .inner_join(schema::teams::table)
                .select((schema::platforms::all_columns, schema::teams::all_columns))
                .first::<(Self, models::Team)>(db)
                .await
                .optional()
        } else {
            return Err(diesel::result::Error::NotFound);
        }
    }

    pub async fn update(
        db: &mut Db,
        id: i32,
        slug: Option<&'_ str>,
        name: Option<&'_ str>,
        description: Option<&'_ str>,
        links: Option<Json>,
        metadata: Option<Json>,
        owner_team_id: Option<i32>,
    ) -> Result<(), diesel::result::Error> {
        #[derive(AsChangeset)]
        #[diesel(table_name = schema::platforms)]
        struct Update<'a> {
            slug: Option<&'a str>,
            name: Option<&'a str>,
            description: Option<&'a str>,
            links: Option<Json>,
            metadata: Option<Json>,
            owner_team_id: Option<i32>,
        }

        diesel::update(schema::platforms::table)
            .filter(schema::platforms::id.eq(id))
            .set(&Update {
                slug,
                name,
                description,
                links,
                metadata,
                owner_team_id,
            })
            .execute(db)
            .await?;
        Ok(())
    }
}

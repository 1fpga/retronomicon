use crate::db::Db;
use crate::{models, schema};
use diesel::prelude::*;
use diesel::{Identifiable, Queryable};
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde_json::Value as Json;

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
}

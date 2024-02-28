use crate::types::FromIdOrSlug;
use crate::Db;
use crate::{models, schema};
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::query_builder::BoxedSelectStatement;
use diesel::{AsExpression, FromSqlRow, Identifiable, Queryable};
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde_json::Value as Json;

mod releases;
use crate::pages::{Paginate, Paginated};
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

    pub async fn list_with_teams_and_releases<'a>(
        db: &'a mut Db,
        page: i64,
        limit: i64,
        platform: Option<i32>,
        system: Option<i32>,
        team: Option<i32>,
        release_date_ge: Option<chrono::NaiveDateTime>,
    ) -> Result<
        (
            Vec<(
                Self,
                models::System,
                models::Team,
                Option<CoreRelease>,
                models::Platform,
            )>,
            i64,
        ),
        String,
    > {
        let mut query = schema::cores::table
            .inner_join(schema::teams::table)
            .left_join(
                schema::core_releases::table.on(schema::core_releases::id.eq(
                    // Diesel does not support subqueries on joins, so we have to use raw SQL.
                    // It's okay because it does not actually need inputs.
                    diesel::dsl::sql(
                        r#"(
                        SELECT id FROM core_releases
                            WHERE cores.id = core_releases.core_id
                            ORDER BY date_released DESC, id DESC
                            LIMIT 1
                        )"#,
                    ),
                )),
            )
            .inner_join(
                schema::platforms::table
                    .on(schema::platforms::id.eq(schema::core_releases::platform_id)),
            )
            .inner_join(schema::systems::table)
            .select((
                schema::cores::all_columns,
                schema::systems::all_columns,
                schema::teams::all_columns,
                schema::core_releases::all_columns.nullable(),
                schema::platforms::all_columns,
            ))
            .into_boxed();

        if let Some(platform) = platform {
            query = query.filter(schema::platforms::id.eq(platform));
        }
        if let Some(system) = system {
            query = query.filter(schema::systems::id.eq(system));
        }
        if let Some(team) = team {
            query = query.filter(schema::teams::id.eq(team));
        }
        if let Some(release_date_ge) = release_date_ge {
            query = query.filter(schema::core_releases::date_released.ge(release_date_ge));
        }

        query
            .paginate(page)
            .per_page(limit)
            .load_and_count_total::<(
                Self,
                models::System,
                models::Team,
                Option<CoreRelease>,
                models::Platform,
            )>(db)
            .await
            .map_err(|e| e.to_string())
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
        id: i32,
    ) -> Result<Option<(Self, models::Team, models::System)>, diesel::result::Error> {
        schema::cores::table
            .inner_join(schema::teams::table)
            .inner_join(schema::systems::table)
            .filter(schema::cores::id.eq(id))
            .first::<(Self, models::Team, models::System)>(db)
            .await
            .optional()
    }

    pub async fn get_by_slug_with_owner_and_system(
        db: &mut Db,
        slug: &str,
    ) -> Result<Option<(Self, models::Team, models::System)>, diesel::result::Error> {
        schema::cores::table
            .inner_join(schema::teams::table)
            .inner_join(schema::systems::table)
            .filter(schema::cores::slug.eq(slug))
            .first::<(Self, models::Team, models::System)>(db)
            .await
            .optional()
    }
}

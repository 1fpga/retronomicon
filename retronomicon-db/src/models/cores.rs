use crate::types::FromIdOrSlug;
use crate::Db;
use crate::{models, schema};
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::query_builder::BoxedSelectStatement;
use diesel::{AsExpression, FromSqlRow, Identifiable, Queryable};
use retronomicon_dto as dto;
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
        dto::Paginated<(
            Self,
            models::System,
            models::Team,
            Option<CoreRelease>,
            models::Platform,
        )>,
        String,
    > {
        use diesel::dsl::sql;
        use diesel::sql_types::{Bool, Integer, Nullable};

        // This is a bit of a mess, but it's the best we can do with Diesel.
        // The problem is that using the Paginate class with `into_boxed()` seems
        // to confuse the borrow checker and it returns a
        // `higher-ranked lifetime error`. This is a workaround where we manually
        // build the query.
        let condition = sql::<Bool>("true")
            .and(
                (schema::platforms::id
                    .nullable()
                    .eq(platform)
                    .or(schema::platforms::id.is_null())),
            )
            .and(
                schema::systems::id
                    .nullable()
                    .eq(system)
                    .or(schema::systems::id.is_null()),
            )
            .and(
                schema::teams::id
                    .nullable()
                    .eq(team)
                    .or(schema::teams::id.is_null()),
            )
            .and(
                schema::core_releases::date_released
                    .nullable()
                    .ge(release_date_ge)
                    .or(schema::core_releases::date_released.is_null()),
            );
        // let condition = sql::<Bool>("true")
        //     .sql(" AND (platforms.id = ")
        //     .bind::<Nullable<Integer>, _>(platform)
        //     .sql(" OR ")
        //     .bind::<Nullable<Integer>, _>(platform)
        //     .sql(" IS NULL)")
        //     .sql(" AND (systems.id = ")
        //     .bind::<Nullable<Integer>, _>(system)
        //     .sql(" OR ")
        //     .bind::<Nullable<Integer>, _>(system)
        //     .sql(" IS NULL)")
        //     .sql(" AND (teams.id = ")
        //     .bind::<Nullable<Integer>, _>(team)
        //     .sql(" OR ")
        //     .bind::<Nullable<Integer>, _>(team)
        //     .sql(" IS NULL)")
        //     .sql(" AND (core_releases.date_released >= ")
        //     .bind::<Nullable<diesel::sql_types::Timestamp>, _>(release_date_ge)
        //     .sql(" OR ")
        //     .bind::<Nullable<diesel::sql_types::Timestamp>, _>(release_date_ge)
        //     .sql(" IS NULL)");

        let (items, total) = schema::cores::table
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
            .filter(condition)
            .paginate(Some(page))
            .per_page(Some(limit))
            .load_and_count_total::<'a, (
                Self,
                models::System,
                models::Team,
                Option<CoreRelease>,
                models::Platform,
            )>(db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(dto::Paginated::new(
            total as u64,
            page as u64,
            limit as u64,
            items,
        ))
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

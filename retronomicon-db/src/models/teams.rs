use crate::models::User;
use crate::types::FromIdOrSlug;
use crate::Db;
use crate::{models, schema};
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::internal::operators_macro::FieldAliasMapper;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use retronomicon_dto as dto;
use retronomicon_dto::types::IdOrSlug;
use rocket::http::Status;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::fmt::{Debug, Formatter};
use std::io::Write;

#[derive(AsChangeset)]
#[diesel(table_name = schema::teams)]
struct TeamUpdate<'a> {
    slug: Option<&'a str>,
    name: Option<&'a str>,
    description: Option<&'a str>,
    links: Option<Json>,
    metadata: Option<Json>,
}

#[derive(Queryable, Debug, Identifiable, Selectable, Serialize)]
#[diesel(table_name = schema::teams)]
pub struct Team {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub links: Json,
    pub metadata: Json,
}

impl From<Team> for dto::teams::TeamRef {
    fn from(value: Team) -> Self {
        Self {
            id: value.id,
            name: value.name,
            slug: value.slug,
        }
    }
}

impl From<Team> for dto::teams::Team {
    fn from(value: Team) -> Self {
        Self {
            team: dto::teams::TeamRef {
                id: value.id,
                name: value.name,
                slug: value.slug,
            },

            description: value.description,
            links: value.links,
            metadata: value.metadata,
        }
    }
}

#[rocket::async_trait]
impl FromIdOrSlug for Team {
    async fn from_id(db: &mut Db, id: i32) -> Result<Option<Self>, diesel::result::Error> {
        schema::teams::table
            .filter(schema::teams::id.eq(id))
            .first::<Team>(db)
            .await
            .optional()
    }

    async fn from_slug(db: &mut Db, slug: &str) -> Result<Option<Self>, diesel::result::Error> {
        schema::teams::table
            .filter(schema::teams::slug.eq(slug))
            .first::<Team>(db)
            .await
            .optional()
    }
}

impl Team {
    pub fn is_root(&self) -> bool {
        self.id == 1
    }

    pub async fn get(db: &mut Db, id: i32) -> Result<Option<Self>, diesel::result::Error> {
        schema::teams::table
            .filter(schema::teams::id.eq(id))
            .first::<Team>(db)
            .await
            .optional()
    }

    pub async fn create(
        db: &mut Db,
        slug: &str,
        name: &str,
        description: &str,
        links: Json,
        metadata: Json,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::teams::table)
            .values((
                schema::teams::slug.eq(slug),
                schema::teams::name.eq(name),
                schema::teams::description.eq(description),
                schema::teams::links.eq(links),
                schema::teams::metadata.eq(metadata),
            ))
            .returning(schema::teams::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn update(
        db: &mut Db,
        id: i32,
        slug: Option<&str>,
        name: Option<&str>,
        description: Option<&str>,
        links: Option<Json>,
        metadata: Option<Json>,
    ) -> Result<(), diesel::result::Error> {
        diesel::update(schema::teams::table)
            .filter(schema::teams::id.eq(id))
            .set(&TeamUpdate {
                slug,
                name,
                description,
                links,
                metadata,
            })
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn delete(db: &mut Db, id: i32) -> Result<(), diesel::result::Error> {
        diesel::delete(schema::teams::table)
            .filter(schema::teams::id.eq(id))
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn list(
        db: &mut Db,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::teams::table
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }

    pub async fn users_ref(
        &self,
        db: &mut Db,
    ) -> Result<Vec<dto::teams::TeamUserRef>, diesel::result::Error> {
        Ok(models::UserTeam::belonging_to(self)
            .inner_join(schema::users::table.on(schema::users::id.eq(schema::user_teams::user_id)))
            .select((
                schema::users::id,
                schema::users::username,
                schema::user_teams::role,
            ))
            .filter(schema::users::username.is_not_null())
            .filter(schema::user_teams::invite_from.is_null())
            .load::<(i32, Option<String>, models::UserTeamRole)>(db)
            .await?
            .into_iter()
            .filter_map(|(id, username, role)| {
                username.map(|username| dto::teams::TeamUserRef {
                    user: dto::user::UserRef { id, username },
                    role: role.into(),
                })
            })
            .collect())
    }
}

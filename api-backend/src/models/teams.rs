use crate::db::Db;
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::fmt::{Debug, Formatter};
use std::io::Write;

#[derive(Queryable, Debug, Identifiable, Selectable, Serialize)]
pub struct Team {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub links: Option<Json>,
}

impl From<Team> for retronomicon_dto::teams::TeamRef {
    fn from(value: Team) -> Self {
        Self {
            id: value.id,
            name: value.name,
            slug: value.slug,
        }
    }
}

impl Team {
    pub async fn from_id(db: &mut Db, id: i32) -> Result<Self, diesel::result::Error> {
        teams::table
            .filter(teams::id.eq(id))
            .first::<Team>(db)
            .await
    }

    pub async fn from_slug(db: &mut Db, slug: &str) -> Result<Self, diesel::result::Error> {
        teams::table
            .filter(teams::slug.eq(slug))
            .first::<Team>(db)
            .await
    }
}

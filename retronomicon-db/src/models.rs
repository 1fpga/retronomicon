#![allow(unused)]
#![allow(clippy::all)]

use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use retronomicon_dto as dto;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::fmt::{Debug, Formatter};
use std::io::Write;

pub mod artifact;
pub use artifact::*;

pub mod cores;
pub use cores::*;

pub mod games;
pub use games::*;

pub mod platforms;
pub use platforms::*;

pub mod systems;
pub use systems::*;

pub mod users;
pub use users::*;

pub mod tags;
pub use tags::*;

pub mod teams;
use crate::{models, schema};
pub use teams::*;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(tag_id, core_id))]
pub struct CoreTag {
    pub core_id: i32,
    pub tag_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(tag_id, platform_id))]
pub struct PlatformTag {
    pub platform_id: i32,
    pub tag_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct SystemRelease {
    pub id: i32,
    pub version: String,
    pub note: Option<String>,
    pub date_released: NaiveDateTime,
    pub prerelease: i32,
    pub yanked: bool,
    pub links: Json,
    pub metadata: Json,
    pub uploader_id: i32,
    pub system_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(artifact_id, system_file_release_id))]
pub struct SystemReleaseArtifact {
    pub system_file_release_id: i32,
    pub artifact_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(tag_id, system_id))]
pub struct SystemTag {
    pub system_id: i32,
    pub tag_id: i32,
}

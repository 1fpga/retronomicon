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

pub mod cores;
pub use cores::*;

pub mod platforms;
pub use platforms::*;

pub mod systems;
pub use systems::*;

pub mod users;
pub use users::*;

pub mod tags;
pub use tags::*;

pub mod teams;
pub use teams::*;

#[derive(Queryable, Debug, Selectable, Identifiable, Serialize, Deserialize)]
pub struct Artifact {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub filename: String,
    pub sha256: Option<Vec<u8>>,
    pub sha512: Option<Vec<u8>>,
    pub size: i32,
    pub download_url: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct CoreRelease {
    pub id: i32,
    pub version: String,
    pub note: Option<String>,
    pub date_released: NaiveDateTime,
    pub prerelease: Option<bool>,
    pub yanked: Option<bool>,
    pub links: Option<Json>,
    pub uploader_id: i32,
    pub core_id: i32,
    pub platform_id: i32,
    pub system_id: i32,
    pub owner_team_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(core_release_id, artifact_id))]
pub struct CoreReleaseArtifact {
    pub core_release_id: i32,
    pub artifact_id: i32,
}

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
    pub date_released: Option<NaiveDateTime>,
    pub date_uploaded: NaiveDateTime,
    pub prerelease: Option<i32>,
    pub yanked: Option<bool>,
    pub links: Option<Json>,
    pub user_id: i32,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, FromSqlRow, AsExpression)]
#[diesel(sql_type = sql_types::UserTeamRole)]
pub enum UserTeamRole {
    Owner = 2,
    Admin = 1,
    Member = 0,
}

impl UserTeamRole {
    pub(crate) fn can_create_systems(&self) -> bool {
        // Admins and owners can create systems.
        *self >= Self::Admin
    }

    pub fn can_create_cores(&self) -> bool {
        // Admins and owners can create cores.
        *self >= Self::Admin
    }
}

impl ToSql<sql_types::UserTeamRole, Pg> for UserTeamRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        match *self {
            Self::Owner => out.write_all(b"owner")?,
            Self::Admin => out.write_all(b"admin")?,
            Self::Member => out.write_all(b"member")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<sql_types::UserTeamRole, Pg> for UserTeamRole {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"owner" => Ok(Self::Owner),
            b"admin" => Ok(Self::Admin),
            b"member" => Ok(Self::Member),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<UserTeamRole> for dto::types::UserTeamRole {
    fn from(value: UserTeamRole) -> Self {
        match value {
            UserTeamRole::Owner => Self::Owner,
            UserTeamRole::Admin => Self::Admin,
            UserTeamRole::Member => Self::Member,
        }
    }
}

impl From<dto::types::UserTeamRole> for UserTeamRole {
    fn from(value: dto::types::UserTeamRole) -> Self {
        match value {
            dto::types::UserTeamRole::Owner => Self::Owner,
            dto::types::UserTeamRole::Admin => Self::Admin,
            dto::types::UserTeamRole::Member => Self::Member,
        }
    }
}

#[derive(Queryable, Debug, Identifiable, Selectable, Associations)]
#[diesel(belongs_to(Team))]
#[diesel(belongs_to(User))]
#[diesel(primary_key(team_id, user_id))]
pub struct UserTeam {
    pub team_id: i32,
    pub user_id: i32,
    pub role: UserTeamRole,
    pub invite_from: Option<i32>,
}

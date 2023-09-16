#![allow(unused)]
#![allow(clippy::all)]

use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::fmt::{Debug, Formatter};
use std::io::Write;

#[derive(Queryable, Debug, Selectable, Identifiable, Serialize, Deserialize)]
pub struct Artifact {
    pub id: i32,
    pub filename: String,
    pub sha256: Option<Vec<u8>>,
    pub sha512: Option<Vec<u8>>,
    pub size: i32,
    pub download_url: Option<String>,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct Core {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub metadata: Option<Json>,
    pub links: Option<Json>,
    pub owner_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct CoreRelease {
    pub id: i32,
    pub version: String,
    pub note: Option<String>,
    pub date_released: Option<NaiveDateTime>,
    pub date_uploaded: NaiveDateTime,
    pub prerelease: Option<bool>,
    pub yanked: Option<bool>,
    pub links: Option<Json>,
    pub uploader_id: i32,
    pub core_id: i32,
    pub platform_id: i32,
    pub system_id: i32,
    pub owner_id: i32,
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

#[derive(Queryable, Debug, Identifiable, Selectable, Serialize)]
pub struct Group {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub links: Option<Json>,
}

impl From<Group> for retronomicon_dto::details::GroupRef {
    fn from(value: Group) -> Self {
        Self {
            id: value.id,
            name: value.name,
            slug: value.slug,
        }
    }
}

#[derive(Queryable, Debug, Identifiable, Serialize)]
pub struct Platform {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub links: Option<Json>,
    pub metadata: Option<Json>,
    pub owner_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(primary_key(tag_id, platform_id))]
pub struct PlatformTag {
    pub platform_id: i32,
    pub tag_id: i32,
}

#[derive(Queryable, Debug, Identifiable)]
pub struct System {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub manufacturer: String,
    pub links: Option<Json>,
    pub metadata: Option<Json>,
    pub owner_id: i32,
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

#[derive(Queryable, Debug, Identifiable)]
pub struct Tag {
    pub id: i32,
    pub slug: String,
    pub description: Option<String>,
    pub color: i32,
}

#[derive(Queryable, Debug, Identifiable, Selectable, Serialize)]
pub struct User {
    pub id: i32,

    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,

    pub email: String,
    #[serde(skip_serializing)]
    pub auth_provider: Option<String>,

    pub need_reset: bool,
    pub deleted: bool,

    pub description: String,
    pub links: Option<Json>,
    pub metadata: Option<Json>,
}

impl From<User> for retronomicon_dto::user::User {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
        }
    }
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = sql_types::UserGroupRole)]
pub enum UserGroupRole {
    Owner,
    Admin,
    Member,
}

impl ToSql<sql_types::UserGroupRole, Pg> for UserGroupRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        match *self {
            Self::Owner => out.write_all(b"owner")?,
            Self::Admin => out.write_all(b"admin")?,
            Self::Member => out.write_all(b"member")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<sql_types::UserGroupRole, Pg> for UserGroupRole {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"owner" => Ok(Self::Owner),
            b"admin" => Ok(Self::Admin),
            b"member" => Ok(Self::Member),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Queryable, Debug, Identifiable, Selectable, Associations)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(User))]
#[diesel(primary_key(group_id, user_id))]
pub struct UserGroup {
    pub group_id: i32,
    pub user_id: i32,
    pub role: UserGroupRole,
}

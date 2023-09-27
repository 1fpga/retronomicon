use crate::db::Db;
use crate::models;
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::upsert::on_constraint;
use diesel::{AsExpression, FromSqlRow};
use jsonwebtoken::DecodingKey;
use retronomicon_dto as dto;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::{IntoOutcome, Outcome};
use rocket::request;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use scoped_futures::ScopedFutureExt;
use serde_json::Value as Json;
use std::fmt::{Debug, Formatter};
use std::io::Write;

#[derive(Clone, Debug, Queryable, Identifiable, Selectable)]
pub struct User {
    pub id: i32,

    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,

    pub email: String,
    pub auth_provider: Option<String>,

    pub need_reset: bool,
    pub deleted: bool,

    pub description: String,
    pub links: Option<Json>,
    pub metadata: Option<Json>,
}

impl From<User> for dto::user::User {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
        }
    }
}

impl User {
    pub async fn from_id(db: &mut Db, id: i32) -> Result<Self, diesel::result::Error> {
        users::table
            .filter(users::id.eq(id))
            .first::<User>(db)
            .await
    }

    pub async fn from_username(db: &mut Db, name: &str) -> Result<Self, diesel::result::Error> {
        users::table
            .filter(users::username.eq(name))
            .first::<User>(db)
            .await
    }

    pub async fn from_userid(
        db: &mut Db,
        user_id: dto::user::UserIdOrUsername<'_>,
    ) -> Result<Self, diesel::result::Error> {
        match user_id {
            dto::user::UserIdOrUsername::Id(id) => Self::from_id(db, id).await,
            dto::user::UserIdOrUsername::Username(name) => Self::from_username(db, &name).await,
        }
    }

    pub async fn invite_to(
        &self,
        db: &mut Db,
        from_id: i32,
        team_id: i32,
        role: models::UserTeamRole,
    ) -> Result<(), diesel::result::Error> {
        use user_teams::dsl;

        db.transaction(|db| {
            async move {
                // Check if we already are part of the team or if there's an invitation
                // pending.
                let maybe_user_team = user_teams::dsl::user_teams
                    .filter(dsl::user_id.eq(self.id))
                    .filter(dsl::team_id.eq(team_id))
                    .first::<models::UserTeam>(db)
                    .await
                    .optional()?;
                if let Some(user_team) = maybe_user_team {
                    if user_team.role < role {
                        diesel::update(dsl::user_teams)
                            .filter(dsl::user_id.eq(self.id))
                            .filter(dsl::team_id.eq(team_id))
                            .set(dsl::role.eq(role))
                            .execute(db)
                            .await?;
                    }
                } else {
                    diesel::insert_into(user_teams::table)
                        .values((
                            user_teams::user_id.eq(self.id),
                            user_teams::team_id.eq(team_id),
                            user_teams::role.eq(role),
                            user_teams::invite_from.eq(from_id),
                        ))
                        .on_conflict_do_nothing()
                        .execute(db)
                        .await?;
                }
                Ok(())
            }
            .scope_boxed()
        })
        .await
    }

    /// Add a user to a team. This does not check for an invitation.
    pub async fn add_team(
        &self,
        db: &mut Db,
        team_id: i32,
        role: models::UserTeamRole,
    ) -> Result<(), diesel::result::Error> {
        db.transaction(|db| {
            async move {
                diesel::insert_into(user_teams::table)
                    .values((
                        user_teams::user_id.eq(self.id),
                        user_teams::team_id.eq(team_id),
                        user_teams::role.eq(role),
                        user_teams::invite_from.eq(None::<i32>),
                    ))
                    .on_conflict(on_constraint("user_teams_pkey"))
                    .do_update()
                    .set((
                        user_teams::role.eq(role),
                        user_teams::invite_from.eq(None::<i32>),
                    ))
                    .execute(db)
                    .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await
    }

    /// Accept an invitation to a team. This will fail if there's no invitation
    /// or if an invitation was already accepted (user is part of the team).
    pub async fn accept_invitation(
        &self,
        db: &mut Db,
        team_id: i32,
    ) -> Result<(), diesel::result::Error> {
        db.transaction(|db| {
            async move {
                diesel::update(user_teams::table)
                    .filter(user_teams::user_id.eq(self.id))
                    .filter(user_teams::team_id.eq(team_id))
                    .filter(user_teams::invite_from.is_not_null())
                    .set(user_teams::invite_from.eq(None::<i32>))
                    .execute(db)
                    .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await
    }

    /// Returns the UserTeamRole, if there's one.
    pub async fn role_in(
        &self,
        db: &mut Db,
        team_id: i32,
    ) -> Result<Option<models::UserTeamRole>, diesel::result::Error> {
        user_teams::dsl::user_teams
            .filter(user_teams::dsl::user_id.eq(self.id))
            .filter(user_teams::dsl::team_id.eq(team_id))
            .filter(user_teams::dsl::invite_from.is_null())
            .select(user_teams::dsl::role)
            .first::<models::UserTeamRole>(db)
            .await
            .optional()
    }
}

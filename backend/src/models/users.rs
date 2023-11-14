use crate::db::Db;
use crate::models;
use crate::schema;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
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
use serde_json::Value as Json;
use std::fmt::{Debug, Formatter};
use std::io::Write;

#[derive(Clone, Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = schema::users)]
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
    pub links: Json,
    pub metadata: Json,
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

impl From<User> for dto::user::UserRef {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username.unwrap_or_default(),
        }
    }
}

impl User {
    pub async fn from_id(db: &mut Db, id: i32) -> Result<Self, diesel::result::Error> {
        schema::users::table
            .filter(schema::users::id.eq(id))
            .first::<User>(db)
            .await
    }

    pub async fn from_username(db: &mut Db, name: &str) -> Result<Self, diesel::result::Error> {
        schema::users::table
            .filter(schema::users::username.eq(name))
            .first::<User>(db)
            .await
    }

    pub async fn from_userid(
        db: &mut Db,
        user_id: dto::user::UserIdOrUsername<'_>,
    ) -> Result<Self, diesel::result::Error> {
        match user_id {
            dto::user::UserIdOrUsername::Id(id) => Self::from_id(db, id).await,
            dto::user::UserIdOrUsername::Username(name) => {
                Self::from_username(db, name.into_inner().as_ref()).await
            }
        }
    }

    pub async fn get_user_team_and_role(
        db: &mut Db,
        user_id: dto::user::UserIdOrUsername<'_>,
        team_id: dto::types::IdOrSlug<'_>,
    ) -> Result<Option<(models::User, models::Team, models::UserTeamRole)>, diesel::result::Error>
    {
        let mut query = schema::user_teams::table
            .inner_join(schema::users::table.on(schema::users::id.eq(schema::user_teams::user_id)))
            .inner_join(schema::teams::table)
            .select((
                schema::users::all_columns,
                schema::teams::all_columns,
                schema::user_teams::role,
            ))
            .into_boxed();

        if let Some(id) = user_id.as_id() {
            query = query.filter(schema::users::dsl::id.eq(id));
        } else if let Some(username) = user_id.as_username() {
            query = query.filter(schema::users::dsl::username.eq(username));
        } else {
            return Ok(None);
        }

        if let Some(id) = team_id.as_id() {
            query = query.filter(schema::user_teams::dsl::team_id.eq(id));
        } else if let Some(slug) = team_id.as_slug() {
            query = query.filter(schema::teams::dsl::slug.eq(slug));
        } else {
            return Ok(None);
        }

        query
            .first::<(models::User, models::Team, models::UserTeamRole)>(db)
            .await
            .optional()
    }

    pub async fn list(
        db: &mut Db,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::users::table
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }

    pub async fn get_user_with_teams(
        db: &mut Db,
        user_id: dto::user::UserIdOrUsername<'_>,
    ) -> Result<
        Option<(Self, Vec<(i32, String, String, models::UserTeamRole)>)>,
        diesel::result::Error,
    > {
        let user = Self::from_userid(db, user_id).await.optional()?;
        if user.is_none() {
            return Ok(None);
        }
        let user = user.unwrap();

        let username = user
            .username
            .as_ref()
            .ok_or(diesel::result::Error::NotFound)?;
        if user.deleted {
            return Err(diesel::result::Error::NotFound);
        }

        let teams = models::UserTeam::belonging_to(&user)
            .inner_join(schema::teams::table)
            .select((
                schema::teams::id,
                schema::teams::name,
                schema::teams::slug,
                schema::user_teams::role,
            ))
            .load::<(i32, String, String, models::UserTeamRole)>(db)
            .await?;

        Ok(Some((user, teams)))
    }

    pub async fn create(
        db: &mut Db,
        username: Option<&str>,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        email: &str,
        auth_provider: Option<&str>,
        description: Option<&str>,
        links: Json,
        metadata: Json,
    ) -> Result<Self, diesel::result::Error> {
        use schema::users::dsl;

        diesel::insert_into(schema::users::table)
            .values((
                dsl::username.eq(username),
                dsl::display_name.eq(display_name),
                dsl::email.eq(email),
                dsl::auth_provider.eq(auth_provider),
                dsl::description.eq(description.unwrap_or_default()),
                dsl::need_reset.eq(false),
                dsl::deleted.eq(false),
                dsl::links.eq(links),
                dsl::metadata.eq(metadata),
            ))
            .returning(schema::users::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn invite_to(
        &self,
        db: &mut Db,
        from_id: i32,
        team_id: i32,
        role: models::UserTeamRole,
    ) -> Result<(), diesel::result::Error> {
        use schema::user_teams::dsl;

        // Check if we already are part of the team or if there's an invitation
        // pending.
        let maybe_user_team = dsl::user_teams
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
            diesel::insert_into(schema::user_teams::table)
                .values((
                    schema::user_teams::user_id.eq(self.id),
                    schema::user_teams::team_id.eq(team_id),
                    schema::user_teams::role.eq(role),
                    schema::user_teams::invite_from.eq(from_id),
                ))
                .on_conflict_do_nothing()
                .execute(db)
                .await?;
        }

        Ok(())
    }

    /// Add a user to a team. This does not check for an invitation.
    pub async fn join_team(
        &self,
        db: &mut Db,
        team_id: i32,
        role: models::UserTeamRole,
    ) -> Result<(), diesel::result::Error> {
        diesel::insert_into(schema::user_teams::table)
            .values((
                schema::user_teams::user_id.eq(self.id),
                schema::user_teams::team_id.eq(team_id),
                schema::user_teams::role.eq(role),
                schema::user_teams::invite_from.eq(None::<i32>),
            ))
            .on_conflict(on_constraint("user_teams_pkey"))
            .do_update()
            .set((
                schema::user_teams::role.eq(role),
                schema::user_teams::invite_from.eq(None::<i32>),
            ))
            .execute(db)
            .await?;

        Ok(())
    }

    /// Accept an invitation to a team. This will fail if there's no invitation
    /// or if an invitation was already accepted (user is part of the team).
    pub async fn accept_invitation(
        &self,
        db: &mut Db,
        team_id: i32,
    ) -> Result<(), diesel::result::Error> {
        diesel::update(schema::user_teams::table)
            .filter(schema::user_teams::user_id.eq(self.id))
            .filter(schema::user_teams::team_id.eq(team_id))
            .filter(schema::user_teams::invite_from.is_not_null())
            .set(schema::user_teams::invite_from.eq(None::<i32>))
            .execute(db)
            .await?;

        Ok(())
    }

    /// Returns the UserTeamRole, if there's one.
    pub async fn role_in(
        &self,
        db: &mut Db,
        team_id: i32,
    ) -> Result<Option<models::UserTeamRole>, diesel::result::Error> {
        schema::user_teams::dsl::user_teams
            .filter(schema::user_teams::dsl::user_id.eq(self.id))
            .filter(schema::user_teams::dsl::team_id.eq(team_id))
            .filter(schema::user_teams::dsl::invite_from.is_null())
            .select(schema::user_teams::dsl::role)
            .first::<models::UserTeamRole>(db)
            .await
            .optional()
    }
}

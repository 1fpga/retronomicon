use crate::models;
use crate::models::Team;
use crate::schema;
use crate::schema::sql_types;
use crate::Db;
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
use rocket_db_pools::diesel::scoped_futures::ScopedFutureExt;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};
use serde_json::{Value as Json, Value};
use std::collections::BTreeMap;
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

    pub async fn from_auth(
        db: &mut Db,
        email: &str,
        auth_provider: &str,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use schema::users::dsl;
        dsl::users
            .filter(dsl::email.eq(email))
            .filter(dsl::auth_provider.eq(auth_provider))
            .first::<models::User>(db)
            .await
            .optional()
    }

    pub async fn exists(
        db: &mut Db,
        user_id: dto::user::UserIdOrUsername<'_>,
    ) -> Result<bool, diesel::result::Error> {
        Ok(Self::from_userid(db, user_id).await.optional()?.is_some())
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

    pub async fn update(
        &self,
        db: &mut Db,
        form: dto::user::UserUpdate<'_>,
    ) -> Result<(), diesel::result::Error> {
        #[derive(AsChangeset)]
        #[diesel(table_name = schema::users)]
        struct UserSignupChangeset<'a> {
            username: Option<&'a str>,
            display_name: Option<&'a str>,
            description: Option<&'a str>,
            links: Option<Json>,
            metadata: Option<Json>,
        }

        db.transaction(|db| {
            async move {
                let mut changeset = UserSignupChangeset {
                    username: form.username,
                    display_name: form.display_name,
                    description: form.description,
                    links: None,
                    metadata: None,
                };

                if let Some(links) = form.links.as_ref() {
                    changeset.links = Some(serde_json::to_value(links).unwrap());
                } else if form.add_links.is_some() || form.remove_links.is_some() {
                    let mut links = BTreeMap::new();
                    let user: models::User = schema::users::table.find(self.id).first(db).await?;

                    if let Value::Object(user_links) = user.links {
                        links.extend(user_links.into_iter());
                    }

                    if let Some(user_links) = form.add_links {
                        for (k, v) in user_links.into_iter() {
                            links.insert(k.to_string(), v.into());
                        }
                    }
                    if let Some(user_links) = form.remove_links {
                        for k in user_links.into_iter() {
                            links.remove(&k.to_string());
                        }
                    }

                    changeset.links = Some(serde_json::to_value(links).unwrap());
                }

                diesel::update(schema::users::table)
                    .filter(schema::users::id.eq(self.id))
                    .set(changeset)
                    .execute(db)
                    .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await
    }

    pub async fn invite_to(
        &self,
        db: &mut Db,
        from_id: i32,
        team_id: i32,
        role: UserTeamRole,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, FromSqlRow, AsExpression)]
#[diesel(sql_type = sql_types::UserTeamRole)]
pub enum UserTeamRole {
    Owner = 2,
    Admin = 1,
    Member = 0,
}

impl UserTeamRole {
    pub fn can_create_systems(&self) -> bool {
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
#[diesel(table_name = schema::user_teams)]
#[diesel(primary_key(team_id, user_id))]
pub struct UserTeam {
    pub team_id: i32,
    pub user_id: i32,
    pub role: UserTeamRole,
    pub invite_from: Option<i32>,
}

impl UserTeam {
    pub async fn user_is_in_team(
        db: &mut crate::Db,
        user_id: i32,
        team_id: i32,
    ) -> Result<bool, diesel::result::Error> {
        use schema::user_teams;

        Ok(user_teams::table
            .filter(user_teams::user_id.eq(user_id))
            .filter(user_teams::team_id.eq(team_id))
            .first::<Self>(db)
            .await
            .optional()?
            .is_some())
    }
}

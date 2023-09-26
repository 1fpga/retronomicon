use crate::db::Db;
use crate::guards::users::AuthenticatedUserGuard;
use crate::models::{User, UserTeamRole};
use crate::{models, schema};
use retronomicon_dto as dto;
use retronomicon_dto::user::UserId;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::diesel::prelude::*;
use scoped_futures::ScopedFutureExt;

#[get("/teams/?<paging..>")]
async fn teams(
    mut db: Db,
    paging: rocket::form::Result<'_, dto::params::PagingParams>,
) -> Result<Json<Vec<models::Team>>, (Status, crate::error::Error)> {
    let dto::params::PagingParams { page, limit } =
        paging.map_err(|e| (Status::BadRequest, e.into()))?;

    schema::teams::table
        .offset(page * limit)
        .limit(limit)
        .load::<models::Team>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.into()))
        .map(Json)
}

async fn team_details_from_(
    mut db: Db,
    team: models::Team,
) -> Result<Json<dto::teams::TeamDetails>, crate::error::Error> {
    let users = models::UserTeam::belonging_to(&team)
        .inner_join(schema::users::table.on(schema::users::id.eq(schema::user_teams::user_id)))
        .select((schema::users::id, schema::users::username))
        .filter(schema::users::username.is_not_null())
        .filter(schema::user_teams::invite_from.is_null())
        .load::<(i32, Option<String>)>(&mut db)
        .await?
        .into_iter()
        .filter_map(|(id, username)| username.map(|username| dto::user::UserRef { id, username }))
        .collect();

    Ok(Json(dto::teams::TeamDetails {
        team: dto::teams::TeamRef {
            id: team.id,
            slug: team.slug,
            name: team.name,
        },
        description: team.description,
        links: team.links,
        users,
    }))
}

#[get("/teams/<id>", rank = 1)]
async fn teams_details_id(
    mut db: Db,
    id: i32,
) -> Result<Json<dto::teams::TeamDetails>, crate::error::Error> {
    let team = schema::teams::table
        .filter(schema::teams::id.eq(id))
        .first::<models::Team>(&mut db)
        .await?;

    team_details_from_(db, team).await
}

#[get("/teams/<slug>", rank = 2)]
async fn teams_details_slug(
    mut db: Db,
    slug: String,
) -> Result<Json<dto::teams::TeamDetails>, crate::error::Error> {
    let team = schema::teams::table
        .filter(schema::teams::slug.eq(&slug))
        .first::<models::Team>(&mut db)
        .await?;

    team_details_from_(db, team).await
}

#[post("/teams/<team_id>/invite", data = "<form>", rank = 1)]
async fn invite(
    mut db: Db,
    invitee: AuthenticatedUserGuard,
    team_id: i32,
    form: Json<dto::teams::TeamInvite<'_>>,
) -> Result<Json<dto::Ok>, crate::error::Error> {
    db.transaction(|db| {
        async move {
            let dto::teams::TeamInvite { user, role } = form.into_inner();
            let user = match user {
                UserId::Id(id) => User::from_id(db, id).await?,
                UserId::Username(ref name) => User::from_username(db, name).await?,
            };

            let role = UserTeamRole::from(role);

            // Verify that the admin has the right to invite.
            let invitee_user = User::from_id(db, invitee.id).await?;
            let invitee_role = invitee_user
                .role_in(db, team_id)
                .await?
                .ok_or(crate::error::Error::RecordNotFound)?;
            if invitee_role > role {
                return Err(crate::error::Error::Unauthorized);
            }

            user.invite_to(db, invitee.id, team_id, role).await?;

            Ok(Json(dto::Ok))
        }
        .scope_boxed()
    })
    .await
}

#[post("/teams/<team_id>/invite/accept", rank = 1)]
async fn invite_accept(
    mut db: Db,
    invited: AuthenticatedUserGuard,
    team_id: i32,
) -> Result<Json<dto::Ok>, crate::error::Error> {
    db.transaction(|db| {
        async move {
            let user = models::User::from_id(db, invited.id).await?;
            user.accept_invitation(db, team_id).await?;

            Ok(Json(dto::Ok))
        }
        .scope_boxed()
    })
    .await
}

pub fn routes() -> Vec<Route> {
    routes![
        teams,
        teams_details_id,
        teams_details_slug,
        invite,
        invite_accept
    ]
}

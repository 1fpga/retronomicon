use crate::db::Db;
use crate::guards::users::AuthenticatedUserGuard;
use crate::models::{User, UserTeamRole};
use crate::{models, schema};
use retronomicon_dto as dto;
use retronomicon_dto::user::UserIdOrUsername;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;
use serde_json::json;

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams/?<paging..>")]
pub async fn teams(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::teams::Team>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    schema::teams::table
        .offset(page * limit)
        .limit(limit)
        .load::<models::Team>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
        .map(|t| Json(t.into_iter().map(Into::into).collect()))
}

async fn team_details_from_(
    mut db: Db,
    team: models::Team,
) -> Result<Json<dto::teams::TeamDetails>, (Status, String)> {
    let users = models::UserTeam::belonging_to(&team)
        .inner_join(schema::users::table.on(schema::users::id.eq(schema::user_teams::user_id)))
        .select((schema::users::id, schema::users::username))
        .filter(schema::users::username.is_not_null())
        .filter(schema::user_teams::invite_from.is_null())
        .load::<(i32, Option<String>)>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
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
        links: team.links.unwrap_or_else(|| json!({})),
        metadata: json!({}),
        users,
    }))
}

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams/<id>", rank = 1)]
pub async fn teams_details_id(
    mut db: Db,
    id: i32,
) -> Result<Json<dto::teams::TeamDetails>, (Status, String)> {
    let team = schema::teams::table
        .filter(schema::teams::id.eq(id))
        .first::<models::Team>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    team_details_from_(db, team).await
}

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams/<slug>", rank = 2)]
pub async fn teams_details_slug(
    mut db: Db,
    slug: String,
) -> Result<Json<dto::teams::TeamDetails>, (Status, String)> {
    let team = schema::teams::table
        .filter(schema::teams::slug.eq(&slug))
        .first::<models::Team>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    team_details_from_(db, team).await
}

#[openapi(tag = "Teams", ignore = "db")]
#[post("/teams/<team_id>/invite", data = "<form>", rank = 1)]
pub async fn invite(
    mut db: Db,
    invitee: AuthenticatedUserGuard,
    team_id: i32,
    form: Json<dto::teams::TeamInvite<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let dto::teams::TeamInvite { user, role } = form.into_inner();
    let user = match user {
        UserIdOrUsername::Id(id) => User::from_id(db, id).await,
        UserIdOrUsername::Username(ref name) => User::from_username(db, name).await,
    }
    .map_err(|e| (Status::NotFound, e.to_string()))?;

    let role = UserTeamRole::from(role);

    // Verify that the admin has the right to invite.
    let invitee_user = User::from_id(db, invitee.id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;
    let invitee_role = invitee_user
        .role_in(db, team_id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?
        .ok_or((Status::NotFound, "User not part of the team".to_string()))?;
    if invitee_role > role {
        return Err((Status::Unauthorized, "Insufficient permissions".to_string()));
    }

    user.invite_to(db, invitee.id, team_id, role)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;

    Ok(Json(dto::Ok))
}

#[openapi(tag = "Teams", ignore = "db")]
#[post("/teams/<team_id>/invite/accept", rank = 1)]
pub async fn invite_accept(
    mut db: Db,
    invited: AuthenticatedUserGuard,
    team_id: i32,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let user = models::User::from_id(db, invited.id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;
    user.accept_invitation(db, team_id)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;

    Ok(Json(dto::Ok))
}

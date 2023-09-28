use crate::db::Db;
use crate::guards::users::AuthenticatedUserGuard;
use crate::models::{User, UserTeamRole};
use crate::types::FetchModel;
use crate::utils::json;
use crate::{models, schema};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_db_pools::diesel::prelude::*;
use rocket_okapi::openapi;
use serde_json::{json, Value};

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams?<paging..>")]
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

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams/<id>")]
pub async fn teams_details(
    mut db: Db,
    id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::teams::TeamDetails>, (Status, String)> {
    let team = if let Some(id) = id.as_id() {
        schema::teams::table
            .filter(schema::teams::id.eq(id))
            .first::<models::Team>(&mut db)
            .await
    } else if let Some(slug) = id.as_slug() {
        schema::teams::table
            .filter(schema::teams::slug.eq(slug))
            .first::<models::Team>(&mut db)
            .await
    } else {
        return Err((Status::BadRequest, "Invalid id or slug".to_string()));
    }
    .optional()
    .map_err(|e| (Status::InternalServerError, e.to_string()))?
    .ok_or((Status::NotFound, "Team not found".to_string()))?;

    let users = models::UserTeam::belonging_to(&team)
        .inner_join(schema::users::table.on(schema::users::id.eq(schema::user_teams::user_id)))
        .select((
            schema::users::id,
            schema::users::username,
            schema::user_teams::role,
        ))
        .filter(schema::users::username.is_not_null())
        .filter(schema::user_teams::invite_from.is_null())
        .load::<(i32, Option<String>, models::UserTeamRole)>(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .into_iter()
        .filter_map(|(id, username, role)| {
            username.map(|username| dto::teams::TeamUserRef {
                user: dto::user::UserRef { id, username },
                role: role.into(),
            })
        })
        .collect();

    let links =
        json::links_into_btree_map(team.links).map_err(|e| (Status::InternalServerError, e))?;
    let metadata = json::metadata_into_btree_map(team.metadata)
        .map_err(|e| (Status::InternalServerError, e))?;

    Ok(Json(dto::teams::TeamDetails {
        team: dto::teams::TeamRef {
            id: team.id,
            slug: team.slug,
            name: team.name,
        },
        description: team.description,
        links,
        metadata,
        users,
    }))
}

/// Create a new team, and make the current user its owner.
#[openapi(tag = "Teams", ignore = "db")]
#[post("/teams", data = "<form>", rank = 1)]
pub async fn teams_create(
    mut db: Db,
    owner: AuthenticatedUserGuard,
    form: Json<dto::teams::TeamCreateRequest<'_>>,
) -> Result<Json<dto::teams::TeamCreateResponse>, (Status, String)> {
    let db = &mut db;
    let dto::teams::TeamCreateRequest {
        slug,
        name,
        description,
        links,
        metadata,
    } = form.into_inner();

    // Links must be only a map of strings.
    let links = links.map(|l| json!(l));
    // Metadata must be a map.
    let metadata = metadata.map(|m| json!(m));
    let user = owner
        .into_model(db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    let team = models::Team::create(
        db,
        slug,
        name,
        description,
        links.unwrap_or_else(|| json!({})),
        metadata.unwrap_or_else(|| json!({})),
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    user.join_team(db, team.id, UserTeamRole::Owner)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::teams::TeamCreateResponse {
        id: team.id,
        slug: team.slug,
    }))
}

/// Create a new team, and make the current user its owner.
#[openapi(tag = "Teams", ignore = "db")]
#[put("/teams/<team_id>", data = "<form>", rank = 1)]
pub async fn teams_update(
    mut db: Db,
    owner: AuthenticatedUserGuard,
    team_id: dto::types::IdOrSlug<'_>,
    form: Json<dto::teams::TeamUpdateRequest<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let mut team = models::Team::from_id_or_slug(db, team_id).await?;

    let user = owner
        .into_model(db)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;
    if user.role_in(db, team.id).await != Ok(Some(UserTeamRole::Owner)) {
        return Err((Status::Unauthorized, "Insufficient permissions".to_string()));
    }
    let dto::teams::TeamUpdateRequest {
        slug,
        name,
        description,
        links,
        metadata,
        add_links,
        remove_links,
    } = form.into_inner();

    let links = if let Some(links) = links {
        Some(json!(links))
    } else if add_links.is_some() || remove_links.is_some() {
        let links = team.links.as_object_mut().unwrap();
        if let Some(add_links) = add_links {
            for (k, v) in add_links {
                links[k] = Value::String(v.to_string());
            }
        }
        if let Some(remove_links) = remove_links {
            for k in remove_links {
                links.remove(&k.to_string());
            }
        }
        Some(json!(links))
    } else {
        None
    };

    let metadata = metadata.map(|m| json!(m));

    models::teams::Team::update(db, team.id, slug, name, description, links, metadata)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;
    Ok(Json(dto::Ok))
}

#[openapi(tag = "Teams", ignore = "db")]
#[post("/teams/<team_id>/invitation", data = "<form>", rank = 1)]
pub async fn invite(
    mut db: Db,
    invitee: AuthenticatedUserGuard,
    team_id: i32,
    form: Json<dto::teams::TeamInvite<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let dto::teams::TeamInvite { user, role } = form.into_inner();
    let user = models::User::from_userid(db, user)
        .await
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
#[post("/teams/<team_id>/invitation/accept", rank = 1)]
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

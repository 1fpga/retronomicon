use crate::guards::users::AuthenticatedUserGuard;
use crate::utils::{acls, json};
use retronomicon_db::models;
use retronomicon_db::models::Team;
use retronomicon_db::types::FetchModel;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use rocket_okapi::openapi;
use serde_json::{json, Value};

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams?<paging..>")]
pub async fn teams(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::teams::Team>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    models::Team::list(&mut db, page, limit)
        .await
        .map(|t| Json(t.into_iter().map(Into::into).collect()))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

#[openapi(tag = "Teams", ignore = "db")]
#[get("/teams/<id>")]
pub async fn teams_details(
    mut db: Db,
    id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::teams::TeamDetails>, (Status, String)> {
    let team = Team::from_id_or_slug(&mut db, id).await?;
    let users = team
        .users_ref(&mut db)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

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
#[post("/teams", data = "<form>")]
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

    if !acls::can_create_team(&user) {
        return Err((Status::Unauthorized, "Insufficient permissions".to_string()));
    }

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

    user.join_team(db, team.id, models::UserTeamRole::Owner)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::teams::TeamCreateResponse {
        id: team.id,
        slug: team.slug,
    }))
}

/// Create a new team, and make the current user its owner.
#[openapi(tag = "Teams", ignore = "db")]
#[put("/teams/<team_id>", data = "<form>")]
pub async fn teams_update(
    mut db: Db,
    owner: AuthenticatedUserGuard,
    team_id: dto::types::IdOrSlug<'_>,
    form: Json<dto::teams::TeamUpdateRequest<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let (user, mut team, role) = models::User::get_user_team_and_role(db, owner.id.into(), team_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Not found".to_string()))?;

    if !acls::can_update_team(&user, &team, &role) {
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
#[delete("/teams/<team_id>")]
pub async fn teams_delete(
    mut db: Db,
    admin: AuthenticatedUserGuard,
    team_id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let (user, team, role) = models::User::get_user_team_and_role(db, admin.into(), team_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Not found".to_string()))?;

    if !acls::can_delete_team(&user, &team, &role) {
        return Err((Status::Unauthorized, "Insufficient permissions".to_string()));
    }

    models::Team::delete(db, team.id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;
    Ok(Json(dto::Ok))
}

#[openapi(tag = "Teams", ignore = "db")]
#[post("/teams/<team_id>/invitation", data = "<form>")]
pub async fn invite(
    mut db: Db,
    admin: AuthenticatedUserGuard,
    team_id: i32,
    form: Json<dto::teams::TeamInvite<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let db = &mut db;
    let (admin_user, team, admin_role) =
        models::User::get_user_team_and_role(db, admin.id.into(), team_id.into())
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::NotFound, "Not found".to_string()))?;

    let dto::teams::TeamInvite { user, role } = form.into_inner();
    let user = models::User::from_userid(db, user)
        .await
        .map_err(|e| (Status::NotFound, e.to_string()))?;
    let role = models::UserTeamRole::from(role);

    if !acls::can_invite_to_team(&team, &admin_user, &admin_role, &user, &role) {
        return Err((Status::Unauthorized, "Insufficient permissions".to_string()));
    }

    user.invite_to(db, admin_user.id, team.id, role)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::Ok))
}

#[openapi(tag = "Teams", ignore = "db")]
#[post("/teams/<team_id>/invitation/accept")]
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

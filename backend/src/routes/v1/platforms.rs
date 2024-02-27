use crate::guards;
use retronomicon_db::models;
use retronomicon_db::types::FetchModel;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_okapi::openapi;
use serde_json::json;

#[openapi(tag = "Platforms", ignore = "db")]
#[get("/platforms?<paging..>")]
pub async fn platforms_list(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::platforms::Platform>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    models::Platform::list(&mut db, page, limit)
        .await
        .map(|p| Json(p.into_iter().map(Into::into).collect()))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

#[openapi(tag = "Platforms", ignore = "db")]
#[get("/platforms/<platform_id>")]
pub async fn platforms_details(
    mut db: Db,
    platform_id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::platforms::PlatformDetails>, (Status, String)> {
    let (platform, owner_team) = models::Platform::get_with_owner(&mut db, platform_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "User not found".to_string()))?;

    Ok(Json(dto::platforms::PlatformDetails {
        id: platform.id,
        slug: platform.slug,
        name: platform.name,
        description: platform.description,
        links: platform.links,
        metadata: platform.metadata,
        owner_team: owner_team.into(),
    }))
}

#[openapi(tag = "Platforms", ignore = "db")]
#[post("/platforms/new", format = "application/json", data = "<form>")]
pub async fn platforms_create(
    mut db: Db,
    user: guards::users::AuthenticatedUserGuard,
    form: Json<dto::platforms::PlatformCreateRequest<'_>>,
) -> Result<Json<dto::platforms::PlatformCreateResponse>, (Status, String)> {
    let dto::platforms::PlatformCreateRequest {
        slug,
        name,
        description,
        links,
        metadata,
        owner_team,
    } = form.into_inner();

    // Get team.
    let team = models::Team::from_id_or_slug(&mut db, owner_team).await?;

    let user = user.into_model(&mut db).await?;

    // Check permissions.
    let role = user
        .role_in(&mut db, team.id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::Forbidden, "Not a member of the team".to_string()))?;
    if role < models::UserTeamRole::Admin {
        return Err((Status::Forbidden, "Not enough permission".to_string()));
    }

    // Create platform.
    let platform = models::Platform::create(
        &mut db,
        slug,
        name,
        description,
        json!(links.unwrap_or_default()),
        json!(metadata.unwrap_or_default()),
        &team,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::platforms::PlatformCreateResponse {
        id: platform.id,
        slug: platform.slug,
    }))
}

#[openapi(tag = "Platforms", ignore = "db")]
#[put(
    "/platforms/<platform_id>",
    format = "application/json",
    data = "<form>"
)]
pub async fn platforms_update(
    mut db: Db,
    user: guards::users::AuthenticatedUserGuard,
    platform_id: dto::types::IdOrSlug<'_>,
    form: Json<dto::platforms::PlatformUpdateRequest<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    let dto::platforms::PlatformUpdateRequest {
        slug,
        name,
        description,
        links,
        metadata,
        team_id,
    } = form.into_inner();

    let platform = models::Platform::from_id_or_slug(&mut db, platform_id).await?;

    let user = user.into_model(&mut db).await?;

    // Check permissions.
    let role_in_old_team = user
        .role_in(&mut db, platform.owner_team_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::Forbidden, "Not a member of the team".to_string()))?;
    if role_in_old_team < models::UserTeamRole::Admin {
        return Err((Status::Forbidden, "Not enough permission".to_string()));
    }
    if let Some(team_id) = team_id {
        let role_in_new_team = user
            .role_in(&mut db, team_id)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((
                Status::Forbidden,
                "Not a member of the new team".to_string(),
            ))?;
        if role_in_new_team < models::UserTeamRole::Admin {
            return Err((Status::Forbidden, "Not enough permission".to_string()));
        }
    }

    // Create platform.
    models::Platform::update(
        &mut db,
        platform.id,
        slug,
        name,
        description,
        links,
        metadata,
        team_id,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::Ok))
}

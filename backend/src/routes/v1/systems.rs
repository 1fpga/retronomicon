use crate::guards;
use crate::utils::json;
use retronomicon_db::models;
use retronomicon_db::types::FetchModel;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::openapi;
use serde_json::json;
use std::collections::BTreeMap;

#[openapi(tag = "Systems", ignore = "db")]
#[get("/systems?<paging..>")]
pub async fn systems_list(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::systems::SystemListItem>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;
    let system_list = models::System::list_with_team(&mut db, page, limit)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(
        system_list
            .into_iter()
            .map(|(s, t)| dto::systems::SystemListItem {
                id: s.id,
                slug: s.slug,
                manufacturer: s.manufacturer,
                name: s.name,
                owner_team: t.into(),
            })
            .collect(),
    ))
}

#[openapi(tag = "Systems", ignore = "db")]
#[post("/systems/new", format = "application/json", data = "<form>")]
pub async fn systems_create(
    mut db: Db,
    user: guards::users::AuthenticatedUserGuard,
    form: Json<dto::systems::SystemCreateRequest<'_>>,
) -> Result<Json<dto::systems::SystemCreateResponse>, (Status, String)> {
    let dto::systems::SystemCreateRequest {
        slug,
        name,
        description,
        manufacturer,
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

    if !role.can_create_systems() {
        return Err((Status::Forbidden, "Not enough permission".to_string()));
    }

    // Create system.
    let system = models::System::create(
        &mut db,
        slug,
        name,
        description,
        manufacturer,
        json!(links.unwrap_or_else(|| BTreeMap::new())),
        json!(metadata.unwrap_or_else(|| BTreeMap::new())),
        team.id,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::systems::SystemCreateResponse {
        id: system.id,
        slug: system.slug,
    }))
}

#[openapi(tag = "Systems", ignore = "db")]
#[get("/systems/<id>")]
pub async fn systems_details(
    mut db: Db,
    id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::systems::SystemDetails>, (Status, String)> {
    let system = models::System::from_id_or_slug(&mut db, id).await?;
    let team = models::Team::from_id(&mut db, system.owner_team_id).await?;
    let links =
        json::links_into_btree_map(system.links).map_err(|e| (Status::InternalServerError, e))?;
    let metadata = json::metadata_into_btree_map(system.metadata)
        .map_err(|e| (Status::InternalServerError, e))?;

    Ok(Json(dto::systems::SystemDetails {
        id: system.id,
        slug: system.slug,
        name: system.name,
        description: system.description,
        manufacturer: system.manufacturer,
        links,
        metadata,
        owner_team: team.into(),
    }))
}

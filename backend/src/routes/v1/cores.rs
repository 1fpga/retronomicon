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

pub mod releases;

/// List cores.
#[openapi(tag = "Cores", ignore = "db")]
#[get("/cores?<filter..>")]
pub async fn cores_list(
    mut db: Db,
    filter: dto::cores::CoreListQueryParams<'_>,
) -> Result<Json<dto::Paginated<dto::cores::CoreListItem>>, (Status, String)> {
    let (page, limit) = filter
        .paging()
        .validate()
        .map_err(|e| (Status::BadRequest, e))?;

    let platform = match filter.platform {
        Some(platform) => Some(models::Platform::from_id_or_slug(&mut db, platform).await?),
        None => None,
    };
    let system = match filter.system {
        Some(system) => Some(models::System::from_id_or_slug(&mut db, system).await?),
        None => None,
    };
    let team = match filter.owner_team {
        Some(team) => Some(models::Team::from_id_or_slug(&mut db, team).await?),
        None => None,
    };
    let release = filter
        .release_date_ge
        .and_then(|release| chrono::NaiveDateTime::from_timestamp_opt(release, 0));

    let paginated = models::Core::list_with_teams_and_releases(
        &mut db,
        page,
        limit,
        platform.as_ref(),
        system.as_ref(),
        team.as_ref(),
        release,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(paginated.map_items(
        |(core, system, team, core_release, platform)| dto::cores::CoreListItem {
            id: core.id,
            slug: core.slug,
            name: core.name,
            owner_team: team.into(),
            system: system.into(),
            latest_release: core_release.map(|cr| cr.into_ref(platform)),
        },
    )))
}

#[openapi(tag = "Cores", ignore = "db")]
#[get("/cores/<core_id>")]
pub async fn cores_details(
    mut db: Db,
    core_id: dto::types::IdOrSlug<'_>,
) -> Result<Json<dto::cores::CoreDetailsResponse>, (Status, String)> {
    let (core, owner_team, system) = models::Core::get_with_owner_and_system(&mut db, core_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Core not found".to_string()))?;

    Ok(Json(dto::cores::CoreDetailsResponse {
        id: core.id,
        slug: core.slug,
        name: core.name,
        description: core.description,
        metadata: json::metadata_into_btree_map(core.metadata)
            .map_err(|e| (Status::InternalServerError, e.to_string()))?,
        links: json::links_into_btree_map(core.links)
            .map_err(|e| (Status::InternalServerError, e.to_string()))?,
        system: system.into(),
        owner_team: owner_team.into(),
    }))
}

#[openapi(tag = "Cores", ignore = "db")]
#[post("/cores", format = "application/json", data = "<form>")]
pub async fn cores_create(
    mut db: Db,
    user: guards::users::AuthenticatedUserGuard,
    form: Json<dto::cores::CoreCreateRequest<'_>>,
) -> Result<Json<dto::cores::CoreCreateResponse>, (Status, String)> {
    let dto::cores::CoreCreateRequest {
        slug,
        name,
        description,
        metadata,
        links,
        system,
        owner_team,
    } = form.into_inner();

    let system = models::System::from_id_or_slug(&mut db, system).await?;
    let (_user, team, role) =
        models::User::get_user_team_and_role(&mut db, user.into(), owner_team)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::NotFound, "Not found".to_string()))?;

    if !role.can_create_cores() {
        return Err((Status::Forbidden, "User cannot create cores".to_string()));
    }

    let core = models::Core::create(
        &mut db,
        slug,
        name,
        description,
        json!(links),
        json!(metadata),
        &system,
        &team,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::cores::CoreCreateResponse {
        id: core.id,
        slug: core.slug,
    }))
}

use crate::db::Db;
use crate::types::FetchModel;
use crate::{guards, models};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::openapi;
use serde_json::json;

#[openapi(tag = "Cores", ignore = "db")]
#[get("/cores?<paging..>")]
pub async fn cores_list(
    mut db: Db,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::cores::CoreListItem>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    Ok(Json(
        models::Core::list_with_teams(&mut db, page, limit)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .into_iter()
            .map(|(core, team)| dto::cores::CoreListItem {
                id: core.id,
                slug: core.slug,
                name: core.name,
                owner_team: team.into(),
            })
            .collect(),
    ))
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
    let (_user, team, role) = models::User::get_user_team_and_role(&mut db, user.id, owner_team)
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

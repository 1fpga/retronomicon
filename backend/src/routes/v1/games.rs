use crate::db::Db;
use crate::utils::acls;
use crate::{guards, models};
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::openapi;
use serde_json::json;

#[openapi(tag = "Games", ignore = "db")]
#[post("/games", format = "application/json", data = "<form>")]
pub async fn games_create(
    mut db: Db,
    user: guards::users::AuthenticatedUserGuard,
    form: Json<dto::games::GameCreateRequest<'_>>,
) -> Result<Json<dto::games::GameCreateResponse>, (Status, String)> {
    if !acls::can_create_games(&mut db, user.id).await {
        return Err((Status::Forbidden, "Insufficient permissions".to_string()));
    }

    let dto::games::GameCreateRequest {
        name,
        short_description,
        description,
        year,
        publisher,
        developer,
        links,
        system,
        system_unique_id,
    } = form.into_inner();
    let system = models::System::get(&mut db, system)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Not found".to_string()))?;

    let game = models::Game::create(
        &mut db,
        name,
        description,
        short_description,
        year,
        publisher,
        developer,
        json!(links),
        system.id,
        system_unique_id,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::games::GameCreateResponse { id: game.id }))
}

#[openapi(tag = "Games", ignore = "db")]
#[get("/games?<filter..>")]
pub async fn games_list(
    mut db: Db,
    filter: dto::games::GameListQueryParams<'_>,
) -> Result<Json<Vec<dto::games::GameListItemResponse>>, (Status, String)> {
    let (page, limit) = filter
        .paging
        .validate()
        .map_err(|e| (Status::BadRequest, e))?;

    let year = filter.year.unwrap_or_default().into();
    let name = filter.name.as_deref();

    Ok(Json(
        models::Game::list(&mut db, page, limit, filter.system, year, name)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .into_iter()
            .map(|(game, system)| dto::games::GameListItemResponse {
                id: game.id,
                name: game.name,
                short_description: game.short_description,
                year: game.year,
                system_id: system.into(),
                system_unique_id: game.system_unique_id,
            })
            .collect(),
    ))
}

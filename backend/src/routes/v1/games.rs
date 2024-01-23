use crate::guards;
use retronomicon_db::models;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_db_pools::diesel::AsyncConnection;
use rocket_okapi::openapi;
use scoped_futures::ScopedFutureExt;
use serde_json::json;
use std::collections::BTreeMap;

#[openapi(tag = "Games", ignore = "db")]
#[post("/games", format = "application/json", data = "<form>")]
pub async fn games_create(
    mut db: Db,
    _root_user: guards::users::RootUserGuard,
    form: Json<dto::games::GameCreateRequest<'_>>,
) -> Result<Json<dto::games::GameCreateResponse>, (Status, String)> {
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
#[get("/games?<filter..>", format = "application/json", data = "<form>")]
pub async fn games_list(
    mut db: Db,
    filter: dto::games::GameListQueryParams<'_>,
    form: Json<dto::games::GameListBody>,
) -> Result<Json<Vec<dto::games::GameListItemResponse>>, (Status, String)> {
    let (page, limit) = filter
        .paging
        .validate()
        .map_err(|e| (Status::BadRequest, e))?;

    let year = filter.year.unwrap_or_default().into();
    let name = filter.name.as_deref();
    let exact_name = filter.exact_name.as_deref();

    let mut result = BTreeMap::new();
    let form = form.into_inner();
    let md5 = form
        .md5
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.into())
        .collect();
    let sha1 = form
        .sha1
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.into())
        .collect();
    let sha256 = form
        .sha256
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.into())
        .collect();

    for (g, s, a) in models::Game::list(
        &mut db,
        page,
        limit,
        filter.system,
        year,
        name,
        exact_name,
        md5,
        sha1,
        sha256,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?
    .into_iter()
    {
        let entry = result
            .entry(g.id)
            .or_insert_with(|| dto::games::GameListItemResponse {
                id: g.id,
                name: g.name,
                short_description: g.short_description,
                year: g.year,
                system_id: s.into(),
                system_unique_id: g.system_unique_id,
                artifacts: vec![],
            });
        if let Some(a) = a {
            entry.artifacts.push(a.into());
        }
    }

    Ok(Json(result.into_values().collect::<Vec<_>>()))
}

#[openapi(tag = "Games", ignore = "db")]
#[get("/games/<game_id>")]
pub async fn games_details(
    mut db: Db,
    game_id: u32,
) -> Result<Json<dto::games::GameDetails>, (Status, String)> {
    let (game, system) = models::Game::details(&mut db, game_id as i32)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::games::GameDetails {
        id: game.id,
        name: game.name,
        description: game.description,
        short_description: game.short_description,
        year: game.year,
        publisher: game.publisher,
        developer: game.developer,
        links: game.links,
        system: system.into(),
        system_unique_id: game.system_unique_id,
    }))
}

#[openapi(tag = "Games", ignore = "db")]
#[put("/games/<game_id>", format = "application/json", data = "<form>")]
pub async fn games_update(
    mut db: Db,
    _root_user: guards::users::RootUserGuard,
    game_id: u32,
    form: Json<dto::games::GameUpdateRequest<'_>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    models::Game::update(
        &mut db,
        game_id as i32,
        form.name,
        form.description,
        form.short_description,
        form.year,
        form.publisher,
        form.developer,
        form.add_links.clone(),
        form.remove_links.clone(),
        form.system_unique_id,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::Ok))
}

#[openapi(tag = "Games", ignore = "db")]
#[post(
    "/games/<game_id>/artifacts",
    format = "application/json",
    data = "<form>"
)]
pub async fn games_add_artifact(
    mut db: Db,
    _root_user: guards::users::RootUserGuard,
    game_id: u32,
    form: Json<Vec<dto::games::GameAddArtifactRequest<'_>>>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    db.transaction(|db| {
        async move {
            let game = models::Game::get(db, game_id as i32).await?;
            let game = match game {
                Some(g) => g,
                None => return Ok(None),
            };

            for a in form.into_inner() {
                let artifact = models::Artifact::create_with_checksum(
                    db,
                    "",
                    a.mime_type,
                    a.md5.as_ref().map(|s| s.as_slice()),
                    a.sha1.as_ref().map(|s| s.as_slice()),
                    a.sha256.as_ref().map(|s| s.as_slice()),
                    None,
                    a.size,
                )
                .await?;

                models::GameArtifact::create(db, game.id, artifact.id).await?;
            }

            Ok(Some(()))
        }
        .scope_boxed()
    })
    .await
    .map_err(|e: diesel::result::Error| (Status::InternalServerError, e.to_string()))?
    .ok_or((Status::NotFound, "Not found".to_string()))?;

    Ok(Json(dto::Ok))
}

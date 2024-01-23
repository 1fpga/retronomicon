use crate::fairings::config::RetronomiconConfig;
use crate::guards;
use crate::utils::acls;
use image::{GenericImageView, ImageFormat};
use retronomicon_db::models;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::data::ToByteUnit;
use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use rocket::{get, post, put, Data, State};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, Repetition,
};
use rocket_okapi::openapi;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::BufReader;

const MAX_IMAGE_WIDTH: u32 = 4096;
const MAX_IMAGE_HEIGHT: u32 = 4096;

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
    let game = models::Game::get(&mut db, game_id as i32)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Not found".to_string()))?;

    for a in form.into_inner() {
        let artifact = models::Artifact::create_with_checksum(
            &mut db,
            "",
            a.mime_type,
            a.md5.as_ref().map(|s| s.as_slice()),
            a.sha1.as_ref().map(|s| s.as_slice()),
            a.sha256.as_ref().map(|s| s.as_slice()),
            None,
            a.size,
        )
        .await
        .map_err(|e: _| (Status::InternalServerError, e.to_string()))?;

        models::GameArtifact::create(&mut db, game.id, artifact.id)
            .await
            .map_err(|e: _| (Status::InternalServerError, e.to_string()))?;
    }

    Ok(Json(dto::Ok))
}

#[openapi(tag = "Games", ignore = "db")]
#[get("/games/<game_id>/images?<filter..>")]
pub async fn games_images(
    mut db: Db,
    game_id: u32,
    filter: dto::games::GameImageListQueryParams,
) -> Result<Json<Vec<dto::images::Image>>, (Status, String)> {
    let (page, limit) = filter
        .paging
        .validate()
        .map_err(|e| (Status::BadRequest, e))?;
    let images = models::GameImage::list(&mut db, page, limit, game_id as i32)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .into_iter()
        .map(|(i, _)| {
            let url = format!("/images/games/{}/{}", game_id, i.image_name);
            dto::images::Image {
                name: i.image_name,
                url,
                mime_type: i.mime_type,
            }
        })
        .collect();

    Ok(Json(images))
}

/// Upload an image to a game. This can be done multiple times (as long
/// as the filename is unique).
/// The upload will be refused if the user does not have permission to
/// upload images to this game.
#[openapi(tag = "Games", ignore = "config", ignore = "db", ignore = "storage")]
#[post("/cores/<game_id>/images", data = "<file>")]
pub async fn games_images_upload(
    mut db: Db,
    admin: guards::users::AuthenticatedUserGuard,
    config: &State<RetronomiconConfig>,
    storage: guards::storage::Storage,
    game_id: i32,
    content_type: &ContentType,
    file: Data<'_>,
) -> Result<Json<dto::Ok>, (Status, String)> {
    // Check the uploader's role.
    let (user, team, role) =
        models::User::get_user_team_and_role(&mut db, admin.into(), config.root_team_id.into())
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::Unauthorized, "Not logged in".to_string()))?;

    if !acls::can_upload_image(&user, &team, &role) {
        return Err((Status::Forbidden, "Forbidden".to_string()));
    }

    let _game = models::Game::get(&mut db, game_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Not found".to_string()))?;

    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file")
            .size_limit(24.mebibytes().as_u64())
            .repetition(Repetition::infinite()),
    ]);
    options.max_data_bytes = 2.mebibytes().as_u64();
    let multipart_form_data = MultipartFormData::parse(content_type, file, options)
        .await
        .map_err(|e| (Status::BadRequest, e.to_string()))?;

    for (_name, files) in &multipart_form_data.files {
        for file in files {
            let filename = file
                .file_name
                .clone()
                .ok_or((Status::BadRequest, "Filename not specified.".to_string()))?;
            let mimetype = file.content_type.clone().ok_or((
                Status::BadRequest,
                "Content-Type not specified.".to_string(),
            ))?;
            if !models::GameImage::is_filename_conform(&mut db, game_id, &filename)
                .await
                .map_err(|e| (Status::InternalServerError, e.to_string()))?
            {
                return Err((Status::BadRequest, "Filename is invalid".to_string()));
            }
            let f = std::fs::File::open(&file.path)
                .map_err(|e| (Status::InternalServerError, e.to_string()))?;

            // Try to figure out the image format based on mimetype.
            let image_format = match mimetype.essence_str() {
                "image/png" => ImageFormat::Png,
                "image/jpeg" => ImageFormat::Jpeg,
                "image/gif" => ImageFormat::Gif,
                _ => {
                    return Err((
                        Status::BadRequest,
                        format!("Unsupported image format: {}", mimetype),
                    ))
                }
            };

            let image = image::load(BufReader::new(f), image_format)
                .map_err(|e| (Status::InternalServerError, e.to_string()))?;

            let (width, height) = image.dimensions();
            if width > MAX_IMAGE_WIDTH || height > MAX_IMAGE_HEIGHT {
                return Err((
                    Status::BadRequest,
                    format!(
                        "Image is too large ({}x{} > {}x{})",
                        width, height, MAX_IMAGE_WIDTH, MAX_IMAGE_HEIGHT
                    ),
                ));
            }

            storage
                .upload_image(
                    &format!("games/{}/{}", game_id, filename),
                    image.as_bytes(),
                    mimetype.essence_str(),
                )
                .await
                .map_err(|e| (Status::InternalServerError, e.to_string()))?;

            let _artifact = models::GameImage::create(
                &mut db,
                game_id,
                &filename,
                width as i32,
                height as i32,
                mimetype.essence_str(),
            )
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?;
        }
    }

    Ok(Json(dto::Ok))
}

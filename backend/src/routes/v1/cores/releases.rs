use crate::guards;
use crate::utils::acls;
use retronomicon_db::models;
use retronomicon_db::types::FetchModel;
use retronomicon_db::Db;
use retronomicon_dto as dto;
use rocket::data::ToByteUnit;
use rocket::http::{ContentType, Header, Status};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{get, post, Data, Request, Response};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions, Repetition,
};
use rocket_okapi::openapi;
use serde_json::json;
use sha1::Digest;
use std::io::Cursor;
use std::path::PathBuf;

#[openapi(tag = "Core Releases", ignore = "db")]
#[get("/cores/<core_id>/releases?<paging>&<filter>")]
pub async fn cores_releases_list(
    mut db: Db,
    core_id: dto::types::IdOrSlug<'_>,
    paging: dto::params::PagingParams,
    filter: dto::cores::releases::CoreReleaseFilterParams<'_>,
) -> Result<Json<Vec<dto::cores::releases::CoreReleaseListItem>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    Ok(Json(
        models::CoreRelease::list(&mut db, core_id, page, limit, filter)
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .into_iter()
            .map(
                |(release, platform, core, uploader)| dto::cores::releases::CoreReleaseListItem {
                    release: release.into_ref(platform),
                    core: dto::cores::CoreRef {
                        id: core.id,
                        slug: core.slug,
                        name: core.name,
                    },
                    uploader: uploader.into(),
                },
            )
            .collect(),
    ))
}

/// Create a release for a core. This does not include any artifacts, which
/// must be uploaded separately.
#[openapi(tag = "Core Releases", ignore = "db")]
#[post("/cores/<core_id>/releases", format = "json", data = "<input>")]
pub async fn cores_releases_create(
    mut db: Db,
    admin: guards::users::AuthenticatedUserGuard,
    core_id: dto::types::IdOrSlug<'_>,
    input: Json<dto::cores::releases::CoreReleaseCreateRequest<'_>>,
) -> Result<Json<dto::cores::releases::CoreReleaseCreateResponse>, (Status, String)> {
    let dto::cores::releases::CoreReleaseCreateRequest {
        version,
        notes,
        date_released,
        prerelease,
        links,
        metadata,
        platform,
    } = input.into_inner();

    if version == "latest" {
        return Err((Status::BadRequest, "Version cannot be 'latest'".to_string()));
    }

    let core = models::Core::from_id_or_slug(&mut db, core_id).await?;
    let platform = models::Platform::from_id_or_slug(&mut db, platform).await?;

    let (user, team, role) =
        models::User::get_user_team_and_role(&mut db, admin.into(), core.owner_team_id.into())
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::Unauthorized, "Not logged in".to_string()))?;

    if !acls::can_create_core_releases(&user, &team, &role, &core).await {
        return Err((Status::Forbidden, "Not authorized".to_string()));
    }

    let timestamp = chrono::NaiveDateTime::from_timestamp_opt(
        date_released.unwrap_or(chrono::Utc::now().timestamp()),
        0,
    )
    .ok_or((Status::BadRequest, "Invalid date_released".to_string()))?;

    // Create the release.
    let release = models::CoreRelease::create(
        &mut db,
        version,
        notes,
        timestamp,
        prerelease,
        json!(links),
        json!(metadata),
        &user,
        &core,
        &platform,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(Json(dto::cores::releases::CoreReleaseCreateResponse {
        id: release.id,
    }))
}

pub struct ArtifactDownload {
    filename: String,
    mime_type: String,
    data: Vec<u8>,
}

impl<'r> Responder<'r, 'static> for ArtifactDownload {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut response = Response::build()
            .sized_body(self.data.len(), Cursor::new(self.data))
            .ok()?;
        response.set_header(ContentType::parse_flexible(&self.mime_type).unwrap());
        response.set_header(Header::new(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", self.filename),
        ));

        Ok(response)
    }
}

/// Download an artifact.
#[openapi(tag = "Core Releases", ignore = "db", skip)]
#[get(
    "/cores/<core_id>/releases/<release_id>/artifacts/<artifact_id>/download",
    rank = 1
)]
pub async fn cores_releases_artifacts_download(
    mut db: Db,
    core_id: dto::types::IdOrSlug<'_>,
    release_id: u32,
    artifact_id: u32,
) -> Result<ArtifactDownload, (Status, String)> {
    let (artifact, data) = models::Artifact::get_file(&mut db, core_id, release_id, artifact_id)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    let models::File { data, .. } = data.ok_or((Status::NotFound, "File not found".to_string()))?;

    Ok(ArtifactDownload {
        filename: artifact.filename,
        mime_type: artifact.mime_type,
        data,
    })
}

/// Download an artifact by its filename.
#[openapi(tag = "Core Releases", ignore = "db", skip)]
#[get(
    "/cores/<core_id>/releases/<release_id>/artifacts/download/<filename..>",
    rank = 2
)]
pub async fn cores_releases_artifacts_download_filename(
    mut db: Db,
    core_id: dto::types::IdOrSlug<'_>,
    release_id: u32,
    filename: PathBuf,
) -> Result<ArtifactDownload, (Status, String)> {
    let (artifact, data) = models::Artifact::get_fileby_filename(
        &mut db,
        core_id,
        release_id,
        filename.to_string_lossy().as_ref(),
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    let models::File { data, .. } = data.ok_or((Status::NotFound, "File not found".to_string()))?;

    Ok(ArtifactDownload {
        filename: artifact.filename,
        mime_type: artifact.mime_type,
        data,
    })
}

/// Get a release's artifact list, including everything except the data itself.
#[openapi(tag = "Core Releases", ignore = "db")]
#[get("/cores/<core_id>/releases/<release_id>/artifacts?<paging>")]
pub async fn cores_releases_artifacts_list(
    mut db: Db,
    core_id: dto::types::IdOrSlug<'_>,
    release_id: u32,
    paging: dto::params::PagingParams,
) -> Result<Json<Vec<dto::artifact::CoreReleaseArtifactListItem>>, (Status, String)> {
    let (page, limit) = paging.validate().map_err(|e| (Status::BadRequest, e))?;

    let release = models::CoreRelease::from_id(&mut db, release_id as i32)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Release not found".to_string()))?;

    let core = models::Core::from_id_or_slug(&mut db, core_id).await?;

    let artifacts = models::Artifact::list(&mut db, &release, page, limit)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;
    rocket::info!("artifacts: {:?}", artifacts);

    Ok(Json(
        artifacts
            .into_iter()
            .map(|artifact| {
                let download_url = artifact.download_url.clone().unwrap_or_else(|| {
                    rocket::uri!(
                        "/api/v1/",
                        cores_releases_artifacts_download(
                            core.slug.as_str(),
                            release.id as u32,
                            artifact.id as u32
                        )
                    )
                    .to_string()
                });

                let r#ref = artifact.clone().into();

                dto::artifact::CoreReleaseArtifactListItem {
                    id: artifact.id,
                    filename: artifact.filename,
                    download_url,
                    mime_type: artifact.mime_type,
                    created_at: artifact.created_at.timestamp(),
                    r#ref,
                }
            })
            .collect(),
    ))
}

async fn upload_single_artifact(
    db: &mut Db,
    core: &models::Core,
    release: &models::CoreRelease,
    storage: &guards::storage::Storage,
    file_name: &str,
    mime_type: &str,
    file_data: &[u8],
) -> Result<dto::artifact::ArtifactCreateResponse, (Status, String)> {
    if file_data.len() > 24.mebibytes().as_u64() as usize {
        return Err((
            Status::BadRequest,
            "File is too large (max 24 MiB)".to_string(),
        ));
    }

    let md5 = md5::compute(file_data).to_vec();
    let sha1 = sha1::Sha1::digest(file_data).to_vec();
    let sha256 = sha2::Sha256::digest(file_data).to_vec();

    // Upload to storage.
    let download_url = storage
        .upload_core(
            &format!("{}/{}/{}", core.slug, release.version, file_name),
            file_data,
            mime_type,
        )
        .await
        .map_err(|e| (Status::InternalServerError, e))?;

    let artifact = models::Artifact::create_with_checksum(
        db,
        file_name,
        mime_type,
        Some(&md5),
        Some(&sha1),
        Some(&sha256),
        Some(&download_url),
        file_data.len() as i32,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    models::CoreReleaseArtifact::create(db, release, &artifact)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    Ok(dto::artifact::ArtifactCreateResponse {
        id: artifact.id,
        url: Some(download_url),
    })
}

/// Upload an artifact to a release. This can be done multiple times.
/// The upload will be refused if the user does not have permission to
/// upload artifacts to the release's core.
#[openapi(tag = "Core Releases", ignore = "db", ignore = "storage")]
#[post("/cores/<core_id>/releases/<release_id>/artifacts", data = "<file>")]
pub async fn cores_releases_artifacts_upload(
    mut db: Db,
    admin: guards::users::AuthenticatedUserGuard,
    storage: guards::storage::Storage,
    core_id: dto::types::IdOrSlug<'_>,
    release_id: u32,
    content_type: &ContentType,
    file: Data<'_>,
) -> Result<Json<Vec<dto::artifact::ArtifactCreateResponse>>, (Status, String)> {
    // Check the uploader's role.
    let core = models::Core::from_id_or_slug(&mut db, core_id).await?;

    let (user, team, role) =
        models::User::get_user_team_and_role(&mut db, admin.into(), core.owner_team_id.into())
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .ok_or((Status::Unauthorized, "Not logged in".to_string()))?;

    if !acls::can_create_core_releases(&user, &team, &role, &core).await {
        return Err((Status::Forbidden, "Not authorized".to_string()));
    }

    let release = models::CoreRelease::from_id(&mut db, release_id as i32)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
        .ok_or((Status::NotFound, "Release not found".to_string()))?;

    let mut result = Vec::new();

    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file")
            .size_limit(24.mebibytes().as_u64())
            .repetition(Repetition::infinite()),
        MultipartFormDataField::file("artifact")
            .size_limit(24.mebibytes().as_u64())
            .repetition(Repetition::infinite()),
    ]);
    options.max_data_bytes = 40.mebibytes().as_u64();
    let multipart_form_data = MultipartFormData::parse(content_type, file, options)
        .await
        .map_err(|e| (Status::BadRequest, e.to_string()))?;

    for files in multipart_form_data.files.values() {
        for file in files {
            let filename = file
                .file_name
                .clone()
                .ok_or((Status::BadRequest, "Filename not specified.".to_string()))?;
            let mimetype = file.content_type.clone().ok_or((
                Status::BadRequest,
                "Content-Type not specified.".to_string(),
            ))?;
            let file_data = std::fs::read(&file.path)
                .map_err(|e| (Status::InternalServerError, e.to_string()))?;

            // Make sure the filename is unique.
            // TODO: figure out if we can make this check in the database itself.
            if !models::CoreReleaseArtifact::is_filename_unique_for_release(
                &mut db, &release, &filename,
            )
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            {
                return Err((
                    Status::Conflict,
                    "Filename already exists for this release".to_string(),
                ));
            }

            if !models::CoreReleaseArtifact::is_filename_conform(&mut db, &release, &filename)
                .await
                .map_err(|e| (Status::InternalServerError, e.to_string()))?
            {
                return Err((Status::BadRequest, "Filename is invalid".to_string()));
            }

            let artifact = upload_single_artifact(
                &mut db,
                &core,
                &release,
                &storage,
                &filename,
                mimetype.as_ref(),
                &file_data,
            )
            .await?;

            result.push(artifact);
        }
    }

    Ok(Json(result))
}

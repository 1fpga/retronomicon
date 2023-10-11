use crate::db::Db;
use crate::types::FetchModel;
use crate::utils::acls;
use crate::{guards, models};
use retronomicon_dto as dto;
use rocket::http::{ContentType, Header, Status};
use rocket::request::Outcome;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{get, post, Data, Request, Response};
use rocket_okapi::openapi;
use serde_json::json;
use std::io::Cursor;
use std::path::PathBuf;

#[openapi(tag = "Core Releases", ignore = "db")]
#[get("/cores/<core_id>/releases?<paging>&<filter>")]
pub async fn cores_releases_list(
    mut db: Db,
    core_id: dto::types::IdOrSlug<'_>,
    paging: dto::params::PagingParams,
    filter: dto::params::CoreReleaseFilterParams<'_>,
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
        return (Status::BadRequest, "Version cannot be 'latest'".to_string());
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

    Ok(Json(
        artifacts
            .into_iter()
            .map(|artifact| {
                let download_url = artifact.download_url.clone().unwrap_or_else(|| {
                    rocket::uri!(
                        "/api/v1/",
                        cores_releases_artifacts_download(
                            &core.slug,
                            release.id as u32,
                            artifact.id as u32
                        )
                    )
                    .to_string()
                });

                dto::artifact::CoreReleaseArtifactListItem {
                    id: artifact.id,
                    filename: artifact.filename,
                    mime_type: artifact.mime_type,
                    created_at: artifact.created_at.timestamp(),
                    md5: hex::encode(artifact.md5),
                    sha256: hex::encode(artifact.sha256),
                    sha512: hex::encode(artifact.sha512),
                    size: artifact.size,
                    download_url: Some(download_url),
                }
            })
            .collect(),
    ))
}

use once_cell::sync::Lazy;
use rocket::data::ToByteUnit;

static FILENAME_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r#".*filename="(.*)""#).unwrap());

#[derive(Debug)]
pub struct ContentHeaders<'v> {
    content_type: ContentType,
    filename: &'v str,
}

#[rocket::async_trait]
impl<'v> rocket::request::FromRequest<'v> for ContentHeaders<'v> {
    type Error = String;

    async fn from_request(request: &'v Request<'_>) -> Outcome<Self, Self::Error> {
        let content_disposition = match request.headers().get_one("Content-Disposition") {
            Some(s) => s,
            None => {
                return Outcome::Failure((
                    Status::BadRequest,
                    "Missing Content-Disposition header".to_string(),
                ))
            }
        };

        let filename = if let Some((_, [filename])) = FILENAME_RE
            .captures(content_disposition)
            .map(|x| x.extract())
        {
            filename
        } else {
            return Outcome::Failure((
                Status::BadRequest,
                "Missing filename in Content-Disposition header".to_string(),
            ));
        };

        let content_type = match request.content_type().ok_or_else(|| {
            ContentType::from_extension(&filename[filename.rfind('.').unwrap_or_default()..])
        }) {
            Ok(content_type) => content_type.clone(),
            Err(Some(ct)) => ct,
            Err(None) => {
                return Outcome::Failure((Status::BadRequest, "Missing Content-Type".to_string()))
            }
        };

        Outcome::Success(ContentHeaders {
            content_type,
            filename,
        })
    }
}

/// Upload an artifact to a release. This can be done multiple times.
/// The upload will be refused if the user does not have permission to
/// upload artifacts to the release's core.
#[openapi(tag = "Core Releases", ignore = "db", ignore = "headers")]
#[post("/cores/<core_id>/releases/<release_id>/artifacts", data = "<file>")]
pub async fn cores_releases_artifacts_upload(
    mut db: Db,
    admin: guards::users::AuthenticatedUserGuard,
    headers: ContentHeaders<'_>,
    core_id: dto::types::IdOrSlug<'_>,
    release_id: u32,
    file: Data<'_>,
) -> Result<Json<dto::Ok>, (Status, String)> {
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

    // Make sure the filename is unique.
    // TODO: figure out if we can make this check in the database itself.
    if !models::CoreReleaseArtifact::is_filename_unique_for_release(
        &mut db,
        &release,
        headers.filename,
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?
    {
        return Err((
            Status::Conflict,
            "Filename already exists for this release".to_string(),
        ));
    }

    if !models::CoreReleaseArtifact::is_filename_conform(&mut db, &release, headers.filename)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?
    {
        return Err((Status::BadRequest, "Filename is invalid".to_string()));
    }

    let artifact = models::Artifact::create_with_data(
        &mut db,
        headers.filename,
        &headers.content_type.to_string(),
        file.open(20.megabytes())
            .into_bytes()
            .await
            .map_err(|e| (Status::InternalServerError, e.to_string()))?
            .as_slice(),
    )
    .await
    .map_err(|e| (Status::InternalServerError, e.to_string()))?;

    models::CoreReleaseArtifact::create(&mut db, &release, &artifact)
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))?;
    Ok(Json(dto::Ok))
}

use rocket::routes;
use rocket_okapi::openapi_get_routes;

mod auth;
mod me;
mod platforms;
mod tags;
mod teams;
mod users;

// Re-export the guard types for authentication.
pub use auth::{GitHubUserInfo, GoogleUserInfo};

pub fn routes() -> Vec<rocket::Route> {
    [
        openapi_get_routes![
            auth::github_login,
            auth::google_login,
            auth::logout,
            me::me,
            me::me_check,
            me::me_token,
            me::me_update,
            platforms::platforms,
            tags::tags,
            tags::tags_create,
            tags::tags_delete,
            teams::invite,
            teams::invite_accept,
            teams::teams,
            teams::teams_details_id,
            teams::teams_details_slug,
            users::users,
            users::users_id,
            users::users_id_update,
        ],
        routes![auth::github_callback, auth::google_callback,],
    ]
    .concat()
}

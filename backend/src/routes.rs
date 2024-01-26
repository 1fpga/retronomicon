pub mod v1;

use rocket::{routes, Route};

pub mod auth;

pub fn routes() -> Vec<Route> {
    routes![
        auth::github_callback,
        auth::google_callback,
        auth::patreon_callback,
        auth::login_token_callback
    ]
}

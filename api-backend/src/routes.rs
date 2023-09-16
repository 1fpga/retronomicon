use rocket::Route;

mod groups;
mod me;
mod platforms;
mod users;

mod auth;
pub use auth::{GitHubUserInfo, GoogleUserInfo};

pub fn routes() -> Vec<Route> {
    [
        auth::routes(),
        groups::routes(),
        me::routes(),
        platforms::routes(),
        users::routes(),
    ]
    .concat()
}

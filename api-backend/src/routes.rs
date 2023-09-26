use rocket::Route;

mod me;
mod platforms;
mod teams;
mod users;

mod auth;
pub use auth::{GitHubUserInfo, GoogleUserInfo};

pub fn routes() -> Vec<Route> {
    [
        auth::routes(),
        teams::routes(),
        me::routes(),
        platforms::routes(),
        users::routes(),
    ]
    .concat()
}

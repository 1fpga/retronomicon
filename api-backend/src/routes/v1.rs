use rocket_okapi::openapi_get_routes;

mod auth;
mod cores;
mod me;
mod platforms;
mod systems;
mod tags;
mod teams;
mod users;

pub fn routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        auth::github_login,
        auth::google_login,
        auth::logout,
        me::me,
        me::me_token,
        me::me_update,
        platforms::platforms_create,
        platforms::platforms_list,
        platforms::platforms_update,
        systems::systems_create,
        systems::systems_details,
        systems::systems_list,
        tags::tags,
        tags::tags_create,
        tags::tags_delete,
        teams::invite,
        teams::invite_accept,
        teams::teams,
        teams::teams_create,
        teams::teams_details,
        teams::teams_update,
        users::check_username,
        users::users,
        users::users_id,
        users::users_id_update,
    ]
}

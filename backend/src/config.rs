use rocket::figment::providers::Format;
use rocket::figment::{providers, Profile};
use std::path::PathBuf;

pub fn create_figment(
    additional_config: &[PathBuf],
    default_profile: &str,
) -> Result<rocket::figment::Figment, anyhow::Error> {
    let figment = rocket::Config::figment()
        .merge(providers::Toml::file("backend/Rocket.toml").nested())
        .merge(providers::Toml::file("Rocket.toml").nested());

    // Add local configuration files in debug.
    #[cfg(debug_assertions)]
    let figment = figment.admerge(providers::Toml::file("Rocket.debug.toml"));

    let mut f = figment;
    for path in additional_config {
        f = f.admerge(providers::Toml::file(path));
    }

    Ok(
        f.admerge(providers::Env::prefixed("ROCKET_").split("__").global())
            .select(Profile::from_env_or("APP_PROFILE", default_profile)),
    )
}

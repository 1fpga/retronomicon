use crate::World;
use cucumber::{given, then, when};
use retronomicon_dto as dto;
use std::str::FromStr;

#[derive(Debug, cucumber::Parameter)]
#[param(name = "team_role", regex = "(owner|admin|member)")]
struct TeamRole(pub dto::types::UserTeamRole);

impl FromStr for TeamRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse().map_err(
            |e: dto::reexports::strum::ParseError| e.to_string(),
        )?))
    }
}

#[derive(Debug, cucumber::Parameter)]
#[param(name = "user", regex = r#"(admin (\w+)|user (\w+)|anonymous user)"#)]
pub enum UserParam {
    Admin(String),
    User(String),
    Anonymous,
}

impl FromStr for UserParam {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "anonymous user" {
            Ok(Self::Anonymous)
        } else if let Some(name) = s.strip_prefix("admin ") {
            Ok(Self::Admin(name.to_string()))
        } else if let Some(name) = s.strip_prefix("user ") {
            Ok(Self::User(name.to_string()))
        } else {
            Err(format!("Invalid user: {}", s))
        }
    }
}

#[given(expr = "{user} is not authenticated")]
async fn given_a_user_no_auth(w: &mut World, user: UserParam) {
    let _ = w.user(&user).await.expect("No user");
}

#[given(expr = "{user}")]
async fn given_a_user_auth(w: &mut World, user: UserParam) {
    let user = w.user(&user).await.unwrap();
    user.lock().await.authenticate().await.unwrap();
}

#[given(expr = "team {word} is owned by {user}")]
async fn given_team_owned(w: &mut World, team: String, user: UserParam) {
    w.team(&user, &team).await.unwrap();
}

#[when(expr = "{user} gets their details")]
async fn user_gets_their_details(w: &mut World, user: UserParam) {
    let user = w.user(&user).await.unwrap();
    let result = user.lock().await.get_user_details(None).await;
    w.record_result(result);
}

#[when(expr = "{user} invites {user} to team {word} as {team_role}")]
async fn user_can_invite_to_team(
    w: &mut World,
    inviter: UserParam,
    invitee: UserParam,
    team: String,
    role: TeamRole,
) {
    w.assert_result_ok();
    let role = role.0;

    // Just make sure user and team exists.
    let invitee = w.auth_user(&invitee).await.unwrap().lock().await.id();
    let team_id = w.team(&inviter, &team).await.unwrap().id;
    let inviter = w.user(&inviter).await.unwrap();

    let result = inviter
        .lock()
        .await
        .invite_to_team(team_id, invitee, role)
        .await;
    w.record_result(result);
}

#[then(expr = "an error occured")]
async fn an_error_occured(w: &mut World) {
    w.assert_result_err();
    w.reset_result();
}

#[then(expr = "no error occured")]
async fn no_error_occured(w: &mut World) {
    w.assert_result_ok();
    w.reset_result();
}

#[then(expr = "team {word} will have {user} as {team_role}")]
async fn team_has_user_role(w: &mut World, team: String, user: UserParam, role: TeamRole) {
    w.assert_result_ok();

    let role = role.0;

    let team = w.team(&user, &team).await.unwrap().clone();
    let user = w.user(&user).await.unwrap().clone();
    let user_id = user.lock().await.id();
    let team_details = user.lock().await.team_details(team.id).await.unwrap();
    let user_role = team_details
        .users
        .iter()
        .find(|u| u.user.id == user_id)
        .unwrap()
        .role;
    assert_eq!(user_role, role);
}

#[when(expr = "{user} accepts the invitation to team {word}")]
async fn user_accepts_invitation(w: &mut World, user: UserParam, team: String) {
    w.assert_result_ok();

    let team = w.team(&user, &team).await.unwrap();

    let user = w.user(&user).await.unwrap();
    let result = user.lock().await.accept_team_invitation(team.id).await;
    w.record_result(result);
}

#[then(expr = "{user} will not be able to invite {user} to team {word} as {team_role}")]
async fn team_cannot_invite(
    w: &mut World,
    inviter: UserParam,
    invitee: UserParam,
    team: String,
    role: TeamRole,
) {
    w.assert_result_ok();
    let role = role.0;

    // Just make sure user and team exists.
    let invitee = w.auth_user(&invitee).await.unwrap().lock().await.id();
    let team_id = w.team(&inviter, &team).await.unwrap().id;
    let inviter = w.auth_user(&inviter).await.unwrap();
    let result = inviter
        .lock()
        .await
        .invite_to_team(team_id, invitee, role)
        .await;
    assert!(result.is_err());
}

#[given(expr = "a system {word} created by {user} owned by team {word}")]
async fn system_owned(w: &mut World, system: String, user: UserParam, team: String) {
    let team = w.team(&user, &team).await.unwrap();
    let user = w.auth_user(&user).await.unwrap();

    let s = user
        .lock()
        .await
        .create_system(team.id, &system)
        .await
        .unwrap();

    w.systems.insert(system.clone(), s.id);
}

#[when(expr = "{user} creates a game {word} on system {word}")]
async fn game_create(w: &mut World, user: UserParam, game: String, system: String) {
    w.assert_result_ok();

    let user = w.auth_user(&user).await.unwrap();
    let system_id = *w.systems.get(&system).unwrap();

    let result = user.lock().await.create_game(system_id, &game).await;
    if let Ok(g) = &result {
        w.games.insert(game.clone(), g.id);
    }
    w.record_result(result);
}

#[then(expr = "game {word} exists on system {word}")]
async fn game_exists(w: &mut World, game: String, system: String) {
    w.assert_result_ok();

    let user = w.user(&UserParam::Anonymous).await.unwrap();
    let game_id = *w.games.get(&game).unwrap();
    let system_id = *w.systems.get(&system).unwrap();
    let result = user.lock().await.get_game_by_id(game_id).await.unwrap();

    assert_eq!(result.system.id, system_id);
}

/// Create a game on a default system.
#[given(expr = "game {word}")]
async fn given_a_game(w: &mut World, game: String) {
    given_a_user_auth(w, UserParam::Admin("default".to_string())).await;
    given_team_owned(
        w,
        "default".to_string(),
        UserParam::Admin("default".to_string()),
    )
    .await;

    system_owned(
        w,
        "default".to_string(),
        UserParam::Admin("default".to_string()),
        "default".to_string(),
    )
    .await;
    game_create(
        w,
        UserParam::Admin("default".to_string()),
        game,
        "default".to_string(),
    )
    .await;
    w.assert_result_ok();
}

#[when(expr = "{user} uploads image {word} to game {word}")]
async fn game_upload_image(w: &mut World, user: UserParam, image: String, game: String) {
    let user = w.auth_user(&user).await.unwrap();
    let game_id = *w.games.get(&game).unwrap();

    let result = user.lock().await.upload_image(game_id, &image).await;
    w.record_result(result);
}

#[then(expr = "{user} can see image {word} of game {word}")]
async fn then_user_can_see(w: &mut World, user: UserParam, image: String, game: String) {
    w.assert_result_ok();

    let user = w.user(&user).await.unwrap();
    let game_id = *w.games.get(&game).unwrap();
    let result = user.lock().await.get_game_images(game_id).await.unwrap();

    let i = result
        .iter()
        .find(|i| i.name == format!("{image}.png"))
        .expect("Image not in the list of images.");

    // Download the image.
    reqwest::get(&i.url)
        .await
        .expect("Could not get a response")
        .error_for_status()
        .expect("Could not download the image");
}

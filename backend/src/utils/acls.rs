use retronomicon_db::models;

pub fn can_create_team(_user: &models::User) -> bool {
    true
}

pub fn can_update_team(
    _user: &models::User,
    _team: &models::Team,
    role: &models::UserTeamRole,
) -> bool {
    role == &models::UserTeamRole::Owner
}

pub fn can_delete_team(
    _user: &models::User,
    _team: &models::Team,
    role: &models::UserTeamRole,
) -> bool {
    role == &models::UserTeamRole::Owner
}

pub fn can_invite_to_team(
    team: &models::Team,
    _admin_user: &models::User,
    admin_role: &models::UserTeamRole,
    _invited_user: &models::User,
    invited_role: &models::UserTeamRole,
) -> bool {
    if team.is_root() {
        admin_role == &models::UserTeamRole::Owner
    } else if admin_role == &models::UserTeamRole::Owner {
        true
    } else {
        admin_role > invited_role
    }
}

pub(crate) async fn can_create_core_releases(
    _user: &models::User,
    _team: &models::Team,
    role: &models::UserTeamRole,
    _core: &models::Core,
) -> bool {
    // All members can do releases.
    role >= &models::UserTeamRole::Member
}

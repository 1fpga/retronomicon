# List of ACLs

## Roles

Owners can invite users to a team as any roles, can update team information, and can delete the team.
They can update cores and releases.

Admin can invite users as members.
They can update cores and releases.

Members can create releases and update information.

## Team Specific ACLs

- `can_create_team`. Allows the user to create a team. Anyone can create a team.
- `can_update_team`. Allows the user to edit a team (description, links, etc). Only the team owner can edit a team.
- `can_delete_team`. Allows the user to delete a team. Only the team owner can delete a team.
- `can_invite_to_team`. Allows the user to invite other users to a team. Owners and admins can invite other users to a team, but only owners can invite with the admin (or owner) role.


## Releases

All members can do a release.

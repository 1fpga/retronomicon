-- These are done in this weird order to avoid dependencies error.

DROP TABLE user_teams;
DROP TABLE system_tags;
DROP TABLE system_release_artifacts;
DROP TABLE system_releases;
DROP TABLE platform_tags;
DROP TABLE core_tags;
DROP TABLE core_release_artifacts;
DROP INDEX core_releases_core_id_platform_id_system_id_version_idx;
DROP TABLE core_releases;
DROP TABLE cores;
DROP TABLE artifacts;

DROP TABLE tags;
DROP TABLE "users";
DROP TABLE "systems";
DROP TABLE platforms;
DROP TABLE "teams";

DROP DOMAIN slug;
DROP DOMAIN username;
DROP TYPE user_team_role;

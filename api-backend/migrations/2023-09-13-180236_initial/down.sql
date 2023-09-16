-- These are done in this weird order to avoid dependencies error.

DROP TABLE user_groups;
DROP INDEX users_username_idx;
DROP INDEX tags_slug_idx;
DROP TABLE system_tags;
DROP TABLE system_release_artifacts;
DROP INDEX system_releases_version_idx;
DROP TABLE system_releases;
DROP INDEX systems_name_idx;
DROP INDEX systems_slug_idx;
DROP TABLE platform_tags;
DROP INDEX platforms_name_idx;
DROP INDEX platforms_slug_idx;
DROP TABLE core_tags;
DROP TABLE core_release_artifacts;
DROP INDEX core_releases_core_id_platform_id_system_id_version_idx;
DROP TABLE core_releases;
DROP INDEX cores_name_idx;
DROP INDEX cores_slug_idx;
DROP TABLE cores;
DROP TABLE artifacts;

DROP TABLE tags;
DROP TABLE "users";
DROP TABLE "systems";
DROP TABLE platforms;
DROP TABLE "groups";

DROP DOMAIN slug;
DROP DOMAIN username;
DROP DOMAIN displayname;
DROP TYPE user_group_role;

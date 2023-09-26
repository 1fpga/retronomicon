SET check_function_bodies = false
;

CREATE TYPE user_team_role AS ENUM('owner', 'admin', 'member');

CREATE DOMAIN slug varchar(255)
    CHECK ( value ~ '^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$' )
    CONSTRAINT slug_constraint NOT NULL;

CREATE DOMAIN username varchar(255)
    CHECK ( value ~ '^[a-z_]([a-z0-9_.-]*[a-z0-9_])?$' )
    CHECK ( value != 'me' )
    CHECK ( value != 'root' )
    CHECK ( value != 'admin' )
    CHECK ( value != 'owner' )
;

COMMENT ON DOMAIN slug IS 'A URL compatible path component that is unique.';

CREATE TABLE artifacts(
                          id SERIAL PRIMARY KEY NOT NULL,
                          filename varchar NOT NULL,
                          sha256 bytea,
                          sha512 bytea,
                          size integer NOT NULL,
                          download_url varchar
);

COMMENT ON TABLE artifacts IS
    'Artifact (a file). There might be multiple per release/core/systems (e.g. a source tree, or a file that needs to accompany the release). Download URL can be NULL for artifacts we only know through their checksums.'
;

CREATE TABLE core_release_artifacts(
                                       core_release_id integer NOT NULL, artifact_id integer NOT NULL,
                                       CONSTRAINT core_release_artifacts_pkey PRIMARY KEY(core_release_id, artifact_id)
);

CREATE TABLE core_releases(
                              id SERIAL PRIMARY KEY NOT NULL,
                              "version" varchar NOT NULL,
                              note text,
                              date_released timestamp,
                              date_uploaded timestamp NOT NULL,
                              prerelease bool,
                              yanked bool,
                              links jsonb,
                              uploader_id integer NOT NULL,
                              core_id integer NOT NULL,
                              platform_id integer NOT NULL,
                              system_id integer NOT NULL,
                              owner_id integer NOT NULL
);

CREATE UNIQUE INDEX core_releases_core_id_platform_id_system_id_version_idx ON
    core_releases(
                  core_id,
                  platform_id,
                  system_id,
                  "version" DESC
        );

COMMENT ON TABLE core_releases IS 'Downloadable release of a core.';

CREATE TABLE core_tags(
                          core_id integer NOT NULL, tag_id integer NOT NULL,
                          CONSTRAINT core_tags_pkey PRIMARY KEY(tag_id, core_id)
);

CREATE TABLE cores(
                      id SERIAL PRIMARY KEY NOT NULL,
                      slug slug NOT NULL,
                      "name" varchar NOT NULL,
                      description text NOT NULL,
                      metadata jsonb,
                      links jsonb,
                      owner_id integer NOT NULL
);

CREATE UNIQUE INDEX cores_slug_idx ON cores(slug);

CREATE UNIQUE INDEX cores_name_idx ON cores("name");

COMMENT ON TABLE cores IS
    'Core being able to run a SYSTEM either in software or hardware.';

CREATE TABLE "teams"(
                         id SERIAL PRIMARY KEY NOT NULL,
                         slug slug NOT NULL,
                         "name" varchar NOT NULL,
                         description text NOT NULL,
                         links jsonb
);

COMMENT ON TABLE "teams" IS 'Team/group of users that own and manage artifacts.';

CREATE TABLE platform_tags(
                              platform_id integer NOT NULL, tag_id integer NOT NULL,
                              CONSTRAINT platform_tags_pkey PRIMARY KEY(tag_id, platform_id)
);

CREATE TABLE platforms(
                          id SERIAL PRIMARY KEY NOT NULL,
                          slug slug NOT NULL,
                          "name" varchar(255) NOT NULL,
                          description text NOT NULL,
                          links jsonb,
                          metadata jsonb,
                          owner_id integer NOT NULL
);

CREATE UNIQUE INDEX platforms_slug_idx ON platforms(slug);

CREATE UNIQUE INDEX platforms_name_idx ON platforms("name");

COMMENT ON TABLE platforms IS
    'The platform that supports running Cores, e.g. "openFPGA" or "MiSTer-de10".';

CREATE TABLE system_release_artifacts(
                                         system_file_release_id integer NOT NULL, artifact_id integer NOT NULL,
                                         CONSTRAINT system_release_artifacts_pkey PRIMARY KEY
                                             (artifact_id, system_file_release_id)
);

CREATE TABLE system_releases(
                                id SERIAL PRIMARY KEY NOT NULL,
                                "version" varchar NOT NULL,
                                note text,
                                date_released timestamp,
                                date_uploaded timestamp NOT NULL,
                                prerelease integer,
                                yanked bool,
                                links jsonb,
                                user_id integer NOT NULL,
                                system_id integer NOT NULL
);

CREATE UNIQUE INDEX system_releases_version_idx ON system_releases
    ("version" DESC);

COMMENT ON TABLE system_releases IS
    'Downloadable release of system''s artifacts (e.g. a BIOS). These are platform and core independent.'
;

CREATE TABLE system_tags(
                            system_id integer NOT NULL, tag_id integer NOT NULL,
                            CONSTRAINT system_tags_pkey PRIMARY KEY(tag_id, system_id)
);

CREATE TABLE systems(
                        id SERIAL PRIMARY KEY NOT NULL,
                        slug slug NOT NULL,
                        "name" varchar NOT NULL,
                        description text NOT NULL,
                        manufacturer varchar NOT NULL,
                        links jsonb,
                        metadata jsonb,
                        owner_id integer NOT NULL
);

CREATE UNIQUE INDEX systems_slug_idx ON systems(slug);

CREATE UNIQUE INDEX systems_name_idx ON systems("name");

COMMENT ON TABLE systems IS
    'A hardware target system, e.g. "NES" or "Arcade-TMNT".';

CREATE TABLE tags(
                     id SERIAL PRIMARY KEY NOT NULL,
                     slug slug NOT NULL,
                     description text,
                     color integer NOT NULL
);

CREATE UNIQUE INDEX tags_slug_idx ON tags(slug);

CREATE TABLE users(
                      id SERIAL PRIMARY KEY NOT NULL,
                      username username,
                      display_name varchar(255),
                      avatar_url varchar(255),

                      email varchar(255) NOT NULL,
                      auth_provider varchar(255),

                      need_reset bool NOT NULL,
                      deleted bool NOT NULL,

                      description text NOT NULL,
                      links jsonb,
                      metadata jsonb
);

CREATE UNIQUE INDEX users_username_idx ON users(username);

COMMENT ON TABLE users IS
    'A list of users for the website. If the user does not have a password, it cannot be logged in using the regular username+password scheme (it needs to use OAuth2).'
;

CREATE TABLE user_teams(
                             team_id integer NOT NULL REFERENCES "teams",
                             user_id integer NOT NULL REFERENCES "users",
                             "role" user_team_role NOT NULL,
                             invite_from integer REFERENCES "users",
                             CONSTRAINT user_teams_pkey PRIMARY KEY(team_id, user_id)
);

ALTER TABLE core_releases
    ADD CONSTRAINT core_releases_uploader_id_fkey
        FOREIGN KEY (uploader_id) REFERENCES users (id);

ALTER TABLE core_releases
    ADD CONSTRAINT core_releases_core_id_fkey
        FOREIGN KEY (core_id) REFERENCES cores (id);

ALTER TABLE core_releases
    ADD CONSTRAINT core_releases_platform_id_fkey
        FOREIGN KEY (platform_id) REFERENCES platforms (id);

ALTER TABLE core_releases
    ADD CONSTRAINT core_releases_system_id_fkey
        FOREIGN KEY (system_id) REFERENCES systems (id);

ALTER TABLE cores
    ADD CONSTRAINT cores_owner_id_fkey
        FOREIGN KEY (owner_id) REFERENCES "teams" (id);

ALTER TABLE core_tags
    ADD CONSTRAINT core_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES tags (id)
;

ALTER TABLE core_tags
    ADD CONSTRAINT core_tags_core_id_fkey
        FOREIGN KEY (core_id) REFERENCES cores (id);

ALTER TABLE platform_tags
    ADD CONSTRAINT platform_tags_tag_id_fkey
        FOREIGN KEY (tag_id) REFERENCES tags (id);

ALTER TABLE platform_tags
    ADD CONSTRAINT platform_tags_platform_id_fkey
        FOREIGN KEY (platform_id) REFERENCES platforms (id);

ALTER TABLE system_tags
    ADD CONSTRAINT system_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES tags (id)
;

ALTER TABLE system_tags
    ADD CONSTRAINT system_tags_system_id_fkey
        FOREIGN KEY (system_id) REFERENCES systems (id);

ALTER TABLE platforms
    ADD CONSTRAINT platforms_owner_id_fkey
        FOREIGN KEY (owner_id) REFERENCES "teams" (id);

ALTER TABLE systems
    ADD CONSTRAINT systems_owner_id_fkey
        FOREIGN KEY (owner_id) REFERENCES "teams" (id);

ALTER TABLE core_release_artifacts
    ADD CONSTRAINT core_release_artifacts_core_release_id_fkey
        FOREIGN KEY (core_release_id) REFERENCES core_releases (id);

ALTER TABLE core_release_artifacts
    ADD CONSTRAINT core_release_artifacts_artifact_id_fkey
        FOREIGN KEY (artifact_id) REFERENCES artifacts (id);

ALTER TABLE system_releases
    ADD CONSTRAINT system_releases_system_id_fkey
        FOREIGN KEY (system_id) REFERENCES systems (id);

ALTER TABLE system_releases
    ADD CONSTRAINT system_releases_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users (id);

ALTER TABLE system_release_artifacts
    ADD CONSTRAINT system_release_artifacts_artifact_id_fkey
        FOREIGN KEY (artifact_id) REFERENCES artifacts (id);

ALTER TABLE system_release_artifacts
    ADD CONSTRAINT system_release_artifacts_system_file_release_id_fkey
        FOREIGN KEY (system_file_release_id) REFERENCES system_releases (id);

ALTER TABLE core_releases
    ADD CONSTRAINT core_releases_owner_id_fkey
        FOREIGN KEY (owner_id) REFERENCES "teams" (id);

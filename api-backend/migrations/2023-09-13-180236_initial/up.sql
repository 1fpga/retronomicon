SET
check_function_bodies = false
;

CREATE TYPE user_team_role AS ENUM('owner', 'admin', 'member');

CREATE DOMAIN slug varchar(255)
    CHECK ( value ~ '^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$' )
    CHECK ( value != 'me' AND value != 'owner' AND value != 'all' AND value != 'update' AND value != 'updates' AND value != 'release' AND value != 'releases' )
    CONSTRAINT slug_constraint NOT NULL;

CREATE DOMAIN username varchar(255)
    CHECK ( value ~ '^[a-z_]([a-z0-9_.-]*[a-z0-9_])?$' )
    CHECK ( value ~ '^.{2,32}$' )
    CHECK ( value != 'me' )
    CHECK ( value != 'root' )
    CHECK ( value != 'admin' )
    CHECK ( value != 'owner' )
;

COMMENT
ON DOMAIN slug IS 'A URL compatible path component that is unique.';

CREATE TABLE tags
(
    id          SERIAL PRIMARY KEY NOT NULL,
    slug        slug               NOT NULL UNIQUE,
    description text,
    color       INT8               NOT NULL
);

CREATE TABLE users
(
    id            SERIAL PRIMARY KEY NOT NULL,
    username      username UNIQUE,
    display_name  varchar(255),
    avatar_url    varchar(255),

    email         varchar(255)       NOT NULL,
    auth_provider varchar(255),

    need_reset    bool               NOT NULL,
    deleted       bool               NOT NULL,

    description   text               NOT NULL,
    links         jsonb              NOT NULL,
    metadata      jsonb              NOT NULL
);

COMMENT
ON TABLE users IS
    'A list of users for the website. If the user does not have a password,\n'
    'it cannot be logged in using the regular username+password scheme (it\n'
    'needs to use OAuth2).'
;

CREATE TABLE "teams"
(
    id          SERIAL PRIMARY KEY NOT NULL,
    slug        slug               NOT NULL,
    "name"      varchar            NOT NULL,
    description text               NOT NULL,
    links       jsonb              NOT NULL,
    metadata    jsonb              NOT NULL
);

COMMENT
ON TABLE "teams" IS 'Team/group of users that own and manage artifacts.';

CREATE TABLE platforms
(
    id            SERIAL PRIMARY KEY NOT NULL,
    slug          slug               NOT NULL UNIQUE,
    "name"        varchar(255)       NOT NULL UNIQUE,
    description   text               NOT NULL,
    links         jsonb              NOT NULL,
    metadata      jsonb              NOT NULL,
    owner_team_id integer            NOT NULL REFERENCES "teams"
);

COMMENT
ON TABLE platforms IS
    'The platform that supports running Cores, e.g. "openFPGA" or \n'
    '"MiSTer-de10".';

CREATE TABLE systems
(
    id            SERIAL PRIMARY KEY NOT NULL,
    slug          slug               NOT NULL UNIQUE,
    "name"        varchar            NOT NULL UNIQUE,
    description   text               NOT NULL,
    manufacturer  varchar            NOT NULL,
    links         jsonb              NOT NULL,
    metadata      jsonb              NOT NULL,
    owner_team_id integer            NOT NULL REFERENCES "teams"
);

COMMENT
ON TABLE systems IS
    'A hardware target system, e.g. "NES" or "Arcade-TMNT".';

CREATE TABLE cores
(
    id            SERIAL PRIMARY KEY NOT NULL,
    slug          slug               NOT NULL UNIQUE,
    "name"        varchar            NOT NULL UNIQUE,
    description   text               NOT NULL,
    metadata      jsonb              NOT NULL,
    links         jsonb              NOT NULL,
    system_id     integer            NOT NULL REFERENCES systems,
    owner_team_id integer            NOT NULL REFERENCES "teams"
);

COMMENT
ON TABLE cores IS
    'Core being able to run a SYSTEM either in software or hardware.';

CREATE TABLE artifacts
(
    id           SERIAL PRIMARY KEY NOT NULL,
    filename     varchar            NOT NULL,
    created_at   timestamp          NOT NULL,
    sha256       bytea,
    sha512       bytea,
    size         integer            NOT NULL,
    download_url varchar
);

COMMENT
ON TABLE artifacts IS
    'Artifact (a file). There might be multiple per release/core/systems\n'
    '(e.g. a source tree, or a file that needs to accompany the release).\n'
    'Download URL can be NULL for artifacts we only know through their\n'
    'checksums.'
;

CREATE TABLE core_releases
(
    id            SERIAL PRIMARY KEY NOT NULL,
    "version"     varchar            NOT NULL,
    notes         text,
    date_released timestamp          NOT NULL,
    prerelease    bool,
    yanked        bool,
    links         jsonb              NOT NULL,
    uploader_id   integer            NOT NULL REFERENCES users,
    core_id       integer            NOT NULL REFERENCES cores,
    platform_id   integer            NOT NULL REFERENCES platforms,
    owner_team_id integer            NOT NULL REFERENCES "teams"
);

CREATE UNIQUE INDEX core_releases_core_id_platform_id_system_id_version_idx ON
    core_releases (
                   core_id,
                   platform_id,
                   "version" DESC
        );

COMMENT
ON TABLE core_releases IS 'Downloadable release of a core.';

CREATE TABLE core_release_artifacts
(
    core_release_id integer NOT NULL REFERENCES core_releases,
    artifact_id     integer NOT NULL REFERENCES artifacts,
    CONSTRAINT core_release_artifacts_pkey PRIMARY KEY (core_release_id, artifact_id)
);

CREATE TABLE system_releases
(
    id            SERIAL PRIMARY KEY NOT NULL,
    "version"     varchar            NOT NULL UNIQUE,
    note          text,
    date_released timestamp,
    date_uploaded timestamp          NOT NULL,
    prerelease    integer,
    yanked        bool,
    links         jsonb              NOT NULL,
    user_id       integer            NOT NULL REFERENCES users,
    system_id     integer            NOT NULL REFERENCES systems
);

COMMENT
ON TABLE system_releases IS
    'Downloadable release of system''s artifacts (e.g. a BIOS). These are\n'
    'platform and core independent.'
;

CREATE TABLE core_tags
(
    core_id integer NOT NULL REFERENCES cores,
    tag_id  integer NOT NULL REFERENCES tags,
    CONSTRAINT core_tags_pkey PRIMARY KEY (tag_id, core_id)
);

CREATE TABLE platform_tags
(
    platform_id integer NOT NULL REFERENCES platforms,
    tag_id      integer NOT NULL REFERENCES tags,
    CONSTRAINT platform_tags_pkey PRIMARY KEY (tag_id, platform_id)
);

CREATE TABLE system_release_artifacts
(
    system_release_id integer NOT NULL REFERENCES system_releases,
    artifact_id       integer NOT NULL REFERENCES artifacts,
    CONSTRAINT system_release_artifacts_pkey PRIMARY KEY
        (artifact_id, system_release_id)
);

CREATE TABLE system_tags
(
    system_id integer NOT NULL,
    tag_id    integer NOT NULL,
    CONSTRAINT system_tags_pkey PRIMARY KEY (tag_id, system_id)
);

CREATE TABLE user_teams
(
    team_id     integer        NOT NULL REFERENCES "teams",
    user_id     integer        NOT NULL REFERENCES "users",
    "role"      user_team_role NOT NULL,
    invite_from integer REFERENCES "users",
    CONSTRAINT user_teams_pkey PRIMARY KEY (team_id, user_id)
);

-- Create the root group.
-- Add root user
INSERT INTO "teams"
VALUES (1,
        'root', 'root',
        'The root team which has administrative right.',
        '{
          "github": "https://github.com/golem-fpga/retronomicon"
        }'::jsonb,
        '{}');

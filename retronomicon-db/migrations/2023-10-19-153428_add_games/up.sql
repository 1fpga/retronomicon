CREATE TABLE games
(
    id                SERIAL PRIMARY KEY NOT NULL,
    name              VARCHAR(255)       NOT NULL,
    description       VARCHAR            NOT NULL,
    short_description VARCHAR(255)       NOT NULL,
    year              INTEGER            NOT NULL,
    publisher         VARCHAR            NOT NULL,
    developer         VARCHAR            NOT NULL,
    links             jsonb              NOT NULL,
    system_id         INTEGER            NOT NULL REFERENCES systems,
    system_unique_id  INTEGER            NOT NULL,

    CONSTRAINT system_id_ UNIQUE (system_id, system_unique_id)
);

COMMENT ON TABLE games IS 'A table of games/roms that can be run (or implemented by) cores.';

CREATE TABLE game_artifacts
(
    game_id     integer NOT NULL REFERENCES games,
    artifact_id integer NOT NULL REFERENCES artifacts,
    CONSTRAINT game_artifacts_pkey PRIMARY KEY (game_id, artifact_id)
);

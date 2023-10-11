-- Your SQL goes here
CREATE DOMAIN version_number varchar(255)
    CHECK (
            value ~ '^[a-z0-9][a-z0-9]*(?:[\.-][a-z0-9]+)*$'
            AND value != 'latest'
        )
    CONSTRAINT version_number_constraint NOT NULL;


ALTER TABLE core_releases
    ALTER COLUMN version TYPE version_number
        USING version::version_number;

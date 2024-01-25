-- Your SQL goes here

CREATE TABLE user_passwords
(
    user_id          INT REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE NOT NULL,
    password         VARCHAR(255)                                                  NOT NULL,
    updated_at       TIMESTAMP                                                     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    needs_reset      BOOLEAN                                                       NOT NULL DEFAULT FALSE,
    validation_token VARCHAR(255),
    PRIMARY KEY (user_id)
);

COMMENT ON TABLE user_passwords IS 'User passwords, separated from the regular user table to allow for password resets.';
ALTER TABLE users
    DROP COLUMN need_reset;

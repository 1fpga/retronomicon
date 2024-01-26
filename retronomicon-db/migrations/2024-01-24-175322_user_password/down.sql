-- This file should undo anything in `up.sql`

ALTER TABLE users ADD COLUMN need_reset bool NOT NULL DEFAULT false;

DROP TABLE user_passwords;

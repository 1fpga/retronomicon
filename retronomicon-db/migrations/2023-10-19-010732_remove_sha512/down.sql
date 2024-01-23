ALTER TABLE artifacts ADD COLUMN sha512 bytea NOT NULL DEFAULT E''::bytea;

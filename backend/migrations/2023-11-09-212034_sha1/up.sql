ALTER TABLE artifacts ADD COLUMN sha1 bytea NOT NULL DEFAULT E''::bytea;

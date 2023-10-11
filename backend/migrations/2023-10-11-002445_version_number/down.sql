-- This file should undo anything in `up.sql`
ALTER TABLE core_releases ALTER COLUMN version TYPE varchar(255);
DROP DOMAIN IF EXISTS version_number;

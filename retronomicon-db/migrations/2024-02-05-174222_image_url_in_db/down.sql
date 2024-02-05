-- This file should undo anything in `up.sql`

ALTER TABLE game_images DROP COLUMN IF EXISTS "url";

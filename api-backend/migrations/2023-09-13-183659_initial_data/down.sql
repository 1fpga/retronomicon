-- This file should undo anything in `up.sql`
DELETE FROM "user_teams" WHERE "team_id" = (SELECT "id" FROM "teams" WHERE "name" = 'root');
DELETE FROM "teams" WHERE "name" = 'root';

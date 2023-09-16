-- This file should undo anything in `up.sql`
DELETE FROM user_groups WHERE user_id = 1;
DELETE FROM "groups" WHERE "name" = 'root';
DELETE FROM "users" WHERE "username" = 'root';

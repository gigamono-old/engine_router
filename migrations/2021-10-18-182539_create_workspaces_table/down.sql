-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_workspaces_name;
DROP TABLE IF EXISTS workspaces;

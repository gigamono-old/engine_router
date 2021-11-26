-- Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_workspaces_name;
DROP TABLE IF EXISTS workspaces;

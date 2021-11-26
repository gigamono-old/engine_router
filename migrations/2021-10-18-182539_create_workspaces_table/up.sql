-- Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

-- Your SQL goes here
CREATE TABLE workspaces (
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone,
    name VARCHAR(255) NOT NULL UNIQUE
);
CREATE UNIQUE INDEX idx_workspaces_name ON workspaces(name);

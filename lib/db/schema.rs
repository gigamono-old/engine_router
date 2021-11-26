// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

table! {
    workspaces (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        name -> Varchar,
    }
}

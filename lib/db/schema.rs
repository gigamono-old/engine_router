table! {
    workspaces (id) {
        id -> Uuid,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        name -> Varchar,
    }
}

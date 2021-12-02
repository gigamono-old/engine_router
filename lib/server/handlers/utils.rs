// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use hyper::{Body, Request};
use parking_lot::Mutex;
use utilities::{
    database::DB,
    errors::{self, HandlerError, HandlerErrorMessage},
    http::{StatusCode, WORKSPACE_ID_HEADER, WORKSPACE_NAME_HEADER},
    result::HandlerResult,
};
use uuid::Uuid;

pub(crate) fn get_workspace_id(
    db_mutex: &Mutex<DB<PgConnection>>,
    req: &Request<Body>,
) -> HandlerResult<String> {
    // Get the workspace id from the headers.
    if let Some(header_val) = req.headers().get(WORKSPACE_ID_HEADER) {
        // Convert header bytes to string.
        let workspace_id = header_val.to_str().map_err(|err| HandlerError::Client {
            ctx: HandlerErrorMessage::InvalidWorkspaceID,
            code: StatusCode::BadRequest,
            src: errors::wrap_error("converting workspace id bytes in header to string", err)
                .unwrap_err(),
        })?;

        // Convert string to uuid.
        let workspace_uuid = Uuid::parse_str(workspace_id).map_err(|err| HandlerError::Client {
            ctx: HandlerErrorMessage::InvalidWorkspaceID,
            code: StatusCode::BadRequest,
            src: errors::wrap_error("parsing workspace id to uuid", err).unwrap_err(),
        })?;

        use crate::db::models::*;
        use crate::db::schema::workspaces::dsl::*;

        // Making sure id exists in the db.
        let db = db_mutex.lock(); // Lock Resource.
        let results = workspaces
            .find(workspace_uuid)
            .load::<Workspace>(&db.conn)
            .map_err(|err| HandlerError::Client {
                ctx: HandlerErrorMessage::InvalidWorkspaceID,
                code: StatusCode::BadRequest,
                src: errors::wrap_error(
                    "getting workspace id associated with name from the db",
                    err,
                )
                .unwrap_err(),
            })?;

        if !results.is_empty() {
            return Ok(String::from(workspace_id));
        }
    } // Drop mutex guard.

    // Otherwise get the workspace name from the headers.
    if let Some(header_val) = req.headers().get(WORKSPACE_NAME_HEADER) {
        // Convert header bytes to string.
        let workspace_name = header_val.to_str().map_err(|err| HandlerError::Client {
            ctx: HandlerErrorMessage::InvalidWorkspaceID,
            code: StatusCode::BadRequest,
            src: errors::wrap_error("converting workspace name bytes in header to string", err)
                .unwrap_err(),
        })?;

        use crate::db::models::*;
        use crate::db::schema::workspaces::dsl::*;

        // Use workspace name to get id from the db.
        let db = db_mutex.lock(); // Lock Resource.
        let results = workspaces
            .filter(name.eq(workspace_name))
            .limit(1)
            .load::<Workspace>(&db.conn)
            .map_err(|err| HandlerError::Client {
                ctx: HandlerErrorMessage::InvalidWorkspaceID,
                code: StatusCode::BadRequest,
                src: errors::wrap_error(
                    "getting workspace id associated with name from the db",
                    err,
                )
                .unwrap_err(),
            })?;

        if !results.is_empty() {
            return Ok(results[0].id.to_string());
        }
    } // Drop mutex guard.

    // Error.
    Err(HandlerError::Client {
        ctx: HandlerErrorMessage::InvalidWorkspaceID,
        code: StatusCode::BadRequest,
        src: errors::any_error("getting workspace id").unwrap_err(),
    })
}

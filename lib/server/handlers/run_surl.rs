use crate::diesel::prelude::*;
use hyper::{Body, Request, Response};
use log::{debug, info};
use parking_lot::Mutex;
use std::{mem, sync::Arc};
use utilities::{
    database::DB,
    errors::{self, HandlerError, HandlerErrorMessage},
    http::{StatusCode, WORKSPACE_ID_HEADER, WORKSPACE_NAME_HEADER},
    natsio::{self, WorkspacesAction},
    result::HandlerResult,
    setup::APISetup,
};
use uuid::Uuid;

pub(crate) async fn run_surl(
    req: Request<Body>,
    stream_buf: Arc<Mutex<Vec<u8>>>,
    setup: Arc<APISetup>,
) -> HandlerResult<Response<Body>> {
    // Get id from header.
    let workspace_id = get_workspace_id(&setup.db, &req)?;

    debug!(r#"Retrieved workspace id "{}""#, workspace_id);

    // Get config.
    let config = &setup.common.config;

    // Get workspace subject.
    let subj = natsio::get_workpace_subject(config, WorkspacesAction::RunSurl, Some(&workspace_id));

    // Make request to backend and wait for response.
    let nats_conn = &setup.common.nats;

    // Create NATS message headers.
    let headers = [
        (natsio::WORKSPACE_ID_HEADER, workspace_id.as_str()),
        (natsio::URL_PATH_HEADER, req.uri().path()),
    ]
    .iter()
    .collect();

    debug!(r#"Set NATS message headers "{:?}""#, headers);

    // Move stream buffer from mutex. We used `stream_buf.lock()` temp MutexGuard value here to avoid async !Send issues.
    let stream_buf = mem::replace(&mut *stream_buf.lock(), Vec::new());

    // TODO(appcypher): Set reply channel.
    let reply = "";

    // Publish.
    nats_conn
        .publish_with_reply_or_headers(&subj, Some(reply), Some(&headers), &stream_buf)
        .await
        .map_err(|err| HandlerError::Internal {
            ctx: HandlerErrorMessage::InternalError,
            src: errors::wrap_error("requesting and waiting for response", err).unwrap_err(),
        })?;

    info!(r#"Published message with subject "{}""#, subj);

    // TODO(appcypher):
    // - Wait for response.
    // - Send as is to the user.
    // - Response bytes is raw HTTP response bytes. Response builder(?)
    Ok(Response::new(Body::from("Hello there!")))
}

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

        // Making sure id exists in the db.
        use crate::db::models::*;
        use crate::db::schema::workspaces::dsl::*;

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

        // Use workspace name to get id from the db.
        use crate::db::models::*;
        use crate::db::schema::workspaces::dsl::*;

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

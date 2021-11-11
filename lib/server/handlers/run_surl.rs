use crate::diesel::prelude::*;
use hyper::{Body, Request, Response};
use log::info;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use utilities::{database::DB, errors::{self, HandlerError, HandlerErrorMessage}, http::{StatusCode, WORKSPACE_ID_HEADER, WORKSPACE_NAME_HEADER}, natsio::{self, Message, WorkspacesAction}, result::HandlerResult, setup::APISetup};
use uuid::Uuid;

pub(crate) async fn run_surl(
    req: Request<Body>,
    stream_buf_mutex: Arc<Mutex<Vec<u8>>>,
    setup: Arc<APISetup>,
) -> HandlerResult<Response<Body>> {
    // Get id from header.
    let workspace_id = get_workspace_id(&setup.db, &req)?;

    info!(r#"Retrieved workspace id "{}""#, workspace_id);

    // Get config.
    let config = &setup.common.config;

    // Get workspace subject.
    let subj = natsio::get_workpace_subject(config, WorkspacesAction::RunSurl, Some(&workspace_id));

    info!(r#"Sending message with subject "{}""#, subj);

    // Get stream buffer from mutex.
    let stream_buf_guard = stream_buf_mutex.lock().unwrap(); // Lock Resource.
    let stream_buf: &[u8] = stream_buf_guard.as_ref();

    // Make request to backend and wait for response.
    let nats_conn = &setup.common.nats;

    // TODO(appcypher): How to send workspace_id and url_path in nats message headers.
    // let msg =

    let resp = nats_conn
        .request_timeout(
            &subj,
            stream_buf,
            Duration::from_secs(config.engines.api.reply_timeout),
        )
        .map_err(|err| HandlerError::Internal {
            ctx: HandlerErrorMessage::InternalError,
            src: err,
        })?;

    // Drop guard.
    drop(stream_buf_guard);

    // TODO(appcypher): Get response bytes and send. Response bytes is raw HTTP response.
    Ok(Response::new(Body::from(resp.data)))
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

        let db = db_mutex.lock().unwrap(); // Lock Resource.
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

        let db = db_mutex.lock().unwrap(); // Lock Resource.
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

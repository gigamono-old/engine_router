use crate::diesel::prelude::*;
use actix_web::{HttpRequest, HttpResponse, http::StatusCode, web::{Bytes, Data}};
use log::info;
use std::{sync::{Arc, Mutex}, time::Duration};
use utilities::{
    database::DB,
    http::{self, WORKSPACE_ID_HEADER, WORKSPACE_NAME_HEADER},
    messages::error::{HandlerError, HandlerErrorMessage, SystemError},
    natsio::{self, WorkspacesAction, Payload},
    result::HandlerResult,
    setup::APISetup,
};
use uuid::Uuid;

pub(crate) async fn run_surl(
    bytes: Bytes,
    req: HttpRequest,
    state: Data<Arc<APISetup>>,
) -> HandlerResult<HttpResponse> {
    // Get id from header.
    let workspace_id = get_workspace_id(&state.db, &req)?;

    info!(r#"Retrieved workspace id "{}""#, workspace_id);

    // Get config.
    let config = &state.common.config;

    // Get workspace subject.
    let subj = natsio::get_workpace_subject(config, WorkspacesAction::RunSurl, Some(&workspace_id));

    info!(r#"Sending message with subject "{}""#, subj);

    // Convert payload to bytes.
    let payload = Payload::new(workspace_id, http::HttpRequest::from((&req, &bytes)));
    let bytes = natsio::serialize(&payload).map_err(|err| HandlerError::Internal {
        ctx: HandlerErrorMessage::BadRequest,
        src: err,
    })?;

    // Make request to backend and wait for response.
    let nats_conn = &state.common.nats;
    let resp = nats_conn
        .request_timeout(
            &subj,
            bytes,
            Duration::from_secs(config.engines.api.reply_timeout),
        )
        .map_err(|err| HandlerError::Internal {
            ctx: HandlerErrorMessage::InternalError,
            src: err,
        })?;

    // TODO: Convert bytes to request object.

    Ok(HttpResponse::Ok().body(resp.data))
}

pub(crate) fn get_workspace_id(
    db_mutex: &Mutex<DB<PgConnection>>,
    req: &HttpRequest,
) -> HandlerResult<String> {
    // Get the workspace id from the headers.
    if let Some(header_val) = req.headers().get(WORKSPACE_ID_HEADER) {
        // Convert header bytes to string.
        let workspace_id = header_val.to_str().map_err(|err| HandlerError::Client {
            ctx: HandlerErrorMessage::InvalidWorkspaceID,
            code: StatusCode::BAD_REQUEST,
            src: SystemError::ToStr {
                ctx: "converting workspace id bytes in header to string".to_string(),
                src: err,
            },
        })?;

        // Convert string to uuid.
        let workspace_uuid = Uuid::parse_str(workspace_id).map_err(|err| HandlerError::Client {
            ctx: HandlerErrorMessage::InvalidWorkspaceID,
            code: StatusCode::BAD_REQUEST,
            src: SystemError::Uuid {
                ctx: "parsing workspace id to uuid".to_string(),
                src: err,
            },
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
                code: StatusCode::BAD_REQUEST,
                src: SystemError::Diesel {
                    ctx: "getting workspace id associated with name from the db".to_string(),
                    src: err,
                },
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
            code: StatusCode::BAD_REQUEST,
            src: SystemError::ToStr {
                ctx: "converting workspace name bytes in header to string".to_string(),
                src: err,
            },
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
                code: StatusCode::BAD_REQUEST,
                src: SystemError::Diesel {
                    ctx: "getting workspace id associated with name from the db".to_string(),
                    src: err,
                },
            })?;

        if !results.is_empty() {
            return Ok(results[0].id.to_string());
        }
    } // Drop mutex guard.

    // Error.
    Err(HandlerError::Client {
        ctx: HandlerErrorMessage::InvalidWorkspaceID,
        code: StatusCode::BAD_REQUEST,
        src: SystemError::Generic {
            ctx: "getting workspace id".to_string(),
        },
    })
}

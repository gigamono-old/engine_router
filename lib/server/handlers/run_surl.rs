// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::{server::handlers::utils, RouterStreamer};
use hyper::{Body, Request, Response};
use log::{debug, info};
use std::sync::Arc;
use utilities::{
    errors::{HandlerError, HandlerErrorMessage},
    http,
    natsio::{self, RequestResponseSubjects},
    result::HandlerResult,
    setup::RouterSetup,
};

pub(crate) async fn run_surl(
    req: Request<Body>,
    setup: Arc<RouterSetup>,
) -> HandlerResult<Response<Body>> {
    // Get id from http request header.
    let workspace_id = &utils::get_workspace_id(&setup.db, &req)?;

    debug!(r#"Retrieved workspace id "{}""#, workspace_id);

    // The subjects for handling streaming.
    let subjects = RequestResponseSubjects {
        directives: nuid::next(),
        request_body: nuid::next(),
        response_body: nuid::next(),
    };

    info!(r#"Handler specific subjects "{}""#, subjects);

    // Prepare NATS headers.
    let headers = create_nats_headers(&subjects, &workspace_id, req.uri().path());

    debug!(r#"NATS message headers "{:?}""#, headers);

    // Convert hyper request and serialize.
    let request = http::Request::from_hyper_request(req)
        .await
        .map_err(|err| HandlerError::Internal {
            ctx: HandlerErrorMessage::InternalError,
            src: err,
        })?;

    let streamer = RouterStreamer::new(setup, subjects, request);

    streamer.publish_request(headers, workspace_id).await?;

    streamer.listen().await
}

pub(crate) fn create_nats_headers(
    subjects: &RequestResponseSubjects,
    workspace_id: &str,
    uri_path: &str,
) -> natsio::Headers {
    [
        (natsio::WORKSPACE_ID_HEADER, workspace_id),
        (natsio::URL_PATH_HEADER, uri_path),
        (natsio::DIRECTIVES_HEADER, subjects.directives.as_str()),
        (natsio::REQUEST_BODY_HEADER, subjects.request_body.as_str()),
        (
            natsio::RESPONSE_BODY_HEADER,
            subjects.response_body.as_str(),
        ),
    ]
    .iter()
    .collect()
}

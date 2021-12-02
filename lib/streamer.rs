// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::sync::Arc;

use hyper::{Body, Response};
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage},
    http::Request,
    natsio::{
        self, Connection, Headers, Message, RequestResponseSubjects, Subscription, WorkspacesAction,
    },
    result::HandlerResult,
    setup::RouterSetup,
};

pub struct RouterStreamer {
    pub setup: Arc<RouterSetup>,
    pub subjects: RequestResponseSubjects,
    pub request: Request,
}

impl RouterStreamer {
    pub fn new(
        setup: Arc<RouterSetup>,
        subjects: RequestResponseSubjects,
        request: Request,
    ) -> Self {
        Self {
            setup,
            subjects,
            request,
        }
    }

    pub async fn publish_request(&self, headers: Headers, workspace_id: &str) -> HandlerResult<()> {
        let nats_conn = &self.setup.common.nats;
        let config = &self.setup.common.config;
        let workspaces_subject =
            natsio::create_workpaces_subject(config, WorkspacesAction::RunSurl, Some(workspace_id));

        // Serialize request.
        let request_bytes =
            bincode::serialize(&self.request).map_err(|err| HandlerError::Internal {
                ctx: HandlerErrorMessage::InternalError,
                src: errors::wrap_error("serializing request", err).unwrap_err(),
            })?;

        // Publish message to workspace subjects with necessary headers.
        nats_conn
            .publish_with_reply_or_headers(
                &workspaces_subject,
                None,
                Some(&headers),
                &request_bytes,
            ) // TODO(appcypher)
            .await
            .map_err(|err| HandlerError::Internal {
                ctx: HandlerErrorMessage::InternalError,
                src: errors::wrap_error(
                    "publishing headers and request to workspaces subject",
                    err,
                )
                .unwrap_err(),
            })
    }

    pub async fn listen(&self) -> HandlerResult<Response<Body>> {
        let nats_conn = &self.setup.common.nats;
        let (directives_sub, response_body_sub) = self.subscribe(nats_conn).await?;
        let mut req_body_handled = false;

        loop {
            // TODO(appcypher): SEC: We need a timeout to avoid leaking resource.
            match directives_sub.next().await {
                Some(msg) => {
                    if natsio::has_header_key(&msg, natsio::SENDING_RESPONSE_BODY_HEADER) {
                        // Loop ends once we handle "sending_response_body"
                        break self.handle_sending_response_body(response_body_sub).await;
                    } else if !req_body_handled
                        && natsio::has_header_key(&msg, natsio::NEED_REQUEST_BODY_HEADER)
                    {
                        // Otherwise we handle "need_request_body" message if not already handled.
                        self.handle_need_request_body().await?;
                        req_body_handled = true;
                    }
                }
                None => {
                    break Err(HandlerError::Internal {
                        ctx: HandlerErrorMessage::InternalError,
                        src: errors::any_error(
                            "directives subject connection closed or subscription canceled",
                        )
                        .unwrap_err(),
                    });
                }
            }
        }
    }

    async fn handle_need_request_body(&self) -> HandlerResult<()> {
        todo!()
    }

    async fn handle_sending_response_body(
        &self,
        response_body_sub: Subscription,
    ) -> HandlerResult<Response<Body>> {
        // TODO: How to implement acknowledgement
        Ok(Response::default())
    }

    async fn subscribe(
        &self,
        nats_conn: &Connection,
    ) -> HandlerResult<(Subscription, Subscription)> {
        let directives_sub = nats_conn
            .subscribe(&self.subjects.directives)
            .await
            .map_err(|err| HandlerError::Internal {
                ctx: HandlerErrorMessage::InternalError,
                src: errors::wrap_error("subscribing to directives subject", err).unwrap_err(),
            })?;

        let response_body_sub = nats_conn
            .subscribe(&self.subjects.directives)
            .await
            .map_err(|err| HandlerError::Internal {
                ctx: HandlerErrorMessage::InternalError,
                src: errors::wrap_error("subscribing to response body subject", err).unwrap_err(),
            })?;

        Ok((directives_sub, response_body_sub))
    }
}

// trace!(
//     r#"Request bytes "{}""#,
//     String::from_utf8_lossy(&request_bytes)
// );

// Get workspace subject.
// let workspaces_subject =
//     natsio::get_workpace_subject(config, WorkspacesAction::RunSurl, Some(&workspace_id));

// let request_bytes = bincode::serialize(&request).map_err(|err| HandlerError::Internal {
//     ctx: HandlerErrorMessage::InternalError,
//     src: errors::wrap_error("serializing request", err).unwrap_err(),
// })?;

// // Publish.
// nats_conn
//     .publish_with_reply_or_headers(&workspaces_subject, None, Some(&headers), &request_bytes) // TODO(appcypher)
//     .await
//     .map_err(|err| HandlerError::Internal {
//         ctx: HandlerErrorMessage::InternalError,
//         src: errors::wrap_error("sending message to subscriber", err).unwrap_err(),
//     })?;

// info!(r#"Published message with subject "{}""#, workspaces_subject);

// Ok(Response::new(Body::from("Hello there!")))

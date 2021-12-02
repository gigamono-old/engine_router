// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::sync::Arc;

use hyper::{Body, Request, Response};
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage},
    http::{utils, StatusCode},
    result::HandlerResult,
    setup::RouterSetup,
};

use super::handlers::run_surl;

pub(crate) async fn route(
    req: Request<Body>,
    setup: Arc<RouterSetup>,
) -> HandlerResult<Response<Body>> {
    let _ = req.method();
    let path = req.uri().path();

    // If the path starts with "/r/".
    if path.starts_with("/r/") {
        run_surl(req, setup).await

    // If the path starts with a number (like "/2/system/load/prometheus/index.css").
    } else if let Ok(_) = utils::parse_url_path_number(path) {
        run_surl(req, setup).await

    // The other routes.
    } else {
        // Not found error
        Err(HandlerError::Client {
            ctx: HandlerErrorMessage::NotFound,
            code: StatusCode::NotFound,
            src: errors::any_error(format!(r#"resource not found "{}""#, path)).unwrap_err(),
        })
    }
}

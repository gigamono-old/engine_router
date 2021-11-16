use std::sync::{Arc, Mutex};

use hyper::{Body, Request, Response};
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage},
    http::{utils, StatusCode},
    result::HandlerResult,
    setup::APISetup,
};

pub(crate) async fn router(
    req: Request<Body>,
    stream_buf_mutex: Arc<Mutex<Vec<u8>>>,
    setup: Arc<APISetup>,
) -> HandlerResult<Response<Body>> {
    let _ = req.method();
    let path = req.uri().path();

    // If the path starts with "/r/".
    if path.starts_with("/r/") {
        super::run_surl(req, stream_buf_mutex, setup).await

    // If the path starts with a number (like "/2/system/load/prometheus/index.css").
    } else if let Ok(_) = utils::parse_url_path_number(path) {
        super::run_surl(req, stream_buf_mutex, setup).await

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

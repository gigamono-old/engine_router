// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use super::router;
use futures::{Future, FutureExt};
use hyper::service::make_service_fn;
use hyper::{service::service_fn, Body};
use hyper::{Request, Response, Server};
use log::error;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use utilities::errors::{self, HandlerError, HandlerErrorMessage};
use utilities::result::HandlerResult;
use utilities::setup::RouterSetup;

pub struct RouterServer {
    pub setup: Arc<RouterSetup>,
}

impl RouterServer {
    pub fn new(setup: Arc<RouterSetup>) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> utilities::result::Result<()> {
        // Initialize logger.
        env_logger::init();

        // Get port info and create socket address.
        let port = self.setup.common.config.engines.api.port;
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // Cloen setup.
        let setup = Arc::clone(&self.setup);

        // Create service.
        let make_svc = make_service_fn(move |_| {
            let setup = Arc::clone(&setup);

            async {
                Ok::<_, Infallible>(service_fn(move |req| {
                    Self::error_wrap(router::router, req, Arc::clone(&setup))
                }))
            }
        });

        // Serve.
        let server = Server::bind(&addr).serve(make_svc);

        Ok(server.await?)
    }

    async fn error_wrap<F, Fut, A>(
        func: F,
        req: Request<A>,
        setup: Arc<RouterSetup>,
    ) -> Result<Response<Body>, Infallible>
    where
        F: FnOnce(Request<A>, Arc<RouterSetup>) -> Fut,
        Fut: Future<Output = HandlerResult<Response<Body>>>,
    {
        // AssertUnwindSafe to catch handler panics and return appropriate 500 error and log errors.
        match AssertUnwindSafe(func(req, Arc::clone(&setup)))
            .catch_unwind()
            .await
        {
            // Handler returned a result.
            Ok(Ok(response)) => Ok(response),
            Ok(Err(err)) => {
                // Log error.
                error!("{:?}", err);

                // Send appropriate server response.
                let resp = Response::builder()
                    .status(err.status_code())
                    .body(Body::from(err.error_json()))
                    .unwrap();

                Ok(resp)
            }
            // Handler panicked.
            Err(err) => {
                // Log panic error.
                error!("{:?}", err);

                // Send 500 response.
                let handler_err = HandlerError::Internal {
                    ctx: HandlerErrorMessage::InternalError,
                    src: errors::any_error(format!("handler panic: {:?}", err)).unwrap_err(),
                };

                let resp = Response::builder()
                    .status(handler_err.status_code())
                    .body(Body::from(handler_err.error_json()))
                    .unwrap();

                Ok(resp)
            }
        }
    }
}

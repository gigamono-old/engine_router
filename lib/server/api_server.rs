use super::{handlers, stream::RecordStream};
use futures::{Future, FutureExt};
use hyper::server::conn::Http;
use hyper::{service::service_fn, Body};
use hyper::{Request, Response};
use log::error;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use utilities::errors::{self, HandlerError, HandlerErrorMessage};
use utilities::result::HandlerResult;
use utilities::setup::APISetup;

pub struct APIServer {
    pub setup: Arc<APISetup>,
}

impl APIServer {
    pub fn new(setup: Arc<APISetup>) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> utilities::result::Result<()> {
        // Initialize logger.
        env_logger::init();

        // Get port info and create socket address.
        let port = self.setup.common.config.engines.api.port;
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // Bind to address.
        let tcp_listener = TcpListener::bind(addr).await.unwrap();

        loop {
            // Clone setup object.
            let setup = Arc::clone(&self.setup);

            // Accept client connection.
            let (tcp_stream, _) = tcp_listener.accept().await.unwrap();

            // Spawn a task for each connection.
            tokio::task::spawn(async move {
                // Create a shared buffer and pass that to a custom stream that wraps tcp stream.
                let stream_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
                let record_stream = RecordStream::new(tcp_stream, Arc::clone(&stream_buf));

                // TODO(appcypher):
                // - /r/*
                // - static files
                // let files = Static::new("../web-ui/dist");
                // static.serve

                // Set up http handling context. HTTP/1.x supported for now.
                if let Err(http_err) = Http::new()
                    .http1_only(true)
                    .http1_keep_alive(true)
                    .serve_connection(
                        record_stream,
                        service_fn(|req: Request<Body>| {
                            Self::error_wrap(
                                handlers::router,
                                req,
                                Arc::clone(&stream_buf),
                                Arc::clone(&setup),
                            )
                        }),
                    )
                    .await
                {
                    panic!("ERROR = {}", http_err) // TODO(appcypher)
                }
            });
        }
    }

    async fn error_wrap<F, Fut, A>(
        func: F,
        req: Request<A>,
        stream_buf: Arc<Mutex<Vec<u8>>>,
        setup: Arc<APISetup>,
    ) -> Result<Response<Body>, Infallible>
    where
        F: FnOnce(Request<A>, Arc<Mutex<Vec<u8>>>, Arc<APISetup>) -> Fut,
        Fut: Future<Output = HandlerResult<Response<Body>>>,
    {
        // AssertUnwindSafe to catch handler panics and return appropriate 500 error and log errors.
        match AssertUnwindSafe(func(req, Arc::clone(&stream_buf), Arc::clone(&setup)))
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

// impl APIServer {
//     pub fn new(setup: Arc<APISetup>) -> Self {
//         Self { setup }
//     }

//     // TODO(appcypher):
//     // In the future we are going to use tokio::TcpListener or create something similar to tokio-minihttp.
//     // We need to be able to read bytes from socket directly in order to discard natsio::Payload.
//     // Right now we are doing this:
//     //      engine-api [parse-request -> convert-to-payload -> serialize-payload -> send-payload-bytes]
//     //      engine-backend [deserialise-payload]
//     // When we can do this instead:
//     //      engine-api [parse-request? -> send-request-bytes]
//     //      engine-backend [parse-request]
//     pub async fn listen(&self) -> Result<()> {
//         // Initialize logger.
//         env_logger::init();

//         let setup = Arc::clone(&self.setup);

//         // Get port info.
//         let port = self.setup.common.config.engines.api.port;

//         // Set server up and run
//         HttpServer::new(move || {
//             App::new()
//                 .wrap(Logger::default())
//                 .configure(|c| Self::app_config(Arc::clone(&setup), c)) // This part needs to come before static files serving.
//                 .service(
//                     fs::Files::new("/", &setup.common.config.web_ui.dir)
//                         .index_file("index.html")
//                         .use_last_modified(true),
//                 )
//         })
//         .bind(("127.0.0.1", port))
//         .context("starting api server")?
//         .run()
//         .await
//         .context("starting api server")
//     }

//     fn app_config(setup: Arc<APISetup>, config: &mut ServiceConfig) {
//         config.service(
//             web::resource("/r/*").data(setup).route(
//                 web::route()
//                     .guard(http::any_method_guard())
//                     .to(handlers::run_surl),
//             ),
//         );
//     }
// }

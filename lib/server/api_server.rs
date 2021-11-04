use super::handlers;
use actix_files as fs;
use actix_web::{
    self,
    middleware::Logger,
    web::{self, ServiceConfig},
    App, HttpServer,
};
use std::sync::Arc;
use utilities::result::{Context, Result};
use utilities::{http, setup::APISetup};

pub struct APIServer {
    pub setup: Arc<APISetup>,
}

impl APIServer {
    pub fn new(setup: Arc<APISetup>) -> Self {
        Self { setup }
    }

    // TODO(appcypher): In the future we are going to use tokio::TcpListener to allow reading bytes from socket directly
    // so that we won't need natsio::Payload. Right now we are doing this:
    //      engine-api [parse-request -> convert-to-payload -> serialize-payload -> send-payload]
    //      engine-backend [deserialise-payload]
    // when we can do this instead:
    //      engine-api [parse-request -> send-request]
    //      engine-backend [parse-request]
    pub async fn listen(&self) -> Result<()> {
        // Initialize logger.
        env_logger::init();

        let setup = Arc::clone(&self.setup);

        // Get port info.
        let port = self.setup.common.config.engines.api.port;

        // Set server up and run
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .configure(|c| Self::app_config(Arc::clone(&setup), c)) // This part needs to come before static files serving.
                .service(
                    fs::Files::new("/", &setup.common.config.web_ui.dir)
                        .index_file("index.html")
                        .use_last_modified(true),
                )
        })
        .bind(("127.0.0.1", port))
        .context("starting api server")?
        .run()
        .await
        .context("starting api server")
    }

    fn app_config(setup: Arc<APISetup>, config: &mut ServiceConfig) {
        config.service(
            web::resource("/r/*").data(setup).route(
                web::route()
                    .guard(http::any_method_guard())
                    .to(handlers::run_surl),
            ),
        );
    }
}

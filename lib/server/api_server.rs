use super::handlers;
use actix_web::{
    self,
    middleware::Logger,
    web::{self, ServiceConfig},
    App, HttpServer,
};
use std::sync::Arc;
use utilities::result::Result;
use utilities::{http, messages::error::SystemError, setup::APISetup};

pub struct APIServer {
    pub setup: Arc<APISetup>,
}

impl APIServer {
    pub fn new(setup: Arc<APISetup>) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> Result<()> {
        // Initialize logger.
        env_logger::init();

        let setup = Arc::clone(&self.setup);

        // Get port info.
        let port = self.setup.common.config.engines.api.port;

        // Set server up and run.
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .configure(|c| Self::app_config(Arc::clone(&setup), c))
        })
        .bind(("127.0.0.1", port))
        .map_err(|err| SystemError::Io {
            ctx: "starting api server".to_string(),
            src: err,
        })?
        .run()
        .await
        .map_err(|err| SystemError::Io {
            ctx: "starting api server".to_string(),
            src: err,
        })
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

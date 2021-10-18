use std::sync::{Arc, Mutex};

use super::handlers;
use actix_web::middleware::Logger;
use actix_web::web::ServiceConfig;
use actix_web::{web, App, HttpServer};
use env_logger;
use utilities::result::Result;
use utilities::{http, messages::error::SystemError, setup::APISetup};

pub struct APIServer {
    pub setup: Arc<Mutex<APISetup>>,
}

impl APIServer {
    pub fn new(setup: Arc<Mutex<APISetup>>) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> Result<()> {
        let setup = self.setup.clone();

        // Get port data from self and drop to release lock after.
        let port = self.setup.lock().unwrap().common.config.engines.api.port;
        drop(self);

        // Initialize logger.
        env_logger::init();

        // Set server up and run.
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .wrap(Logger::new("%a %{User-Agent}i"))
                .configure(|c| Self::app_config(setup.clone(), c))
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

    fn app_config(setup: Arc<Mutex<APISetup>>, config: &mut ServiceConfig) {
        config.service(
            web::resource("/r/*").data(setup).route(
                web::route()
                    .guard(http::any_method_guard())
                    .to(handlers::run),
            ),
        );
    }
}

use actix_web::{App, HttpServer};
use futures::join;
use utilities::messages::error::Result;
use utilities::{messages::error::SystemError, setup::SharedSetup};

pub struct APIServer<'a> {
    setup: &'a SharedSetup,
}

impl<'a> APIServer<'a> {
    pub fn new(setup: &'a SharedSetup) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> Result<()> {
        // Set server up and run.
        let server = HttpServer::new(|| App::new().service(routes::greet))
            .bind(("127.0.0.1", self.setup.config.engines.api.port))
            .map_err(|err| SystemError::Io {
                ctx: "starting api server".to_string(),
                src: err,
            })?
            .run();

        // Add running acknowledgement after server starts running.
        let (result, _) = join!(server, self.run_acknowledgement());
        result.map_err(|err| SystemError::Io {
            ctx: "starting api server".to_string(),
            src: err,
        })?;

        Ok(())
    }

    async fn run_acknowledgement(&self) {
        println!(
            "server listening on port {}",
            self.setup.config.engines.api.port
        );
    }
}

mod routes {
    use actix_web::{get, Responder};

    #[get("/greet")]
    pub async fn greet() -> impl Responder {
        format!("Hello world!")
    }
}

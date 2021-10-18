extern crate actix_web;
extern crate engine_api;
extern crate utilities;

use std::sync::{Arc, Mutex};

use engine_api::APIServer;
use utilities::result::Result;
use utilities::setup::APISetup;

#[actix_web::main]
async fn main() -> Result<()> {
    let setup = Arc::new(Mutex::new(APISetup::new().unwrap()));
    let server = APIServer::new(setup);
    server.listen().await
}

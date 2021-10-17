extern crate actix_web;
extern crate engine_api;
extern crate utilities;

use engine_api::APIServer;
use utilities::setup::SharedSetup;
use utilities::messages::error::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    let setup = SharedSetup::new().unwrap();
    let server = APIServer::new(&setup);
    server.listen().await
}

extern crate actix_web;
extern crate engine_api;
extern crate utilities;

use engine_api::APIServer;
use utilities::result::Result;
use utilities::setup::SharedSetup;

#[actix_web::main]
async fn main() -> Result<()> {
    let setup = SharedSetup::new().unwrap();
    let server = APIServer::new(&setup);
    server.listen().await
}

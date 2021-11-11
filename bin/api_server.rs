extern crate engine_api;
extern crate utilities;

use engine_api::APIServer;
use std::sync::Arc;
use utilities::result::Result;
use utilities::setup::APISetup;

#[tokio::main]
async fn main() -> Result<()> {
    let setup = Arc::new(APISetup::new()?);
    let server = APIServer::new(setup);
    server.listen().await
}

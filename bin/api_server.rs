// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate engine_api;
extern crate utilities;

use engine_api::APIServer;
use std::sync::Arc;
use utilities::result::Result;
use utilities::setup::APISetup;

#[tokio::main]
async fn main() -> Result<()> {
    let setup = Arc::new(APISetup::new().await?);
    let server = APIServer::new(setup);
    server.listen().await
}

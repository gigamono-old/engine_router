// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate engine_router;
extern crate utilities;

use engine_router::RouterServer;
use std::sync::Arc;
use utilities::result::Result;
use utilities::setup::RouterSetup;

#[tokio::main]
async fn main() -> Result<()> {
    let setup = Arc::new(RouterSetup::new().await?);
    let server = RouterServer::new(setup);
    server.listen().await
}

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::{web, HttpResponse};
use utilities::setup::APISetup;

pub(crate) fn run(state: web::Data<Arc<Mutex<APISetup>>>) -> HttpResponse {
    {
        let setup = state.as_ref().as_ref();
        let nc = &setup
            .lock() // TODO: Handle error
            .unwrap()
            .common
            .nats
            .conn;

        let resp = nc
            .request_timeout("v1.run.workspaces", "message", Duration::from_secs(2))
            .unwrap(); // TODO: Handle error

        println!(">> returned reply = {}", resp);
    }

    HttpResponse::Ok().body("Published")
}

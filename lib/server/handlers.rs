use actix_web::{HttpRequest, Responder};

pub(crate) async fn greet(req: HttpRequest) -> impl Responder {
    format!("Hello world!")
}

mod info;
mod search;

use actix_web::{get, http::header, web, HttpResponse, Responder};

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .service(info::service)
        .service(search::service)
        .service(index);
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .header(header::CONTENT_TYPE, "text/html")
        .body(include_str!("index.html"))
}

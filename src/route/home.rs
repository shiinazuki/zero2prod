mod login;

pub use login::*;

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;

pub async fn home() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("home/home.html"))
}

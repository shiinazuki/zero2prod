use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

pub async fn login_from(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .cookie(Cookie::build("_flash", "").max_age(Duration::ZERO).finish())
        .body(format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Login</title>
            </head>
            <body>
            {error_html}
            <form action="/login" method="post">
            <label>Username
            <input
            type="text"
            placeholder="Enter Username"
            name="username"
            >
            </label>
            <label>Password
            <input
            type="password"
            placeholder="Enter Password"
            name="password"
            >
            </label>
            <button type="submit">Login</button>
            </form>
            </body>
            </html>"#,
        ));
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
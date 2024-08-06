use std::net::TcpListener;
use actix_web::{App, HttpResponse, HttpServer,  web};
use actix_web::dev::Server;



async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}


// 注意不同的签名！
// 我们在快乐路径上返回 `Server`，并且删除了 `async` 关键字
// 我们没有 .await 调用，因此不再需要它。
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
    })
        .listen(listener)?
        .run();

    // No .await here!
    Ok(server)
}
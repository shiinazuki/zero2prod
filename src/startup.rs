use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use sqlx::PgPool;
use crate::route::*;

// 注意不同的签名！
// 我们在快乐路径上返回 `Server`，并且删除了 `async` 关键字
// 我们没有 .await 调用，因此不再需要它。
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {

    // 将连接包装在智能指针中
    let connection = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            // 我们的路由表中为 POST /subscribe 请求添加一个新条目
            .route("/subscriptions", web::post().to(subscribe))
            // 将连接注册为应用程序状态的一部分
            .app_data(connection.clone())
    })
        .listen(listener)?
        .run();

    // No .await here!
    Ok(server)
}

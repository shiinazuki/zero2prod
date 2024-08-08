use crate::route::*;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

// 注意不同的签名！
// 我们在快乐路径上返回 `Server`，并且删除了 `async` 关键字
// 我们没有 .await 调用，因此不再需要它。
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // 将连接包装在智能指针中
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            // 使用 `App` 上的 `wrap` 方法添加 logger 中间件
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // 我们的路由表中为 POST /subscribe 请求添加一个新条目
            .route("/subscriptions", web::post().to(subscribe))
            // 将数据库连接注册为应用程序状态的一部分
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    // No .await here!
    Ok(server)
}

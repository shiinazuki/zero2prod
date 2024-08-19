use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::route::*;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

// 新类型，用于保存新建的服务器及其端口
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // 我们已经将 `build` 函数转换为 `Application` 的构造函数
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        // 拿到pgsql的连接
        let connection_pool = get_connection_pool(&configuration.database);

        // 使用 `configuration` 构建 `EmailClient`
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool.await,
            email_client,
            configuration.application.base_url,
        )?;

        // 我们将绑定的端口“保存”在 `Application` 的一个字段中
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // 一个更具表现力的名称，清楚地表明此函数仅在应用程序停止时返回。
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

// 提取获取连接池代码
pub async fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPool::connect(configuration.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres")
    // PgPoolOptions::new()
    // .acquire_timeout(Duration::from_secs(20))
    // .connect_lazy_with(configuration.with_db())
}

// 我们需要定义一个包装器类型，以便在 `subscribe` 处理程序中检索 URL
// 在 actix-web 中，从上下文中检索是基于类型的：使用
// 原始 `String` 会让我们面临冲突。
pub struct ApplicationBaseUrl(pub String);

// 注意不同的签名！
// 我们在快乐路径上返回 `Server`，并且删除了 `async` 关键字
// 我们没有 .await 调用，因此不再需要它。
fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    // 将连接包装在智能指针中
    let db_pool = web::Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));

    let server = HttpServer::new(move || {
        App::new()
            // 使用 `App` 上的 `wrap` 方法添加 logger 中间件
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // 我们的路由表中为 POST /subscribe 请求添加一个新条目
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            // 将数据库连接注册为应用程序状态的一部分
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

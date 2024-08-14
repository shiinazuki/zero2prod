use once_cell::sync::Lazy;
use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::{Application, get_connection_pool};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// 确保“tracing”堆栈仅使用“once_cell”初始化一次
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// 以某种方式在后台启动我们的应用程序
pub async fn spawn_app() -> TestApp {
    // 第一次调用 `initialize` 时，将执行 `TRACING` 中的代码。
    // 所有其他调用都将跳过执行
    Lazy::force(&TRACING);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let address = format!("http://127.0.0.1:{}", application.port());

    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database).await,
    }
}

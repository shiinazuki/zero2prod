use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 如果不能读取配置  panic
    let configuration = get_configuration().expect("Failed to read configuration.");

    // 拿到pgsql的连接
    let connection_pool =
        PgPool::connect(&configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    // 我们已经删除了硬编码的“8000”——它现在来自我们的设置！
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listen = TcpListener::bind(address)?;
    startup::run(listen, connection_pool)?.await?;
    Ok(())
}

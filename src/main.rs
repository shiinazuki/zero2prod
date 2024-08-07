use std::net::TcpListener;
use sqlx::{PgPool};
use zero2prod::configuration::get_configuration;
use zero2prod::startup;

const HOST_ADDRESS: &str = "127.0.0.1";


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    // 如果不能读取配置  panic
    let configuration = get_configuration().expect("Failed to read configuration.");

    // 拿到pgsql的连接
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // 我们已经删除了硬编码的“8000”——它现在来自我们的设置！
    let address = format!("{}:{}", HOST_ADDRESS, configuration.application_port);

    let listen = TcpListener::bind(address)?;
    startup::run(listen, connection_pool)?.await
}

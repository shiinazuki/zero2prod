use once_cell::sync::Lazy;
use sqlx::PgPool;
use wiremock::MockServer;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::{get_connection_pool, Application};
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

/// 嵌入在电子邮件 API 请求中的确认链接
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    /// 将请求中嵌入的确认链接提取到电子邮件 API。
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // 从其中一个请求字段中提取链接
        let get_link = |s: &str| {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect::<Vec<_>>();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // 确保我们不会在网络上调用随机 API
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["MessageStream"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }
}

// 以某种方式在后台启动我们的应用程序
pub async fn spawn_app() -> TestApp {
    // 第一次调用 `initialize` 时，将执行 `TRACING` 中的代码。
    // 所有其他调用都将跳过执行
    Lazy::force(&TRACING);

    // 启动一个模拟服务器来代替 Postmark 的 API
    let email_server = MockServer::start().await;

    // 随机配置以确保测试隔离
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        // 使用模拟服务器作为电子邮件 API
        c.email_client.base_url = email_server.uri();
        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let address = format!("http://127.0.0.1:{}", application.port());
    let application_port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        port: application_port,
        db_pool: get_connection_pool(&configuration.database).await,
        email_server,
    }
}

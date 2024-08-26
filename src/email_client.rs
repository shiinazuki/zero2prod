use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use std::time::Duration;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            message_stream: text_content,
        };
        let _builder = self
            .http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    message_stream: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use std::time::Duration;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    /// 生成随机电子邮件主题
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// 生成随机电子邮件内容
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// 生成随机订阅者的电子邮件
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// 获取 `EmailClient` 的测试实例。
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 通过 Mock 来指示模拟服务器以不同的方式运行
        // 通过传入any() 来达到无论请求是什么，总是匹配
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // 使用我们的自定义匹配器！
            .and(SendEmailBodyMatcher)
            // 按照这个响应
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // MockServer::start 向操作系统请求一个随机可用端口，并在后台线程上启动服务器 准备监听传入的请求
        let mock_server = MockServer::start().await;
        // 发送者邮件
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200)
            // 延迟3分钟响应
            .set_delay(Duration::from_secs(180));

        // 通过 Mock 来指示模拟服务器以不同的方式运行
        // 通过传入any() 来达到无论请求是什么，总是匹配
        Mock::given(any())
            // 按照这个响应
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // MockServer::start 向操作系统请求一个随机可用端口，并在后台线程上启动服务器 准备监听传入的请求
        let mock_server = MockServer::start().await;
        // 发送者邮件
        let email_client = email_client(mock_server.uri());

        // 通过 Mock 来指示模拟服务器以不同的方式运行
        // 通过传入any() 来达到无论请求是什么，总是匹配
        Mock::given(any())
            // 按照这个响应
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_success_if_the_server_returns_200() {
        // MockServer::start 向操作系统请求一个随机可用端口，并在后台线程上启动服务器 准备监听传入的请求
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 通过 Mock 来指示模拟服务器以不同的方式运行
        // 通过传入any() 来达到无论请求是什么，总是匹配
        Mock::given(any())
            // 按照这个响应
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_ok!(outcome);
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // 尝试将主体解析为 JSON 值
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                // 检查所有必填字段是否已填充
                // 不检查字段值
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("MessageStream").is_some()
            } else {
                // 如果解析失败  不匹配请求
                false
            }
        }
    }
}

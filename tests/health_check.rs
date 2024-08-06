//! tests/health_check.rs

use std::net::TcpListener;

// `tokio::test` 是 `tokio::main` 的测试等价物。
// 它还使您不必指定 `#[test]` 属性。
//
// 您可以使用以下方法检查生成的代码
// `cargo expand --test health_check` (<- 测试文件的名称)
#[tokio::test]
async fn health_check_works() {
    // 安排
    let address = spawn_app();

    let client = reqwest::Client::new();

    // Act
    let response = client
        // 使用返回的应用程序地址
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// 以某种方式在后台启动我们的应用程序

// 没有 .await 调用，因此现在不需要 `spawn_app` 异步。
// 我们也正在运行测试，因此传播错误是不值得的：
// 如果我们无法执行所需的设置，我们就会惊慌失措并崩溃
// 所有的事情
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // 我们检索操作系统分配给我们的端口
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    // 将服务器作为后台任务启动
    // tokio::spawn 返回生成的未来的句柄，
    // 但这里我们用不着它，因此非绑定 let
    let _ = tokio::spawn(server);

    // 我们将应用程序地址返回给调用者！
    format!("http://127.0.0.1:{}", port)

}


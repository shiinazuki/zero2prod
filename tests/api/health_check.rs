use crate::helpers::spawn_app;

// `tokio::test` 是 `tokio::main` 的测试等价物。
// 它还使您不必指定 `#[test]` 属性。
//
// 您可以使用以下方法检查生成的代码
// `cargo expand --test health_check` (<- 测试文件的名称)
#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        // 使用返回的应用程序地址
        .get(&format!("{}/health_check", &address.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

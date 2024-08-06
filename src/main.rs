use std::net::TcpListener;

const HOST_ADDRESS: &str = "127.0.0.1:0";
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    let listen = TcpListener::bind(HOST_ADDRESS).expect("Failed to bind random port");
    let port = listen.local_addr().unwrap().port();
    println!("随机端口是: {}", port);
    // 如果绑定地址失败，则将 io::Error
    // 否则，在我们的服务器上调用 .await
    zero2prod::run(listen)?.await
}


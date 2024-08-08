use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// 将多个层组合成 `tracing` 的订阅者。
///
/// # 实施说明
///
/// 我们使用 `impl Subscriber` 作为返回类型，以避免必须说明返回订阅者的实际类型，这确实非常复杂
/// 我们需要明确指出返回的订阅者是 `Send` 和 `Sync`，以便稍后将其传递给 `init_subscriber`
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // 如果未设置 RUST_LOG 环境变量，我们将返回打印 info 级别或更高级别的所有日志。
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(
        name, // 将格式化的跨度输出到标准输出。
        sink,
    );

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// 将订阅者注册为全局默认值以处理跨度数据。
///
/// 它应该只被调用一次！
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // 将所有“log”事件重定向到我们的订阅者
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

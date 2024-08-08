# 我们使用最新的 Rust 稳定版本作为基础镜像
FROM lukemathwalker/cargo-chef:latest-rust:1.80.0 AS chef


# 让我们将工作目录切换到 `app`（相当于 `cd app`）
# 如果 `app` 文件夹不存在，Docker 将为我们创建它
WORKDIR /app

# 安装链接配置所需的系统依赖项
RUN apt update && apt install lld clang -y

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 将工作环境中的所有文件复制到 Docker 镜像中
COPY . .

# 让我们构建我们的二进制文件！
# 我们将使用发布配置文件使其尽快完成
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
&& apt-get install -y --no-install-recommends openssl ca-certificates \
&& apt-get autoremove -y \
&& apt-get clean -y \
&& rm -rf /var/lib/apt/lists/*


COPY --from=builder /app/target/release/zero2prod zero2prod

COPY configuration configuration

ENV APP_ENVIRONMENT production

# 当执行`docker run`时，启动二进制文件！
ENTRYPOINT ["./zero2prod"]


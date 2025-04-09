# マルチステージビルドを使用し、Rustのプログラムをビルド
FROM rust:1.86.0-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# sqlxクレートを使ったビルドで必要な環境変数を設定
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# 不要なソフトウェアはいらないので、軽量なbookworm-slimを使用
FROM debian:bookworm-slim
WORKDIR /app

# ユーザー作成
RUN adduser book && chown -R book /app
USER book

# ビルドしたプログラムをコピー
COPY --from=builder ./app/target/release/app ./target/release/app

# 8080番ポートを解放し、アプリケーションを起動
ENV PORT=8080
EXPOSE ${PORT}
ENTRYPOINT [ "./target/release/app" ]
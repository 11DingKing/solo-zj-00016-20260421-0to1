FROM --platform=linux/arm64 rust:1.75-alpine as builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app

RUN rustup target add aarch64-unknown-linux-musl

COPY Cargo.toml Cargo.toml

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release --target aarch64-unknown-linux-musl

RUN rm -rf src

COPY src src

RUN touch src/main.rs

RUN cargo build --release --target aarch64-unknown-linux-musl

FROM --platform=linux/arm64 alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/shortlink_backend /app/shortlink_backend

EXPOSE 8080

CMD ["/app/shortlink_backend"]

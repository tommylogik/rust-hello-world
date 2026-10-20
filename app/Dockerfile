ARG APP_PORT=3000

FROM rust:alpine

ARG APP_PORT

WORKDIR /app

RUN apk add --no-cache musl-dev

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    set -e cargo build --locked --release


ENV PORT=${APP_PORT}

CMD ["/app/target/release/hello-world"]

EXPOSE ${APP_PORT}

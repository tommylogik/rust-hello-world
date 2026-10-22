# --- Stage 1: Initialize cargo-chef ---
FROM lukemathwalker/cargo-chef:latest-rust-1.85-alpine AS chef
WORKDIR /app

# --- Stage 2: Compute dependency recipe ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Stage 3: Build statically linked binary ---
FROM chef AS builder

# Automatically provided by Docker Buildx
ARG TARGETARCH

# Install system certificates
RUN apk add --no-cache ca-certificates

# Map Docker's $TARGETARCH variable to Rust's target triples
RUN if [ "$TARGETARCH" = "amd64" ]; then \
        echo "x86_64-unknown-linux-musl" > /target.txt; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        echo "aarch64-unknown-linux-musl" > /target.txt; \
    else \
        echo "Unsupported architecture: $TARGETARCH" && exit 1; \
    fi && \
    rustup target add $(cat /target.txt)

COPY --from=planner /app/recipe.json recipe.json

# Build external dependencies for the designated architecture
RUN cargo chef cook --release --target $(cat /target.txt) --recipe-path recipe.json

# Copy source and compile the hello-world release binary
COPY . .
RUN cargo build --release --target $(cat /target.txt) --bin hello-world

# Move the binary to a predictable path independent of architecture name
RUN TARGET_TRIPLE=$(cat /target.txt) && \
    cp /app/target/${TARGET_TRIPLE}/release/hello-world /app/hello-world

# --- Stage 4: Completely Empty Multi-Arch Scratch Runtime ---
FROM scratch

# 1. Accept the initial build argument with a default of 3000
ARG APP_PORT=3000

# 2. Assign it to a runtime Environment Variable so the Rust app can read it
ENV PORT=${APP_PORT}

# 3. Copy the system certificates from the builder stage
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# 4. Copy the statically compiled hello-world binary
COPY --from=builder /app/hello-world /hello-world

# Inform Docker that the container listens on the chosen port at runtime
EXPOSE ${PORT}

# Run the app directly
CMD ["/hello-world"]

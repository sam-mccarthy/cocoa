# cargo-chef using Alpine image of rust as a backend
FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

# Plan out cargo-chef build
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Copy over the recipe
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies + cache
RUN cargo chef cook --release --recipe-path recipe.json
# Build the final binary
COPY . .
RUN cargo build --release bin cocoa

# We don't need Rust anymore, so let's move to pure alpine
FROM alpine AS runtime
WORKDIR /app

# Copy over and run the binary
COPY --from=builder /app/target/release/cocoa /usr/local/bin
ENTRYPOINT ["/usr/local/bin/cocoa"]
# We use the rust-musl-builder image instead of official
# rust in order to support Alpine properly.
FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

# Prepare recipe for building later
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Copy recipe over to builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build and cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Build final binary
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin cocoa

# Finally, we pull Alpine as our runtime.
FROM alpine AS runtime
# Add a new user to avoid running as root.
RUN addgroup -S bot && adduser -S bot -G bot
# Copy over binary. Self-explanatory.
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cocoa /usr/local/bin/
USER bot
CMD ["/usr/local/bin/cocoa"]
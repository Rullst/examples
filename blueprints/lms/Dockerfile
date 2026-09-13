# ==============================================================================
# Rullst LMS Blueprint - Optimized OCI Containerfile
# Powered by cargo-chef for blazing-fast layer caching (< 30s rebuilds)
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Cargo Chef Base
# ------------------------------------------------------------------------------
FROM lukemathwalker/cargo-chef:latest-rust-1.98-bookworm AS chef
WORKDIR /app

# ------------------------------------------------------------------------------
# Stage 2: Recipe Planner (Dependency Skeleton)
# ------------------------------------------------------------------------------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Stage 3: Dependency Cooker (Cached Layer)
# ------------------------------------------------------------------------------
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build the application source code
COPY . .
RUN cargo build --release --bin lms

# ------------------------------------------------------------------------------
# Stage 4: Minimal Production Runtime
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies for SQLite and TLS certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/lms /app/lms

# Copy assets and static resources
COPY static /app/static
COPY Rullst.toml /app/Rullst.toml

# Persistent storage directory for SQLite database
RUN mkdir -p /app/data && chown -R 1000:1000 /app
VOLUME ["/app/data"]

ENV PORT=3000
ENV RUST_LOG="info,rullst=info"
ENV DATABASE_URL="sqlite:///app/data/lms.db?mode=rwc"

EXPOSE 3000

# Run as non-root container user for defense-in-depth security
USER 1000:1000

CMD ["/app/lms"]

# ==============================================================================
# Rullst LMS Blueprint - Optimized OCI Containerfile
# Powered by cargo-chef for blazing-fast layer caching (< 30s rebuilds)
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Cargo Chef Base
# ------------------------------------------------------------------------------
FROM lukemathwalker/cargo-chef:latest-rust-1.98-bookworm AS chef
WORKDIR /app/blueprints/lms

# ------------------------------------------------------------------------------
# Stage 2: Recipe Planner (Dependency Skeleton)
# ------------------------------------------------------------------------------
FROM chef AS planner
COPY . /app
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Stage 3: Dependency Cooker (Cached Layer)
# ------------------------------------------------------------------------------
FROM chef AS builder
COPY --from=planner /app/blueprints/lms/recipe.json recipe.json
# The standalone shared crate is not part of cargo-chef's blueprint recipe.
COPY crates/blueprint-ai /app/crates/blueprint-ai
RUN cargo chef cook --release --recipe-path recipe.json

# Build the application source code
COPY . /app
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
COPY --from=builder /app/blueprints/lms/target/release/lms /app/lms

# Copy assets and static resources
COPY blueprints/lms/static /app/static
COPY blueprints/lms/Rullst.toml /app/Rullst.toml

RUN chown -R 1000:1000 /app

ENV HOST="0.0.0.0"
ENV RULLST_HOST="0.0.0.0"
ENV RULLST_ENV="production"
ENV PORT=3000
ENV RUST_LOG="info,rullst=info"
ENV DATABASE_URL="sqlite:///app/db.sqlite?mode=rwc"
ENV LMS_SHOWCASE_MODE="true"
# Configure a unique APP_KEY via the hosting platform secret store.

EXPOSE 3000

# Run as non-root container user for defense-in-depth security
USER 1000:1000

CMD ["/app/lms"]

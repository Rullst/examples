FROM rust:1.98.1-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM docker.io/library/debian:bookworm-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 rullst \
    && useradd --system --uid 10001 --gid 10001 --home-dir /app --shell /usr/sbin/nologin rullst \
    && install --directory --owner=10001 --group=10001 /app
WORKDIR /app
COPY --from=builder --chown=10001:10001 /app/target/release/saas /app/saas
COPY --chown=10001:10001 static /app/static
COPY --chown=10001:10001 Rullst.toml /app/Rullst.toml
ENV RULLST_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000
USER 10001:10001
EXPOSE 3000
# Staging runs at most one replica and opts into the migration gate explicitly.
# Production should run migrations as a separate one-off deployment job.
CMD ["/bin/sh", "-c", "if [ \"${MIGRATE_ON_STARTUP:-false}\" = \"true\" ]; then /app/saas db:migrate; fi && exec /app/saas"]

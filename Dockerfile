FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/release/blog001 /usr/local/bin/blog001
COPY style /app/style
COPY migrations /app/migrations
COPY start.sh /app/start.sh

RUN chmod +x /app/start.sh \
    && mkdir -p /app/data

ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATABASE_URL=sqlite:/app/data/data.db?mode=rwc

EXPOSE 8080

CMD ["/app/start.sh"]

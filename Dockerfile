# Etapa 1: Compilar el binario en Rust
FROM rust:1-slim AS builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock build.rs* ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

# Etapa 2: Imagen ligera de ejecución (~80 MB)
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/mtg-store-web /app/mtg-store-web
COPY static /app/static

RUN mkdir -p /app/data

ENV PORT=3000
ENV DATABASE_URL=sqlite:/app/data/store_inventory.db
ENV FALLBACK_USD_TO_CLP=950.0

EXPOSE 3000

CMD ["/app/mtg-store-web"]
FROM rust:1.91 AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY schema.sql faq.json ./

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
COPY schema.sql faq.json ./

ENV DATABASE_URL=mysql://faq_user:faq_pass@db:3306/faq_db
CMD ["backend"]

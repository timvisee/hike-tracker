FROM rust:1.87-bookworm as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hike-tracker /usr/local/bin/
COPY templates /app/templates
COPY Rocket.toml /app/

WORKDIR /app

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8888

EXPOSE 8888

CMD ["hike-tracker"]

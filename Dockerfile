FROM rust:1.44.0-stretch as builder

WORKDIR /app

COPY . .

RUN cargo install --path .

FROM debian:buster-slim
WORKDIR /app

COPY --from=builder /app /app

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates

RUN update-ca-certificates

EXPOSE 8001
CMD ["/app/target/release/dontslack"]


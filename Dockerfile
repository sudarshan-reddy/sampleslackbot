FROM rust:1.43.1-stretch as rustbuild

workdir /app

COPY Cargo.toml .
COPY Cargo.lock .
COPY src/ src

RUN cargo build --release


CMD /app/target/release/dontslack 


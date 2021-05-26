FROM rust:buster as builder
RUN apt-get update && apt-get install -y firefox-esr && rm -rf /var/lib/apt/lists/*
RUN mkdir /app
WORKDIR /app/
COPY . /app/
RUN cargo install wasm-pack
RUN cargo build
RUN cargo test
RUN wasm-pack test --headless --firefox

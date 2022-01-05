FROM registry.gitlab.com/asuran-rs/containers/rust-sccache-docker:1.56 as builder
RUN apt-get update && apt-get install -y firefox-esr && rm -rf /var/lib/apt/lists/*
RUN mkdir /app
WORKDIR /app/
COPY . /app/
RUN wget -O wasm.tar.gz https://github.com/rustwasm/wasm-pack/releases/download/v0.10.2/wasm-pack-v0.10.2-x86_64-unknown-linux-musl.tar.gz ; \
    tar -xvf wasm.tar.gz ; \
    rm wasm.tar.gz ; \
    mv wasm*/wasm-pack /usr/bin/
RUN rm /opt/.cargo/config
RUN cargo build
RUN cargo test
RUN wasm-pack test --headless --firefox

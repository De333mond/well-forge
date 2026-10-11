FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        file \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libssl-dev \
        libwebkit2gtk-4.1-dev \
        libxdo-dev \
        pkg-config \
        wget \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install just --locked \
    && rm -rf /usr/local/cargo/registry

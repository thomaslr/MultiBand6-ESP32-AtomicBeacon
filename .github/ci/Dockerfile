FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/cargo/bin:/opt/rustup/toolchains/nightly-2026-04-01-x86_64-unknown-linux-gnu/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    cmake \
    ninja-build \
    python3 \
    python3-pip \
    python3-venv \
    libclang-dev \
    libudev-dev \
    build-essential \
    pkg-config \
    unzip \
    flex \
    bison \
    gperf \
  && rm -rf /var/lib/apt/lists/*

# Install pinned nightly toolchain with rust-src for -Zbuild-std
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain nightly-2026-04-01 && \
    rustup component add rust-src

# Install prebuilt ldproxy & espflash binaries for fast and reproducible builds
RUN mkdir -p /opt/cargo/bin && \
    curl -sSfL -o /tmp/ldproxy.zip https://github.com/esp-rs/embuild/releases/download/ldproxy-v0.3.2/ldproxy-x86_64-unknown-linux-gnu.zip && \
    unzip /tmp/ldproxy.zip -d /opt/cargo/bin && \
    chmod +x /opt/cargo/bin/ldproxy && \
    curl -sSfL -o /tmp/espflash.zip https://github.com/esp-rs/espflash/releases/latest/download/espflash-x86_64-unknown-linux-gnu.zip && \
    unzip /tmp/espflash.zip -d /opt/cargo/bin && \
    chmod +x /opt/cargo/bin/espflash && \
    rm -f /tmp/ldproxy.zip /tmp/espflash.zip

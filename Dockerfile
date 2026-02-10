FROM debian:trixie AS build

WORKDIR /build

RUN apt-get update \
    && DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        ca-certificates \
        rustup \
        build-essential \
        libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml .
ENV RUST_BACKTRACE=full
RUN cargo --version

WORKDIR /app
COPY . .

RUN cargo build --release --bins

FROM debian:trixie AS runtime

RUN apt-get update \
    && DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        ca-certificates \
        libssl3 \
        libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/target/release/repeater-atlas /app/repeater-atlas

ENTRYPOINT ["/app/repeater-atlas"]

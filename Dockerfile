FROM debian:trixie AS chef

WORKDIR /app

RUN apt-get update \
    && DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        ca-certificates \
        rustup \
        build-essential \
        libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml .
RUN cargo --version
RUN cargo install cargo-chef --version 0.1.73

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Copy over the rest of the application
COPY . .

# And build everything
RUN cargo build --release --bins

FROM debian:trixie AS runtime

RUN apt-get update \
    && DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        ca-certificates \
        libssl3 \
        libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/repeater-atlas /app/repeater-atlas

ENTRYPOINT ["/app/repeater-atlas"]

FROM debian:trixie AS deb

RUN apt-get update

FROM deb AS typst-vendor

RUN DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        ca-certificates \
        wget \
        tar \
        xz-utils \
    && rm -rf /var/lib/apt/lists/*

COPY bin/typst /usr/local/bin/typst
RUN typst --version

FROM deb AS chef

WORKDIR /app

RUN DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        build-essential \
        ca-certificates \
        just \
        libpq-dev \
        npm \
        rustup \
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
COPY askama.toml ./
COPY src src
COPY templates templates
COPY static static
COPY package.json package-lock.json Justfile ./

# And build everything
RUN just assets-ci
RUN cargo build --release --bins

FROM deb AS runtime

RUN DEBIAN_FRONTEND="noninteractive" apt-get install --yes --no-install-recommends \
        ca-certificates \
        libpq5 \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/static static
COPY --from=builder /app/target/release/repeater-atlas bin/repeater-atlas
COPY --from=typst-vendor /usr/local/bin/typst /usr/local/bin/typst
# Just to have an index over everything in the image
RUN find

ENTRYPOINT ["/app/bin/repeater-atlas"]

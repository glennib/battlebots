FROM rust:1.84.1 AS builder-base

# Install cargo-binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
    | bash

# Install protobuf compiler
RUN apt-get update && apt-get install -y --no-install-recommends libprotobuf-dev protobuf-compiler 

FROM builder-base AS chef

RUN cargo binstall -y cargo-chef


FROM chef AS planner

COPY Cargo.lock Cargo.toml build.rs /code/
COPY proto /code/proto
COPY src /code/src
WORKDIR /code
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder

COPY --from=planner /code/recipe.json /code/recipe.json
WORKDIR /code
RUN cargo chef cook \
    --release \
    --recipe-path \
    recipe.json
COPY Cargo.lock Cargo.toml build.rs /code/
COPY proto /code/proto
COPY src /code/src
RUN cargo build --release


FROM builder AS tini
# Download init system
RUN mkdir /downloads
ARG TINI_VERSION="v0.19.0"
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini -o /downloads/tini


FROM ubuntu:24.04 AS certs
# Download certificates
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates


FROM ubuntu:24.04 AS runtime

COPY --from=certs /etc/ssl/certs/* /etc/ssl/certs/

COPY --from=tini /downloads/tini /usr/local/bin/tini
RUN chmod +x /usr/local/bin/tini
ENTRYPOINT ["tini", "--"]

# non-root user:
RUN useradd user
RUN mkdir /app && chown user /app
USER user
ENV PATH="/app:${PATH}"
WORKDIR /app

COPY --from=builder /code/target/release/battlebots /app/battlebots
CMD ["battlebots"]

FROM rust:1.80-bookworm AS builder
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked -p ayame-spell

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /source/target/release/ayame-spell /usr/local/bin/ayame-spell
USER 65532:65532
ENTRYPOINT ["ayame-spell"]
CMD ["--help"]

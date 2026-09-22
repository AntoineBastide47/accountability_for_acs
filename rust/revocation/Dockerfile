# From rust/revocation/:
#   docker build -t revocation-bench-rs -f Dockerfile .
#
# Server profile (8 CPUs, 16 GB):
#   docker run --rm --cpus=8 --memory=16g --memory-swap=16g \
#     -v "$(pwd)/results:/bench/results" \
#     revocation-bench-rs \
#     bash -lc 'cargo run --release --bin run_experiments'
#
FROM rust:1-bookworm

WORKDIR /bench

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash ca-certificates python3 python3-matplotlib python3-numpy \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY lib/ ./lib/
COPY direct-decrypt/ ./direct-decrypt/
COPY link-decrypt/ ./link-decrypt/
COPY mpc/ ./mpc/
COPY scripts/ ./scripts/

RUN cargo build --release --locked

RUN ./target/release/verify_link

CMD ["bash"]

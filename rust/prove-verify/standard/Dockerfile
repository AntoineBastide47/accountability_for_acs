# Build: cd rust/prove-verify/standard && docker build -t standard-bench-rs -f Dockerfile .
#
#   docker run --rm --cpus=2 --memory=4g --memory-swap=4g standard-bench-rs \
#     bash -lc 'cd /bench && ./target/release/bench_prove_verify >/tmp/log 2>&1 \
#       && cat prove-verify/artifacts_bench_prove_verify/summary_latest.json'
#
# Binaries: bench_prove_verify | bench_prove_verify_no_cft
#         | bench_prove_verify_revocation | bench_merkle_vs_flat
#         | bench_communication_size
#
# The Longfellow C++ tree is a vendored dependency and is not duplicated into
# the Rust stack, so it is copied from the original checkout. Build this image
# from rust/prove-verify/standard with the repository root as context:
#
#   docker build -t standard-bench-rs \
#     -f rust/prove-verify/standard/Dockerfile .
#
FROM rust:1-bookworm

WORKDIR /bench

ENV DEBIAN_FRONTEND=noninteractive

# C++ toolchain + Google Test / Benchmark (same stack as the Longfellow README).
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash ca-certificates \
    clang cmake git make \
    libssl-dev libzstd-dev libgtest-dev libbenchmark-dev zlib1g-dev \
 && rm -rf /var/lib/apt/lists/*

ENV CXX=clang++

ENV LONGFELLOW_BUILD_DIR=/bench/clang-build-release
ENV LONGFELLOW_CRED_BENCH_BIN=${LONGFELLOW_BUILD_DIR}/circuits/tests/ec/prove_verify_test
ENV LONGFELLOW_NO_CFT_BENCH_BIN=${LONGFELLOW_BUILD_DIR}/circuits/tests/ec/prove_verify_no_cft_test
ENV LONGFELLOW_ATTR_BENCH_BIN=${LONGFELLOW_BUILD_DIR}/circuits/tests/ec/attr_commitment_experiment_test
ENV LONGFELLOW_REVOC_BENCH_BIN=${LONGFELLOW_BUILD_DIR}/circuits/tests/ec/prove_verify_revocation_test

# Build context: the repository root.
COPY prove-verify/standard/longfellow-zk/lib/ /bench/lib/
COPY rust/prove-verify/standard/Cargo.toml rust/prove-verify/standard/Cargo.lock /bench/
COPY rust/prove-verify/standard/scripts/ /bench/scripts/
COPY rust/prove-verify/standard/prove-verify/ /bench/prove-verify/
COPY rust/prove-verify/standard/prove-verify-no-cft/ /bench/prove-verify-no-cft/
COPY rust/prove-verify/standard/prove-verify-revocation/ /bench/prove-verify-revocation/
COPY rust/prove-verify/standard/merkle-vs-flat/ /bench/merkle-vs-flat/
COPY rust/prove-verify/standard/communication-costs/ /bench/communication-costs/

RUN cmake -S /bench/lib -B "${LONGFELLOW_BUILD_DIR}" \
      -DCMAKE_BUILD_TYPE=Release \
  && cmake --build "${LONGFELLOW_BUILD_DIR}" \
      --target prove_verify_test prove_verify_no_cft_test attr_commitment_experiment_test prove_verify_revocation_test -j"$(nproc)" \
  && test -x "${LONGFELLOW_CRED_BENCH_BIN}" \
  && test -x "${LONGFELLOW_NO_CFT_BENCH_BIN}" \
  && test -x "${LONGFELLOW_ATTR_BENCH_BIN}" \
  && test -x "${LONGFELLOW_REVOC_BENCH_BIN}"

RUN cargo build --release --locked && cargo test --release --locked

ENV PATH="/bench/target/release:${PATH}"

CMD ["bash"]

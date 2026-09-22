# Build: cd rust/prove-verify/zk-friendly && docker build -t zk-friendly-bench-rs -f Dockerfile .
#
#   docker run --rm --cpus=2 --memory=4g --memory-swap=4g zk-friendly-bench-rs \
#     bash -lc 'cd /bench && ./target/release/bench_prove_verify >/tmp/log 2>&1 \
#       && cat prove-verify/artifacts_bench_prove_verify/summary_latest.json'
#
# Binaries: bench_prove_verify | bench_prove_verify_no_cft
#         | bench_prove_verify_revocation | bench_merkle_vs_flat
#         | bench_communication_size
#
# On Apple Silicon, Docker builds linux/arm64 natively and rapidsnark is compiled
# with ARM assembly (same class of optimizations as a phone wallet prover).
# On amd64, the x86_64 host build is used (also USE_ASM=ON by default).
FROM rust:1-bullseye

WORKDIR /bench

# System deps for circom + rapidsnark (see iden3/rapidsnark README)
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash ca-certificates curl git \
    build-essential cmake pkg-config m4 nasm \
    libgmp-dev libsodium-dev nlohmann-json3-dev \
    nodejs npm python3 \
 && rm -rf /var/lib/apt/lists/*

# snarkjs provides the Groth16 trusted setup CLI. Proving and verifying are
# native: rapidsnark proves, this crate verifies.
RUN npm install -g snarkjs@0.7.5

# Install circom (build from source)
RUN git clone --depth 1 https://github.com/iden3/circom.git /tmp/circom \
 && cd /tmp/circom \
 && cargo build --release \
 && install -m 0755 target/release/circom /usr/local/bin/circom \
 && rm -rf /tmp/circom

# Build rapidsnark (provides `prover`). Architecture-specific:
#   arm64  → ./build_gmp.sh aarch64 && make host_arm64  (ARM asm, USE_ASM=ON)
#   amd64  → ./build_gmp.sh host     && make host       (x86 asm, USE_ASM=ON)
# Fallback on unknown arch: host_noasm (portable C++, slower).
ARG TARGETARCH
RUN set -eux; \
  git clone --depth 1 https://github.com/iden3/rapidsnark.git /tmp/rapidsnark; \
  cd /tmp/rapidsnark; \
  git submodule update --init --recursive; \
  ARCH="${TARGETARCH:-$(uname -m)}"; \
  case "$ARCH" in \
    arm64|aarch64) \
      ./build_gmp.sh aarch64; \
      make host_arm64; \
      install -m 0755 package_arm64/bin/prover /usr/local/bin/prover; \
      ;; \
    amd64|x86_64) \
      ./build_gmp.sh host; \
      make host; \
      install -m 0755 package/bin/prover /usr/local/bin/prover; \
      ;; \
    *) \
      ./build_gmp.sh host_noasm; \
      make host_noasm; \
      install -m 0755 package_noasm/bin/prover /usr/local/bin/prover; \
      ;; \
  esac; \
  rm -rf /tmp/rapidsnark

# Build context: rust/prove-verify/zk-friendly/
COPY Cargo.toml Cargo.lock ./
COPY lib/ ./lib/
COPY scripts/ ./scripts/
# (If you don't have this file in your checkout, run scripts/fetch_ptau.sh first.)
COPY powersOfTau/ ./powersOfTau/
COPY prove-verify/ ./prove-verify/
COPY prove-verify-no-cft/ ./prove-verify-no-cft/
COPY prove-verify-revocation/ ./prove-verify-revocation/
COPY merkle-vs-flat/ ./merkle-vs-flat/
COPY communication-costs/ ./communication-costs/

RUN bash scripts/fetch_circomlib.sh \
 && cargo build --release --locked \
 && cargo test --release --locked \
 && test -f powersOfTau/powersOfTau28_hez_final_19.ptau \
 && command -v circom \
 && command -v prover \
 && command -v snarkjs

ENV PATH="/bench/target/release:${PATH}"

CMD ["bash"]

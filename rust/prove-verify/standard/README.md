# standard (Longfellow), Rust

Google Benchmark timings for the age-check presentation (P-256 + SHA-256).
Rust port of `../../../prove-verify/standard/`.

The C++ benchmark binaries are unchanged. Longfellow itself is a vendored
third-party tree (Apache 2.0, Google LLC) and is **not** duplicated here: the
harness looks for it at `$LONGFELLOW_ROOT`, then `./longfellow-zk`, then the
original checkout at `../../../prove-verify/standard/longfellow-zk`.

## Setup

```bash
cd prove-verify/standard/longfellow-zk        # the original, vendored tree
cmake -S lib -B clang-build-release -DCMAKE_BUILD_TYPE=Release
cmake --build clang-build-release --target \
  prove_verify_test prove_verify_no_cft_test \
  attr_commitment_experiment_test prove_verify_revocation_test -j 8

cd ../../../rust/prove-verify/standard
cargo build --release
```

## Benchmarks

```bash
cargo run --release --bin bench_prove_verify
cargo run --release --bin bench_prove_verify_no_cft
cargo run --release --bin bench_prove_verify_revocation
cargo run --release --bin bench_merkle_vs_flat
cargo run --release --bin bench_communication_size
```

Every binary takes `--help`. Each option also reads its environment variable, so
the Node stack's invocations still work.

| Env | Flag | Default | Meaning |
|-----|------|---------|---------|
| `BENCH_N` | `--repetitions` / `--n` | `10` | Outer samples |
| `BENCH_ITERATIONS` | `--iterations` | `1` | Inner iterations; `auto`/`0` = adaptive |
| `BENCH_MIN_TIME` | `--min_time` | `0.05s` | Google Benchmark `--benchmark_min_time` |
| `BENCH_FILTER` | `--filter` | per benchmark | Google Benchmark filter regex |
| `BENCH_METRIC` | `--metric` | `both` | `--verbose` timing column |
| `BENCH_WARMUP` | — | on | `0`/`false`/`no` skips the discarded repetition |
| `ARTIFACTS_DIR` | `--out-dir` | per benchmark | Summary directory |
| `REVOC_LOG2_LIST` | `--revoc-log2` | `12,16,20,24` | Revocation population scales |
| `TOTAL_ATTRS` | `--total-attrs` / `--attr` | `8,16,32,64` | merkle-vs-flat: credential sizes \(n\) |
| `USED_ATTRS` | `--used-attrs` / `--used-attr` | `1,2,4,8,16` | merkle-vs-flat: disclosed counts \(k\) (skipped when \(k>n\)) |
| `LONGFELLOW_ROOT` | — | search | The Longfellow C++ tree |
| `LONGFELLOW_*_BENCH_BIN` | `--bin` | build path | Benchmark binary |
| `STANDARD_ROOT` | — | crate dir | Overrides where benchmark folders are looked up |

| Folder | Role |
|--------|------|
| `prove-verify/` | age-check + CFT |
| `prove-verify-no-cft/` | no-CFT baseline |
| `prove-verify-revocation/` | + packed status-list |
| `merkle-vs-flat/` | attribute-commitment sweep |
| `communication-costs/` | wire-size measurement (C++ sources + build scripts) |
| `scripts/` | the crate library (shared helpers) + clean scripts |

## Docker

The image needs both the Rust stack and the vendored Longfellow tree, so it is
built from the repository root:

```bash
docker build -t standard-bench-rs -f rust/prove-verify/standard/Dockerfile .
```

| Binary | `cat` path inside the container |
|--------|----------------------------------|
| `bench_prove_verify` | `prove-verify/artifacts_bench_prove_verify/summary_latest.json` |
| `bench_prove_verify_no_cft` | `prove-verify-no-cft/artifacts_bench_prove_verify_no_cft/summary_latest.json` |
| `bench_prove_verify_revocation` | `prove-verify-revocation/artifacts_bench_prove_verify_revocation/summary_latest.json` |
| `bench_merkle_vs_flat` | `merkle-vs-flat/artifacts_bench_merkle_vs_flat/summary_latest.json` |
| `bench_communication_size` | `communication-costs/artifacts_measure_communication_size/summary_latest.json` |

### Example with prove-verify

```bash
# mobile-like
docker run --rm --cpus=2 --memory=4g --memory-swap=4g standard-bench-rs \
  bash -lc 'cd /bench && ./target/release/bench_prove_verify >/tmp/log 2>&1 \
    && cat prove-verify/artifacts_bench_prove_verify/summary_latest.json' \
  > standard_mobile_prove_verify_summary.json
```

```bash
# server-like
docker run --rm --cpus=8 --memory=16g --memory-swap=16g standard-bench-rs \
  bash -lc 'cd /bench && ./target/release/bench_prove_verify >/tmp/log 2>&1 \
    && cat prove-verify/artifacts_bench_prove_verify/summary_latest.json' \
  > standard_server_prove_verify_summary.json
```

## Clean

```bash
bash scripts/clean_results.sh  # delete local artifact/summary folders (asks for confirmation)
bash scripts/clean_all.sh      # same + target/ and clang-build-release
```

## Notes on the port

- **Nothing that is measured changed.** This stack only launches the C++
  binaries and reads their `--benchmark_out` JSON, so the reported numbers are
  directly comparable with the Node ones.
- **One driver, not two.** `bench_prove_verify` and `bench_prove_verify_no_cft`
  differed in three strings; `scripts/driver.rs` holds the shared body and each
  binary supplies its own `Bench` description.
- **`clap` replaces the hand-written parsers** and the hand-maintained usage
  text that each JavaScript driver carried.
- **`--out-dir` resolves against the working directory**, and `ARTIFACTS_DIR`
  against the stack root, exactly as before.

## License

Benchmark harness and circuit wrappers here are under the workspace **MIT**
license (`../../../LICENSE`). The vendored `longfellow-zk/` tree, which this
stack builds against but does not contain, remains **Apache 2.0** (Google LLC).

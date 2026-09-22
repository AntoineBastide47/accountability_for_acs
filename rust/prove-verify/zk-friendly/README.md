# zk-friendly (Circom / Groth16), Rust

Age-check presentation benchmarks: witness + rapidsnark prove + native Groth16
verify. Rust port of `../../../prove-verify/zk-friendly/`.

## Setup

Needs `circom`, rapidsnark `prover` and the `snarkjs` CLI on `PATH`, plus
`circomlib`'s circuit sources and the powers-of-tau file.

```bash
cd rust/prove-verify/zk-friendly
bash scripts/fetch_ptau.sh        # powersOfTau/powersOfTau28_hez_final_19.ptau
bash scripts/fetch_circomlib.sh   # circom-libs/circomlib
cargo build --release
```

`snarkjs` is used only for the one-off Groth16 trusted setup
(`snarkjs groth16 setup`, `snarkjs zkey export verificationkey`). Proving is
rapidsnark; verification is this crate.

## Benchmarks

```bash
cargo run --release --bin bench_prove_verify             # age-check + CFT
cargo run --release --bin bench_prove_verify_no_cft      # same, no CFT
cargo run --release --bin bench_prove_verify_revocation  # + non-revocation claim (2^12…2^24)
cargo run --release --bin bench_merkle_vs_flat           # attribute-commitment sweep
cargo run --release --bin bench_communication_size       # wire size
```

Every binary takes `--help`. Each option also reads its environment variable, so
the Node stack's invocations still work.

| Env | Flag | Default | Meaning |
|-----|------|---------|---------|
| `BENCH_N` | `--n` | `10` | Measured iterations |
| `BENCH_WARMUP` | `--warmup` | `1` | Discarded iterations |
| `BENCH_VERIFY_WARMUP` | `--verify-warmup` | `0` | Extra verify calls on the warm-up iteration |
| `REVOC_LOG2_LIST` | `--revoc-log2` | `12,16,20,24` | Revocation population scales |
| `REVOC_BITS_PER_LEAF` | `--bits-per-leaf` | `253` | Status-list bits per leaf |
| `REVOC_SLOT` | `--revoc-slot` | `14` | Attribute slot holding the revocation index |
| `TOTAL_ATTRS` | `--totals` | `8,16,32,64` | merkle-vs-flat: credential sizes \(n\) |
| `USED_ATTRS` | `--used` | `1,2,4,8,16` | merkle-vs-flat: disclosed counts \(k\) (skipped when \(k>n\)) |
| `KEEP_ARTIFACTS` | `--keep-artifacts` | off | Keep per-iteration files |
| `CIRCOM_BIN` / `CIRCOM` | — | `circom` | circom binary |
| `RAPIDSNARK_BIN` | — | `prover` | rapidsnark binary |
| `SNARKJS_BIN` | — | `snarkjs` | snarkjs CLI |
| `CIRCOM_LIB_PATH` | — | `circom-libs` | circom `-l` include root |
| `ZK_FRIENDLY_ROOT` | — | crate dir | Overrides where benchmark folders are looked up |

| Folder | Role |
|--------|------|
| `prove-verify/` | age-check + CFT |
| `prove-verify-no-cft/` | no-CFT baseline |
| `prove-verify-revocation/` | + packed status-list |
| `merkle-vs-flat/` | attribute-commitment sweep |
| `communication-costs/` | wire-size measurement |
| `lib/` | shared crypto, toolchain driver, Groth16 verifier |
| `scripts/` | `clean.sh`, `fetch_ptau.sh`, `fetch_circomlib.sh` |

## Docker

```bash
cd rust/prove-verify/zk-friendly
bash scripts/fetch_ptau.sh   # once, if powersOfTau/ is empty
docker build -t zk-friendly-bench-rs -f Dockerfile .
```

```bash
# mobile-like
docker run --rm --cpus=2 --memory=4g --memory-swap=4g zk-friendly-bench-rs \
  bash -lc 'cd /bench && ./target/release/bench_prove_verify >/tmp/log 2>&1 \
    && cat prove-verify/artifacts_bench_prove_verify/summary_latest.json' \
  > zkfriendly_mobile_prove_verify_summary.json
```

```bash
# server-like
docker run --rm --cpus=8 --memory=16g --memory-swap=16g zk-friendly-bench-rs \
  bash -lc 'cd /bench && ./target/release/bench_prove_verify >/tmp/log 2>&1 \
    && cat prove-verify/artifacts_bench_prove_verify/summary_latest.json' \
  > zkfriendly_server_prove_verify_summary.json
```

## Clean

```bash
bash scripts/clean.sh          # drop proofs/witnesses + generated/ (keep summary_*.json)
bash scripts/clean_results.sh  # delete artifact folders (summaries too)
bash scripts/clean.sh --all    # the above + target/
```

## Notes on the port

- **The circuits are unchanged.** `prove_verify.circom` and friends are copied
  verbatim, and `prove-verify-revocation/lib/circom_codegen.rs` reproduces the
  generated `prove_verify_revocation_l*.circom` byte for byte — a unit test
  compares its output against the checked-in files.
- **Verification moved in-process.** `snarkjs.groth16.verify` is replaced by
  `lib/groth16.rs`, which reads the same `vkey-*.json` / `proof.json` /
  `public.json` encodings and evaluates the pairing check with `arkworks`. This
  is the one measurement that is not comparable one-to-one with the Node
  numbers: the reported `verify` time is now a Rust verifier's, not snarkjs'.
  Witness generation and proving are unchanged external binaries, so those
  numbers stay comparable.
- **The verification key is parsed once**, and `e(α, β)` with it; the timed
  region is the pairing check alone, as it was in the JS.
- **`circomlib` is a circuit dependency**, not a JavaScript one. It is fetched
  into `circom-libs/` instead of `node_modules/`.
- **Commands are spawned with an argument vector**, not through a shell, so the
  JS `shellQuote` helper has no counterpart.
- **Merkle proofs share one tree.** The JS rebuilt every level for each proof;
  `lib/poseidon_merkle.rs` builds the levels once, which matters for the
  merkle-vs-flat sweep at `k` disclosed attributes.
- **Node-only knobs are gone**: `BENCH_GC_BEFORE_VERIFY` and the `gcAvailable`
  summary field described V8's garbage collector.

## License

Code in this folder is under the workspace **MIT** license (`../../../LICENSE`).

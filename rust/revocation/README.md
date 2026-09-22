# Anonymity revocation benchmarks (Rust)

Baby Jubjub CFT revocation throughput. C4 binding matches zk-friendly
`prove_verify.circom`. Rust port of `../../revocation/`.

| Dir | Experiment |
|-----|------------|
| `direct-decrypt/` | Decrypt every CFT (police + judge + NGO) |
| `link-decrypt/` | Link to PIDs, keep ≥10% recurring, decrypt those |
| `mpc/` | PET + MP-SPDZ predicate matrix (needs `MP_SPDZ_PATH`) |
| `lib/` | Shared crypto / experiment harness (the crate library) |
| `results/` | CSVs; PDFs under `results/plots/` |

## Setup

```bash
cd rust/revocation
cargo build --release
./target/release/verify_link      # optional: protocol sanity check
```

## Benchmarks

```bash
cargo run --release --bin bench_direct_decrypt
cargo run --release --bin bench_link_decrypt
cargo run --release --bin run_experiments    # both (direct then link)

# MPC (requires local MP-SPDZ)
MP_SPDZ_PATH=/path/to/mp-spdz bash mpc/run_sweep.sh
python3 mpc/summarize.py mpc/sweep_*/results.csv

python3 results/plots/plot_experiment_figures.py   # PDFs → results/plots/
cargo run --release --bin regenerate_from_runs -- direct-decrypt
```

| Env | Default | Meaning |
|-----|---------|---------|
| `EXPERIMENT_SIZES` | `10,20,50,100,500,1000,2000,4000,8000` | CFT set sizes (direct/link) |
| `EXPERIMENT_RUNS` | `10` | Runs per (size × recurring%) cell |
| `EXPERIMENT_RECURRING_PCTS` | per benchmark | Recurring rates, in percent |
| `NUMS` | `10 20 50 100 200 500 1000` | CFT sizes for MPC sweep |
| `ITERATIONS` | `10` | Runs per size (MPC) |
| `TAU` | `2` | MPC predicate threshold |
| `MP_SPDZ_PATH` | — | Path to MP-SPDZ install (**required** for MPC) |
| `REVOCATION_ROOT` | crate dir | Overrides where `results/` and `mpc/` are looked up |

Short run:

```bash
EXPERIMENT_SIZES=100,500 EXPERIMENT_RUNS=2 cargo run --release --bin bench_direct_decrypt
```

Outputs: `results/{direct,link,mpc}-decrypt_{runs,summary,fit}.csv` and
`results/plots/*_total_time.pdf`.

## Docker

```bash
cd rust/revocation
docker build -t revocation-bench-rs -f Dockerfile .
```

```bash
# server-like; bind-mount results/
mkdir -p results
docker run --rm --cpus=8 --memory=16g --memory-swap=16g \
  -v "$(pwd)/results:/bench/results" \
  revocation-bench-rs \
  bash -lc 'cargo run --release --bin run_experiments'
```

## Notes on the port

- **Curve.** `lib/babyjub.rs` declares Baby Jubjub for `arkworks` in circomlib's
  parameterisation (`a = 168700`, `d = 168696`, generator `Base8`), so point
  coordinates are byte-for-byte the ones `circomlibjs` produced. Unit tests pin
  this against reference vectors from `circomlibjs@0.1.7`.
- **Scalars.** `crypto_common.js` (`mod`, `modInv`, `randomScalarMod`) has no
  counterpart: the scalar field type carries reduction and inversion. Random
  scalars are now rejection-sampled rather than `random_bytes % order`, so the
  slight modulo bias is gone.
- **EdDSA.** The JS `signC4` already signed with a raw random scalar under the
  circomlib cofactor convention (`S·Base8 == R8 + 8·hm·A`); `lib/eddsa.rs` is a
  direct port, and the circuits accept the signatures unchanged.
- **MP-SPDZ.** `MP_SPDZ_PATH` is now required; the JS fell back to a hard-coded
  developer path.
- **Memory.** The MPC pairwise matrices are stored as one upper triangle instead
  of a mirrored `n × n` matrix, and the fully decrypted `C1` differences are
  folded into `M'` in place. The same group operations are performed.

## License

Code in this folder is under the workspace **MIT** license (`../../LICENSE`).

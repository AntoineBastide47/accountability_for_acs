# Rust port

A Rust duplicate of this repository's benchmark code. The layout mirrors the
JavaScript tree one directory at a time, so `rust/revocation/link-decrypt/`
holds what `revocation/link-decrypt/` holds.

```
rust/
├── revocation/                 crate `revocation-bench`
│   ├── lib/                    the crate library
│   ├── direct-decrypt/  link-decrypt/  mpc/  scripts/  results/
├── prove-verify/
│   ├── zk-friendly/            crate `zk-friendly-bench`
│   │   ├── lib/                the crate library
│   │   ├── prove-verify/  prove-verify-no-cft/  prove-verify-revocation/
│   │   ├── merkle-vs-flat/  communication-costs/  scripts/
│   └── standard/               crate `standard-bench`
│       ├── scripts/            the crate library + clean scripts
│       ├── prove-verify/  prove-verify-no-cft/  prove-verify-revocation/
│       └── merkle-vs-flat/  communication-costs/
└── prove-verify/results/       pre-recorded summaries (copied)
```

Each stack is its own crate with its own `Cargo.lock`, mirroring the three
independent `package.json` files, so each can still be built and Dockerized on
its own. Every binary's source stays in the directory its JavaScript
counterpart occupied, via explicit `[[bin]] path` entries.

```bash
cd rust/revocation              && cargo build --release && cargo test --release
cd rust/prove-verify/zk-friendly && cargo build --release && cargo test --release
cd rust/prove-verify/standard    && cargo build --release && cargo test --release
```

## What was migrated, and what was not

**User-written code (migrated).** All 32 JavaScript files across the three npm
packages — the CFT revocation protocol, the credential model, the experiment
harnesses, the CSV and JSON reporting, the MP-SPDZ and Circom toolchain drivers,
and the Google Benchmark report parsing.

**Third-party dependencies (not migrated).** These are inputs to the
benchmarks, not part of the codebase under test:

| Dependency | Status |
|---|---|
| `prove-verify/standard/longfellow-zk/` (Apache 2.0, Google LLC) | Vendored C++ tree, built and measured as-is. Not duplicated; `standard-bench` resolves it in the original checkout. |
| `circom`, rapidsnark `prover`, `snarkjs` CLI | External binaries, invoked as before |
| `circomlib` circuits | Circuit sources, fetched by `scripts/fetch_circomlib.sh` instead of npm |
| MP-SPDZ | External, invoked as before |
| `powersOfTau28_hez_final_19.ptau` | Downloaded by `scripts/fetch_ptau.sh` |
| `circomlibjs`, `@noble/curves`, `snarkjs` (library), `big-integer` | Replaced by Rust crates — see below |

**Not JavaScript, copied unchanged.** The `.circom` circuits, the `.mpc`
MP-SPDZ program, the C++ proof-size measurement sources, the matplotlib plot
scripts and `mpc/summarize.py`, and the recorded result CSVs and JSON. The
shell scripts were carried over with their Node-specific paths updated.

## Renamed modules

Most files kept their name. These were renamed because the old name referred to
a JavaScript library or to a distinction the type system now carries:

| JavaScript | Rust |
|---|---|
| `lib/crypto_babyjub.js`, `lib/babyjub_noble.js` | `lib/babyjub.rs` |
| `lib/crypto_common.js` | `lib/babyjub.rs` (field arithmetic), `lib/hash.rs` (SHA-256) |
| `lib/poseidon_cjs.js` | `lib/poseidon.rs` |
| `lib/c4_binding.js` | `lib/c4.rs` |
| `lib/cft_bench_lib.js` | `lib/cft.rs`, `lib/stats.rs` |
| `lib/run_experiment.js` | `lib/experiment.rs` |
| `scripts/bench_gbench_common.js` | `scripts/gbench.rs`, `scripts/cli.rs`, `scripts/runner.rs`, `scripts/driver.rs` |

`crypto_common.js` mostly disappears: `mod`, `modInv` and `randomScalarMod`
were doing by hand what the field and scalar types do.

## Replacing the JavaScript crypto stack

| JavaScript | Rust |
|---|---|
| `circomlibjs` Baby Jubjub | `ark-ec` / `ark-ed-on-bn254`, with circomlib's parameters declared in each stack's `lib/babyjub.rs` |
| `circomlibjs` Poseidon | `light-poseidon` (`new_circom`) |
| `circomlibjs` EdDSA-Poseidon | `lib/eddsa.rs` |
| `@noble/curves` babyjubjub | the same arkworks curve |
| `snarkjs.groth16.verify` | `zk-friendly/lib/groth16.rs`, over `ark-bn254` |
| `big-integer` | the field and scalar types |

Unit tests pin the curve and Poseidon against reference vectors produced with
`circomlibjs@0.1.7`, and pin the generated `prove_verify_revocation_l*.circom`
files against the ones checked into the JavaScript tree.

## Comparability of the numbers

- `revocation/` measures Rust code, and is meant to be faster than the Node
  original — that is the point of the port.
- `prove-verify/standard/` only launches the unchanged C++ binaries and reads
  their JSON, so its numbers are directly comparable.
- `prove-verify/zk-friendly/` still shells out to `circom` and rapidsnark, so
  `witness` and `prove` stay comparable. `verify` is now a Rust Groth16
  verifier rather than snarkjs, so that one column measures a different
  implementation by construction.

Two Node-only knobs disappeared with the runtime: `BENCH_GC_BEFORE_VERIFY` and
the `gcAvailable` summary field both described V8's garbage collector.

## License

Original code here is under the workspace **MIT** license (`../LICENSE`), like
the JavaScript it was ported from.

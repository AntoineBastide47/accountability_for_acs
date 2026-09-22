# JavaScript sources

The user-written JavaScript in this repository: **32 files, 6 547 lines**, and
what each became in the Rust port.

Everything under `prove-verify/standard/longfellow-zk/` is excluded. That
vendored Google tree (Apache 2.0) holds 84 further `.js` files, all belonging to
its Hugo documentation site, and is a dependency rather than code under test.

To regenerate this list:

```bash
git ls-files '*.js' | grep -v '^prove-verify/standard/longfellow-zk/' | xargs wc -l
```

## `revocation/` — 15 files, 1 778 lines

| File                                     | Lines | Role                                         | Rust counterpart                         |
|------------------------------------------|------:|----------------------------------------------|------------------------------------------|
| `lib/crypto_babyjub.js`                  |    10 | Baby Jubjub subgroup order                   | `lib/babyjub.rs`                         |
| `lib/babyjub_noble.js`                   |    40 | `@noble/curves` wrapper, `Base8` check       | `lib/babyjub.rs`                         |
| `lib/crypto_common.js`                   |    46 | `mod`, `modInv`, `randomScalarMod`           | `lib/babyjub.rs` (the scalar type)       |
| `lib/poseidon_cjs.js`                    |    50 | `circomlibjs` Poseidon loader                | `lib/poseidon.rs`                        |
| `lib/c4_binding.js`                      |    85 | C4 tag sign and verify                       | `lib/c4.rs`, `lib/eddsa.rs`              |
| `lib/cft_bench_lib.js`                   |   297 | CFT batches, direct/link decrypt, OLS fit    | `lib/cft.rs`, `lib/stats.rs`             |
| `lib/run_experiment.js`                  |   289 | Grid driver, CSV writing                     | `lib/experiment.rs`, `lib/csv.rs`        |
| `lib/run_experiments.js`                 |    36 | Runs both benchmarks                         | `lib/run_experiments.rs`                 |
| `direct-decrypt/bench_direct_decrypt.js` |     9 | Entry point                                  | `direct-decrypt/bench_direct_decrypt.rs` |
| `link-decrypt/bench_link_decrypt.js`     |     9 | Entry point                                  | `link-decrypt/bench_link_decrypt.rs`     |
| `link-decrypt/verify_link.js`            |    54 | Protocol sanity check                        | `link-decrypt/verify_link.rs`            |
| `mpc/bench_mpc.js`                       |    15 | Entry point for the sweep                    | `mpc/bench_mpc.rs`                       |
| `mpc/mpc_runner.js`                      |   151 | MP-SPDZ compile/run, stderr parsing          | `lib/mpc_runner.rs`                      |
| `mpc/pet_mpc.js`                         |   396 | PET phase, predicate matrix, integrity check | `mpc/pet_mpc.rs`                         |
| `scripts/regenerate_from_runs.js`        |   291 | Rebuilds summary and fit CSVs                | `scripts/regenerate_from_runs.rs`        |

## `prove-verify/zk-friendly/` — 11 files, 2 813 lines

| File                                                       | Lines | Role                                                 | Rust counterpart                                           |
|------------------------------------------------------------|------:|------------------------------------------------------|------------------------------------------------------------|
| `lib/crypto_babyjub.js`                                    |    10 | Subgroup order (duplicate of the above)              | `lib/babyjub.rs`                                           |
| `lib/crypto_common.js`                                     |    52 | The above plus `sha256Utf8ToField`                   | `lib/babyjub.rs`, `lib/hash.rs`                            |
| `lib/poseidon_merkle.js`                                   |    69 | Poseidon leaves, root, proofs                        | `lib/poseidon_merkle.rs`                                   |
| `lib/zk_common.js`                                         |   447 | circom / snarkjs / rapidsnark driver, macOS patching | `lib/zk_common.rs`                                         |
| `prove-verify/bench_prove_verify.js`                       |   423 | Credential model plus the CFT benchmark              | `prove-verify/bench_prove_verify.rs`, `lib/credential.rs`  |
| `prove-verify-no-cft/bench_prove_verify_no_cft.js`         |   416 | No-CFT baseline                                      | `prove-verify-no-cft/bench_prove_verify_no_cft.rs`         |
| `prove-verify-revocation/bench_prove_verify_revocation.js` |   317 | Population scale sweep                               | `prove-verify-revocation/bench_prove_verify_revocation.rs` |
| `prove-verify-revocation/lib/circom_codegen.js`            |   185 | Per-scale circuit generator                          | `prove-verify-revocation/lib/circom_codegen.rs`            |
| `prove-verify-revocation/lib/revocation_tree.js`           |    90 | Packed zero-leaf status-list tree                    | `prove-verify-revocation/lib/revocation_tree.rs`           |
| `merkle-vs-flat/bench_merkle_vs_flat.js`                   |   572 | Commitment sweep and its circuit generators          | `merkle-vs-flat/bench_merkle_vs_flat.rs`                   |
| `communication-costs/bench_communication_size.js`          |   232 | Wire-size report                                     | `communication-costs/bench_communication_size.rs`          |

The in-process `snarkjs.groth16.verify` call these benchmarks made has no file
of its own; it is replaced by `lib/groth16.rs`.

## `prove-verify/standard/` — 6 files, 1 956 lines

| File                                                       | Lines | Role                          | Rust counterpart                                                                |
|------------------------------------------------------------|------:|-------------------------------|---------------------------------------------------------------------------------|
| `scripts/bench_gbench_common.js`                           |   137 | Google Benchmark JSON helpers | `scripts/gbench.rs`, `scripts/cli.rs`, `scripts/runner.rs`, `scripts/driver.rs` |
| `prove-verify/bench_prove_verify.js`                       |   424 | Longfellow CFT driver         | `prove-verify/bench_prove_verify.rs`                                            |
| `prove-verify-no-cft/bench_prove_verify_no_cft.js`         |   388 | No-CFT driver                 | `prove-verify-no-cft/bench_prove_verify_no_cft.rs`                              |
| `prove-verify-revocation/bench_prove_verify_revocation.js` |   248 | Population scale sweep        | `prove-verify-revocation/bench_prove_verify_revocation.rs`                      |
| `merkle-vs-flat/bench_merkle_vs_flat.js`                   |   625 | Commitment sweep              | `merkle-vs-flat/bench_merkle_vs_flat.rs`                                        |
| `communication-costs/bench_communication_size.js`          |   134 | Runs the C++ measure scripts  | `communication-costs/bench_communication_size.rs`                               |

## Duplication in the original

Two patterns account for a large share of the 6 547 lines:

- `lib/crypto_babyjub.js` and `lib/crypto_common.js` are duplicated between
  `revocation/` and `prove-verify/zk-friendly/`, because each stack is a
  standalone npm package.
- The two `prove-verify/standard/` presentation drivers, 812 lines together,
  differ in three strings: the binary name, the default filter and the label.
  `scripts/driver.rs` holds that shared body once.

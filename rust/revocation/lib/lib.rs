//! Baby Jubjub CFT anonymity-revocation benchmarks.
//!
//! Rust port of the Node.js `revocation/` stack. The C4 binding matches the
//! zk-friendly `prove_verify.circom`, so tags produced here are the ones the
//! circuit accepts.
//!
//! | JavaScript module | Rust module |
//! |---|---|
//! | `lib/crypto_babyjub.js`, `lib/babyjub_noble.js` | [`babyjub`] |
//! | `lib/crypto_common.js` | [`babyjub`] (field arithmetic is in the types) |
//! | `lib/poseidon_cjs.js` | [`poseidon`] |
//! | `circomlibjs` EdDSA | [`eddsa`] |
//! | `lib/c4_binding.js` | [`c4`] |
//! | `lib/cft_bench_lib.js` | [`cft`], [`stats`] |
//! | `lib/run_experiment.js` | [`experiment`] |
//! | `mpc/mpc_runner.js` | [`mpc_runner`] |

pub mod babyjub;
pub mod c4;
pub mod cft;
pub mod csv;
pub mod eddsa;
pub mod env;
pub mod experiment;
pub mod mpc_runner;
pub mod paths;
pub mod poseidon;
pub mod stats;
pub mod time;

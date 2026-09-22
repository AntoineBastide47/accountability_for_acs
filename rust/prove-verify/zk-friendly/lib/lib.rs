//! Circom / Groth16 prove-verify benchmarks.
//!
//! Rust port of the Node.js `prove-verify/zk-friendly/` stack. The circuits,
//! `circom`, `rapidsnark` and the `snarkjs` CLI are unchanged external tools;
//! what moved into Rust is the credential model, the witness inputs, the
//! toolchain driver and the Groth16 verifier.
//!
//! | JavaScript module | Rust module |
//! |---|---|
//! | `lib/crypto_babyjub.js` | [`babyjub`] |
//! | `lib/crypto_common.js` | [`babyjub`], [`hash`] |
//! | `lib/poseidon_merkle.js` | [`poseidon_merkle`] |
//! | `lib/zk_common.js` | [`zk_common`] |
//! | `circomlibjs` Poseidon / EdDSA | [`poseidon`], [`eddsa`] |
//! | credential half of `bench_prove_verify.js` | [`credential`] |
//! | `snarkjs.groth16.verify` | [`groth16`] |

pub mod babyjub;
pub mod credential;
pub mod eddsa;
pub mod groth16;
pub mod hash;
pub mod paths;
pub mod poseidon;
pub mod poseidon_merkle;
pub mod stats;
pub mod summary;
pub mod time;
pub mod zk_common;

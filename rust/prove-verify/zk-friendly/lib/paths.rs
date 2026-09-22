//! Locations inside the stack, replacing Node's `__dirname`.
//!
//! The crate directory is baked in at build time and can be overridden with
//! `ZK_FRIENDLY_ROOT`, which is what a relocated binary needs.

use std::path::PathBuf;

pub fn crate_dir() -> PathBuf {
    std::env::var_os("ZK_FRIENDLY_ROOT")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
}

/// The directory of one benchmark, e.g. `prove-verify/`.
pub fn bench_dir(name: &str) -> PathBuf {
    crate_dir().join(name)
}

pub fn powers_of_tau() -> PathBuf {
    crate_dir().join("powersOfTau")
}

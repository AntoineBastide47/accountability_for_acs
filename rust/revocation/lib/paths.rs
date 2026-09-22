//! Locations inside the benchmark tree, replacing Node's `__dirname`.
//!
//! The crate directory is baked in at build time and can be overridden with
//! `REVOCATION_ROOT`, which is what a relocated binary needs.

use std::path::{Path, PathBuf};

pub fn crate_dir() -> PathBuf {
    std::env::var_os("REVOCATION_ROOT")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
}

pub fn results_dir() -> PathBuf {
    crate_dir().join("results")
}

pub fn mpc_dir() -> PathBuf {
    crate_dir().join("mpc")
}

/// `results/direct-decrypt` + `_runs.csv` -> `results/direct-decrypt_runs.csv`.
pub fn with_suffix(prefix: &Path, suffix: &str) -> PathBuf {
    let mut name = prefix.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    prefix.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_extends_the_file_name() {
        let p = with_suffix(Path::new("/tmp/results/link-decrypt"), "_runs.csv");
        assert_eq!(p, Path::new("/tmp/results/link-decrypt_runs.csv"));
    }
}

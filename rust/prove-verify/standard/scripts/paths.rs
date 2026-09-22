//! Locations inside the stack and in the vendored Longfellow tree.

use std::path::PathBuf;

/// The crate root, overridable with `STANDARD_ROOT` for a relocated binary.
pub fn crate_dir() -> PathBuf {
    std::env::var_os("STANDARD_ROOT")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
}

/// The directory of one benchmark, e.g. `prove-verify/`.
pub fn bench_dir(name: &str) -> PathBuf {
    crate_dir().join(name)
}

/// The Longfellow C++ tree.
///
/// Longfellow is a vendored third-party dependency (Apache 2.0, Google LLC) and
/// is not duplicated into the Rust tree, so the original checkout is the last
/// candidate. `LONGFELLOW_ROOT` overrides the search.
pub fn longfellow_root() -> PathBuf {
    if let Some(root) = std::env::var_os("LONGFELLOW_ROOT").filter(|v| !v.is_empty()) {
        return PathBuf::from(root);
    }

    let crate_dir = crate_dir();
    let candidates = [
        // Alongside the crate, as in the Docker image.
        crate_dir.join("longfellow-zk"),
        // A flat layout where the crate root *is* the Longfellow tree.
        crate_dir.clone(),
        // The original Node stack, three levels up at the repository root.
        repo_root(&crate_dir).join("prove-verify/standard/longfellow-zk"),
    ];

    candidates
        .into_iter()
        .find(|dir| dir.join("lib").join("CMakeLists.txt").is_file())
        .unwrap_or(crate_dir)
}

/// `rust/prove-verify/standard` -> the repository root.
fn repo_root(crate_dir: &std::path::Path) -> PathBuf {
    crate_dir
        .ancestors()
        .nth(3)
        .unwrap_or(crate_dir)
        .to_path_buf()
}

/// Where the CMake release build puts a benchmark binary.
pub fn default_bin_path(test_name: &str) -> PathBuf {
    longfellow_root()
        .join("clang-build-release")
        .join("circuits")
        .join("tests")
        .join("ec")
        .join(test_name)
}

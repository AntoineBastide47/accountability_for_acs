//! Writing the summary JSON.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::time;

/// Writes `summary_<timestamp>.json` and `summary_latest.json` into `dir`.
pub fn write<T: Serialize>(dir: &Path, summary: &T) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    let body = serde_json::to_vec_pretty(summary)?;
    std::fs::write(
        dir.join(format!("summary_{}.json", time::iso8601_now_for_filename())),
        &body,
    )?;
    std::fs::write(dir.join("summary_latest.json"), &body)?;
    Ok(())
}

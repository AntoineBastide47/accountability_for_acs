//! The summary JSON both prove/verify benchmarks write.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::stats::SummaryMs;
use crate::time;

#[derive(Serialize)]
pub struct Meta {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub variant: &'static str,
    #[serde(rename = "N")]
    pub n: usize,
    #[serde(rename = "credentialMode")]
    pub credential_mode: &'static str,
    #[serde(rename = "timestampIso")]
    pub timestamp_iso: String,
}

#[derive(Serialize)]
pub struct Results {
    #[serde(rename = "successfulIters")]
    pub successful_iters: usize,
}

#[derive(Serialize)]
pub struct AvgMs {
    pub witness: Option<f64>,
    pub prove: Option<f64>,
    pub verify: Option<f64>,
}

#[derive(Serialize)]
pub struct StatsMs {
    pub witness: Option<SummaryMs>,
    pub prove: Option<SummaryMs>,
    pub verify: Option<SummaryMs>,
    #[serde(rename = "proverTotal")]
    pub prover_total: Option<SummaryMs>,
    #[serde(rename = "fullCycle")]
    pub full_cycle: Option<SummaryMs>,
}

#[derive(Serialize)]
pub struct TimingSummary {
    pub meta: Meta,
    pub results: Results,
    #[serde(rename = "avgMs")]
    pub avg_ms: AvgMs,
    #[serde(rename = "statsMs")]
    pub stats_ms: StatsMs,
}

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

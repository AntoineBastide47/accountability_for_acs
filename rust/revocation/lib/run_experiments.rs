//! Runs direct-decrypt then link-decrypt, sharing one crypto context.
//!
//! Port of `lib/run_experiments.js`.

use anyhow::Result;
use revocation::experiment::{self, Benchmark, Context, Options};
use revocation::paths;

fn main() -> Result<()> {
    let mut ctx = Context::new();

    for (index, benchmark) in Benchmark::ALL.into_iter().enumerate() {
        if index > 0 {
            println!("\n{}\n", "─".repeat(60));
        }
        let options = Options::from_env(benchmark)?;
        experiment::run(benchmark, &mut ctx, &options)?;
    }

    let results = paths::results_dir();
    println!("\nAll experiments done. CSVs in {}:", results.display());
    for benchmark in Benchmark::ALL {
        for suffix in ["_runs.csv", "_summary.csv", "_fit.csv"] {
            println!("  {}{suffix}", results.join(benchmark.as_str()).display());
        }
    }
    Ok(())
}

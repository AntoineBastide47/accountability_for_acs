//! Decrypt every CFT (police + judge + NGO).

use anyhow::Result;
use revocation::experiment::{self, Benchmark, Context, Options};

fn main() -> Result<()> {
    let benchmark = Benchmark::DirectDecrypt;
    let options = Options::from_env(benchmark)?;
    experiment::run(benchmark, &mut Context::new(), &options)?;
    Ok(())
}

//! Link CFTs to pseudonyms, keep the recurring ones, decrypt only those.

use anyhow::Result;
use revocation::experiment::{self, Benchmark, Context, Options};

fn main() -> Result<()> {
    let benchmark = Benchmark::LinkDecrypt;
    let options = Options::from_env(benchmark)?;
    experiment::run(benchmark, &mut Context::new(), &options)?;
    Ok(())
}

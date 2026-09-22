//! Longfellow (Google Benchmark) prove-verify benchmarks.
//!
//! Rust port of the Node.js `prove-verify/standard/` stack. The C++ benchmark
//! binaries are unchanged; what moved into Rust is the harness that drives them
//! and turns their JSON report into the summaries the paper reports.
//!
//! | JavaScript module | Rust module |
//! |---|---|
//! | `scripts/bench_gbench_common.js` | [`gbench`] |
//! | per-benchmark argument parsing and process launching | [`cli`], [`runner`] |

pub mod cli;
pub mod driver;
pub mod gbench;
pub mod paths;
pub mod runner;
pub mod summary;
pub mod time;

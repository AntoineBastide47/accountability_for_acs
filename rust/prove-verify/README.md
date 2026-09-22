# `prove-verify/` (Rust)

Side-by-side prove/verify benchmarks for the credential presentation proof.
Rust port of `../../prove-verify/`.

| Dir | Stack |
|-----|--------|
| `zk-friendly/` | Circom + Groth16 (rapidsnark proves, this crate verifies) |
| `standard/` | Longfellow C++ (Google Benchmark) |

Both expose the same five benchmarks (run from the stack dir after setup):

```bash
cargo run --release --bin bench_prove_verify             # age-check + CFT
cargo run --release --bin bench_prove_verify_no_cft      # same, no CFT
cargo run --release --bin bench_prove_verify_revocation  # + non revocation claim (2^12…2^24)
cargo run --release --bin bench_merkle_vs_flat           # attribute-commitment sweep
cargo run --release --bin bench_communication_size       # wire size (CFT + non revoc claim 2^12…2^24)
```

Pre-recorded summaries from the JavaScript stacks: `results/{mobile,server}-env/`,
`results/proof-sizes/`.

See each stack README for Docker and local setup.

## License

Original code in this folder is under the workspace **MIT** license
(`../../LICENSE`). The vendored Longfellow tree that `standard/` builds against
lives at `../../prove-verify/standard/longfellow-zk/` and is **Apache 2.0**
(Google LLC).

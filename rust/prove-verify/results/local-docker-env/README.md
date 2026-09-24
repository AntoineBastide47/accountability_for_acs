# Local runs of the Rust port

Written by `rust/benchmark.py`; the plots come from
`../plots/plot_local_vs_server.py`. `environment.json` records the machine,
the Docker limits, the git commit and which benchmarks succeeded.

| Files | Environment |
|---|---|
| `standard_local_*`, `zkfriendly_local_*` | Docker, `--cpus` (default 2) and `--memory` (default 16g) |
| `zkfriendly_local_12cpu_*` | Docker, `--cpus-full` (default 12): rapidsnark scaling |
| `zkfriendly_local_2cpu_pinned_*` | Docker, `--cpuset-cpus=0,1`, run by hand (rapidsnark sees 2 cores) |
| `revocation-native/` | Native on the host, no CPU limit; `mpc_results.csv` only when `MP_SPDZ_PATH` is set |

- The zk-friendly keys are set up with `ppot_0080_19.ptau` from the PSE
  Perpetual Powers of Tau, because the Hermez `powersOfTau28_hez_final_19.ptau`
  mirrors return 403. The circuits and timings do not depend on the ptau.
- `--cpus` is a CPU-time quota: rapidsnark still sees every core and is
  throttled, which is why `prove` is slower at 2 CPUs than in `server-env/`.

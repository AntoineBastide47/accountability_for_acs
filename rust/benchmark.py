#!/usr/bin/env python3
"""Runs every benchmark of the Rust port, collects the results and draws the plots.

  1. Builds the two Docker images (standard, zk-friendly).
  2. Runs the five standard (Longfellow) benchmarks and the five zk-friendly
     benchmarks in Docker, with the CPU and memory limits below.
  3. Reruns the three zk-friendly prove benchmarks with --cpus-full, to show
     how rapidsnark scales with cores.
  4. Runs the revocation CFT experiments natively (and the MPC sweep when
     MP_SPDZ_PATH is set).
  5. Writes flat summaries, CSVs, environment.json and logs/ into --out, then
     calls prove-verify/results/plots/plot_local_vs_server.py.

  python3 rust/benchmark.py                      # everything, ~3.5 h on 2 CPUs
  python3 rust/benchmark.py --only zk-friendly --skip-build
  BENCH_N=3 python3 rust/benchmark.py            # BENCH_* and friends pass through

Needs Docker (VM memory >= --memory), cargo, and matplotlib + numpy for the
plots. The zk-friendly image needs the ptau file in
prove-verify/zk-friendly/powersOfTau/ before it is built.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

RUST = Path(__file__).resolve().parent
REPO = RUST.parent
STANDARD = RUST / "prove-verify" / "standard"
ZK = RUST / "prove-verify" / "zk-friendly"
REVOCATION = RUST / "revocation"
PLOTS = RUST / "prove-verify" / "results" / "plots"
DEFAULT_OUT = RUST / "prove-verify" / "results" / "local-docker-env"
PTAU = ZK / "powersOfTau" / "powersOfTau28_hez_final_19.ptau"
PTAU_URL = "https://pse-trusted-setup-ppot.s3.eu-central-1.amazonaws.com/pot28_0080/ppot_0080_19.ptau"

STACKS = ("standard", "zk-friendly", "revocation")

# Benchmark options forwarded into the containers when set on the host.
PASS_ENV = (
    "BENCH_N", "BENCH_REPETITIONS", "BENCH_WARMUP", "BENCH_VERIFY_WARMUP",
    "BENCH_ITERATIONS", "BENCH_MIN_TIME", "BENCH_FILTER", "REVOC_LOG2_LIST",
    "REVOC_LOG2", "REVOC_BITS_PER_LEAF", "REVOC_SLOT", "TOTAL_ATTRS", "USED_ATTRS",
)

BENCHES = ("prove_verify", "prove_verify_no_cft", "prove_verify_revocation",
           "merkle_vs_flat", "communication_size")
PROVE_BENCHES = BENCHES[:3]

# Where each zk-friendly binary leaves summary_latest.json inside the image.
ZK_SUMMARY = {
    "prove_verify": "prove-verify/artifacts_bench_prove_verify",
    "prove_verify_no_cft": "prove-verify-no-cft/artifacts_bench_prove_verify_no_cft",
    "prove_verify_revocation":
        "prove-verify-revocation/artifacts_bench_prove_verify_revocation",
    "merkle_vs_flat": "merkle-vs-flat/artifacts_bench_merkle_vs_flat",
    "communication_size": "/tmp/artifacts",
}


def result_name(prefix: str, bench: str) -> str:
    """Flat file name, e.g. `zkfriendly_local_prove_verify_summary.json`,
    matching server-env/ and proof-sizes/."""
    if bench == "communication_size":
        return f"{prefix}_prove_verify_size_report.json"
    return f"{prefix}_{bench}_summary.json"


def run(cmd: list[str], **kwargs) -> int:
    print("$ " + " ".join(cmd), flush=True)
    return subprocess.run(cmd, check=False, **kwargs).returncode


def docker_memory_bytes() -> int:
    out = subprocess.run(["docker", "info", "--format", "{{.MemTotal}}"],
                         capture_output=True, text=True, check=False)
    if out.returncode != 0:
        sys.exit("Docker is not reachable; start Docker Desktop (or the daemon) first.")
    return int(out.stdout.strip())


def parse_memory(text: str) -> int:
    units = {"k": 2**10, "m": 2**20, "g": 2**30}
    return int(float(text[:-1]) * units[text[-1].lower()]) if text[-1].isalpha() else int(text)


def build_images(stacks: list[str]) -> None:
    if "standard" in stacks and run(
            ["docker", "build", "-t", "standard-bench-rs",
             "-f", str(STANDARD / "Dockerfile"), str(REPO)]) != 0:
        sys.exit("standard image build failed")
    if "zk-friendly" in stacks:
        if not PTAU.is_file():
            sys.exit(f"Missing {PTAU}.\nThe Hermez mirrors behind scripts/fetch_ptau.sh "
                     f"return 403; download the PSE file instead:\n"
                     f"  curl -L --fail -o {PTAU} {PTAU_URL}")
        if run(["docker", "build", "-t", "zk-friendly-bench-rs",
                "-f", str(ZK / "Dockerfile"), str(ZK)]) != 0:
            sys.exit("zk-friendly image build failed")


def docker_suite(image: str, stack: str, benches: tuple[str, ...], limits: list[str],
                 out: Path, tag: str) -> dict[str, bool]:
    """Runs `benches` in one container and copies each summary to `out`.

    One container per suite keeps the circuit setup cache between benchmarks.
    Returns, per benchmark, whether it exited 0 and wrote its summary.
    """
    prefix = f"{stack}_local_{tag}" if tag else f"{stack}_local"
    lines = ["cd /bench"]
    for bench in benches:
        dest = result_name(prefix, bench)
        (out / dest).unlink(missing_ok=True)  # never leave a stale result behind
        src = ("/tmp/artifacts" if stack == "standard" else ZK_SUMMARY[bench])
        log = f"/out/logs/{prefix}_{bench}.log"
        lines += [
            f"echo '=== {prefix} {bench}' $(date -u +%T)",
            f"ARTIFACTS_DIR=/tmp/artifacts ./target/release/bench_{bench} > {log} 2>&1",
            f"echo {bench} $? >> /out/logs/{prefix}.status",
            f"cp {src}/summary_latest.json /out/{dest} 2>/dev/null || true",
            "rm -rf /tmp/artifacts",
        ]
    status = out / "logs" / f"{prefix}.status"
    status.unlink(missing_ok=True)

    env = [arg for name in PASS_ENV if name in os.environ for arg in ("-e", name)]
    run(["docker", "run", "--rm", *limits, *env, "-v", f"{out}:/out", image,
         "bash", "-c", "\n".join(lines)])

    codes = dict.fromkeys(benches, -1)  # -1: never reached (container died)
    if status.is_file():
        for line in status.read_text().split("\n"):
            if line:
                bench, code = line.split()
                codes[bench] = int(code)
        status.unlink()
    return {
        f"{prefix}:{b}": codes[b] == 0 and (out / result_name(prefix, b)).is_file()
        for b in benches
    }


def revocation_native(out: Path) -> dict[str, bool]:
    """CFT experiments on the host; results go to out/revocation-native/."""
    if run(["cargo", "build", "--release"], cwd=REVOCATION) != 0:
        return {"revocation:build": False}
    dest = out / "revocation-native"
    dest.mkdir(parents=True, exist_ok=True)
    for stale in dest.glob("*.csv"):
        stale.unlink()
    codes = {}
    with tempfile.TemporaryDirectory() as root:
        # REVOCATION_ROOT redirects results/ away from the recorded CSVs.
        env = {**os.environ, "REVOCATION_ROOT": root}
        with open(out / "logs" / "revocation_run_experiments.log", "w") as log:
            codes["revocation:run_experiments"] = run(
                [str(REVOCATION / "target/release/run_experiments")],
                env=env, stdout=log, stderr=subprocess.STDOUT) == 0
        for csv in Path(root, "results").glob("*.csv"):
            shutil.copy(csv, dest / csv.name)

        if os.environ.get("MP_SPDZ_PATH"):
            sweep = Path(root, "mpc-sweep")
            codes["revocation:mpc"] = run(
                ["bash", str(REVOCATION / "mpc/run_sweep.sh")],
                env={**os.environ, "OUTDIR": str(sweep)}) == 0
            if (sweep / "results.csv").is_file():
                shutil.copy(sweep / "results.csv", dest / "mpc_results.csv")
        else:
            print("Skip revocation MPC (MP_SPDZ_PATH not set)")
    return codes


def host_description() -> dict:
    cpu = platform.processor()
    if sys.platform == "darwin":
        cpu = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"],
                             capture_output=True, text=True).stdout.strip() or cpu
    info = subprocess.run(["docker", "info", "--format", "{{.NCPU}} {{.MemTotal}}"],
                          capture_output=True, text=True).stdout.split()
    commit = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"],
                            capture_output=True, text=True).stdout.strip()
    return {
        "host": {"platform": platform.platform(), "cpu": cpu,
                 "logicalCpus": os.cpu_count()},
        "docker": {"cpus": int(info[0]), "memBytes": int(info[1])} if len(info) == 2 else None,
        "gitCommit": commit,
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__.splitlines()[0],
        formatter_class=argparse.RawDescriptionHelpFormatter, epilog=__doc__.split("\n", 2)[2])
    parser.add_argument("--only", action="append", choices=STACKS,
                        help="run only this stack (repeatable)")
    parser.add_argument("--skip-build", action="store_true", help="reuse the Docker images")
    parser.add_argument("--cpus", default="2", help="Docker CPU limit (default 2)")
    parser.add_argument("--cpus-full", default="12",
                        help="CPU limit of the zk-friendly prove rerun; 0 skips it")
    parser.add_argument("--memory", default="16g",
                        help="Docker memory limit (default 16g; merkle-vs-flat needs > 7g)")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT, help="results directory")
    parser.add_argument("--no-plots", action="store_true")
    args = parser.parse_args()

    stacks = args.only or list(STACKS)
    out = args.out.resolve()
    (out / "logs").mkdir(parents=True, exist_ok=True)
    docker_stacks = [s for s in stacks if s != "revocation"]

    if docker_stacks:
        if docker_memory_bytes() < parse_memory(args.memory):
            sys.exit(f"Docker VM has less memory than --memory {args.memory}; "
                     f"raise it in Docker Desktop (Settings -> Resources).")
        if not args.skip_build:
            build_images(docker_stacks)

    started = datetime.now(timezone.utc)
    limits = ["--cpus", args.cpus, "--memory", args.memory, "--memory-swap", args.memory]
    codes: dict[str, bool] = {}
    if "standard" in stacks:
        codes |= docker_suite("standard-bench-rs", "standard", BENCHES, limits, out, "")
    if "zk-friendly" in stacks:
        codes |= docker_suite("zk-friendly-bench-rs", "zkfriendly", BENCHES, limits, out, "")
        if args.cpus_full != "0":
            full = ["--cpus", args.cpus_full, *limits[2:]]
            codes |= docker_suite("zk-friendly-bench-rs", "zkfriendly", PROVE_BENCHES,
                                  full, out, f"{args.cpus_full}cpu")
    if "revocation" in stacks:
        codes |= revocation_native(out)

    environment = {
        **host_description(),
        "started": started.isoformat(timespec="seconds"),
        "finished": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "limits": {"cpus": args.cpus, "cpusFull": args.cpus_full, "memory": args.memory},
        "ptau": PTAU.name,
        "succeeded": codes,
    }
    (out / "environment.json").write_text(json.dumps(environment, indent=2) + "\n")

    if not args.no_plots:
        plot_out = PLOTS if out == DEFAULT_OUT else out / "plots"
        codes["plots"] = run([sys.executable, str(PLOTS / "plot_local_vs_server.py"),
                              "--local", str(out), "--out-dir", str(plot_out)]) == 0

    failed = [name for name, ok in codes.items() if not ok]
    print(f"\nResults: {out}  (logs in {out / 'logs'})")
    if failed:
        sys.exit(f"Failed: {', '.join(failed)}")
    print("All benchmarks succeeded.")


if __name__ == "__main__":
    main()

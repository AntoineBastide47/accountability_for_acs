//! Age-check presentation with a CFT and a packed status-list non-revocation
//! proof, swept over population scales.
//!
//! Timing matches the other prove/verify benches: `witness` covers the per-show
//! input preparation, the Merkle path and the C++ witness calculator; `prove`
//! is rapidsnark alone; `verify` is the Groth16 pairing check alone.
//!
//! Port of `prove-verify-revocation/bench_prove_verify_revocation.js`.

#[path = "lib/circom_codegen.rs"]
mod circom_codegen;
#[path = "lib/revocation_tree.rs"]
mod revocation_tree;

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{bail, Result};
use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;

use revocation_tree::PackedTree;
use zk_friendly::credential::{claim_name, write_json, Credential, ShowInput};
use zk_friendly::groth16::{ProofBundle, VerifyingKey};
use zk_friendly::paths;
use zk_friendly::poseidon::Poseidon;
use zk_friendly::stats::{self, Summary};
use zk_friendly::summary;
use zk_friendly::time::{self, elapsed_ms};
use zk_friendly::zk_common::{Groth16Spec, IterArgs, Toolchain};

const PTAU: &str = "../powersOfTau/powersOfTau28_hez_final_19.ptau";

#[derive(Parser)]
#[command(about = "Groth16 prove/verify benchmark for CFT + packed status-list revocation")]
struct Args {
    /// Population scales as log2, e.g. `12,16,20,24`.
    #[arg(long, env = "REVOC_LOG2_LIST", value_delimiter = ',', default_values_t = [12u32, 16, 20, 24])]
    revoc_log2: Vec<u32>,
    /// Status-list bits packed into one Merkle leaf.
    #[arg(long, env = "REVOC_BITS_PER_LEAF", default_value_t = 253)]
    bits_per_leaf: u32,
    /// Attribute slot holding the revocation index.
    #[arg(long, env = "REVOC_SLOT", default_value_t = 14)]
    revoc_slot: usize,
    #[command(flatten)]
    iters: IterArgs,
    /// Print setup sections instead of per-iteration lines.
    #[arg(long)]
    verbose: bool,
}

/// `input.json` for one revocation scale: the base show plus the packed
/// status-list witness.
#[derive(Serialize)]
struct RevocationInput {
    #[serde(flatten)]
    base: ShowInput,
    #[serde(rename = "revClaimName")]
    rev_claim_name: String,
    #[serde(rename = "revocationRoot")]
    revocation_root: String,
    #[serde(rename = "leafIndex")]
    leaf_index: String,
    #[serde(rename = "bitIndex")]
    bit_index: String,
    #[serde(rename = "leafValue")]
    leaf_value: String,
    #[serde(rename = "pathElements")]
    path_elements: Vec<String>,
    #[serde(rename = "pathIndices")]
    path_indices: Vec<String>,
}

#[derive(Serialize)]
struct ScaleResult {
    #[serde(rename = "revocLog2")]
    revoc_log2: u32,
    population: u64,
    #[serde(rename = "merkleDepth")]
    merkle_depth: u32,
    witness: Option<Summary>,
    prove: Option<Summary>,
    verify: Option<Summary>,
    #[serde(rename = "proverTotal")]
    prover_total: Option<Summary>,
}

#[derive(Serialize)]
struct Meta {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "revocLog2List")]
    revoc_log2_list: Vec<u32>,
    #[serde(rename = "bitsPerLeaf")]
    bits_per_leaf: u32,
    #[serde(rename = "revocSlot")]
    revoc_slot: usize,
    #[serde(rename = "benchN")]
    bench_n: usize,
    warmup: usize,
    #[serde(rename = "verifyWarmup")]
    verify_warmup: usize,
    #[serde(rename = "timestampIso")]
    timestamp_iso: String,
}

#[derive(Serialize)]
struct RevocationSummary {
    meta: Meta,
    #[serde(rename = "byScale")]
    by_scale: Vec<ScaleResult>,
}

struct Context<'a> {
    args: &'a Args,
    base_dir: &'a Path,
    artifacts_dir: &'a Path,
    zk: &'a Toolchain,
    poseidon: &'a Poseidon,
    credential: &'a Credential,
    rev_claim_name: String,
}

fn bench_scale(ctx: &Context<'_>, revoc_log2: u32, rng: &mut StdRng) -> Result<ScaleResult> {
    let args = ctx.args;
    let population = 1u64 << revoc_log2;
    let suffix = format!("_l{revoc_log2}");

    let generated = circom_codegen::write_circuit(
        ctx.base_dir,
        population,
        args.bits_per_leaf,
        args.revoc_slot,
        &suffix,
    )?;
    let circuit_name = generated
        .file_name
        .strip_suffix(".circom")
        .expect("generated circuit file name")
        .to_string();
    let out_dir = PathBuf::from("generated").join(&circuit_name);
    std::fs::create_dir_all(ctx.base_dir.join(&out_dir))?;

    let circom_file = PathBuf::from(format!("./{}", generated.file_name));
    let spec = Groth16Spec {
        circom_file: &circom_file,
        r1cs_file: &out_dir.join(format!("{circuit_name}.r1cs")),
        ptau_file: Path::new(PTAU),
        zkey_file: &out_dir.join(format!("{circuit_name}.zkey")),
        vkey_file: &out_dir.join(format!("vkey-{circuit_name}.json")),
        out_dir: &out_dir,
    };
    let artifacts = ctx.zk.prepare_groth16(&spec)?;
    let vkey = VerifyingKey::read(&ctx.zk.resolve(&artifacts.vkey))?;

    let tree = PackedTree::build(ctx.poseidon, population, args.bits_per_leaf);

    let mut witness_ms = Vec::with_capacity(args.iters.n);
    let mut prove_ms = Vec::with_capacity(args.iters.n);
    let mut verify_ms = Vec::with_capacity(args.iters.n);

    println!(
        "\n── 2^{revoc_log2} (depth {}, N={population}) ──\n  timed={}{}",
        generated.merkle_depth,
        args.iters.describe(),
        if args.iters.verify_warmup > 0 {
            format!(", verifyWarmup={}", args.iters.verify_warmup)
        } else {
            String::new()
        }
    );

    for i in 0..(args.iters.n + args.iters.warmup) {
        let is_warmup = i < args.iters.warmup;
        let iter_dir = ctx
            .artifacts_dir
            .join(format!("{circuit_name}_l{revoc_log2}_iter_{i:04}"));
        std::fs::create_dir_all(&iter_dir)?;

        let input_path = iter_dir.join("input.json");
        let witness_path = iter_dir.join("witness.wtns");
        let proof_path = iter_dir.join("proof.json");
        let public_path = iter_dir.join("public.json");

        let started = Instant::now();
        let credential_index = rng.gen_range(0..population);
        let rev = tree.proof_for(credential_index);
        let cred = ctx.credential.with_revocation_index(
            ctx.poseidon,
            args.revoc_slot,
            credential_index,
            rng,
        );
        let input = RevocationInput {
            base: cred.show_input(ctx.poseidon, credential_index + 1, rng),
            rev_claim_name: ctx.rev_claim_name.clone(),
            revocation_root: rev.revocation_root,
            leaf_index: rev.leaf_index,
            bit_index: rev.bit_index,
            leaf_value: rev.leaf_value,
            path_elements: rev.path_elements,
            path_indices: rev.path_indices,
        };
        write_json(&input_path, &input)?;
        let witness = ctx
            .zk
            .run_witness(&artifacts.witness_bin, &input_path, &witness_path)?;
        let w_ms = elapsed_ms(started);
        if !witness.ok {
            bail!("witness failed @2^{revoc_log2}: {}", witness.message());
        }

        let started = Instant::now();
        let proved =
            ctx.zk
                .run_prover(&artifacts.zkey, &witness_path, &proof_path, &public_path)?;
        let p_ms = elapsed_ms(started);
        if !proved.ok {
            bail!("prove failed @2^{revoc_log2}: {}", proved.message());
        }

        let bundle = ProofBundle::read(&proof_path, &public_path)?;
        if is_warmup {
            for _ in 0..args.iters.verify_warmup {
                bundle.verify(&vkey)?;
            }
        }

        let started = Instant::now();
        let ok = bundle.verify(&vkey)?;
        let v_ms = elapsed_ms(started);
        if !ok {
            bail!("verify failed @2^{revoc_log2}");
        }

        if !is_warmup {
            witness_ms.push(w_ms);
            prove_ms.push(p_ms);
            verify_ms.push(v_ms);
        }

        if !args.verbose {
            let label = if is_warmup {
                format!("warmup {}/{}", i + 1, args.iters.warmup)
            } else {
                format!("{}/{}", i - args.iters.warmup + 1, args.iters.n)
            };
            println!("  {label} witness={w_ms:.0}ms prove={p_ms:.0}ms verify={v_ms:.0}ms");
        }
    }

    Ok(ScaleResult {
        revoc_log2,
        population,
        merkle_depth: generated.merkle_depth,
        witness: stats::summary(&witness_ms),
        prove: stats::summary(&prove_ms),
        verify: stats::summary(&verify_ms),
        prover_total: stats::summary(&stats::add_series(&witness_ms, &prove_ms)),
    })
}

fn main() -> Result<()> {
    let args = Args::parse();
    let base_dir = paths::bench_dir("prove-verify-revocation");
    let artifacts_dir = base_dir.join("artifacts_bench_prove_verify_revocation");
    std::fs::create_dir_all(&artifacts_dir)?;

    println!("Zk-friendly ProveVerify + CFT + packed revocation");
    println!(
        "  scales: {}",
        args.revoc_log2
            .iter()
            .map(|x| format!("2^{x}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "  bits/leaf = {}, BENCH_N = {}, warmup = {}, verifyWarmup = {}\n",
        args.bits_per_leaf, args.iters.n, args.iters.warmup, args.iters.verify_warmup
    );

    let zk = Toolchain::from_env(&base_dir, args.verbose);
    let poseidon = Poseidon::new();
    let mut rng = StdRng::from_entropy();
    let credential = Credential::issue(&poseidon, &mut rng);
    let rev_claim_name = claim_name(&poseidon, "revocationIndex").to_string();

    let ctx = Context {
        args: &args,
        base_dir: &base_dir,
        artifacts_dir: &artifacts_dir,
        zk: &zk,
        poseidon: &poseidon,
        credential: &credential,
        rev_claim_name,
    };

    let mut by_scale = Vec::with_capacity(args.revoc_log2.len());
    for &revoc_log2 in &args.revoc_log2 {
        by_scale.push(bench_scale(&ctx, revoc_log2, &mut rng)?);
    }

    println!("\n── Summary (avg prover ms) ──");
    for r in &by_scale {
        println!(
            "  2^{} (depth {})  witness={}  prove={}  verify={}  prover={}",
            r.revoc_log2,
            r.merkle_depth,
            avg(r.witness),
            avg(r.prove),
            avg(r.verify),
            avg(r.prover_total)
        );
    }

    let report = RevocationSummary {
        meta: Meta {
            kind: "zk-friendly",
            revoc_log2_list: args.revoc_log2.clone(),
            bits_per_leaf: args.bits_per_leaf,
            revoc_slot: args.revoc_slot,
            bench_n: args.iters.n,
            warmup: args.iters.warmup,
            verify_warmup: args.iters.verify_warmup,
            timestamp_iso: time::iso8601_now(),
        },
        by_scale,
    };
    summary::write(&artifacts_dir, &report)?;
    println!(
        "\nSummary: {}",
        artifacts_dir.join("summary_latest.json").display()
    );
    Ok(())
}

fn avg(s: Option<Summary>) -> String {
    s.map_or_else(|| "?".to_string(), |s| format!("{:.1}", s.avg))
}

//! Drives the external Circom / Groth16 toolchain: `circom`, `make`, the
//! `snarkjs` CLI and `rapidsnark`.
//!
//! Port of `lib/zk_common.js`. Commands are spawned with an explicit argument
//! vector rather than through a shell, so the JS `shellQuote` helper has no
//! counterpart.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use regex::Regex;

/// Iteration counts shared by the prove/verify benchmarks; each flag also
/// reads the environment variable the Node stack used.
#[derive(clap::Args, Debug)]
pub struct IterArgs {
    /// Measured iterations.
    #[arg(long, env = "BENCH_N", default_value_t = 10)]
    pub n: usize,
    /// Discarded iterations before the measured ones.
    #[arg(long, env = "BENCH_WARMUP", default_value_t = 1)]
    pub warmup: usize,
    /// Extra verify calls during the warm-up iteration.
    #[arg(long, env = "BENCH_VERIFY_WARMUP", default_value_t = 0)]
    pub verify_warmup: usize,
}

impl IterArgs {
    /// `Iterations: N (+W warmup discarded)`.
    pub fn describe(&self) -> String {
        if self.warmup > 0 {
            format!("{} (+{} warmup discarded)", self.n, self.warmup)
        } else {
            self.n.to_string()
        }
    }
}

/// Where each external tool lives, and how loud to be about using it.
pub struct Toolchain {
    /// Benchmark directory; every relative path resolves against it.
    pub base_dir: PathBuf,
    pub circom: String,
    pub rapidsnark: String,
    pub snarkjs: String,
    /// `-l` include root; must contain `circomlib/circuits/`.
    pub circom_lib: PathBuf,
    pub verbose: bool,
}

impl Toolchain {
    /// Reads `CIRCOM_BIN`/`CIRCOM`, `RAPIDSNARK_BIN`, `SNARKJS_BIN` and
    /// `CIRCOM_LIB_PATH`, falling back to the names on `PATH`.
    pub fn from_env(base_dir: impl Into<PathBuf>, verbose: bool) -> Self {
        let base_dir = base_dir.into();
        let stack_root = crate::paths::crate_dir();
        Self {
            base_dir,
            circom: env_or("CIRCOM_BIN", || env_or("CIRCOM", || "circom".into())),
            rapidsnark: env_or("RAPIDSNARK_BIN", || "prover".into()),
            snarkjs: env_or("SNARKJS_BIN", || "snarkjs".into()),
            circom_lib: std::env::var_os("CIRCOM_LIB_PATH")
                .map_or_else(|| stack_root.join("circom-libs"), PathBuf::from),
            verbose,
        }
    }

    pub fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }

    fn progress(&self, message: &str) {
        if std::env::var("BENCH_SILENT_SETUP").as_deref() != Ok("1") {
            println!("{message}");
        }
    }

    pub fn section(&self, title: &str) {
        if !self.verbose {
            return;
        }
        println!("\n{}", "━".repeat(51));
        println!("  {title}");
        println!("{}", "━".repeat(51));
    }

    /// Runs a command in the benchmark directory and captures its output.
    pub fn exec(&self, program: &str, args: &[&str]) -> Result<Output> {
        self.exec_in(&self.base_dir, program, args)
    }

    pub fn exec_in(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<Output> {
        let output = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .output()
            .with_context(|| format!("spawn {program}"))?;
        Ok(Output {
            ok: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// What a spawned tool produced.
pub struct Output {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    /// The tool's own diagnostics, preferring stderr.
    pub fn message(&self) -> &str {
        if self.stderr.trim().is_empty() {
            &self.stdout
        } else {
            &self.stderr
        }
    }
}

fn env_or(name: &str, fallback: impl FnOnce() -> String) -> String {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(fallback)
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// True when every output exists and is at least as new as the source.
fn up_to_date(source: &Path, outputs: &[&Path]) -> bool {
    let Some(source_time) = modified(source) else {
        return false;
    };
    outputs
        .iter()
        .all(|out| modified(out).is_some_and(|t| t >= source_time))
}

fn assert_non_empty(path: &Path, label: &str) -> Result<()> {
    let size = fs::metadata(path)
        .with_context(|| format!("missing {label} at {}", path.display()))?
        .len();
    if size == 0 {
        bail!("empty {label} at {}", path.display());
    }
    Ok(())
}

/// Everything one circuit needs, relative to [`Toolchain::base_dir`].
pub struct Groth16Spec<'a> {
    pub circom_file: &'a Path,
    pub r1cs_file: &'a Path,
    pub ptau_file: &'a Path,
    pub zkey_file: &'a Path,
    pub vkey_file: &'a Path,
    pub out_dir: &'a Path,
}

impl Groth16Spec<'_> {
    fn sym_file(&self) -> PathBuf {
        self.r1cs_file.with_extension("sym")
    }

    fn circuit_name(&self) -> &str {
        self.circom_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
    }
}

/// The prover-side artifacts a benchmark iteration needs.
pub struct Artifacts {
    pub witness_bin: PathBuf,
    pub zkey: PathBuf,
    pub vkey: PathBuf,
    pub r1cs: PathBuf,
}

impl Toolchain {
    /// Compiles the circuit to R1CS unless the cached output is still fresh.
    fn ensure_circuit_compiled(&self, spec: &Groth16Spec<'_>) -> Result<()> {
        let circom_abs = self.resolve(spec.circom_file);
        let r1cs_abs = self.resolve(spec.r1cs_file);
        let sym_abs = self.resolve(spec.sym_file());

        if up_to_date(&circom_abs, &[&r1cs_abs, &sym_abs]) {
            self.progress("  [setup] Using cached R1CS/sym (skipped circom compile).");
            return Ok(());
        }

        self.section("Setup (Compiling circuit)");
        self.progress("  [setup] Compiling circuit (circom)...");
        fs::create_dir_all(self.resolve(spec.out_dir))?;

        let circom_lib = self.circom_lib.display().to_string();
        let out_dir = spec.out_dir.display().to_string();
        let source = file_name(spec.circom_file)?;
        let result = self.exec(
            &self.circom,
            &[source, "--r1cs", "--sym", "-o", &out_dir, "-l", &circom_lib],
        )?;
        if !result.ok {
            bail!("circom failed:\n{}", result.message());
        }
        Ok(())
    }

    /// Builds circom's C++ witness calculator unless it is still fresh.
    fn ensure_witness_generator(&self, spec: &Groth16Spec<'_>) -> Result<PathBuf> {
        let circom_abs = self.resolve(spec.circom_file);
        let name = spec.circuit_name().to_string();
        let cpp_dir_rel = spec.out_dir.join(format!("{name}_cpp"));
        let cpp_dir_abs = self.resolve(&cpp_dir_rel);
        let bin_rel = cpp_dir_rel.join(&name);
        let bin_abs = self.resolve(&bin_rel);
        let dat_abs = cpp_dir_abs.join(format!("{name}.dat"));

        if up_to_date(&circom_abs, &[&bin_abs, &dat_abs]) {
            self.progress("  [setup] Using cached C++ witness binary (skipped make).");
            return Ok(bin_rel);
        }

        self.section("Setup (Building C++ witness generator)");
        self.progress("  [setup] Building C++ witness generator (circom --c + make)...");

        let circom_lib = self.circom_lib.display().to_string();
        let out_dir = spec.out_dir.display().to_string();
        let source = file_name(spec.circom_file)?;
        let generated = self.exec(
            &self.circom,
            &[source, "--c", "--no_asm", "-o", &out_dir, "-l", &circom_lib],
        )?;
        if !generated.ok {
            bail!("circom --c failed:\n{}", generated.message());
        }

        // Best effort: circom's generated Makefile does not know about
        // Homebrew's prefix, and its `--no_asm` backend passes `uint64_t*` to
        // GMP's `mpn_*`, which Clang rejects on macOS.
        if cfg!(target_os = "macos") {
            let _ = self.patch_makefile_for_brew(&cpp_dir_abs.join("Makefile"));
            let _ = patch_fr_backend_for_darwin(&cpp_dir_abs);
        }

        let cpp_dir = cpp_dir_rel.display().to_string();
        let made = self.exec("make", &["-C", &cpp_dir])?;
        if !made.ok {
            bail!(
                "{}\n\nIf this is a missing dependency on macOS, try:\n  brew install gmp nlohmann-json",
                made.message()
            );
        }

        if !bin_abs.is_file() {
            bail!("C++ witness binary not found at {}", bin_abs.display());
        }
        Ok(bin_rel)
    }

    /// Adds Homebrew's include and library paths to a generated Makefile.
    fn patch_makefile_for_brew(&self, makefile: &Path) -> Result<()> {
        if !makefile.is_file() {
            return Ok(());
        }
        let brew = self.exec("brew", &["--prefix"])?;
        if !brew.ok || brew.stdout.trim().is_empty() {
            return Ok(());
        }

        static CFLAGS: OnceLock<Regex> = OnceLock::new();
        let cflags = CFLAGS.get_or_init(|| Regex::new(r"(?m)^(CFLAGS=.*)$").expect("valid regex"));

        let mut text = fs::read_to_string(makefile)?;
        if !text.contains("BREW_PREFIX") && text.contains("CFLAGS=") {
            text = cflags
                .replace(
                    &text,
                    "$1\nBREW_PREFIX ?= $(shell brew --prefix 2>/dev/null)\n\
                     CFLAGS += -I$(BREW_PREFIX)/include\n\
                     LDFLAGS += -L$(BREW_PREFIX)/lib",
                )
                .into_owned();
        }

        // Link steps that pull in -lgmp must also honour LDFLAGS.
        text = text
            .lines()
            .map(|line| {
                if line.contains("-lgmp") && line.contains("$(CC)") && !line.contains("$(LDFLAGS)")
                {
                    line.replacen("$(CC) ", "$(CC) $(LDFLAGS) ", 1)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(makefile, text)?;
        Ok(())
    }
}

/// Retypes the generated field backend so Clang accepts GMP's `mpn_*` calls.
fn patch_fr_backend_for_darwin(cpp_dir: &Path) -> Result<()> {
    static RAW_ELEMENT: OnceLock<Regex> = OnceLock::new();
    static P_RAW_B: OnceLock<Regex> = OnceLock::new();
    static UINT64: OnceLock<Regex> = OnceLock::new();
    static FIRST_INCLUDE: OnceLock<Regex> = OnceLock::new();

    let raw_element = RAW_ELEMENT.get_or_init(|| {
        Regex::new(r"typedef\s+uint64_t\s+FrRawElement\[Fr_N64\];").expect("valid regex")
    });
    let p_raw_b = P_RAW_B.get_or_init(|| Regex::new(r"\buint64_t\s+pRawB\b").expect("valid regex"));
    let uint64 = UINT64.get_or_init(|| Regex::new(r"\buint64_t\b").expect("valid regex"));
    let first_include =
        FIRST_INCLUDE.get_or_init(|| Regex::new(r"(?m)^(#include[^\n]*\n)").expect("valid regex"));

    let header = cpp_dir.join("fr.hpp");
    if header.is_file() {
        let original = fs::read_to_string(&header)?;
        let mut text = raw_element
            .replace_all(&original, "typedef mp_limb_t FrRawElement[Fr_N64];")
            .into_owned();
        text = p_raw_b.replace_all(&text, "mp_limb_t pRawB").into_owned();

        // Some generated headers use the non-standard `uint` without a typedef.
        if text.contains("uint base") && !text.contains("<sys/types.h>") {
            const APPLE_BLOCK: &str =
                "\n#ifdef __APPLE__\n#include <sys/types.h> // typedef unsigned int uint;\n#endif // __APPLE__\n";
            text = if text.contains("#include <gmp.h>") {
                text.replace(
                    "#include <gmp.h>",
                    &format!("#include <gmp.h>{APPLE_BLOCK}"),
                )
            } else {
                first_include
                    .replace(&text, format!("${{1}}{APPLE_BLOCK}\n").as_str())
                    .into_owned()
            };
        }

        if text != original {
            fs::write(&header, text)?;
        }
    }

    let source = cpp_dir.join("fr.cpp");
    if source.is_file() {
        let text = fs::read_to_string(&source)?;
        let uses_gmp_backend = text.contains("mpn_add_n") || text.contains("mpn_mul_1");
        if uses_gmp_backend && text.contains("uint64_t") {
            fs::write(&source, uint64.replace_all(&text, "mp_limb_t").as_ref())?;
        }
    }

    Ok(())
}

impl Toolchain {
    /// Generates (or reuses) the proving and verification keys.
    fn ensure_keys(&self, spec: &Groth16Spec<'_>) -> Result<()> {
        self.ensure_circuit_compiled(spec)?;

        let r1cs_abs = self.resolve(spec.r1cs_file);
        let zkey_abs = self.resolve(spec.zkey_file);
        let vkey_abs = self.resolve(spec.vkey_file);

        let zkey_fresh = up_to_date(&r1cs_abs, &[&zkey_abs])
            && fs::metadata(&zkey_abs).is_ok_and(|m| m.len() > 0);
        let vkey_fresh = up_to_date(&r1cs_abs, &[&vkey_abs])
            && fs::metadata(&vkey_abs).is_ok_and(|m| m.len() > 0);

        if zkey_fresh && vkey_fresh {
            self.progress("  [setup] Using cached zkey/vkey (skipped snarkjs setup).");
            return Ok(());
        }

        if zkey_fresh {
            self.progress("  [setup] Exporting vkey from cached zkey...");
            self.export_verification_key(spec)?;
            return Ok(());
        }

        self.section("Setup (Generating Groth16 zkey/vkey)");
        self.progress(
            "  [setup] Generating Groth16 zkey (snarkjs groth16 setup — may take a few minutes)...",
        );
        let ptau_abs = self.resolve(spec.ptau_file);
        if !ptau_abs.is_file() {
            bail!(
                "Missing ptau file {}. Fetch it with scripts/fetch_ptau.sh.",
                ptau_abs.display()
            );
        }

        let setup = self.exec(
            &self.snarkjs,
            &[
                "groth16",
                "setup",
                &spec.r1cs_file.display().to_string(),
                &spec.ptau_file.display().to_string(),
                &spec.zkey_file.display().to_string(),
            ],
        )?;
        if !setup.ok {
            bail!("snarkjs groth16 setup failed:\n{}", setup.message());
        }
        assert_non_empty(&zkey_abs, "proving key (zkey)")?;

        self.progress("  [setup] Exporting verification key...");
        self.export_verification_key(spec)
    }

    fn export_verification_key(&self, spec: &Groth16Spec<'_>) -> Result<()> {
        let exported = self.exec(
            &self.snarkjs,
            &[
                "zkey",
                "export",
                "verificationkey",
                &spec.zkey_file.display().to_string(),
                &spec.vkey_file.display().to_string(),
            ],
        )?;
        if !exported.ok {
            bail!("snarkjs vkey export failed:\n{}", exported.message());
        }
        assert_non_empty(&self.resolve(spec.vkey_file), "verification key")
    }

    /// Checks that `rapidsnark` is reachable before a long setup starts.
    pub fn ensure_rapidsnark(&self) -> Result<()> {
        if self.rapidsnark.contains('/') {
            if !self.resolve(&self.rapidsnark).is_file() {
                bail!("rapidsnark binary not found at path: {}", self.rapidsnark);
            }
            return Ok(());
        }

        if which(&self.rapidsnark).is_some() {
            return Ok(());
        }
        bail!(
            "rapidsnark not found in PATH (looked for {:?}).\n\
             Install rapidsnark, or point to it via RAPIDSNARK_BIN.",
            self.rapidsnark
        );
    }

    /// Compiles the circuit, generates the keys and builds the witness binary.
    pub fn prepare_groth16(&self, spec: &Groth16Spec<'_>) -> Result<Artifacts> {
        self.ensure_rapidsnark()?;
        self.ensure_keys(spec)?;
        let witness_bin = self.ensure_witness_generator(spec)?;

        assert_non_empty(&self.resolve(spec.zkey_file), "proving key (zkey)")?;
        assert_non_empty(&self.resolve(spec.vkey_file), "verification key")?;
        assert_non_empty(&self.resolve(&witness_bin), "C++ witness binary")?;

        Ok(Artifacts {
            witness_bin,
            zkey: spec.zkey_file.to_path_buf(),
            vkey: spec.vkey_file.to_path_buf(),
            r1cs: spec.r1cs_file.to_path_buf(),
        })
    }

    /// Runs the witness calculator; paths are relative to the base directory.
    pub fn run_witness(&self, witness_bin: &Path, input: &Path, witness: &Path) -> Result<Output> {
        let program = self.resolve(witness_bin).display().to_string();
        self.exec(
            &program,
            &[&input.display().to_string(), &witness.display().to_string()],
        )
    }

    /// Runs rapidsnark over a witness.
    pub fn run_prover(
        &self,
        zkey: &Path,
        witness: &Path,
        proof: &Path,
        public: &Path,
    ) -> Result<Output> {
        self.exec(
            &self.rapidsnark,
            &[
                &zkey.display().to_string(),
                &witness.display().to_string(),
                &proof.display().to_string(),
                &public.display().to_string(),
            ],
        )
    }
}

fn file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("{} has no file name", path.display()))
}

/// Minimal `PATH` lookup, so a missing tool is reported before a long setup.
fn which(program: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(program))
            .find(|candidate| candidate.is_file())
    })
}

/// Deletes everything under `dir` except the `summary_*.json` files.
pub fn clean_artifacts_keep_summaries(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("summary_") && name.ends_with(".json") {
            continue;
        }
        let path = entry.path();
        let _ = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_prefers_stderr_for_diagnostics() {
        let out = Output {
            ok: false,
            stdout: "some stdout".into(),
            stderr: "  ".into(),
        };
        assert_eq!(out.message(), "some stdout");

        let err = Output {
            ok: false,
            stdout: "some stdout".into(),
            stderr: "the real error".into(),
        };
        assert_eq!(err.message(), "the real error");
    }

    #[test]
    fn spec_derives_sym_and_circuit_name() {
        let spec = Groth16Spec {
            circom_file: Path::new("./prove_verify.circom"),
            r1cs_file: Path::new("./generated/prove_verify.r1cs"),
            ptau_file: Path::new("../powersOfTau/pot19.ptau"),
            zkey_file: Path::new("./generated/circuit.zkey"),
            vkey_file: Path::new("./generated/vkey.json"),
            out_dir: Path::new("./generated"),
        };
        assert_eq!(spec.circuit_name(), "prove_verify");
        assert_eq!(spec.sym_file(), Path::new("./generated/prove_verify.sym"));
    }
}

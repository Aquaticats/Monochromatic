//! What:
//!  The `cargo` processes of the closure check:
//!  the tool probe (which also reads the `cargo-dylint` version),
//!  the offline dependency probe and the Dylint run.
//! Why:
//!  Every process the check starts is built here from fixed arguments,
//!  so the only inputs from outside are the paths of the package and the library
//!  and the `cargo` program that the machine owner selects through an environment variable.
//!  Nothing here fetches:
//!  the dependency probe runs offline and a cold cache skips the package.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function cargoProgram(): Path;
//! function dylintProbe(cargo: Path): string; // the version line; throws the reason to skip
//! function dependenciesMissing(cargo: Path, manifest: Path): string | undefined;
//! function runDylint(cargo: Path, library: Path, manifest: Path, setup: PackageSetup): Output; // throws string
//! ```

/// Import the package setup that adds variables and arguments to the run.
use crate::closure_probe::PackageSetup;
/// Import native paths and process types.
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// What:
///  The environment variable that replaces the `cargo` program.
/// Why:
///  The tests of the check point it at a stub,
///  and a machine owner may point it at a specific toolchain's `cargo`;
///  the setting is the machine's own and never comes from repository JSONC.
///  While it is set,
/// the check does not look for the compiler's `rustc-dev` component,
///  because the replacement stands for the whole toolchain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CARGO_VARIABLE = 'MONOCHROMATIC_LINT_CARGO';
/// ```
pub const CARGO_VARIABLE: &str = "MONOCHROMATIC_LINT_CARGO";

/// What:
///  The compiler flags that make Dylint report the check's own findings.
/// Why:
///  Both lints are requested at warning level whatever the environment holds,
///  and the linter maps each finding to the configured severity of its file.
///  Reporting an expectation that suppressed nothing keeps stale `expect` attributes visible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DYLINT_FLAGS = '-W avoidable_closure -W unfulfilled_lint_expectations';
/// ```
pub const DYLINT_FLAGS: &str = "-W avoidable_closure -W unfulfilled_lint_expectations";

/// What:
///  The result of one finished `cargo dylint` run.
/// Why:
///  The caller reads the machine-readable records from standard output
///  and quotes standard error when the run failed without any record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DylintRun = { success: boolean; status: string; stdout: Uint8Array; stderr: string };
/// ```
#[derive(Debug)]
pub struct DylintRun {
    /// Whether the process exited with status zero.
    pub success: bool,
    /// The exit status as text,
    ///  for the failure message.
    pub status: String,
    /// Standard output:
    ///  cargo's JSON records.
    pub stdout: Vec<u8>,
    /// Standard error:
    ///  build progress and tool messages.
    pub stderr: String,
}

/// What:
///  The `cargo` program to run.
/// Why:
///  The default is the `cargo` found on `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cargoProgram(): Path { return process.env.MONOCHROMATIC_LINT_CARGO ?? 'cargo'; }
/// ```
pub fn cargo_program() -> PathBuf {
    return match std::env::var_os(CARGO_VARIABLE) {
        Some(program) => PathBuf::from(program),
        None => PathBuf::from("cargo"),
    };
}

/// What:
///  Check that `cargo dylint` runs on this machine and return what `--version` printed.
/// Why:
///  A machine without `cargo-dylint` skips every package with one note instead of failing them one by one,
///  and the printed line carries the version that the pinned driver crate needs,
///  so one process answers both questions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dylintProbe(cargo: Path): string; // throws string, the reason to skip
/// ```
pub fn dylint_probe(cargo: &Path) -> Result<String, String> {
    let output = match Command::new(cargo)
        .args(["dylint", "--version"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(done) => done,
        Err(error) => {
            return Err(format!(
                "cargo does not run ({error}); install the Rust toolchain, cargo-dylint and dylint-link with mise install"
            ));
        }
    };
    if !output.status.success() {
        return Err(format!(
            "`cargo dylint --version` failed ({}); install cargo-dylint and dylint-link with mise install",
            output.status
        ));
    }
    return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
}

/// What:
///  Explain a missing `rustc-dev` component in the compiler installed at a sysroot.
/// Why:
///  A Dylint library links the compiler's own crates,
/// which the component installs as `librustc_middle-*` files
///  in each target's library directory;
///  without them the library cannot build,
///  which is a gap of the machine and not a failure of any package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rustcDevMissingIn(sysroot: Path): string | undefined;
/// ```
pub fn rustc_dev_missing_in(sysroot: &Path) -> Option<String> {
    let rustlib: PathBuf = sysroot.join("lib").join("rustlib");
    let Ok(targets) = std::fs::read_dir(&rustlib) else {
        return Some(format!(
            "the compiler's library directory {} cannot be read",
            rustlib.display()
        ));
    };
    for target in targets.flatten() {
        let Ok(libraries) = std::fs::read_dir(target.path().join("lib")) else {
            continue;
        };
        for library in libraries.flatten() {
            if library
                .file_name()
                .to_string_lossy()
                .starts_with("librustc_middle-")
            {
                return None;
            }
        }
    }
    return Some(String::from(
        "the active compiler lacks the rustc-dev component; add it with `rustup component add rustc-dev` or mise install",
    ));
}

/// What:
///  Explain a missing `rustc-dev` component in the active compiler,
///  unless a replacement `cargo` is in use.
/// Why:
///  The active compiler is the one `rustc --print sysroot` names.
///  A replacement `cargo` (anything but the bare program name) stands for the whole toolchain,
/// so it is not inspected;
///  the decision follows the program the step was given,
///  not the process environment,
///  so a test that passes a stub is independent of the machine's compiler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rustcDevMissing(cargo: Path): string | undefined;
/// ```
pub fn rustc_dev_missing(cargo: &Path) -> Option<String> {
    if cargo != Path::new("cargo") {
        return None;
    }
    let output = match Command::new("rustc")
        .args(["--print", "sysroot"])
        .stdin(Stdio::null())
        .output()
    {
        Ok(done) => done,
        Err(error) => return Some(format!("rustc does not run ({error})")),
    };
    let sysroot: String = String::from_utf8_lossy(&output.stdout).trim().to_string();
    return rustc_dev_missing_in(Path::new(&sysroot));
}

/// What:
///  Check that the package's dependencies,
/// for every platform in its lockfile,
/// are already in the cargo cache.
/// Why:
///  Dylint resolves the dependencies of every platform,
/// and this program never fetches.
///  A cold cache therefore skips the package with the command that fills it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dependenciesMissing(cargo: Path, manifest: Path): string | undefined;
/// ```
pub fn dependencies_missing(cargo: &Path, manifest: &Path) -> Option<String> {
    let output = match Command::new(cargo)
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output()
    {
        Ok(done) => done,
        Err(error) => return Some(format!("cargo does not run ({error})")),
    };
    if output.status.success() {
        return None;
    }
    let directory: String = manifest.parent().map_or_else(String::new, directory_text);
    return Some(format!(
        "its dependencies are not all in the cargo cache or its lockfile is stale (`cargo metadata --offline --locked` failed); run `cargo fetch --locked` in {directory}, which this linter does not do. First line of cargo's message: {}",
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .next()
            .unwrap_or("none")
    ));
}

/// What:
///  Render a directory path as text for a message.
/// Why:
///  A named function is the mapping that `Option::map_or_else` takes without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directoryText(directory: Path): string;
/// ```
fn directory_text(directory: &Path) -> String {
    return directory.to_string_lossy().into_owned();
}

/// What:
///  Run `cargo dylint` for one package with the library at a path.
/// Why:
///  `--path` makes Dylint build the library with the active toolchain and load it,
///  and the arguments after `--` go to `cargo check`:
///  `--locked` refuses a lockfile change and the JSON message format gives positions in bytes.
///  The default targets (library and binaries) are checked,
///  so tests,
/// examples and benchmarks are not.
///  The run is offline;
///  `DYLINT_RUSTFLAGS` is set here and replaces any inherited value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runDylint(cargo: Path, library: Path, manifest: Path, setup: PackageSetup): DylintRun; // throws string
/// ```
pub fn run_dylint(
    cargo: &Path,
    library: &Path,
    manifest: &Path,
    setup: &PackageSetup,
) -> Result<DylintRun, String> {
    let mut command: Command = Command::new(cargo);
    command
        .arg("dylint")
        .arg("--path")
        .arg(library)
        .arg("--manifest-path")
        .arg(manifest)
        .args(["--", "--locked", "--message-format", "json"])
        .args(setup.arguments.as_slice())
        .env("CARGO_NET_OFFLINE", "true")
        .env("DYLINT_RUSTFLAGS", DYLINT_FLAGS)
        .stdin(Stdio::null());
    for (name, value) in &setup.environment {
        command.env(OsString::from(name), OsString::from(value));
    }
    let output = match command.output() {
        Ok(done) => done,
        Err(error) => return Err(format!("cargo dylint does not start: {error}")),
    };
    return Ok(DylintRun {
        success: output.status.success(),
        status: output.status.to_string(),
        stdout: output.stdout,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    });
}

/// Process construction controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_cargo_tests.rs"]
mod tests;

//! What:
//!  Where this machine keeps the Dylint driver whose absence skips a package,
//!  and what the machine reports about that driver and about its cargo cache:
//!  the active toolchain,
//!  the version the driver binary prints,
//!  and whether the pinned `dylint_driver` crate resolves offline.
//! Why:
//!  `cargo-dylint` keeps one driver per toolchain in `DYLINT_DRIVER_PATH`
//!  or in `.dylint_drivers` under the home directory,
//!  and writes the crate it builds a driver from into the per-user cache directory,
//!  so a check that never fetches has to be told both places before it reads anything.
//!  Every fact here is a record or one command's output,
//!  which is what lets the controls pass their own machine
//!  and change no process-wide variable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DriverSite = { rustup: Path; drivers: Path | undefined; scratch: Path | undefined };
//! function driverSiteFromEnvironment(): DriverSite;
//! function activeToolchain(rustup: Path, directory: Path): string; // throws string
//! function driverVersion(driver: Path): [number, number, number] | undefined;
//! function driverUnresolved(cargo: Path, scratch: Path, version: string): string | undefined; // throws string
//! ```

/// Import native paths for the directories and programs the facts name.
use std::path::{
    Path,
    PathBuf,
};
/// Import the process types of the two commands that answer a fact.
use std::process::{
    Command,
    Stdio,
};

/// What:
///  The machine facts the driver check reads.
/// Why:
///  The facts come from the process environment in production and are explicit in tests,
///  so no test changes process-wide variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DriverSite = { rustup: Path; drivers: Path | undefined; scratch: Path | undefined };
/// ```
#[derive(Clone, Debug)]
pub struct DriverSite {
    /// The `rustup` program that names the active toolchain.
    pub rustup: PathBuf,
    /// The directory of the driver cache:
    ///  `DYLINT_DRIVER_PATH`,
    ///  or `.dylint_drivers` in the home directory,
    ///  or nothing when neither is known.
    pub drivers: Option<PathBuf>,
    /// The directory in which the throwaway crate is written,
    ///  or nothing when no per-user cache directory is known.
    pub scratch: Option<PathBuf>,
}

/// What:
///  The default directory of the Dylint drivers in a home directory.
/// Why:
///  A named function can stand in for the closure that `Option::map` would run over the home directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dylintDriversDirectory(home: Path): Path;
/// ```
fn dylint_drivers_directory(home: &Path) -> PathBuf {
    return home.join(".dylint_drivers");
}

/// What:
///  The per-user cache directory in a home directory.
/// Why:
///  A named function can stand in for the closure that `Option::map` would run over the home directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cacheDirectory(home: Path): Path;
/// ```
fn cache_directory(home: &Path) -> PathBuf {
    return home.join(".cache");
}

/// What:
///  This tool's scratch directory in a cache directory.
/// Why:
///  A named function can stand in for the closure that `Option::map` would run over the cache directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scratchDirectory(cache: Path): Path;
/// ```
fn scratch_directory(cache: &Path) -> PathBuf {
    return cache.join("monochromatic-lint");
}

/// What:
///  Read the machine's facts from the process environment.
/// Why:
///  `DYLINT_DRIVER_PATH` replaces the default driver directory as it does for `cargo-dylint`.
///  The scratch directory is under the per-user cache directory,
///  because the system temporary directory is not a place this repository writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function driverSiteFromEnvironment(): DriverSite;
/// ```
pub fn driver_site_from_environment() -> DriverSite {
    let home: Option<PathBuf> = std::env::home_dir();
    let drivers: Option<PathBuf> = match std::env::var_os("DYLINT_DRIVER_PATH") {
        Some(set) => Some(PathBuf::from(set)),
        None => home.as_deref().map(dylint_drivers_directory),
    };
    let cache: Option<PathBuf> = match std::env::var_os("XDG_CACHE_HOME") {
        Some(set) if !set.is_empty() => Some(PathBuf::from(set)),
        _ => home.as_deref().map(cache_directory),
    };
    return DriverSite {
        rustup: PathBuf::from("rustup"),
        drivers,
        scratch: cache.as_deref().map(scratch_directory),
    };
}

/// What:
///  Parse a plain `major.minor.patch` version from the last word of a tool's version line.
/// Why:
///  `cargo dylint --version` prints `cargo-dylint 6.3.0` and the driver's `-V` prints a line ending in its version;
///  upstream reads the last space-separated word of each.
///  A version with a pre-release or build suffix is not read,
///  because comparing it needs the full ordering rules and Dylint releases do not use them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseVersion(line: string): [number, number, number] | undefined;
/// ```
pub fn parse_version(line: &str) -> Option<(u64, u64, u64)> {
    let word: &str = line.split_whitespace().last()?;
    let mut parts = word.split('.');
    let major: u64 = parts.next()?.parse::<u64>().ok()?;
    let minor: u64 = parts.next()?.parse::<u64>().ok()?;
    let patch: u64 = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    return Some((major, minor, patch));
}

/// What:
///  The first word of the output of `rustup show active-toolchain` in a directory.
/// Why:
///  That word names the driver directory.
///  The environment is cleaned as `cargo-dylint` cleans it,
///  because an inherited `RUSTUP_TOOLCHAIN` is not what it asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function activeToolchain(rustup: Path, directory: Path): string; // throws string
/// ```
pub(crate) fn active_toolchain(
    rustup: &Path,
    directory: &Path,
) -> Result<String, String> {
    let output = match Command::new(rustup)
        .args(["show", "active-toolchain"])
        .current_dir(directory)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env_remove("CARGO")
        .env_remove("RUSTC")
        .stdin(Stdio::null())
        .output()
    {
        Ok(done) => done,
        Err(error) => return Err(format!("rustup does not run ({error})")),
    };
    if !output.status.success() {
        return Err(format!(
            "`rustup show active-toolchain` failed ({})",
            output.status
        ));
    }
    return match String::from_utf8_lossy(&output.stdout)
        .split_ascii_whitespace()
        .next()
    {
        Some(word) => Ok(word.to_string()),
        None => Err(String::from(
            "`rustup show active-toolchain` printed no toolchain",
        )),
    };
}

/// What:
///  The version a driver binary reports,
///  or nothing when it is absent or does not report one.
/// Why:
///  Upstream treats a driver that fails to run or to report a version as outdated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function driverVersion(driver: Path): [number, number, number] | undefined;
/// ```
pub(crate) fn driver_version(driver: &Path) -> Option<(u64, u64, u64)> {
    if !driver.is_file() {
        return None;
    }
    let output = Command::new(driver)
        .arg("-V")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    return parse_version(String::from_utf8_lossy(&output.stdout).trim_end());
}

/// What:
///  The manifest of the throwaway crate that `cargo-dylint` builds a driver from,
///  without the toolchain-specific name.
/// Why:
///  Resolving it offline answers whether the registry holds the pinned `dylint_driver`
///  and everything it depends on.
///  The empty `[workspace]` table keeps cargo from joining a workspace above the scratch directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probeManifest(version: string): string;
/// ```
pub(crate) fn probe_manifest(version: &str) -> String {
    return format!(
        "[package]\nname = \"dylint-driver-probe\"\nversion = \"0.1.0\"\nedition = \"2018\"\n\n[workspace]\n\n[dependencies]\nanyhow = \"1.0\"\nenv_logger = \"0.11\"\ndylint_driver = {{ version = \"={version}\" }}\n"
    );
}

/// What:
///  Resolve the throwaway driver crate offline.
/// Why:
///  A resolution that succeeds means `cargo dylint` can build the driver without the network;
///  the first error line of cargo's message explains a failure.
///  The crate lives in a directory named for this process and is removed afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function driverUnresolved(cargo: Path, scratch: Path, version: string): string | undefined; // throws string
/// ```
pub(crate) fn driver_unresolved(
    cargo: &Path,
    scratch: &Path,
    version: &str,
) -> Result<Option<String>, String> {
    let run_directory: PathBuf = scratch.join(format!("driver-probe-{}", std::process::id()));
    let package: PathBuf = run_directory.join("dylint-driver-probe");
    let prepared: std::io::Result<()> = prepare_probe(&package, version);
    let verdict: Result<Option<String>, String> = match prepared {
        Err(error) => Err(format!(
            "the scratch crate in {} cannot be written ({error})",
            package.display()
        )),
        Ok(()) => resolve_probe(cargo, &package),
    };
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(&run_directory);
    return verdict;
}

/// Write the throwaway crate.
fn prepare_probe(
    package: &Path,
    version: &str,
) -> std::io::Result<()> {
    std::fs::create_dir_all(package.join("src"))?;
    std::fs::write(package.join("Cargo.toml"), probe_manifest(version))?;
    return std::fs::write(package.join("src").join("main.rs"), "fn main() {}\n");
}

/// What:
///  Whether a line of Cargo's standard error is an error line.
/// Why:
///  A named function can stand in for the closure that `Iterator::find` would run over the lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isErrorLine(line: string): boolean;
/// ```
fn is_error_line(line: &&str) -> bool {
    return line.starts_with("error");
}

/// What:
///  Run the offline resolution of the throwaway crate.
/// Why:
///  The run starts in the crate's directory,
///  as the build of `cargo-dylint` does in its own temporary directory,
///  so no configuration of the repository applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveProbe(cargo: Path, package: Path): string | undefined; // throws string
/// ```
fn resolve_probe(
    cargo: &Path,
    package: &Path,
) -> Result<Option<String>, String> {
    let output = match Command::new(cargo)
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(package.join("Cargo.toml"))
        .current_dir(package)
        .env_remove("RUSTUP_TOOLCHAIN")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output()
    {
        Ok(done) => done,
        Err(error) => return Err(format!("cargo does not run ({error})")),
    };
    if output.status.success() {
        return Ok(None);
    }
    // Cargo prints progress notes such as "Locking 81 packages" before its error, so the first error line is quoted.
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    let explanation: &str = stderr
        .lines()
        .find(is_error_line)
        .or(stderr.lines().next())
        .unwrap_or("none");
    return Ok(Some(
        explanation
            .strip_prefix("error: ")
            .unwrap_or(explanation)
            .to_string(),
    ));
}

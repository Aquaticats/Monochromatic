//! What:
//!  The offline check that the Dylint driver `cargo dylint` needs exists or can be built from the cargo cache.
//! Why:
//!  `cargo dylint` keeps one driver binary per toolchain and rebuilds it whenever its version is lower than the
//!  version of `cargo-dylint` itself,
//!  from a throwaway crate that pins `dylint_driver = "=<version of cargo-dylint>"`
//!  (`dylint/src/driver_builder.rs` in the `dylint` crate:
//!  `get`,
//!  `is_outdated`,
//!  `initialize` and `cargo_toml`).
//!  That crate belongs to no package lockfile,
//!  so the dependency probe of the package never sees it,
//!  and this program never fetches:
//!  after a `cargo-dylint` upgrade the offline rebuild failed with exit status 2 instead of a skip.
//!  The check mirrors upstream's decisions,
//!  reading nothing it would have to write:
//!  the toolchain is the first word of `rustup show active-toolchain` in the library directory
//!  (`dylint_internal/src/rustup.rs`, `active_toolchain`),
//!  the driver is `<drivers>/<toolchain>/dylint-driver`
//!  with `<drivers>` being `DYLINT_DRIVER_PATH` or `<home>/.dylint_drivers`,
//!  and a driver counts as current unless its `-V` fails or reports a version below `cargo-dylint`'s.
//!  A driver that is not current is acceptable when `cargo metadata --offline` resolves the same throwaway crate.
//!  Doc: `doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DriverState = { kind: 'ready' } | { kind: 'missing'; reason: string } | { kind: 'unknown'; note: string };
//! function driverState(cargo: Path, site: DriverSite, library: Path, version: string, warm: string): DriverState;
//! function driverGaps(packages: ClosurePackage[], cargo: Path, site: DriverSite, version: string, cwd: Path): Gaps;
//! ```

/// Import the package grouping and the display of paths.
use crate::{closure_packages::ClosurePackage, run_paths::display_name};
/// Import native paths and the process types.
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
///  The result of checking one library's driver.
/// Why:
///  A machine that lacks the driver is a skip,
///  and a check that could not be made must not hide the run,
///  so it falls through to the run itself and is noted for `--debug`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DriverState = { kind: 'ready' } | { kind: 'missing'; reason: string } | { kind: 'unknown'; note: string };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum DriverState {
    /// A current driver exists,
    ///  or the cargo cache can build one.
    Ready,
    /// The driver is absent or outdated and the cargo cache cannot build one.
    Missing(String),
    /// The check itself could not be made.
    Unknown(String),
}

/// What:
///  The libraries whose packages are skipped and the debug notes of the checks.
/// Why:
///  The caller turns each gap into one skip message and leaves the packages of those libraries out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gaps = { gaps: Gap[]; notes: string[] };
/// ```
#[derive(Debug, Default)]
pub struct DriverGaps {
    /// One entry per library whose driver is missing.
    pub gaps: Vec<DriverGap>,
    /// Notes for checks that could not be made.
    pub notes: Vec<String>,
}

/// What:
///  One library whose driver is missing and the packages that would have used it.
/// Why:
///  The message names the packages as a group,
///  so a repository with many packages gets one message per library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gap = { library: Path; subject: string; reason: string };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct DriverGap {
    /// The library directory.
    pub library: PathBuf,
    /// Which packages are skipped,
    ///  as the subject of the skip message.
    pub subject: String,
    /// Why they are skipped,
    ///  including the command that fills the cache.
    pub reason: String,
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
        None => home
            .as_ref()
            .map(|directory| return directory.join(".dylint_drivers")),
    };
    let cache: Option<PathBuf> = match std::env::var_os("XDG_CACHE_HOME") {
        Some(set) if !set.is_empty() => Some(PathBuf::from(set)),
        _ => home.map(|directory| return directory.join(".cache")),
    };
    return DriverSite {
        rustup: PathBuf::from("rustup"),
        drivers,
        scratch: cache.map(|directory| return directory.join("monochromatic-lint")),
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
fn active_toolchain(rustup: &Path, directory: &Path) -> Result<String, String> {
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
fn driver_version(driver: &Path) -> Option<(u64, u64, u64)> {
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
fn probe_manifest(version: &str) -> String {
    return format!(
        "[package]\nname = \"dylint-driver-probe\"\nversion = \"0.1.0\"\nedition = \"2018\"\n\n[workspace]\n\n[dependencies]\nanyhow = \"1.0\"\nenv_logger = \"0.11\"\ndylint_driver = {{ version = \"={version}\" }}\n"
    );
}

/// What:
///  Resolve the throwaway driver crate offline.
/// Why:
///  A resolution that succeeds means `cargo dylint` can build the driver without the network;
///  the first line of cargo's message explains a failure.
///  The crate lives in a directory named for this process and is removed afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function driverUnresolved(cargo: Path, scratch: Path, version: string): string | undefined; // throws string
/// ```
fn driver_unresolved(
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
fn prepare_probe(package: &Path, version: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(package.join("src"))?;
    std::fs::write(package.join("Cargo.toml"), probe_manifest(version))?;
    return std::fs::write(package.join("src").join("main.rs"), "fn main() {}\n");
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
fn resolve_probe(cargo: &Path, package: &Path) -> Result<Option<String>, String> {
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
    return Ok(Some(
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .next()
            .unwrap_or("none")
            .to_string(),
    ));
}

/// What:
///  Format a version as dotted text.
/// Why:
///  Messages quote versions in the form the tools print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dotted(version: [number, number, number]): string;
/// ```
fn dotted(version: (u64, u64, u64)) -> String {
    return format!("{}.{}.{}", version.0, version.1, version.2);
}

/// What:
///  Decide whether the driver for one library is ready.
/// Why:
///  The order follows `cargo-dylint`:
///  a current driver is reused,
///  and only otherwise does it build,
///  which needs the pinned crates in the cargo cache.
///  `warm` is the command that fills the caches and is quoted in the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function driverState(cargo: Path, site: DriverSite, library: Path, needed: [number, number, number], warm: string): DriverState;
/// ```
pub fn driver_state(
    cargo: &Path,
    site: &DriverSite,
    library: &Path,
    needed: (u64, u64, u64),
    warm: &str,
) -> DriverState {
    let Some(drivers) = site.drivers.as_ref() else {
        return DriverState::Unknown(String::from(
            "no driver directory is known (no DYLINT_DRIVER_PATH and no home directory)",
        ));
    };
    let toolchain: String = match active_toolchain(&site.rustup, library) {
        Ok(name) => name,
        Err(reason) => return DriverState::Unknown(reason),
    };
    let driver: PathBuf = drivers.join(&toolchain).join("dylint-driver");
    let found: Option<(u64, u64, u64)> = driver_version(&driver);
    if found.is_some_and(|version| return version >= needed) {
        return DriverState::Ready;
    }
    let Some(scratch) = site.scratch.as_ref() else {
        return DriverState::Unknown(String::from(
            "no cache directory is known to resolve the driver crate in",
        ));
    };
    let refusal: String = match driver_unresolved(cargo, scratch, &dotted(needed)) {
        Ok(None) => return DriverState::Ready,
        Ok(Some(first_line)) => first_line,
        Err(reason) => return DriverState::Unknown(reason),
    };
    let held: String = match found {
        Some(version) => format!(
            "reports version {}, below the {} of cargo-dylint",
            dotted(version),
            dotted(needed)
        ),
        None => format!("is absent (cargo-dylint is {})", dotted(needed)),
    };
    return DriverState::Missing(format!(
        "the Dylint driver for toolchain `{toolchain}` at {} {held}, and `dylint_driver` {} with its dependencies is not in the cargo cache, so `cargo dylint` would have to download it, which this linter does not do; run `{warm}` once with network access (it writes the cargo registry and the Dylint driver directory, and is needed again after each cargo-dylint upgrade). First line of cargo's message: {refusal}",
        driver.display(),
        dotted(needed)
    ));
}

/// What:
///  The command that fills the caches,
///  for a library and a package manifest.
/// Why:
///  A normal `cargo dylint` run without `CARGO_NET_OFFLINE` refreshes the index,
///  downloads the pinned crates,
///  builds the driver and checks the package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function warmCommand(library: string, manifest: string): string;
/// ```
pub fn warm_command(library: &str, manifest: &str) -> String {
    return format!(
        "env --unset=CARGO_NET_OFFLINE cargo dylint --path {library} --manifest-path {manifest} -- --locked"
    );
}

/// What:
///  Check the driver once per distinct library of the packages.
/// Why:
///  The driver depends on the toolchain active in the library directory and not on the package,
///  so a repository of many packages with one library is checked once.
///  A `cargo-dylint` version that cannot be read is noted and the run proceeds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function driverGaps(packages: ClosurePackage[], cargo: Path, site: DriverSite, version: string, cwd: Path): DriverGaps;
/// ```
pub fn driver_gaps(
    packages: &[ClosurePackage],
    cargo: &Path,
    site: &DriverSite,
    version_text: &str,
    cwd: &Path,
) -> DriverGaps {
    let mut result: DriverGaps = DriverGaps::default();
    let Some(needed) = parse_version(version_text) else {
        result.notes.push(format!(
            "the Dylint driver check did not run: `cargo dylint --version` printed no readable version ({})",
            version_text.trim()
        ));
        return result;
    };
    let mut seen: Vec<&PathBuf> = Vec::<&PathBuf>::new();
    for package in packages {
        if seen.contains(&&package.library) {
            continue;
        }
        seen.push(&package.library);
        let library_name: String = display_name(&package.library, cwd);
        let warm: String = warm_command(
            library_name.as_str(),
            display_name(&package.manifest, cwd).as_str(),
        );
        match driver_state(cargo, site, &package.library, needed, warm.as_str()) {
            DriverState::Ready => {}
            DriverState::Unknown(note) => result.notes.push(format!(
                "the Dylint driver check for the library {library_name} did not run: {note}"
            )),
            DriverState::Missing(reason) => {
                let count: usize = packages
                    .iter()
                    .filter(|other| return other.library == package.library)
                    .count();
                let subject: String = if count == packages.len() {
                    format!("all {count} package(s)")
                } else {
                    format!("{count} package(s) checked with the library {library_name}")
                };
                result.gaps.push(DriverGap {
                    library: package.library.clone(),
                    subject,
                    reason,
                });
            }
        }
    }
    return result;
}

/// Driver check controls stay outside release artifacts.
#[cfg(test)]
#[path = "closure_driver_tests.rs"]
mod tests;

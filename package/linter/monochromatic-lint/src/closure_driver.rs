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
//!  (`dylint_internal/src/rustup.rs`,
//! `active_toolchain`),
//!  the driver is `<drivers>/<toolchain>/dylint-driver`
//!  with `<drivers>` being `DYLINT_DRIVER_PATH` or `<home>/.dylint_drivers`,
//!  and a driver counts as current unless its `-V` fails or reports a version below `cargo-dylint`'s.
//!  A driver that is not current is acceptable when `cargo metadata --offline` resolves the same throwaway crate.
//!  The facts that are read here and the reads themselves live in `closure_driver_probe`,
//!  and this module turns them into one verdict per library.
//!  Doc:
//! `doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DriverState = { kind: 'ready' } | { kind: 'missing'; reason: string } | { kind: 'unknown'; note: string };
//! function driverState(cargo: Path, site: DriverSite, library: Path, version: string, warm: string): DriverState;
//! function driverGaps(packages: ClosurePackage[], cargo: Path, site: DriverSite, version: string, cwd: Path): Gaps;
//! ```

/// Import the probe manifest,
///  which the controls below name.
#[cfg(test)]
use crate::closure_driver_probe::probe_manifest;
/// Re-export the machine facts and the reader that fills them,
///  so a caller asks the driver check for both.
pub use crate::closure_driver_probe::{
    DriverSite,
    driver_site_from_environment,
};
/// Import the readers of one machine and the package grouping.
use crate::{
    closure_driver_probe::{
        active_toolchain,
        driver_unresolved,
        driver_version,
        parse_version,
    },
    closure_packages::ClosurePackage,
    run_paths::display_name,
};
/// Import native paths of the library and of the driver the check looks for.
use std::path::{
    Path,
    PathBuf,
};

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
    Missing(
        /// Why the driver cannot be built,
        /// as the first error line of the failed resolve.
        String,
    ),
    /// The check itself could not be made.
    Unknown(
        /// What stopped the check.
        String,
    ),
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
///  Whether a library's driver is among the missing ones.
/// Why:
///  The packages of such a library are skipped instead of checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gapsHoldLibrary(gaps: Gap[], library: Path): boolean;
/// ```
pub fn gaps_hold_library(
    gaps: &[DriverGap],
    library: &Path,
) -> bool {
    for gap in gaps {
        if gap.library.as_path() == library {
            return true;
        }
    }
    return false;
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
    if matches!(found, Some(version) if version >= needed) {
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
        "the Dylint driver for toolchain `{toolchain}` at {} {held}, and `dylint_driver` {} with its dependencies is not in the cargo cache, so `cargo dylint` would have to download it, which this linter does not do; run `{warm}` once with network access (it writes the cargo registry and the Dylint driver directory, and is needed again after each cargo-dylint upgrade). Cargo's error: {refusal}",
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
pub fn warm_command(
    library: &str,
    manifest: &str,
) -> String {
    return format!(
        "env --unset=CARGO_NET_OFFLINE cargo dylint --path {library} --manifest-path {manifest} -- --locked"
    );
}

/// What:
///  Count the packages that use one library.
/// Why:
///  The skip message says whether a missing driver costs every package or only some of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packagesUsing(packages: ClosurePackage[], library: Path): number;
/// ```
fn packages_using(
    packages: &[ClosurePackage],
    library: &Path,
) -> usize {
    let mut count: usize = 0;
    for other in packages {
        if other.library.as_path() == library {
            count += 1;
        }
    }
    return count;
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
                let count: usize = packages_using(packages, &package.library);
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

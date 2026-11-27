//! What:
//! The host's platform id and OS family,
//! and whether a tool whose plan entry carries an `os` filter installs on one.
//! Why:
//! The lock's `artifacts` keys are platform ids,
//! and a plan entry's `os` filter is an OS family,
//! so both are named here once.
//! `platform_id` is pure,
//! so a test can ask for every combination of operating system,
//! CPU and C library the lock names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function platformId(os: string, arch: string, musl: boolean): string
//! export function currentPlatformId(): string
//! export function currentFamily(): OsFamily | undefined
//! export function appliesToFamily(os: OsFamily[] | undefined, family: OsFamily | undefined): boolean
//! ```
/// Imports the OS family type the plan's `os` filter is written in.
use super::super::model::OsFamily;

/// What:
/// The platform id the lock's `artifacts` keys use,
/// from the operating system,
/// the CPU and the C library:
///           `linux-x64`,
/// `linux-arm64-musl`,
/// `macos-arm64`,
/// `windows-x64`.
/// Why:
/// Pure so tests can ask for every combination;
/// `current_platform_id` supplies the running host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function platformId(os: string, arch: string, musl: boolean): string
/// ```
pub fn platform_id(
    os: &str,
    arch: &str,
    musl: bool,
) -> String {
    let cpu: &str = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let family: &str = match os {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    };
    if family == "linux" && musl {
        return format!("linux-{cpu}-musl");
    }
    return format!("{family}-{cpu}");
}

/// The platform id of the running host (a musl build runs on a musl host,
/// so the build's C library is the host's).
pub fn current_platform_id() -> String {
    return platform_id(
        std::env::consts::OS,
        std::env::consts::ARCH,
        cfg!(target_env = "musl"),
    );
}

/// The OS family of the running host,
/// or `None` for a family the plan cannot name.
pub fn current_family() -> Option<OsFamily> {
    return OsFamily::parse(std::env::consts::OS);
}

/// Whether a tool with this OS filter installs on `family` (no filter means every family).
pub fn applies_to_family(
    os: &Option<Vec<OsFamily>>,
    family: Option<OsFamily>,
) -> bool {
    let Some(allowed) = os else {
        return true;
    };
    let Some(current) = family else {
        return false;
    };
    return allowed.contains(&current);
}

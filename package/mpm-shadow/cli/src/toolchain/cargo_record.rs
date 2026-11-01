//! What:
//! Cargo's own install record,
//! `<cargo home>/.crates.toml` (the `[v1]` file),
//! read as crate,
//! version and the binaries each install placed.
//! Why:
//! `cargo uninstall` decides from this file alone
//! (measured 2026-11-01 with cargo 1.101.0-nightly in disposable homes:
//! a crate named only in `.crates2.json` is "did not match any packages"),
//! and the old-copy cleanup needs to know which crate a binary in the shared cargo bin directory belongs to.
//! `src/installed.rs` reads the same file for versions only and ignores the binary lists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cargo-record.ts
//! export type RecordedInstall = { crateName: string; version: string; bins: string[] };
//! export function parseRecordedInstalls(text: string): RecordedInstall[]
//! export function installNamingBin(installs: RecordedInstall[], bin: string): RecordedInstall | undefined
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// One install the record names:
/// the crate,
/// its version and the binaries it placed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedInstall {
    /// The crate name.
    pub crate_name: String,
    /// The version cargo recorded.
    pub version: String,
    /// The binaries the install placed in the cargo bin directory.
    pub bins: Vec<String>,
}

/// Every text between a pair of double quotes in `text`,
/// in order (the record never escapes a quote inside a name).
fn quoted_texts(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for character in text.chars() {
        if character == '"' {
            match current.take() {
                Some(finished) => found.push(finished),
                None => current = Some(String::new()),
            }
        } else if let Some(open) = current.as_mut() {
            open.push(character);
        }
    }
    return found;
}

/// The crate and version of a record key such as `monochromatic-lint 0.4.0 (registry+https://...)`.
fn name_and_version(key: &str) -> Option<(String, String)> {
    let mut words = key.split_whitespace();
    let name: &str = words.next()?;
    let version: &str = words.next()?;
    return Some((name.to_string(), version.to_string()));
}

/// What:
/// The installs a `.crates.toml` text names.
/// Why:
/// One entry is `"<crate> <version> (<source>)" = ["bin", ...]`,
/// and a long list is written one binary per line;
/// the keys and binary names never hold a quote,
/// so a line scan is exact and no TOML parser is needed.
/// A line that does not open an entry is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRecordedInstalls(text: string): RecordedInstall[]
/// ```
pub fn parse_recorded_installs(text: &str) -> Vec<RecordedInstall> {
    let mut installs: Vec<RecordedInstall> = Vec::new();
    let mut open: Option<RecordedInstall> = None;
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if let Some(mut install) = open.take() {
            let (names, closed): (&str, bool) = match trimmed.split_once(']') {
                Some((before, _)) => (before, true),
                None => (trimmed, false),
            };
            install.bins.extend(quoted_texts(names));
            if closed {
                installs.push(install);
            } else {
                open = Some(install);
            }
            continue;
        }
        let Some(after_quote) = trimmed.strip_prefix('"') else {
            continue;
        };
        let Some((key, rest)) = after_quote.split_once('"') else {
            continue;
        };
        let Some((crate_name, version)) = name_and_version(key) else {
            continue;
        };
        let Some((_, list)) = rest.split_once('[') else {
            continue;
        };
        let (names, closed): (&str, bool) = match list.split_once(']') {
            Some((before, _)) => (before, true),
            None => (list, false),
        };
        let install: RecordedInstall = RecordedInstall { crate_name, version, bins: quoted_texts(names) };
        if closed {
            installs.push(install);
        } else {
            open = Some(install);
        }
    }
    return installs;
}

/// What:
/// The installs `<cargo home>/.crates.toml` names;
/// a missing or unreadable file names none.
/// Why:
/// No record means cargo knows nothing about the binaries,
/// which the cleanup treats as "remove the file directly".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readRecordedInstalls(cargoHome: string): RecordedInstall[]
/// ```
pub fn read_recorded_installs(cargo_home: &Path) -> Vec<RecordedInstall> {
    return match fs::read_to_string(cargo_home.join(".crates.toml")) {
        Ok(text) => parse_recorded_installs(&text),
        Err(_) => Vec::new(),
    };
}

/// The install that placed the binary `bin`,
/// or `None` when no install names it.
pub fn install_naming_bin<'a>(installs: &'a [RecordedInstall], bin: &str) -> Option<&'a RecordedInstall> {
    for install in installs {
        for name in &install.bins {
            if name == bin {
                return Some(install);
            }
        }
    }
    return None;
}

//! What:
//! Reads the few cargo options the shadow cares about from a command line without changing it: where cargo will look
//! (`-C <dir>`, `--manifest-path <file>`) and whether the user already asked for a frozen lockfile (`--locked`, `--frozen`).
//! Why:
//! The install step runs for the Cargo workspace that holds the place cargo will work in, and the user's own lock assertion
//! is this tool's `--frozen-lockfile`: the shadow never adds an option to the user's command, so the user's words are all
//! there is to read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // options.ts
//! export function manifestPath(args: string[]): string | undefined
//! export function changeDir(args: string[]): string | undefined
//! export function assertsLock(args: string[]): boolean
//! ```

use std::ffi::OsString;
use std::path::PathBuf;

use crate::verb::words_before_double_dash;

/// The value of an option written `--name value` or `--name=value`; the last one wins, as in cargo.
fn long_option_value(words: &[String], name: &str) -> Option<String> {
    let prefix: String = format!("{name}=");
    let mut found: Option<String> = None;
    let mut index: usize = 0;
    while index < words.len() {
        if words[index] == name && index + 1 < words.len() {
            found = Some(words[index + 1].clone());
            index += 2;
            continue;
        }
        if let Some(value) = words[index].strip_prefix(&prefix) {
            found = Some(value.to_string());
        }
        index += 1;
    }
    return found;
}

/// The `--manifest-path` value, when given.
pub fn manifest_path(arguments: &[OsString]) -> Option<PathBuf> {
    return long_option_value(&words_before_double_dash(arguments), "--manifest-path").map(PathBuf::from);
}

/// The `-C <directory>` value, when given (cargo's own option, nightly only).
pub fn change_dir(arguments: &[OsString]) -> Option<PathBuf> {
    let words: Vec<String> = words_before_double_dash(arguments);
    let mut found: Option<PathBuf> = None;
    let mut index: usize = 0;
    while index < words.len() {
        if words[index] == "-C" && index + 1 < words.len() {
            found = Some(PathBuf::from(&words[index + 1]));
            index += 2;
            continue;
        }
        index += 1;
    }
    return found;
}

/// Whether the command line asks cargo to keep the lockfile unchanged (`--locked`) or also stay offline (`--frozen`).
pub fn asserts_lock(arguments: &[OsString]) -> bool {
    return words_before_double_dash(arguments).iter().any(|word| { return word == "--locked" || word == "--frozen"; });
}

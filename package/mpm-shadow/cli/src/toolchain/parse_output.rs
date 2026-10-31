//! What:     Readers for the text other tools print about what they have installed (`bin list`, `cargo binstall
//!           --list`, `uv tool list`, `pnpm list --global --json`), and the version comparison `status` and `sync` use.
//! Why:      Everything here is a pure function of text, so it is tested with captured and hostile output and no tool
//!           is started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parse-output.ts
//! export function parseBinList(output: string): Map<string, string>
//! export function parseCargoBinstallList(output: string): Map<string, string>
//! export function parseUvToolList(output: string): Map<string, string>
//! export function globalPnpmVersions(parsed: unknown): Map<string, string>
//! export function versionMatches(observed: string, locked: string): boolean
//! ```

use serde_json::Value;
use std::collections::BTreeMap;

/// The characters a release tag's name prefix may hold before its `-v<digit>` (`bun-v1.3.14`).
fn is_tag_prefix_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '.' || character == '_' || character == '-';
}

/// What:     A version without the name prefix or `v` a release tag carries: `bun-v1.3.14` and `v1.3.14` both give
///           `1.3.14`; `nightly-2026-10-21` stays as it is.
/// Why:      The `bin` registry records release tags, the lock records bare versions; they must compare equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function normalizeVersion(version: string): string
/// ```
pub fn normalize_version(version: &str) -> String {
    let bytes: &[u8] = version.as_bytes();
    if !bytes.is_empty() && bytes[0].is_ascii_alphabetic() {
        let mut run_length: usize = 1;
        for character in version.chars().skip(1) {
            if !is_tag_prefix_character(character) {
                break;
            }
            run_length += 1;
        }
        let mut dash: usize = run_length;
        while dash > 1 {
            dash -= 1;
            if bytes[dash] == b'-' && bytes.get(dash + 1) == Some(&b'v') && bytes.get(dash + 2).is_some_and(u8::is_ascii_digit) {
                return version[dash + 2..].to_string();
            }
        }
    }
    if bytes.len() > 1 && bytes[0] == b'v' && bytes[1].is_ascii_digit() {
        return version[1..].to_string();
    }
    return version.to_string();
}

/// What:     Whether an observed version is the locked one: equal after `normalize_version`, or one is the other
///           followed by `-` and a suffix (`nightly-2026-10-21-x86_64-unknown-linux-gnu` matches `nightly-2026-10-21`).
/// Why:      Toolchains report platform suffixes the lock does not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionMatches(observed: string, locked: string): boolean
/// ```
pub fn version_matches(observed: &str, locked: &str) -> bool {
    let normalized_observed: String = normalize_version(observed);
    let normalized_locked: String = normalize_version(locked);
    return normalized_observed == normalized_locked
        || normalized_observed.starts_with(&format!("{normalized_locked}-"))
        || normalized_locked.starts_with(&format!("{normalized_observed}-"));
}

/// The version text without a leading `v` (`v1.2.3` gives `1.2.3`; a bare `v` stays).
fn without_version_prefix(word: &str) -> String {
    if word.len() > 1 && word.starts_with('v') {
        return word[1..].to_string();
    }
    return word.to_string();
}

/// Splits text into rows at `\n`, `\r`, U+2028 and U+2029.
fn split_rows(text: &str) -> Vec<&str> {
    return text.split(['\n', '\r', '\u{2028}', '\u{2029}']).collect();
}

/// The columns of a row: the pieces separated by two or more whitespace characters.
fn split_columns(row: &str) -> Vec<String> {
    let mut columns: Vec<String> = Vec::new();
    let mut current: String = String::new();
    let mut pending_whitespace: String = String::new();
    for character in row.chars() {
        if character.is_whitespace() {
            pending_whitespace.push(character);
            continue;
        }
        if pending_whitespace.chars().count() >= 2 {
            columns.push(std::mem::take(&mut current));
        } else {
            current.push_str(&pending_whitespace);
        }
        pending_whitespace.clear();
        current.push(character);
    }
    if !current.is_empty() {
        columns.push(current);
    }
    return columns;
}

/// The final path segment, accepting `/` and `\` as separators.
fn base_name(path: &str) -> String {
    return path.rsplit(['/', '\\']).next().unwrap_or(path).to_string();
}

/// What:     Parses `bin list`: rows of `path  version  url  status` separated by two or more spaces, giving the
///           binary's file name to its version (`*` marker and `v` prefix removed).
/// Why:      `sync` and `status` ask `bin` what it installed; a row starting with whitespace or with fewer than three
///           columns is not a tool row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseBinList(output: string): Map<string, string>
/// ```
pub fn parse_bin_list(output: &str) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();
    for row in split_rows(output) {
        if row.is_empty() || row.starts_with(char::is_whitespace) {
            continue;
        }
        let columns: Vec<String> = split_columns(row);
        if columns.len() < 3 {
            continue;
        }
        let version_word: &str = columns[1].split_whitespace().next().unwrap_or("");
        let marked: &str = if version_word.len() > 1 && version_word.starts_with('*') { &version_word[1..] } else { version_word };
        if marked.is_empty() {
            continue;
        }
        versions.insert(base_name(&columns[0]), without_version_prefix(marked));
    }
    return versions;
}

/// What:     Parses `cargo binstall --list`: rows of `crate  cargo-version  binstall-version  source` giving the crate
///           to its installed version (the binstall column unless it prints `-`).
/// Why:      The `Package` header and `Total:` footer are not crates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseCargoBinstallList(output: string): Map<string, string>
/// ```
pub fn parse_cargo_binstall_list(output: &str) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();
    for row in split_rows(output) {
        if row.starts_with(char::is_whitespace) {
            continue;
        }
        let words: Vec<&str> = row.split_whitespace().collect();
        if words.len() < 4 || words[0] == "Package" || words[0] == "Total:" {
            continue;
        }
        let version: String = if words[2] == "-" { without_version_prefix(words[1]) } else { without_version_prefix(words[2]) };
        versions.insert(words[0].to_string(), version);
    }
    return versions;
}

/// What:     Parses `uv tool list`: `name vX.Y.Z` rows (indented `- bin` rows are skipped), keyed by the name with `-`
///           turned into `_` so a package id matches however uv spells it.
/// Why:      uv normalizes package names; the lookup normalizes the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseUvToolList(output: string): Map<string, string>
/// ```
pub fn parse_uv_tool_list(output: &str) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();
    for row in output.split('\n') {
        if row.starts_with(char::is_whitespace) || row.starts_with('-') {
            continue;
        }
        let mut words = row.split_whitespace();
        let (Some(name), Some(token)) = (words.next(), words.next()) else {
            continue;
        };
        let key: String = name.replace('-', "_");
        let version: String = if token.len() > 1 && token.starts_with('v') { token[1..].to_string() } else { token.to_string() };
        versions.entry(key).or_insert(version);
    }
    return versions;
}

/// What:     Reads the output of `pnpm list --global --depth=0 --json` (an object or a list of objects with a
///           `dependencies` object) into package name to installed version.
/// Why:      The shape differs between pnpm releases; both are accepted and anything else is simply empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function globalPnpmVersions(parsed: unknown): Map<string, string>
/// ```
pub fn global_pnpm_versions(parsed: &Value) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();
    let roots: Vec<&Value> = match parsed {
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    };
    for root in roots {
        let Some(dependencies) = root.get("dependencies").and_then(Value::as_object) else {
            continue;
        };
        for (name, dependency) in dependencies {
            if let Some(version) = dependency.get("version").and_then(Value::as_str) {
                versions.insert(name.clone(), version.to_string());
            }
        }
    }
    return versions;
}

//! What:
//! The selector syntax of `rust-catalog.toml`'s exclude lists,
//! which is pnpm's:
//! `name`,
//! `name@1.2.3`,
//! `name@1.2.3 || 1.2.4`,
//! and `*` wildcards inside the name.
//! Why:
//! The release-age and trust checks skip excluded versions,
//! and the update prunes excludes that no locked version needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // selectors.ts
//! export function parseSelector(selector: string): { pattern: string; versions: string[] }
//! export function isExcluded(args: { selectors: string[]; name: string; version: string }): boolean
//! ```

/// What:
/// `use std::collections::BTreeSet;` imports `BTreeSet` from the standard library so this file can name it unqualified.
/// Why:
/// `BTreeSet` is used by `prune_selectors`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeSet } from 'std/collections';
/// ```
use std::collections::BTreeSet;

/// One selector split into its name pattern and the exact versions it names (empty means every version).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedSelector {
    /// The name or the `*` pattern.
    pub pattern: String,
    /// Exact versions;
    /// empty exempts every version of the matching names.
    pub versions: Vec<String>,
}

/// Splits `name@v1 || v2` at the first `@`.
pub fn parse_selector(selector: &str) -> ParsedSelector {
    let at: usize = match selector.find('@') {
        Some(position) => position,
        None => {
            return ParsedSelector { pattern: selector.trim().to_string(), versions: Vec::new() };
        }
    };
    let mut versions: Vec<String> = Vec::new();
    for piece in selector[at + 1..].split("||") {
        let trimmed: &str = piece.trim();
        if !trimmed.is_empty() {
            versions.push(trimmed.to_string());
        }
    }
    return ParsedSelector { pattern: selector[..at].trim().to_string(), versions };
}

/// What:
/// `pub fn matches_pattern(pattern: &str, name: &str) -> bool` matches a crate name against a pattern whose only
/// metacharacter is `*` (any run of characters).
/// Why:
/// The same wildcard rule pnpm uses;
/// a plain regular expression would need escaping and the `regex` crate for one character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function matchesPattern({ pattern, name }): boolean
/// ```
pub fn matches_pattern(pattern: &str, name: &str) -> bool {
    let pieces: Vec<&str> = pattern.split('*').collect();
    if pieces.len() == 1 {
        return pattern == name;
    }
    let first: &str = pieces[0];
    let last: &str = pieces[pieces.len() - 1];
    if !name.starts_with(first) || !name.ends_with(last) || name.len() < first.len() + last.len() {
        return false;
    }
    let mut offset: usize = first.len();
    for piece in &pieces[1..pieces.len() - 1] {
        let found: usize = match name[offset..].find(piece) {
            Some(position) => offset + position,
            None => return false,
        };
        if found + piece.len() > name.len() - last.len() {
            return false;
        }
        offset = found + piece.len();
    }
    return true;
}

/// Whether any selector covers this exact name and version.
pub fn is_excluded(selectors: &[String], name: &str, version: &str) -> bool {
    for selector in selectors {
        let parsed: ParsedSelector = parse_selector(selector);
        if matches_pattern(&parsed.pattern, name)
            && (parsed.versions.is_empty() || crate::adapt::contains_text(&parsed.versions, version))
        {
            return true;
        }
    }
    return false;
}

/// What:
/// `pub fn prune_selectors(selectors: &[String], locked: &BTreeSet<String>) -> Vec<String>` drops what no locked
/// `name@version` needs:
/// wildcard selectors stay,
/// a bare name stays while any version of it is locked,
/// and a versioned
/// selector keeps only the versions still locked.
/// Why:
/// `minimumReleaseAgeExcludePrune` and `trustPolicyExcludePrune` keep the lists from collecting stale entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pruneSelectors({ selectors, locked }): string[]
/// ```
pub fn prune_selectors(selectors: &[String], locked: &BTreeSet<String>) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for selector in selectors {
        let parsed: ParsedSelector = parse_selector(selector);
        if parsed.pattern.contains('*') {
            kept.push(selector.clone());
            continue;
        }
        if parsed.versions.is_empty() {
            let prefix: String = format!("{}@", parsed.pattern);
            if locked.iter().any(|entry| { return entry.starts_with(&prefix); }) {
                kept.push(selector.clone());
            }
            continue;
        }
        let mut remaining: Vec<String> = Vec::new();
        for version in &parsed.versions {
            if locked.contains(&format!("{}@{}", parsed.pattern, version)) {
                remaining.push(version.clone());
            }
        }
        if !remaining.is_empty() {
            kept.push(format!("{}@{}", parsed.pattern, remaining.join(" || ")));
        }
    }
    return kept;
}

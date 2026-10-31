//! What:     Plans the default-catalog `>=` floor for one group of registry literals that share a dependency name.
//! Why:      Literals are grouped by name across every crate first, so the result never depends on crate order: the
//!           new floor is the highest base version, and it must satisfy every literal's own requirement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // adopt-registry.ts
//! export function planRegistryGroup(args: { name: string; literals: UseSite[]; existing: Entry[]; issues: SyncIssue[] }): Entry[]
//! ```

use std::cmp::Ordering;

use crate::adopt_shared::named_catalog_remedy;
use crate::error::SyncIssue;
use crate::quote::quote;
use crate::requirement::{Operator, Requirement, satisfies, split_requirement};
use crate::site::UseSite;
use crate::types::Entry;
use crate::version::{ParsedVersion, compare_versions, format_version};

/// One candidate floor: the version text as written (keeping the author's precision) and its parse.
struct FloorCandidate {
    /// Version text without its operator, such as `1.50` for `^1.50`.
    text: String,
    /// Parsed form for comparison.
    parsed: ParsedVersion,
}

/// Version text of a requirement without its `>=`, `^` or `~` operator.
fn strip_operator(requirement: &str) -> String {
    let trimmed: &str = requirement.trim();
    for operator in [">=", "^", "~"] {
        if let Some(rest) = trimmed.strip_prefix(operator) {
            return rest.trim().to_string();
        }
    }
    return trimmed.to_string();
}

/// What:     `fn literal_candidates(...)` turns registry literals into candidate floors; exact pins and unreadable
///           requirements become issues.
/// Why:      An exact pin can never be a `>=` floor, so it must live in a named catalog.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function literalCandidates(args: { literals: UseSite[]; issues: SyncIssue[] }): FloorCandidate[]
/// ```
fn literal_candidates(literals: &[&UseSite], issues: &mut Vec<SyncIssue>) -> Vec<FloorCandidate> {
    let mut candidates: Vec<FloorCandidate> = Vec::new();
    for site in literals {
        let Entry::Registry { version, .. } = &site.literal else {
            continue;
        };
        match split_requirement(version) {
            Ok(Requirement { operator: Operator::Exact, .. }) => issues.push(SyncIssue {
                file: site.manifest_path.clone(),
                location: site.location(),
                problem: format!("exact pin {} cannot become a `>=` floor in [catalog]", quote(version)),
                remedy: named_catalog_remedy(site, version),
            }),
            Ok(split) => candidates.push(FloorCandidate {
                text: strip_operator(version),
                parsed: split.version,
            }),
            Err(error) => issues.push(SyncIssue {
                file: site.manifest_path.clone(),
                location: site.location(),
                problem: format!("requirement {} is not one comparator the catalog can adopt ({error})", quote(version)),
                remedy: "write a single requirement such as `\"1.2\"`, `\"~1.2.3\"`, or `\">=1.2\"`".to_string(),
            }),
        }
    }
    return candidates;
}

/// What:     `pub fn plan_registry_group(...) -> Option<Entry>` plans one group: the entry the default catalog
///           should hold, or `None` when the group conflicts (the reasons are in `issues`).
/// Why:      `existing` is the current default entry, so a floor only ever rises.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planRegistryGroup(args: { name: string; literals: UseSite[]; existing: Entry[]; issues: SyncIssue[] }): Entry[]
/// ```
pub fn plan_registry_group(name: &str, literals: &[&UseSite], existing: Option<&Entry>, issues: &mut Vec<SyncIssue>) -> Option<Entry> {
    let issues_before: usize = issues.len();
    let mut candidates: Vec<FloorCandidate> = literal_candidates(literals, issues);
    if let Some(Entry::Registry { version, .. }) = existing
        && let Ok(split) = split_requirement(version)
    {
        candidates.push(FloorCandidate {
            text: strip_operator(version),
            parsed: split.version,
        });
    }
    if issues.len() != issues_before || candidates.is_empty() {
        return None;
    }
    let mut highest: &FloorCandidate = &candidates[0];
    for candidate in &candidates {
        if compare_versions(&candidate.parsed, &highest.parsed) == Ordering::Greater {
            highest = candidate;
        }
    }
    let highest_full: String = format_version(&highest.parsed);
    for site in literals {
        let Entry::Registry { version, .. } = &site.literal else {
            continue;
        };
        if !satisfies(&highest_full, version).unwrap_or(false) {
            issues.push(SyncIssue {
                file: site.manifest_path.clone(),
                location: site.location(),
                problem: format!(
                    "requires {}, which the shared floor >={} for {name} does not allow",
                    quote(version),
                    highest.text
                ),
                remedy: named_catalog_remedy(site, version),
            });
        }
    }
    if issues.len() != issues_before {
        return None;
    }
    let mut renames: Vec<String> = Vec::new();
    for site in literals {
        renames.push(site.literal.package().unwrap_or("").to_string());
    }
    if let Some(entry) = existing {
        renames.push(entry.package().unwrap_or("").to_string());
    }
    renames.sort();
    renames.dedup();
    if renames.len() > 1 {
        let first_file: String = match literals.first() {
            Some(site) => site.manifest_path.clone(),
            None => String::new(),
        };
        issues.push(SyncIssue {
            file: first_file,
            location: name.to_string(),
            problem: format!("crates rename {name} to different packages"),
            remedy: "give each renamed package its own dependency name".to_string(),
        });
        return None;
    }
    let rename: String = renames.into_iter().next().unwrap_or_default();
    return Some(Entry::Registry {
        version: format!(">={}", highest.text),
        package: if rename.is_empty() { None } else { Some(rename) },
    });
}

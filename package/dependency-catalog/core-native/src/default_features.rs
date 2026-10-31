//! What:
//! The per-crate `default-features` choice for generated `[workspace.dependencies]` entries.
//! Why:
//! Cargo before 1.99 rejects `default-features = false` beside `workspace = true` when the workspace entry
//! keeps default features,
//! so a crate whose catalog sites all disable default features gets
//! `default-features = false` in its own generated entry,
//! and its sites keep adding the features they need.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // default-features.ts
//! export function generatedEntries(args: { info: CrateInfo; resolvedEntries: Map<string, Entry>; issues: SyncIssue[] }): Map<string, GeneratedEntry>
//! ```

/// Imports `crate::crate_info::CrateInfo`.
use crate::crate_info::CrateInfo;
/// Imports `crate::error::SyncIssue`.
use crate::error::SyncIssue;
/// Imports `crate::site::{GeneratedEntry, SiteKind}`.
use crate::site::{GeneratedEntry, SiteKind};
/// Imports `crate::types::Entry`.
use crate::types::Entry;

/// What:
/// `pub fn generated_entries(...)` pairs each resolved entry with the crate's default-features choice.
/// Sites that disagree cannot share one generated entry and are reported.
/// Why:
/// The result keeps the order of `resolved`,
/// which the caller built in site order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function generatedEntries(args: { info; resolvedEntries; issues }): Map<string, GeneratedEntry>
/// ```
pub fn generated_entries(info: &CrateInfo, resolved: &[(String, Entry)], issues: &mut Vec<SyncIssue>) -> Vec<(String, GeneratedEntry)> {
    let mut entries: Vec<(String, GeneratedEntry)> = Vec::new();
    for (name, entry) in resolved {
        let mut total: usize = 0;
        let mut disabling: usize = 0;
        for site in &info.sites {
            if site.kind != SiteKind::Path && &site.name == name {
                total += 1;
                if site.disables_default_features {
                    disabling += 1;
                }
            }
        }
        let all_disable: bool = total > 0 && disabling == total;
        let mixed: bool = disabling > 0 && disabling < total;
        if mixed {
            issues.push(SyncIssue {
                file: info.manifest.path.clone(),
                location: name.clone(),
                problem: format!(
                    "some {name} entries write `default-features = false` and others do not, but the crate's one generated [workspace.dependencies] entry can hold only one choice"
                ),
                remedy: format!(
                    "write `default-features = false` on every {name} entry in this manifest (adding the needed features explicitly), or remove it from all of them"
                ),
            });
        }
        entries.push((
            name.clone(),
            GeneratedEntry {
                entry: entry.clone(),
                default_features: !all_disable,
            },
        ));
    }
    return entries;
}

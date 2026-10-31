//! What:
//! Plans how literal versions left in manifests (for example by `cargo add`) join the default catalog.
//! Why:
//! The result must not depend on the order crates are visited,
//! so literals are grouped by dependency name
//! across every crate first;
//! each group becomes one added entry,
//! one raised floor,
//! or a reported conflict.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // adopt.ts
//! export function planAdoption(args: { document: RustCatalogDocument; sites: UseSite[]; selections: Selections; issues: SyncIssue[] }): AdoptionPlan
//! ```

/// Imports `crate::adopt_registry::plan_registry_group`.
use crate::adopt_registry::plan_registry_group;
/// Imports `crate::adopt_selected::check_selected_literals`.
use crate::adopt_selected::check_selected_literals;
/// Imports `crate::adopt_shared::same_entry`.
use crate::adopt_shared::same_entry;
/// Imports `crate::crate_info::CrateInfo`.
use crate::crate_info::CrateInfo;
/// Imports `crate::error::SyncIssue`.
use crate::error::SyncIssue;
/// Imports `crate::site::{SiteKind, UseSite}`.
use crate::site::{SiteKind, UseSite};
/// Imports `crate::types::{CatalogDocument, Entry, EntryMap}`.
use crate::types::{CatalogDocument, Entry, EntryMap};

/// What:
/// `pub struct AdoptionPlan { ... }` lists the catalog changes adoption needs and the default catalog as it
/// will read afterwards.
/// Why:
/// Resolution uses `default_catalog` (so a just-adopted entry resolves),
/// the editor uses the other two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AdoptionPlan = { additions: Map<string, Entry>; floorRaises: Map<string, string>; defaultCatalog: Map<string, Entry> };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionPlan {
    /// New default entries,
    /// in discovery order.
    pub additions: Vec<(String, Entry)>,
    /// Existing default entries whose `>=` floor rises,
    /// with the new requirement text.
    pub floor_raises: Vec<(String, String)>,
    /// The default catalog after adoption.
    pub default_catalog: EntryMap,
}

/// Plans one default-catalog group of git literals:
/// every pin must agree with the others and the existing entry.
fn plan_git_group(name: &str, group: &[&UseSite], existing: Option<&Entry>, issues: &mut Vec<SyncIssue>) -> Option<Entry> {
    let first: &UseSite = group.first()?;
    let pin: &Entry = match existing {
        Some(entry) => entry,
        None => &first.literal,
    };
    let mut agrees: bool = true;
    for site in group {
        if !same_entry(&site.literal, pin) {
            agrees = false;
        }
    }
    if !agrees {
        issues.push(SyncIssue {
            file: first.manifest_path.clone(),
            location: name.to_string(),
            problem: format!("crates pin {name} to different git revisions"),
            remedy: "pin every crate to one `rev`".to_string(),
        });
        return None;
    }
    if existing.is_none() {
        return Some(pin.clone());
    }
    return None;
}

/// Groups literal sites by dependency name in first-seen order;
/// selected literals go to `selected` instead.
fn group_literals<'a>(crates: &'a [CrateInfo], selected: &mut Vec<(&'a UseSite, &'a str)>) -> Vec<(&'a str, Vec<&'a UseSite>)> {
    let mut groups: Vec<(&'a str, Vec<&'a UseSite>)> = Vec::new();
    for info in crates {
        for site in &info.sites {
            if site.kind != SiteKind::Literal {
                continue;
            }
            if let Some(catalog_name) = info.selection(&site.name) {
                selected.push((site, catalog_name));
                continue;
            }
            let mut placed: bool = false;
            for group in &mut groups {
                if group.0 == site.name {
                    group.1.push(site);
                    placed = true;
                }
            }
            if !placed {
                groups.push((&site.name, vec![site]));
            }
        }
    }
    return groups;
}

/// What:
/// `pub fn plan_adoption(...) -> AdoptionPlan` plans adoption of every literal and returns the catalog
/// changes it implies.
/// Conflicts are pushed to `issues`.
/// Why:
/// Git pins must agree;
/// registry literals raise or create a `>=` floor;
/// a git and a registry source for
/// one name need separate catalogs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planAdoption(args: { document; sites; selections; issues }): AdoptionPlan
/// ```
pub fn plan_adoption(document: &CatalogDocument, crates: &[CrateInfo], issues: &mut Vec<SyncIssue>) -> AdoptionPlan {
    let mut selected: Vec<(&UseSite, &str)> = Vec::new();
    let groups: Vec<(&str, Vec<&UseSite>)> = group_literals(crates, &mut selected);
    check_selected_literals(document, &selected, issues);
    let mut additions: Vec<(String, Entry)> = Vec::new();
    let mut floor_raises: Vec<(String, String)> = Vec::new();
    let mut default_catalog: EntryMap = document.catalog.clone();
    for (name, group) in &groups {
        let current: Option<&Entry> = document.catalog.get(name);
        let mut has_git: bool = matches!(current, Some(Entry::Git { .. }));
        let mut has_registry: bool = matches!(current, Some(Entry::Registry { .. }));
        for site in group {
            match site.literal {
                Entry::Git { .. } => has_git = true,
                Entry::Registry { .. } => has_registry = true,
            }
        }
        if has_git && has_registry {
            issues.push(SyncIssue {
                file: group[0].manifest_path.clone(),
                location: (*name).to_string(),
                problem: format!("{name} is a git dependency in some places and a registry dependency in others"),
                remedy: "use one source for every crate, or give the other source its own named catalog".to_string(),
            });
            continue;
        }
        let planned: Option<Entry> = if has_git {
            plan_git_group(name, group, current, issues)
        } else {
            plan_registry_group(name, group, current, issues)
        };
        let Some(entry) = planned else {
            continue;
        };
        match current {
            None => additions.push(((*name).to_string(), entry.clone())),
            Some(existing) => {
                if let Entry::Registry { version, .. } = &entry
                    && !same_entry(&entry, existing)
                {
                    floor_raises.push(((*name).to_string(), version.clone()));
                }
            }
        }
        default_catalog.insert(name, entry);
    }
    return AdoptionPlan {
        additions,
        floor_raises,
        default_catalog,
    };
}

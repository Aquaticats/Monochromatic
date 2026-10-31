//! What:     Validation of one catalog table (`[catalog]` or `[catalogs.<name>]`) and its entries.
//! Why:      The catalog gates CI and every crate manifest, so an invalid entry is reported (all of them at once)
//!           rather than skipped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parse-entry.ts
//! export function parseCatalogTable(args: { location: string; value: unknown; bounded: boolean; issues: Issue[] }): Map<string, Entry>
//! ```

use toml_edit::{Item, TableLike};

use crate::error::CatalogIssue;
use crate::quote::quote;
use crate::syntax::{is_bounded_requirement, is_crate_name, is_floor_requirement};
use crate::types::{Entry, EntryMap};

/// Entry keys that belong at a crate's use site, each with the fix to tell the contributor.
const USE_SITE_KEYS: [(&str, &str); 7] = [
    (
        "features",
        "move `features` to each crate that needs them; Cargo adds use-site features on top of the inherited entry",
    ),
    ("optional", "move `optional = true` to the crate that makes the dependency optional"),
    (
        "default-features",
        "move `default-features = false` to the crate that disables default features",
    ),
    (
        "path",
        "depend on internal crates by `path` in the crate manifest; internal crates stay out of the catalog",
    ),
    ("branch", "pin git entries with `rev = \"<commit>\"` instead of `branch`"),
    ("tag", "pin git entries with `rev = \"<commit>\"` instead of `tag`"),
    ("workspace", "remove `workspace`; catalog entries are the source the crates inherit from"),
];

/// The only keys a table-form entry may declare besides the use-site keys above.
const ENTRY_KEYS: [&str; 4] = ["git", "package", "rev", "version"];

/// Fix text for a `>=` floor rule violation in the default catalog.
const FLOOR_REMEDY: &str = "write a `>=` floor such as `\">=1.0.104\"`; bounded versions belong in a named catalog";

/// Fix text for a bounded rule violation in a named catalog.
const BOUNDED_REMEDY: &str = "named catalogs hold bounded requirements such as `\"0.58\"`, `\"^0.58\"`, `\"~0.58.1\"`, or `\"=0.58.0\"`; a `>=` floor would be pulled to the newest release";

/// What:     `pub fn describe_item(item: Option<&Item>) -> String` renders a value for a message the way
///           `JSON.stringify` did: strings quoted, a missing value as `undefined`, anything else as its TOML text.
/// Why:      Messages quote the offending value so the contributor can find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// String(JSON.stringify(value))
/// ```
pub fn describe_item(item: Option<&Item>) -> String {
    return match item {
        None => "undefined".to_string(),
        Some(present) => match present.as_str() {
            Some(text) => quote(text),
            None => present.to_string().trim().to_string(),
        },
    };
}

/// Whether `requirement` passes the floor rule (default catalog) or the bounded rule (named catalog).
fn accepts(requirement: &str, bounded: bool) -> bool {
    if bounded {
        return is_bounded_requirement(requirement);
    }
    return is_floor_requirement(requirement);
}

/// The fix text of the rule in force.
fn rule_remedy(bounded: bool) -> &'static str {
    if bounded {
        return BOUNDED_REMEDY;
    }
    return FLOOR_REMEDY;
}

/// Whether `item` is a non-empty string.
fn non_empty_string(item: Option<&Item>) -> Option<&str> {
    return match item.and_then(Item::as_str) {
        Some(text) if !text.is_empty() => Some(text),
        _ => None,
    };
}

/// The fix for a key that belongs at the use site, or `None` for any other key.
fn use_site_remedy(key: &str) -> Option<&'static str> {
    for (candidate, remedy) in USE_SITE_KEYS {
        if candidate == key {
            return Some(remedy);
        }
    }
    return None;
}

/// Reports table keys that are unknown or belong at the use site.
fn report_foreign_keys(location: &str, table: &dyn TableLike, issues: &mut Vec<CatalogIssue>) {
    for (key, _) in table.iter() {
        let use_site: Option<&str> = use_site_remedy(key);
        if let Some(remedy) = use_site {
            issues.push(CatalogIssue {
                location: location.to_string(),
                problem: format!("declares `{key}`"),
                remedy: remedy.to_string(),
            });
        } else if !ENTRY_KEYS.contains(&key) {
            issues.push(CatalogIssue {
                location: location.to_string(),
                problem: format!("declares unknown key `{key}`"),
                remedy: "use only `version`, `package`, `git`, and `rev`".to_string(),
            });
        }
    }
}

/// Validates a table-form entry; `None` when the table has problems (already pushed to `issues`).
fn parse_table_entry(location: &str, table: &dyn TableLike, bounded: bool, issues: &mut Vec<CatalogIssue>) -> Option<Entry> {
    let issues_before: usize = issues.len();
    report_foreign_keys(location, table, issues);
    let package_item: Option<&Item> = table.get("package");
    let mut package: Option<String> = None;
    if let Some(item) = package_item {
        match item.as_str() {
            Some(name) if is_crate_name(name) => package = Some(name.to_string()),
            _ => issues.push(CatalogIssue {
                location: location.to_string(),
                problem: "`package` is not a valid crate name".to_string(),
                remedy: "set `package` to the crate name on crates.io that this dependency renames".to_string(),
            }),
        }
    }
    let git: Option<&Item> = table.get("git");
    let rev: Option<&Item> = table.get("rev");
    let version: Option<&Item> = table.get("version");
    if git.is_some() {
        let git_text: Option<&str> = non_empty_string(git);
        let rev_text: Option<&str> = non_empty_string(rev);
        if git_text.is_none() {
            issues.push(CatalogIssue {
                location: location.to_string(),
                problem: "`git` is not a non-empty string".to_string(),
                remedy: "set `git` to the repository URL".to_string(),
            });
        }
        if rev_text.is_none() {
            issues.push(CatalogIssue {
                location: location.to_string(),
                problem: "git entry has no `rev`".to_string(),
                remedy: "pin the entry with `rev = \"<full commit hash>\"`".to_string(),
            });
        }
        if version.is_some() {
            issues.push(CatalogIssue {
                location: location.to_string(),
                problem: "git entry also declares `version`".to_string(),
                remedy: "remove `version` from git entries".to_string(),
            });
        }
        if issues.len() != issues_before {
            return None;
        }
        return match (git_text, rev_text) {
            (Some(git_value), Some(rev_value)) => Some(Entry::Git {
                git: git_value.to_string(),
                rev: rev_value.to_string(),
                package,
            }),
            _ => None,
        };
    }
    if rev.is_some() {
        issues.push(CatalogIssue {
            location: location.to_string(),
            problem: "declares `rev` without `git`".to_string(),
            remedy: "add `git` or remove `rev`".to_string(),
        });
    }
    let version_text: Option<&str> = version.and_then(Item::as_str);
    let accepted: bool = match version_text {
        Some(text) => accepts(text, bounded),
        None => false,
    };
    if !accepted {
        issues.push(CatalogIssue {
            location: location.to_string(),
            problem: format!("version requirement {} is not accepted here", describe_item(version)),
            remedy: rule_remedy(bounded).to_string(),
        });
    }
    if issues.len() != issues_before {
        return None;
    }
    let text: &str = version_text?;
    return Some(Entry::Registry {
        version: text.to_string(),
        package,
    });
}

/// Validates one entry, pushing problems instead of failing; `None` when rejected.
fn parse_entry(location: &str, name: &str, item: &Item, bounded: bool, issues: &mut Vec<CatalogIssue>) -> Option<Entry> {
    let valid_name: bool = is_crate_name(name);
    if !valid_name {
        issues.push(CatalogIssue {
            location: location.to_string(),
            problem: format!("{} is not a valid Cargo dependency name", quote(name)),
            remedy: "use 1 to 64 ASCII letters, digits, `-`, or `_`, starting with a letter".to_string(),
        });
    }
    if let Some(text) = item.as_str() {
        if !accepts(text, bounded) {
            issues.push(CatalogIssue {
                location: location.to_string(),
                problem: format!("version requirement {} is not accepted here", quote(text)),
                remedy: rule_remedy(bounded).to_string(),
            });
            return None;
        }
        if !valid_name {
            return None;
        }
        return Some(Entry::Registry {
            version: text.to_string(),
            package: None,
        });
    }
    let Some(table) = item.as_table_like() else {
        issues.push(CatalogIssue {
            location: location.to_string(),
            problem: "is neither a version string nor an inline table".to_string(),
            remedy: "write `name = \">=1.2.3\"` or `name = { version = \">=1.2.3\", package = \"real-name\" }`".to_string(),
        });
        return None;
    };
    let entry: Option<Entry> = parse_table_entry(location, table, bounded, issues);
    if !valid_name {
        return None;
    }
    return entry;
}

/// What:     `pub fn parse_catalog_table(...)` validates one catalog table into an ordered map. `item` is `None`
///           when the file does not declare the table.
/// Why:      The default catalog takes floors (`bounded` false), named catalogs take bounded requirements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseCatalogTable(args: { location: string; value: unknown; bounded: boolean; issues: Issue[] }): Map<string, Entry>
/// ```
pub fn parse_catalog_table(location: &str, item: Option<&Item>, bounded: bool, issues: &mut Vec<CatalogIssue>) -> EntryMap {
    let mut entries: EntryMap = EntryMap::new();
    let Some(present) = item else {
        return entries;
    };
    let Some(table) = present.as_table_like() else {
        issues.push(CatalogIssue {
            location: location.to_string(),
            problem: "is not a table".to_string(),
            remedy: format!("declare it as `[{location}]`"),
        });
        return entries;
    };
    for (name, entry_item) in table.iter() {
        let entry_location: String = format!("{location}.{name}");
        if let Some(entry) = parse_entry(&entry_location, name, entry_item, bounded, issues) {
            entries.insert(name, entry);
        }
    }
    return entries;
}

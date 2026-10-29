//! Scans logical pathname components without exposing matched names.
//!
//! A component is one directory name or filename,
//!  not the path between them.
//! The engine returns matching rule ids rather than match spans,
//!  so a matching
//! component is masked in full.
//!  All findings for the file share the same masked
//! display path,
//!  including content and read-error findings made by the caller.

/// Imports the filesystem path type for repository-root-relative names.
use std::path::Path;
/// Imports the unwind boundary that keeps a matcher panic from passing a scan.
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Imports loaded matcher sets and their shared rule-identity formatter.
use crate::{frx_load::LoadedRules, frx_scan::rule_token};
/// Import structured results so pathname matching has only one canonical finding model.
use crate::scan_finding::ScanFinding;
/// Native path bytes are matched before any display encoding occurs.
use crate::path_name_bytes::{is_slash, normalized_path, prefix_parts, safe_component};

/// Mask substituted for a pathname component that matches any active rule.
const REDACTED: &str = "[REDACTED]";

/// Redacted label and findings for one scanned pathname.
pub(crate) struct PathScan {
    /// Path safe for content findings and read-error diagnostics.
    pub(crate) display: String,
    /// Distinct name findings,
    ///  one for each matching component and rule.
    pub(crate) findings: Vec<String>,
}

/// Canonical pathname scan returned to embedded callers before terminal rendering.
pub(crate) struct PathScanRecords {
    /// Path with every matching component masked and protocol-sensitive characters escaped.
    pub(crate) display: String,
    /// Location/rule records without candidate bytes or terminal-format parsing.
    pub(crate) findings: Vec<ScanFinding>,
}

/// Discovers the Git root,
///  if cwd or an ancestor has a `.git` entry.
///
/// A worktree's `.git` file counts as a root just as a main repository's
/// `.git` directory does.
///  A standalone invocation returns `None`.
pub(crate) fn repository_root() -> Option<std::path::PathBuf> {
    // `current_dir` owns an absolute path; failure leaves explicit names lexical.
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join(".git").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    return None;
}

/// Chooses a repository-relative name for files inside the repository.
///
/// Paths outside a repository and standalone invocations retain all supplied
/// pathname segments.
///  Lexical normalization removes navigation markers without
/// following symlinks:
///  the selected link name,
///  not its target,
///  must be scanned.
pub(crate) fn logical_path(path: &str, root: Option<&Path>) -> String {
    let Some(repository) = root else {
        return path.to_string();
    };
    let input = Path::new(path);
    let absolute = if input.is_absolute() {
        input.to_path_buf()
    } else {
        let Ok(cwd) = std::env::current_dir() else {
            return path.to_string();
        };
        cwd.join(input)
    };
    let mut normalized = std::path::PathBuf::new();
    for part in absolute.components() {
        if part == std::path::Component::ParentDir {
            normalized.pop();
        } else if part != std::path::Component::CurDir {
            normalized.push(part.as_os_str());
        }
    }
    if let Ok(relative) = normalized.strip_prefix(repository)
        && let Some(name) = relative.to_str() {
            return name.to_string();
        }
    return path.to_string();
}

/// Matches one non-empty,
///  single-line name component against all loaded sets.
///
/// The engine treats a trailing CR or LF as a content-line terminator,
///  which
/// would change anchor semantics for a filename.
///  Callers fail closed on those
/// pathname bytes instead of misrepresenting an incomplete name as a match.
fn matching_rules(component: &[u8], loaded: &LoadedRules) -> Result<Vec<String>, ()> {
    let mut rules: Vec<String> = Vec::new();
    for set in loaded.iter_sets() {
        let matcher = AssertUnwindSafe(|| return set.matcher.line_matches(component, &[0]));
        let Ok(ids) = catch_unwind(matcher) else {
            return Err(());
        };
        rules.extend(ids.into_iter().map(|(_, id)| return rule_token(set.base, &set.names, id)));
    }
    return Ok(rules);
}

/// Scans each logical component and masks every component with a matching rule.
///
/// Component numbers are one-based and ignore the root,
///  every native volume-prefix
/// part,
///  and navigation markers (`.`,
///  `..`),
///  which are not directory names.
///  A matcher
/// panic masks the entire path and returns an engine-error finding instead of printing unsafe input.
pub(crate) fn scan_path_records(path: &Path, loaded: &LoadedRules) -> PathScanRecords {
    // Retain every non-separator native byte, including invalid UTF-8, until the matcher has inspected it.
    let normalized: Vec<u8> = normalized_path(path);
    return scan_normalized_records(&normalized, prefix_parts(path), loaded);
}

/// Scan already normalized bytes and an explicitly counted native prefix,
///  preserving one shared policy implementation.
///
/// The first `prefix_count` non-empty components are prefix parts whatever their bytes,
/// so the skip agrees by construction with `count_prefix_parts`,
///  which counts non-empty separator-delimited runs.
/// Every later non-empty component other than `.` or `..` is a name.
fn scan_normalized_records(normalized: &[u8], prefix_count: usize, loaded: &LoadedRules) -> PathScanRecords {
    // usize counts native prefix components, not bytes; the caller has already identified their boundary.
    let mut remaining_prefix: usize = prefix_count;
    let mut displayed: Vec<String> = Vec::<String>::new();
    let mut matches: Vec<(usize, Vec<String>)> = Vec::new();
    let mut position = 0;
    for component in normalized.split(is_slash) {
        if component.contains(&b'\n') || component.contains(&b'\r') {
            return PathScanRecords {
                display: REDACTED.to_string(),
                findings: vec![ScanFinding::PathnameLineBreak],
            };
        }
        // Empty components come from repeated or leading separators; they are neither names nor prefix parts.
        if component.is_empty() {
            displayed.push(safe_component(component));
            continue;
        }
        // Skip every component of the native volume prefix, including UNC server and share names,
        // before classifying navigation markers: the `.` of `\\.\COM1` is a prefix part, not navigation.
        // A prefix part is any non-empty component whatever its bytes, exactly what `count_prefix_parts` counts.
        if remaining_prefix > 0 {
            remaining_prefix -= 1;
            displayed.push(safe_component(component));
            continue;
        }
        if component == b"." || component == b".." {
            displayed.push(safe_component(component));
            continue;
        }
        position += 1;
        let Ok(rules) = matching_rules(component, loaded) else {
            return PathScanRecords {
                display: REDACTED.to_string(),
                findings: vec![ScanFinding::EngineError],
            };
        };
        if rules.is_empty() {
            displayed.push(safe_component(component));
        } else {
            displayed.push(REDACTED.to_string());
            matches.push((position, rules));
        }
    }
    let display = displayed.join("/");
    let mut findings: Vec<ScanFinding> = Vec::<ScanFinding>::new();
    for (segment, rules) in matches {
        for rule in rules {
            findings.push(ScanFinding::Name { component: segment, rule });
        }
    }
    return PathScanRecords { display, findings };
}

/// Render canonical pathname records for the existing standalone and fuzz text consumers.
pub(crate) fn scan_path(path: &str, loaded: &LoadedRules) -> PathScan {
    // Borrow the rules once; all matching and redaction remains inside the canonical scan.
    let records: PathScanRecords = scan_path_records(Path::new(path), loaded);
    let mut findings: Vec<String> = Vec::<String>::new();
    for finding in records.findings {
        findings.push(finding.render(records.display.as_str()));
    }
    return PathScan { display: records.display, findings };
}

/// Unit tests keep the name-matching and masking contract beside its owner.
#[cfg(test)]
#[path = "path_scan_tests.rs"]
mod tests;

/// Target-independent Windows prefix forms drive production normalization,
///  counting and scanning together.
#[cfg(test)]
#[path = "path_scan_prefix_tests.rs"]
mod prefix_tests;

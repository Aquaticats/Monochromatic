//! What:
//!  The list of findings a layout rule tolerates today,
//!  read from the file its `allow` option names.
//! Why:
//!  A crate that switches a layout rule on does not become correct at once:
//!  the rule's findings it has today are recorded,
//! one entry per finding,
//!  and a run fails when a finding is not on the list,
//!  so new code cannot add one.
//!  An entry that stops matching a finding is a finding of its own,
//!  so the list can only shrink.
//! Decisions:
//!  a folder rule entry and a file rule entry that tolerate one use between two folders or two files
//!  are the only entries that suppress more than one finding;
//!  they suppress that one use,
//! so every loop needs all of its uses on the list;
//!  an entry that names no finding any more says so,
//!  because a list that keeps dead lines is not a list of today's findings;
//!  a key names one finding,
//!  so the rule numbers a finding that would share an earlier one's key,
//!  and one line never tolerates a finding a later edit added;
//!  paths in an entry are relative to the package folder,
//!  the way findings show them,
//!  and never absolute and never climb out with `..`,
//!  because the list is read on every machine that lints the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AllowEntry = { line: number; text: string; kind: AllowKind };
//! type AllowKind = { kind: 'folder-use'; from: Path; to: Path }
//!   | { kind: 'file-use'; folder: Path; from: string; to: string }
//!   | { kind: 'key'; key: string };
//! function readAllowList(path: Path): AllowList;
//! ```

/// Import the entry shapes and the line reader the list is built from,
///  re-exported so a caller asks the list module for both.
pub use crate::layout_allow_entries::{
    AllowEntry,
    AllowKind,
    parse_entries,
};
/// Import the finding shape and the display helper.
use crate::layout_finding::{
    LayoutFinding,
    shown,
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeSet,
    io::Read,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The largest list file this reader accepts:
///  one mebibyte.
/// Why:
///  This repository's largest lists are the git wrapper's,
///  measured on 2026-11-25 from the branch that builds them:
///  `folder-rule.txt` 57,956 bytes with 524 `uses` lines and 416 `missing-link` lines,
///  `file-rule.txt` 53,048 bytes with 544 `file-uses` lines and 42 `no-file-list` lines.
///  A mebibyte is about 18 times the larger one,
///  so it leaves room for a decade of growth and still bounds a file a person controls.
///  The siblings a reader might expect are an unbounded read
///  and a per-entry limit;
///  a bound on the whole file is the one that keeps a mistyped path from reading a disk image,
///  and the read itself is bounded too,
///  so a file that grows between the check and the read is refused rather than swallowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_BYTES = 1024 * 1024;
/// ```
pub const MAX_BYTES: u64 = 1024 * 1024;

/// What:
///  The entries of one list file.
/// Why:
///  A rule asks it whether a use or a key is tolerated,
///  and reports every entry that no longer matches anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowList = { source: Path; entries: AllowEntry[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowList {
    /// The file the entries were read from.
    source: PathBuf,
    /// The entries in file order.
    entries: Vec<AllowEntry>,
}

/// What:
///  Read a whole list from its text.
/// Why:
///  The lines are read by `layout_allow_entries.rs`,
///  and the list they make is the one a rule asks about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAllowList(text: string, source: Path): AllowList;
/// ```
pub fn parse_allow_list(
    text: &str,
    source: &Path,
) -> Result<AllowList, String> {
    return Ok(AllowList {
        source: source.to_path_buf(),
        entries: parse_entries(text)?,
    });
}

/// What:
///  Read one list file,
///  refusing a path that is not a regular file and a file over `MAX_BYTES`.
/// Why:
///  The option names a file a person edits;
///  reading it must not block on a pipe or fill memory from a device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAllowList(path: Path): AllowList;
/// ```
pub fn read_allow_list(path: &Path) -> Result<AllowList, String> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "The allow-list file {} is not a regular file.",
            path.display()
        ));
    }
    if metadata.len() > MAX_BYTES {
        return Err(format!(
            "The allow-list file {} is {} bytes, over the {MAX_BYTES} bytes this reader accepts.",
            path.display(),
            metadata.len()
        ));
    }
    let file: std::fs::File = std::fs::File::open(path).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    // What: at most one byte more than the cap, so the cap is enforced on what was read and not only on what was measured above.
    // Why: a file that grows between the measurement and the read must be refused, not read whole.
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return format!(
                "The allow-list file {} cannot be read: {error}.",
                path.display()
            );
        })?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!(
            "The allow-list file {} is longer than the {MAX_BYTES} bytes this reader accepts.",
            path.display()
        ));
    }
    let text: String = String::from_utf8(bytes).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    return parse_allow_list(text.as_str(), path);
}

/// What:
///  The list as a rule reads it.
/// Why:
///  A rule asks for what it needs and never walks the entries itself,
///  so every rule answers a dead entry the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AllowList { entries: AllowEntry[]; coversUse(from, to): boolean; coversKey(key): boolean }
/// ```
impl AllowList {
    /// The file the entries came from.
    pub fn source(&self) -> &Path {
        return &self.source;
    }

    /// The entries in file order.
    pub fn entries(&self) -> &[AllowEntry] {
        return &self.entries;
    }

    /// Whether one use between two folders is tolerated.
    pub fn covers_use(
        &self,
        from: &Path,
        to: &Path,
    ) -> bool {
        for entry in &self.entries {
            let AllowKind::FolderUse {
                from: listed_from,
                to: listed_to,
            } = &entry.kind
            else {
                continue;
            };
            if listed_from == from && listed_to == to {
                return true;
            }
        }
        return false;
    }

    /// Whether one use between two files of a folder is tolerated.
    pub fn covers_file_use(
        &self,
        folder: &Path,
        from: &str,
        to: &str,
    ) -> bool {
        for entry in &self.entries {
            let AllowKind::FileUse {
                folder: listed_folder,
                from: listed_from,
                to: listed_to,
            } = &entry.kind
            else {
                continue;
            };
            if listed_folder == folder && listed_from == from && listed_to == to {
                return true;
            }
        }
        return false;
    }

    /// Whether one finding's key is tolerated.
    pub fn covers_key(
        &self,
        key: &str,
    ) -> bool {
        for entry in &self.entries {
            let AllowKind::Key(listed) = &entry.kind else {
                continue;
            };
            if listed == key {
                return true;
            }
        }
        return false;
    }
}

/// What:
///  The findings a list tolerates,
///  and the entries that did the tolerating.
/// Why:
///  A rule reports the entries that tolerated nothing,
///  which is how the list shrinks:
///  a round that fixes a finding must delete its line.
///  The use entries are not matched here;
///  a rule matches them against the uses it read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tolerated(findings: LayoutFinding[], list: AllowList): { kept: LayoutFinding[]; used: Set<number> };
/// ```
pub fn tolerated(
    findings: Vec<LayoutFinding>,
    list: &AllowList,
) -> (Vec<LayoutFinding>, BTreeSet<usize>) {
    let mut kept: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    let mut used: BTreeSet<usize> = BTreeSet::<usize>::new();
    for finding in findings {
        if finding.key.is_empty() || !list.covers_key(finding.key.as_str()) {
            kept.push(finding);
            continue;
        }
        for (index, entry) in list.entries().iter().enumerate() {
            let AllowKind::Key(key) = &entry.kind else {
                continue;
            };
            if *key == finding.key {
                used.insert(index);
            }
        }
    }
    return (kept, used);
}

/// What:
///  The findings for the entries that tolerated nothing.
/// Why:
///  A dead line is the sign the list has not shrunk after the work that made it dead,
///  and the run must say so on the line itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadEntryFindings(list: AllowList, used: Set<number>, base: Path): LayoutFinding[];
/// ```
pub fn dead_entry_findings(
    list: &AllowList,
    used: &BTreeSet<usize>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (index, entry) in list.entries().iter().enumerate() {
        if used.contains(&index) {
            continue;
        }
        findings.push(LayoutFinding {
            file: list.source().to_path_buf(),
            line: entry.line,
            message: format!(
                "The allow-list entry `{}` in {} tolerates nothing any more. Delete the line so the list keeps shrinking.",
                entry.text,
                shown(list.source(), base)
            ),
            help: Some(String::from(
                "Delete this line. The list holds today's findings only, and a line that matches none of them must not wait for a later round.",
            )),
            key: String::new(),
        });
    }
    return findings;
}

/// Controls of the list reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_allow_list_tests.rs"]
mod tests;

//! What:
//!  The list of findings a layout rule tolerates today,
//!  read from the file its `allow` option names.
//! Why:
//!  A crate that switches a layout rule on does not become correct at once:
//!  the rule's findings it has today are recorded, one entry per finding,
//!  and a run fails when a finding is not on the list,
//!  so new code cannot add one.
//!  An entry that stops matching a finding is a finding of its own,
//!  so the list can only shrink.
//! Decisions:
//!  a folder rule entry and a file rule entry that tolerate one use between two folders or two files
//!  are the only entries that suppress more than one finding;
//!  they suppress that one use, so every loop needs all of its uses on the list;
//!  an entry that names no finding any more says so,
//!  because a list that keeps dead lines is not a list of today's findings;
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

/// Import the finding shape,
///  the display helper and native paths.
use crate::layout_finding::{
    LayoutFinding,
    shown,
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeSet,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The largest list file this reader accepts:
///  one mebibyte.
/// Why:
///  This repository's largest list is the git wrapper's folder list,
///  about 90 KiB for today's 1,073 tolerated uses,
///  so a mebibyte leaves a decade of growth and still bounds a file a person controls.
///  The siblings a reader might expect are an unbounded read
///  and a per-entry limit;
///  a bound on the whole file is the one that keeps a mistyped path from reading a disk image.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_BYTES = 1024 * 1024;
/// ```
pub const MAX_BYTES: u64 = 1024 * 1024;

/// The word that starts an entry tolerating one use between two folders.
const FOLDER_USE_WORD: &str = "uses";

/// The word that starts an entry tolerating one use between two files of a folder.
const FILE_USE_WORD: &str = "file-uses";

/// The text that joins the two ends of a tolerated use.
const ARROW: &str = "->";

/// What:
///  What one entry of a list tolerates.
/// Why:
///  A use is suppressed before loops are found;
///  a finding key is matched against the findings that were produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowKind = { kind: 'folder-use'; from: Path; to: Path }
///   | { kind: 'file-use'; folder: Path; from: string; to: string }
///   | { kind: 'key'; key: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AllowKind {
    /// One use between two folders of the package.
    FolderUse {
        /// The folder whose code uses.
        from: PathBuf,
        /// The folder it uses.
        to: PathBuf,
    },
    /// One use between two files of one folder.
    FileUse {
        /// The folder both files are in.
        folder: PathBuf,
        /// The file whose code uses.
        from: String,
        /// The file it uses.
        to: String,
    },
    /// The exact key one finding reports.
    Key(String),
}

/// What:
///  One line of a list, as written and as read.
/// Why:
///  A finding about a dead entry points at that line,
///  and its message quotes the line so the reader can find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowEntry = { line: number; text: string; kind: AllowKind };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowEntry {
    /// One-based line of the list file.
    pub line: usize,
    /// The line as written, without surrounding spaces.
    pub text: String,
    /// What the line tolerates.
    pub kind: AllowKind,
}

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
///  Whether a path an entry names may be used:
///  relative, with no component that leaves the package.
/// Why:
///  A finding shows paths relative to the package,
///  and an absolute path or a `..` would mean a different file on every machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usable(path: string): boolean;
/// ```
fn usable(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let parsed: &Path = Path::new(path);
    if parsed.is_absolute() {
        return false;
    }
    for part in parsed.components() {
        if part.as_os_str() == ".." {
            return false;
        }
    }
    return true;
}

/// What:
///  Read one line as an entry,
///  or say why it names nothing.
/// Why:
///  A line that is a misspelling must fail the run,
///  because a silently ignored line would let a finding through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(line: number, text: string): AllowEntry;
/// ```
fn entry_of(
    line: usize,
    text: &str,
) -> Result<AllowEntry, String> {
    let words: Vec<&str> = text.split_whitespace().collect::<Vec<&str>>();
    let kind: AllowKind = if words.first() == Some(&FOLDER_USE_WORD) {
        if words.len() != 4 || words[2] != ARROW {
            return Err(format!(
                "Line {line} reads `{text}`. A use between two folders is written `uses <folder> {ARROW} <folder>`."
            ));
        }
        if !usable(words[1]) || !usable(words[3]) {
            return Err(format!(
                "Line {line} names a folder that is absolute or climbs out with `..`. A path is relative to the package folder, as findings show it."
            ));
        }
        AllowKind::FolderUse {
            from: PathBuf::from(words[1]),
            to: PathBuf::from(words[3]),
        }
    } else if words.first() == Some(&FILE_USE_WORD) {
        if words.len() != 5 || words[3] != ARROW {
            return Err(format!(
                "Line {line} reads `{text}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`."
            ));
        }
        if !usable(words[1]) {
            return Err(format!(
                "Line {line} names a folder that is absolute or climbs out with `..`. A path is relative to the package folder, as findings show it."
            ));
        }
        AllowKind::FileUse {
            folder: PathBuf::from(words[1]),
            from: String::from(words[2]),
            to: String::from(words[4]),
        }
    } else {
        AllowKind::Key(String::from(text))
    };
    return Ok(AllowEntry {
        line,
        text: String::from(text),
        kind,
    });
}

/// What:
///  Read a whole list from its text.
/// Why:
///  A blank line and a line that starts with `#` carry nothing;
///  every other line must read as one entry,
///  and no line may repeat another,
///  because a list that names one thing twice is not a list of distinct findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAllowList(text: string, source: Path): AllowList;
/// ```
pub fn parse_allow_list(
    text: &str,
    source: &Path,
) -> Result<AllowList, String> {
    let mut entries: Vec<AllowEntry> = Vec::<AllowEntry>::new();
    let mut seen: BTreeSet<String> = BTreeSet::<String>::new();
    for (index, raw) in text.lines().enumerate() {
        let line: usize = index + 1;
        let trimmed: &str = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let entry: AllowEntry = entry_of(line, trimmed)?;
        if !seen.insert(entry.text.clone()) {
            return Err(format!(
                "Line {line} repeats the entry `{}` that an earlier line already holds.",
                entry.text
            ));
        }
        entries.push(entry);
    }
    return Ok(AllowList {
        source: source.to_path_buf(),
        entries,
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
    let text: String = std::fs::read_to_string(path).map_err(|error| {
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

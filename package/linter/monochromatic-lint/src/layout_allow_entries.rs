//! What:
//!  One line of a layout rule's allow list:
//!  the shape it reads as,
//!  the words that spell each shape,
//!  and the one way each shape is written.
//! Why:
//!  A line is the unit a person edits,
//!  and a line that reads as no shape must fail the run rather than pass for an entry,
//!  because a silently ignored line would let a finding through.
//!  The list that holds the entries is `layout_allow_list.rs`;
//!  it reads them from here and never walks a line itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AllowEntry = { line: number; text: string; kind: AllowKind };
//! function parseEntries(text: string): AllowEntry[];
//! ```

/// Import the set that keeps one line from repeating another,
/// and native paths.
use std::{
    collections::BTreeSet,
    path::{
        Path,
        PathBuf,
    },
};

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
    Key(
        /// The key text,
        /// as `--debug` prints it.
        String,
    ),
}

/// What:
///  One line of a list,
/// as written and as read.
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
    /// The line as written,
    /// without surrounding spaces.
    pub text: String,
    /// What the line tolerates.
    pub kind: AllowKind,
}

/// What:
///  What one entry tolerates,
///  written the one way every spelling of it reads.
/// Why:
///  No line may repeat another,
///  and `uses ./src/a/ -> src/b` repeats `uses src/a -> src/b`:
///  a `.` segment and a trailing separator name the same folder,
///  so the list must say so rather than hold two lines for one use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entryIdentity(entry): string
/// ```
impl AllowEntry {
    /// The entry's kind in the words the list writes it.
    pub fn identity(&self) -> String {
        return match &self.kind {
            AllowKind::FolderUse {
                from,
                to,
            } => format!(
                "{FOLDER_USE_WORD} {} {ARROW} {}",
                from.display(),
                to.display()
            ),
            AllowKind::FileUse {
                folder,
                from,
                to,
            } => format!("{FILE_USE_WORD} {} {from} {ARROW} {to}", folder.display()),
            AllowKind::Key(key) => key.clone(),
        };
    }
}

/// What:
///  Why a path an entry names may not be used,
///  or nothing when it may.
/// Why:
///  A finding shows paths relative to the package,
///  and an absolute path or a `..` would mean a different file on every machine.
///  Keys hold `/` between segments on every platform,
///  so an entry written with a backslash would name a path no key matches,
///  and a file named with a backslash cannot be listed at all:
///  saying so is better than a line that tolerates nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unusableReason(path: string): string | undefined;
/// ```
fn unusable_reason(path: &str) -> Option<&'static str> {
    if path.is_empty() {
        return Some("names no folder");
    }
    if path.contains('\\') {
        return Some("holds a backslash, and every path is written with `/` on every machine");
    }
    let parsed: &Path = Path::new(path);
    if parsed.is_absolute() {
        return Some("is absolute");
    }
    for part in parsed.components() {
        if part.as_os_str() == ".." {
            return Some("climbs out of the package with `..`");
        }
    }
    return None;
}

/// What:
///  One path as an entry names it:
///  with `.` segments and a trailing separator gone,
///  so `./src/a` and `src/a/` name the folder a finding names.
/// Why:
///  A use is counted from the path the code makes,
///  while an entry is read from its own words,
///  so a spelling difference between them would make a line that records a use tolerate nothing.
///  A `.` on its own names the package folder itself,
///  which is how a key writes it,
///  and `./` is the same folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listedPath = (text) => text === '.' || text === './' ? '.' : lexical_normal(text)
/// ```
fn listed_path(text: &str) -> PathBuf {
    let normal: PathBuf = crate::markdown_lfs_target::lexical_normal(Path::new(text));
    // What: the spelling a key uses for the package folder, kept instead of being read away.
    // Why: an entry about the package folder must compare equal to the key that names it.
    if normal.as_os_str().is_empty() && !text.is_empty() {
        return PathBuf::from(".");
    }
    return normal;
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
        for path in [words[1], words[3]] {
            if let Some(reason) = unusable_reason(path) {
                return Err(format!(
                    "Line {line} names a folder that {reason}. A path is relative to the package folder, as findings show it."
                ));
            }
        }
        AllowKind::FolderUse {
            from: listed_path(words[1]),
            to: listed_path(words[3]),
        }
    } else if words.first() == Some(&FILE_USE_WORD) {
        if words.len() != 5 || words[3] != ARROW {
            return Err(format!(
                "Line {line} reads `{text}`. A use between two files is written `file-uses <folder> <file> {ARROW} <file>`."
            ));
        }
        if let Some(reason) = unusable_reason(words[1]) {
            return Err(format!(
                "Line {line} names a folder that {reason}. A path is relative to the package folder, as findings show it."
            ));
        }
        AllowKind::FileUse {
            folder: listed_path(words[1]),
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
///  Read every line of a list from its text.
/// Why:
///  A blank line and a line that starts with `#` carry nothing;
///  every other line must read as one entry,
///  and no line may repeat another,
///  because a list that names one thing twice is not a list of distinct findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseEntries(text: string): AllowEntry[];
/// ```
pub fn parse_entries(text: &str) -> Result<Vec<AllowEntry>, String> {
    let mut entries: Vec<AllowEntry> = Vec::<AllowEntry>::new();
    let mut seen: BTreeSet<String> = BTreeSet::<String>::new();
    for (index, raw) in text.lines().enumerate() {
        let line: usize = index + 1;
        let trimmed: &str = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let entry: AllowEntry = entry_of(line, trimmed)?;
        if !seen.insert(entry.identity()) {
            return Err(format!(
                "Line {line} repeats the entry `{}` that an earlier line already holds.",
                entry.text
            ));
        }
        entries.push(entry);
    }
    return Ok(entries);
}

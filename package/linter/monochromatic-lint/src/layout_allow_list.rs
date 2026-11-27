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
/// Import the finding shape, the display helper and the one spelling a key gives a path.
use crate::{
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
    },
    markdown_lfs_target::lexical_normal,
};
/// Import ordered collections and native paths.
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    io::Read,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  How much work one run of a rule did on its allow list,
///  for the controls that pin the cost of a lookup.
/// Why:
///  A cost has no test of its own,
///  and a control that timed one would read this machine's load rather than the code,
///  so a control counts the entries a question walked instead and fails when the index is gone.
///  Test builds only, so a release artifact carries no counter and no branch on one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let entriesWalked = 0; // one per entry a question read, and none when an index answers it
/// ```
#[cfg(test)]
pub(crate) mod counted_work {
    /// Import the cell the counter lives in.
    use std::cell::Cell;
    // What: one counter per thread, because the test runner runs several tests at once in one process.
    // Why: a counter shared between tests would add two tests' entries together and make both controls meaningless.
    thread_local! {
        /// The entries of a list that a question read one by one.
        static ENTRIES_WALKED: Cell<usize> = const { Cell::new(0) };
    }

    /// Count one entry a question read.
    pub(crate) fn note_entry() {
        ENTRIES_WALKED.with(|count| return count.set(count.get() + 1));
    }

    /// How many entries this thread walked.
    pub(crate) fn entries_walked() -> usize {
        return ENTRIES_WALKED.with(|count| return count.get());
    }

    /// Start one control from nothing counted.
    pub(crate) fn reset() {
        ENTRIES_WALKED.with(|count| return count.set(0));
    }
}

/// What:
///  The largest list file this reader accepts:
///  four mebibytes.
/// Why:
///  This repository's largest lists are the git wrapper's,
///  measured on 2026-11-25 from the branch that builds them:
///  `folder-rule.txt` 57,956 bytes with 524 `uses` lines and 416 `missing-link` lines,
///  `file-rule.txt` 53,048 bytes with 544 `file-uses` lines and 42 `no-file-list` lines.
///  The list a loop needs is one line per use inside it,
///  and the dense shape of the alpha.15 release pass
///  (200 groups whose own files use one folder that uses a child of each of them,
///  60,100 uses in one loop)
///  needs 60,300 lines and 1,681,290 bytes,
///  measured on 2026-11-26 while that pass's findings were fixed:
///  one mebibyte refused the very list the rule asked the round to write.
///  Four mebibytes leave room for about 150,000 recorded uses
///  and still bound a file a person controls:
///  the bound is what keeps a mistyped path from reading a disk image,
///  and the read itself is bounded too,
///  so a file that grows between the check and the read is refused rather than swallowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_BYTES = 4 * 1024 * 1024;
/// ```
pub const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// What:
///  The entries of one list file.
/// Why:
///  A rule asks it whether a use or a key is tolerated,
///  and reports every entry that no longer matches anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllowList = {
///   source: Path;
///   entries: AllowEntry[];
///   uses: Set<[Path, Path]>;
///   keys: Map<string, number>;
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowList {
    /// The file the entries were read from.
    source: PathBuf,
    /// The entries in file order.
    entries: Vec<AllowEntry>,
    /// The uses the entries record, so one lookup is not a pass over every entry.
    uses: BTreeSet<(PathBuf, PathBuf)>,
    /// Each key the entries record and the line that holds it,
    ///  so one lookup is not a pass over every entry.
    keys: BTreeMap<String, usize>,
}

/// What:
///  Read a whole list from its text.
/// Why:
///  The lines are read by `layout_allow_entries.rs`,
///  and the list they make is the one a rule asks about.
///  The uses, the keys and the line each key sits on are also collected once here,
///  because a rule asks about one use or one key per finding it makes,
///  and a list of a dense loop holds a line for every use inside that loop:
///  a pass over every entry per question cost minutes on a list of 60,300 lines
///  (measured 2026-11-26 on the alpha.15 release pass's densest loop).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAllowList(text: string, source: Path): AllowList;
/// ```
pub fn parse_allow_list(
    text: &str,
    source: &Path,
) -> Result<AllowList, String> {
    let entries: Vec<AllowEntry> = parse_entries(text)?;
    let mut uses: BTreeSet<(PathBuf, PathBuf)> = BTreeSet::<(PathBuf, PathBuf)>::new();
    let mut keys: BTreeMap<String, usize> = BTreeMap::<String, usize>::new();
    for (index, entry) in entries.iter().enumerate() {
        match &entry.kind {
            AllowKind::FolderUse {
                from,
                to,
            } => {
                uses.insert((from.clone(), to.clone()));
            }
            AllowKind::Key(key) => {
                keys.insert(key.clone(), index);
            }
            // What: a file use is matched by walking the entries instead.
            // Why: a folder's file uses are few (the wrapper's largest list holds 544 lines for 2,255 keys),
            // and the index for the last file use of that list measured no cost beyond the run-to-run spread
            // (0.86 to 1.04 against a spread of 0.87 to 0.98 on 2026-11-27, three interleaved rounds).
            // The siblings a reader might expect are a set beside `uses` and a set beside `keys`.
            AllowKind::FileUse {
                ..
            } => {}
        }
    }
    return Ok(AllowList {
        source: source.to_path_buf(),
        entries,
        uses,
        keys,
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
    let bytes: Vec<u8> = read_at_most(file, path)?;
    let text: String = String::from_utf8(bytes).map_err(|error| {
        return format!(
            "The allow-list file {} cannot be read: {error}.",
            path.display()
        );
    })?;
    return parse_allow_list(text.as_str(), path);
}

/// What:
///  Read at most the list cap from one reader,
///  refusing anything longer.
/// Why:
///  The size of the file was measured before this read,
///  and a file that grows between the measurement and the read would get past that check,
///  so the read itself stops one byte past the cap and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAtMost(reader: ReadStream, path: Path): Uint8Array;
/// ```
fn read_at_most<R: std::io::Read>(
    reader: R,
    path: &Path,
) -> Result<Vec<u8>, String> {
    // What: at most one byte more than the cap, so the length read is known even when it is over.
    // Why: reading the whole of a file that grew past the cap is what the bound exists to prevent.
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    reader
        .take(MAX_BYTES + 1)
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
    return Ok(bytes);
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
        return self.uses.contains(&(from.to_path_buf(), to.to_path_buf()));
    }

    /// Whether one use between two files of a folder is tolerated.
    pub fn covers_file_use(
        &self,
        folder: &Path,
        from: &str,
        to: &str,
    ) -> bool {
        for entry in &self.entries {
            // What: one entry this question reads, which a control counts.
            // Why: the walk is what the README says a file-use lookup does, so a control holds the sentence true.
            #[cfg(test)]
            counted_work::note_entry();
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
        return self.keys.contains_key(key);
    }

    /// The line of the entry that holds one finding's key,
    ///  or nothing when no line holds it.
    pub fn key_entry(
        &self,
        key: &str,
    ) -> Option<usize> {
        return self.keys.get(key).copied();
    }

    /// What:
    ///  The list as one package's rule asks it, with every `uses` path written the way a key writes it.
    /// Why:
    ///  A line may climb out of the package with `..` and come back into it:
    ///  with the package folder named `p`, `uses ../p/src/a -> src/b` names the use `src/a -> src/b`,
    ///  which is the spelling every key prints.
    ///  The tolerance lookups compared the listed spelling with the key's and found neither,
    ///  while the dead-entry check read the line as alive,
    ///  so one use had two answers and a mistyped line stayed unreported.
    ///  Reading each path back against the package folder and writing it from there gives one spelling,
    ///  and it is the key's, which is the one a reader copies out of `--debug`.
    ///  A line's own text stays as written, because a dead entry quotes it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// list.inKeySpelling(base): AllowList // 'uses ../p/src/a -> src/b' holds the use 'src/a -> src/b'
    /// ```
    pub fn in_key_spelling(
        &self,
        base: &Path,
    ) -> AllowList {
        // What: the entries with every `uses` path in the key's spelling, in file order.
        // Why: a rule reports a dead entry on its line, so the order and the text of each line must survive.
        let mut entries: Vec<AllowEntry> = Vec::<AllowEntry>::with_capacity(self.entries.len());
        let mut uses: BTreeSet<(PathBuf, PathBuf)> = BTreeSet::<(PathBuf, PathBuf)>::new();
        for entry in &self.entries {
            let kind: AllowKind = match &entry.kind {
                AllowKind::FolderUse {
                    from,
                    to,
                } => {
                    let spelled_from: PathBuf = PathBuf::from(spelled_path(base, from).as_str());
                    let spelled_to: PathBuf = PathBuf::from(spelled_path(base, to).as_str());
                    uses.insert((spelled_from.clone(), spelled_to.clone()));
                    AllowKind::FolderUse {
                        from: spelled_from,
                        to: spelled_to,
                    }
                }
                // A file use names a folder and two file names, and a key holds all three as written.
                AllowKind::FileUse {
                    ..
                }
                | AllowKind::Key(_) => entry.kind.clone(),
            };
            entries.push(AllowEntry {
                line: entry.line,
                text: entry.text.clone(),
                kind,
            });
        }
        return AllowList {
            source: self.source.clone(),
            entries,
            uses,
            keys: self.keys.clone(),
        };
    }
}

/// What:
///  One listed path written the way a key writes that folder:
///  the path read back against the package folder, then shown from it.
/// Why:
///  That is the one spelling both a tolerance lookup and a dead-entry check must compare,
///  and `key_path` is what every finding's key already writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spelledPath = (base, listed) => keyPath(lexicalNormal(base.join(listed)), base);
/// ```
fn spelled_path(
    base: &Path,
    listed: &Path,
) -> String {
    return key_path(&lexical_normal(&base.join(listed)), base);
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
        // What: a finding that carries no key, which no line can hold.
        // Why: the loop findings and the use added inside a tolerated loop are what makes new code unrecordable.
        if finding.key.is_empty() {
            kept.push(finding);
            continue;
        }
        // What: the line that holds this finding's key, which the list read once.
        // Why: a pass over every entry per tolerated finding reads a list of tens of thousands of lines once per finding.
        let Some(index) = list.key_entry(finding.key.as_str()) else {
            kept.push(finding);
            continue;
        };
        used.insert(index);
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

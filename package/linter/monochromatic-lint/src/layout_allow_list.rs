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
//!  and never absolute,
//!  because the list is read on every machine that lints the repository,
//!  while a `..` in one is the walk from the package folder to a module a `#[path]` names elsewhere.
//!  Where each part of the reader lives:
//!  `layout_allow_list_reading` reads one list file under its byte cap,
//!  and `layout_allow_list_dead_entry_findings` holds the finding for a line that tolerates nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AllowEntry = { line: number; text: string; kind: AllowKind };
//! type AllowKind = { kind: 'folder-use'; from: Path; to: Path }
//!   | { kind: 'file-use'; folder: Path; from: string; to: string }
//!   | { kind: 'key'; key: string };
//! function readAllowList(path: Path, base: Path): AllowList;
//! ```

/// Import the entry shapes and the line reader the list is built from,
///  re-exported so a caller asks the list module for both.
pub use crate::layout_allow_entries::{
    AllowEntry,
    AllowKind,
    parse_entries,
};
/// Import the finding for a line that tolerates nothing,
///  re-exported so a caller asks the list module for it.
pub(crate) use crate::layout_allow_list_dead_entry_findings::dead_entry_findings;
/// Import the reader of one list file,
///  re-exported so a caller asks the list module for it.
pub use crate::layout_allow_list_reading::read_allow_list;
/// Import the byte cap and the bounded reader the list's controls name here.
#[cfg(test)]
pub(crate) use crate::layout_allow_list_reading::{
    MAX_BYTES,
    read_at_most,
};
/// Import the finding shape and the path a key writes.
use crate::layout_finding::LayoutFinding;
/// Import ordered collections and native paths.
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
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
///  The package folder is passed with the text,
///  because one listed path has two spellings and the keys write only one of them.
///  The uses, the keys and the line each key sits on are also collected once here,
///  because a rule asks about one use or one key per finding it makes,
///  and a list of a dense loop holds a line for every use inside that loop:
///  a pass over every entry per question cost minutes on a list of 60,300 lines
///  (measured 2026-11-26 on the alpha.15 release pass's densest loop).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAllowList(text: string, source: Path, base: Path): AllowList;
/// ```
pub fn parse_allow_list(
    text: &str,
    source: &Path,
    base: &Path,
) -> Result<AllowList, String> {
    let entries: Vec<AllowEntry> = parse_entries(text, base)?;
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
///  Whether one entry of the list is the line that holds one finding's key.
/// Why:
///  A scan over the entries asks this of every entry before the one it wants,
///  and the index names the one entry to ask,
///  so a control that counts the entries read tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryHoldsKey(entry: AllowEntry, key: string): boolean;
/// ```
fn entry_holds_key(
    entry: &AllowEntry,
    key: &str,
) -> bool {
    // What: one entry this question read, which a control counts.
    // Why: the index answers with the one entry that holds the key, where a scan reads one entry for each before it.
    #[cfg(test)]
    counted_work::note_entry();
    // What: the key this entry holds, or nothing when the entry is a `uses` or a `file-uses` line.
    // Why: only a key line can tolerate a finding by its key.
    let AllowKind::Key(held) = &entry.kind else {
        // What: `false` is the answer for every entry that is no key line.
        // Why: a use line tolerates a use, never a key.
        return false;
    };
    return held == key;
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
        // What: the position the index holds for this key, which a scan would reach by reading one entry at a time.
        // Why: the index is built from the entries once, so one question never walks a list that holds tens of thousands of lines.
        let Some(index) = self.keys.get(key).copied() else {
            // What: `None` is the absent variant of `Option`.
            // Why: no line holds this key, so no line tolerates the finding.
            return None;
        };
        // What: the one entry that position names, read to answer with its position.
        // Why: reading it holds the index to the entries it was built from,
        // and the control that counts the entries a question reads then sees this question at all,
        // where an answer that reads no entry left that control reading zero whatever the lookup did.
        if !entry_holds_key(&self.entries[index], key) {
            // What: `None` is the absent variant of `Option`.
            // Why: an index that disagrees with its entries answers no line, rather than one that holds another key.
            return None;
        }
        return Some(index);
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

/// Controls of the list reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_allow_list_tests.rs"]
mod tests;

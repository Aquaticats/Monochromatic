//! What:
//!  The clone rung for a source with uncommitted changes:
//! clone every tracked path the proof settles,
//! build the new
//!  worktree's index from the tree of `HEAD`,
//! and let Git write the paths the changes reach.
//! Why:
//!  A source checkout that is not clean used to end the clone rung,
//! so the machine whose main checkout other
//!  sessions keep dirty paid the clean proof and then Git's whole checkout on every add.
//! The files Git's
//!  checkout would write are the tree's bytes,
//! and a path whose file its own index entry still proves already holds
//!  them,
//! so those paths are cloned and only the path a change reaches is written by Git
//!  (`doc/planning/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`).
//! The index cannot come
//!  from the source:
//! a staged path records another blob,
//! a staged deletion is missing from it,
//! and a staged addition
//!  names a path no tree has,
//! so the index is built from the tree with the stat fields of the files the clones made.
//! Git writes
//!  the rest with one `git checkout-index --stdin`,
//! which runs the checkout's conversion filters exactly as Git's own
//!  checkout does,
//! and the rung then proves the worktree with one `git status` and one index comparison.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const part = cloneTheUnchanged({ context, source, target, destination, admin }); // throws the reason Git's checkout is needed
//! ```

use crate::add_timing::begin_step;
use crate::add_timing::end_step;
use crate::diagnostic_log::debug;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_index::parse_index;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{FlatEntry, flatten_tree};
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::install_index_in;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::worktree_clone::Created;
use crate::worktree_clone::clone_the_entries_of;
use crate::worktree_clone_partial_index::{fields_the_source_recorded, index_of_the_tree};
use crate::worktree_clean::stored_stats;
use crate::worktree_clone_partial_split::Split;
use crate::worktree_clone_partial_split::split_the_tree;
use crate::worktree_ladder_target::{Target, checkout_environment};
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeClonePartial";

/// The variable that sets the smallest unchanged file this prototype clones;
/// it exists only to measure where cloning wins on this host and is not the shipped rule.
pub const PROTOTYPE_CLONE_MIN_BYTES_VARIABLE: &str = "CLI_GIT_PROTOTYPE_CLONE_MIN_BYTES";

/// The variable that sets the total tracked bytes below which this prototype declines the rung and lets Git's checkout
/// make the worktree;
/// it exists only to measure the whole-tree rule and is not the shipped rule.
pub const PROTOTYPE_DECLINE_BELOW_TOTAL_BYTES_VARIABLE: &str = "CLI_GIT_PROTOTYPE_DECLINE_BELOW_TOTAL_BYTES";

/// What:
///  The two size rules a partial clone can apply to the paths its proof settled.
/// Why:
///   A clone of a small file on ZFS costs more than Git's write of it,
/// so a per-file rule clones only the files at or above a
///  size where cloning wins,
/// and a whole-tree rule declines the rung where the tree holds too little data for cloning to win.
/// Both are measured before either is chosen;
/// the variables above exist for that measurement and for nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SizeRules = { cloneMinBytes: number; declineBelowTotalBytes: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SizeRules {
    /// The smallest unchanged file a clone takes;
    /// zero clones every one of them.
    pub clone_min_bytes: u64,
    /// The tracked bytes below which the rung is declined;
    /// zero never declines.
    pub decline_below_total_bytes: u64,
}

impl SizeRules {
    /// The rules the prototype's variables name,
    /// and no rule at all when neither is set.
    pub fn from_the_environment() -> SizeRules {
        return SizeRules { clone_min_bytes: number_variable(PROTOTYPE_CLONE_MIN_BYTES_VARIABLE), decline_below_total_bytes: number_variable(PROTOTYPE_DECLINE_BELOW_TOTAL_BYTES_VARIABLE) };
    }
}

/// What:
///  The number a variable names,
/// or zero when it is unset or names no number.
/// Why:
///   A measurement sets these variables itself,
/// and a value that does not parse must leave the rung as it was
///  rather than stop an add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = Number(process.env[name] ?? '0'); return Number.isSafeInteger(value) && value > 0 ? value : 0;
/// ```
fn number_variable(name: &str) -> u64 {
    let Some(value) = std::env::var_os(name) else { return 0 };
    return value.to_string_lossy().trim().parse::<u64>().unwrap_or(0);
}

/// What:
///  What one partial clone made.
/// Why:
///   A later step that fails reports the entries the clones created,
/// and the two counts say what the rung did
///  without reading the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PartCloned = { cloned: number; written: number; created: Created[] };
/// ```
#[derive(Debug)]
pub struct PartCloned {
    /// How many tracked paths were cloned.
    pub cloned: usize,
    /// How many tracked paths Git wrote.
    pub written: usize,
    /// The entries the clones created,
    /// in creation order.
    pub created: Vec<Created>,
}

impl PartCloned {
    /// The entries the clones created,
    /// in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }
}

/// What:
///  The stat fields of no path:
/// the index built from the tree already holds the fields of every file the clones
/// made.
/// Why:
///   `install_index_in` refreshes the fields of the entries it is given,
/// and a partial clone hands it bytes whose
///  fields are already the objects it made,
/// so it refreshes none of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NoStats implements StatLookup { statNow() { return undefined; } }
/// ```
struct NoStats;

impl crate::subvolume_index::StatLookup for NoStats {
    fn stat_now(&mut self, _path: &[u8]) -> Option<StatFields> {
        return None;
    }
}

/// What:
///  The tracked paths the source's index and the tree of `HEAD` name,
/// and the tree itself.
/// Why:
///   The split compares three things (the index entry,
/// the tree entry and the file),
/// and the index build
///  takes the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Tracked = { entries: StatEntry[]; indexed: IndexEntry[]; tree: FlatEntry[] };
/// ```
struct Tracked {
    /// The index's entries with their recorded stat data.
    entries: Vec<crate::worktree_clean::StatEntry>,
    /// The same entries as the plain index reader gives them,
    /// with their stage and extended flags.
    indexed: Vec<crate::repo_read_index::IndexEntry>,
    /// The flattened tree of `HEAD`.
    tree: Vec<FlatEntry>,
}

/// What:
///  Read the source's index and the tree of its `HEAD`.
/// Why:
///   The split judges every tree path against the index entry and the file,
/// and the index build takes
///  the tree,
/// so one reading answers both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tracked = readTracked(source); // throws the reason
/// ```
fn read_tracked(source: &SnapshotSource) -> Result<Tracked, String> {
    let hash_length: usize = source.clean.hash_length;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let bytes: &[u8] = source.clean.index_bytes.as_slice();
    let Some(entries) = stored_stats(bytes, hash_length) else { return Err(String::from("the source's index changed shape")) };
    let Some(indexed) = parse_index(bytes, format) else { return Err(String::from("the source's index has a layout the reader does not know")) };
    let Some(database) = ObjectDatabase::open(source.common_dir.as_path(), None, format) else { return Err(String::from("the object database has a layout the reader does not know")) };
    let Some(tree) = flatten_tree(&database, source.clean.head_tree.as_slice(), format) else { return Err(String::from("the tree of HEAD is unreadable")) };
    return Ok(Tracked { entries, indexed, tree });
}

/// What:
///  The request that writes `paths` in the new worktree from its index.
/// Why:
///   `git checkout-index` is the scripted form of Git's own checkout:
/// it writes the entry's object through the
///  checkout's conversion filters and records the new stat data in the index,
/// and it takes the paths on its standard input,
///  so a source with many changed paths names none of them in the argument list.
/// `--index` is what makes the first `git status` of the
///  worktree cheap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = checkoutIndexRequest({ destination, admin, paths });
/// ```
fn checkout_index_request(destination: &Path, admin: &Path, paths: &[Vec<u8>]) -> GitRequest {
    let mut input: Vec<u8> = Vec::with_capacity(paths.iter().map(Vec::len).sum::<usize>() + paths.len());
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    return GitRequest {
        cwd: destination.to_path_buf(),
        arguments: ["checkout-index", "--stdin", "-z", "--index"].iter().map(OsString::from).collect(),
        without_prefix: true,
        environment: checkout_environment(admin, destination),
        input: Some(input),
        ..GitRequest::default()
    };
}

/// What:
///  Let Git write `paths` in the new worktree from its index.
/// Why:
///   The paths are the ones the proof handed to Git,
/// and every one of them is in the tree the index was built from.
/// A run that fails ends the rung,
/// and the checkout that completes the worktree writes the same bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeTheRest({ context, destination, admin, paths }); // throws Git's words
/// ```
fn write_the_rest(context: &GitContext, destination: &Path, admin: &Path, paths: &[Vec<u8>]) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let request: GitRequest = checkout_index_request(destination, admin, paths);
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git checkout-index` could not start ({error})")),
    };
    if output.code == Some(0) {
        return Ok(());
    }
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_string();
    return Err(format!("`git checkout-index` ended with {:?}: {said}", output.code));
}

/// What:
///  Run one captured Git query in the new worktree and answer its status and standard output,
/// or the reason it
///  could not be run at all.
/// Why:
///   The proof of a partial clone asks Git twice,
/// and both runs name the worktree's own Git directory and
///  work tree,
/// so neither can read the source's.
///A run that reports a failure through its status is an answer,
///  not an error:
/// a quiet
///  difference check says "they differ" with status 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout } = askInTheWorktree({ context, target, workTree, words }); // throws the reason it did not start
/// ```
fn ask_in_the_worktree(context: &GitContext, target: &Target, work_tree: &Path, words: &[&str]) -> Result<(Option<i32>, Vec<u8>), String> {
    let request: GitRequest = GitRequest {
        cwd: work_tree.to_path_buf(),
        arguments: words.iter().copied().map(OsString::from).collect(),
        without_prefix: true,
        environment: checkout_environment(target.admin.as_path(), work_tree),
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git {}` could not start ({error})", words.join(" "))),
    };
    return Ok((output.code, output.stdout));
}

/// What:
///  Prove that the new worktree holds what Git's own `worktree add` would have written:
/// no worktree file
///  differs from the index entry Git's own status judges,
/// and the index equals the new `HEAD`'s tree.
/// Why:
///   The clones and the paths Git wrote must leave the state Git's checkout leaves,
/// and Git's status is the
///  judge a person would ask,
/// so the proof is Git's own answer and not this process's opinion of it.
/// `--no-optional-locks` is
///  what keeps the proof from writing an index:
/// a read-only proof must not leave a mark on the worktree it
///  judges,
/// and an entry whose fields stayed stale costs the person one hash later,
/// never a wrong byte.
/// The status is
///  also the honest judge of the entries whose recorded fields a link keeps from its source,
/// which a quiet
///  `git diff-files` reports as changed without ever comparing the target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheWorktree({ context, target, workTree }); // throws the reason it is not Git's result
/// ```
pub fn prove_the_worktree(context: &GitContext, target: &Target, work_tree: &Path) -> Result<(), String> {
    let (listed_code, listed) = ask_in_the_worktree(context, target, work_tree, &["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=all", "--no-renames"])?;
    if listed_code != Some(0) {
        return Err(format!("`git status` ended with {listed_code:?} in the new worktree"));
    }
    if let Some(first) = listed.split(|byte: &u8| { return *byte == 0; }).find(|entry| { return !entry.is_empty(); }) {
        let shown: String = String::from_utf8_lossy(&first[3.min(first.len())..]).into_owned();
        return Err(format!("the new worktree is not what Git's checkout makes: {} differs", crate::worktree_snapshot_source::plain(shown.as_str())));
    }
    // The index's own side: every path of the new `HEAD`'s tree, and nothing else.
    let (compared_code, _quiet) = ask_in_the_worktree(context, target, work_tree, &["--no-optional-locks", "diff-index", "--cached", "--quiet", target.commit.as_str()])?;
    if compared_code != Some(0) {
        return Err(String::from("the new worktree's index does not equal the tree of the new HEAD"));
    }
    return Ok(());
}

/// What:
///  Move the unchanged paths a size rule keeps from the clones to the paths Git writes,
/// or refuse the rung when the whole tree is
///  below the threshold,
/// and answer the split that then holds.
/// Why:
///   The two rules are measured against Git's own checkout before either is chosen,
/// and a rule that only moves a path between
///  the clones and the write step needs nothing else in the rung;
/// the write step already writes any path whose entry it is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const split = applyTheSizeRules({ split, entries, rules }); // throws when the tree is too small for the rung
/// ```
fn apply_the_size_rules(split: Split, entries: &[crate::worktree_clean::StatEntry], rules: &SizeRules) -> Result<Split, String> {
    if rules.decline_below_total_bytes > 0 {
        let total: u64 = entries.iter().map(|entry| { return u64::from(entry.stored.size); }).sum();
        if total < rules.decline_below_total_bytes {
            return Err(format!("the prototype's whole-tree rule declines the rung: {total} tracked bytes are below {}", rules.decline_below_total_bytes));
        }
    }
    if rules.clone_min_bytes == 0 {
        return Ok(split);
    }
    let mut clones: Vec<usize> = Vec::with_capacity(split.clones.len());
    let mut writes: Vec<Vec<u8>> = split.writes;
    for at in split.clones {
        let Some(entry) = entries.get(at) else { continue };
        if u64::from(entry.stored.size) < rules.clone_min_bytes {
            writes.push(entry.path.clone());
            continue;
        }
        clones.push(at);
    }
    // The write step and the proof ask nothing of the order,
    // but a record of what the rung did reads better in path order.
    writes.sort();
    return Ok(Split { clones, writes });
}

/// What:
///  The stat fields of the paths the clones made,
/// with a link's entry keeping the fields its source recorded.
/// Why:
///   A link is cloned by making another link with the same target,
/// and the object made is a different inode with a new change time,
/// so fields taken from it would certify a name another process may have replaced since;
/// the source's fields name the target's length,
/// which is what Git compares a link by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stats = statsOfTheClones({ made, split, entries });
/// ```
fn stats_of_the_clones(made: std::collections::HashMap<Vec<u8>, StatFields>, split: &Split, entries: &[crate::worktree_clean::StatEntry]) -> std::collections::HashMap<Vec<u8>, StatFields> {
    let mut stats: std::collections::HashMap<Vec<u8>, StatFields> = made;
    for at in &split.clones {
        let Some(entry) = entries.get(*at) else { continue };
        if entry.mode & 0o170_000 == 0o120_000 {
            stats.insert(entry.path.clone(), fields_the_source_recorded(entry.stored));
        }
    }
    return stats;
}

/// What:
///  Clone the unchanged tracked paths of a source with changes into the destination,
/// install the index built from
///  the tree of `HEAD`,
/// and let Git write the paths the changes reach.
/// Why:
///   See the module description.
/// Nothing is removed on a failure:
/// the entries the clones created stay and are counted by the
///  caller,
/// and the checkout that completes the worktree overwrites the tracked ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const part = cloneTheUnchanged({ context, source, target, destination, admin }); // throws the reason
/// ```
pub fn clone_the_unchanged(context: &GitContext, source: &SnapshotSource, target: &Target, destination: &Path, held: &File, admin: &File, rules: &SizeRules) -> Result<PartCloned, String> {
    let read_entered: bool = begin_step("partRead");
    let read_outcome = read_tracked(source);
    end_step(read_entered);
    let tracked: Tracked = read_outcome?;
    let classify_entered: bool = begin_step("partClassify");
    let classify_outcome = split_the_tree(tracked.entries.as_slice(), tracked.indexed.as_slice(), tracked.tree.as_slice(), source.root.as_path(), source.clean.index_seconds);
    end_step(classify_entered);
    let settled: Split = classify_outcome.map_err(|refusal| { return refusal.reason; })?;
    let split: Split = apply_the_size_rules(settled, tracked.entries.as_slice(), rules)?;
    let cloned: usize = split.clones.len();
    let written: usize = split.writes.len();
    let mut created: Vec<Created> = Vec::new();
    let taken = split.cloned_entries(tracked.entries.as_slice());
    let made: std::collections::HashMap<Vec<u8>, StatFields> = clone_the_entries_of(source, taken, held, &mut created)?;
    let stats: std::collections::HashMap<Vec<u8>, StatFields> = stats_of_the_clones(made, &split, tracked.entries.as_slice());
    let index_entered: bool = begin_step("partIndex");
    let index_outcome = index_of_the_tree(tracked.tree.as_slice(), &stats, source.clean.hash_length).and_then(|bytes| { return install_index_in(admin, bytes.as_slice(), &mut NoStats, source.clean.hash_length); });
    end_step(index_entered);
    let report: String = index_outcome?;
    debug(TAG, report.as_str());
    let write_entered: bool = begin_step("partWrite");
    let write_outcome = write_the_rest(context, destination, target.admin.as_path(), split.writes.as_slice());
    end_step(write_entered);
    write_outcome?;
    debug(TAG, format!("cloned {cloned} of {} tracked paths and had Git write {written} of them", tracked.tree.len()).as_str());
    return Ok(PartCloned { cloned, written, created });
}

/// Partial-clone controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_partial_tests.rs"]
mod tests;

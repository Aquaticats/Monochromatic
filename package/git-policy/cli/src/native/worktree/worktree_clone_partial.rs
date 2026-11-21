//! What:
//!  The clone rung for a source with uncommitted changes:
//! clone every tracked path the proof settles,
//! build the
//!  new worktree's index from the tree of the source's `HEAD`,
//! and let Git write every other path,
//!  the changed ones and the one the conversion fence takes.
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
//! so the index is built from the tree with the stat fields of the files the clones made,
//!  and written under the fence's own waiting name (`worktree_clone_index`).
//! Git writes the rest through the fence's one
//!  `git checkout-index`,
//! which runs the checkout's conversion filters exactly as Git's own checkout does,
//! and the index takes the
//!  worktree's name only after that run,
//! so a kill in the window leaves no index that could vouch for a path Git had not
//!  written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const part = cloneTheUnchanged({ context, source, destination, adminPath, fence, rules }); // throws the reason Git's checkout is needed
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::diagnostic_log::debug;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_index::parse_index;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{FlatEntry, flatten_tree};
use crate::subvolume_index::StatFields;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::worktree_clone::Created;
use crate::worktree_clone::clone_these_bytes_beside;
use crate::worktree_clone_conversion_beside_the_clone::fence_beside_the_clone;
use crate::worktree_clone_conversion_fence::{AttributeFence, Fence};
use crate::worktree_clone_index::{PendingIndex, install_the_index_beside_the_write};
use crate::worktree_clone_partial_index::{fields_the_source_recorded, index_of_the_tree};
use crate::worktree_clone_partial_split::Split;
use crate::worktree_clone_partial_split::split_the_tree;
use crate::worktree_clean::stored_stats;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeClonePartial";

/// The mode bits that say an entry is a link rather than a file.
const MODE_SYMLINK: u32 = 0o120_000;

/// The type bits of an entry's mode.
const MODE_TYPE: u32 = 0o170_000;

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
///  One partial clone's inputs:
/// what it reads from Git,
/// what the source check proved,
/// where its bytes go,
/// the fence the attributes half already answered,
/// and the prototype's size rules.
/// Why:
///   The rung reads the source,
/// the destination and the administrative directory of the new worktree,
/// and it must not choose the conversion fence itself:
/// the ladder decided the decline and read the attributes before any copy (`the_fence_of_this_build`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PartialClone = { context: GitContext; source: SnapshotSource; destination: File; admin: File; adminPath: Path; fence: AttributeFence; rules: SizeRules };
/// ```
pub(crate) struct PartialClone<'a> {
    /// The Git face the rung's own runs go through.
    pub context: &'a GitContext,
    /// What the source check proved about the source.
    pub source: &'a SnapshotSource,
    /// The directory Git made for the new worktree,
    /// with no checkout,
    /// held open.
    pub destination: &'a File,
    /// The new worktree's administrative directory,
    /// held open.
    pub admin: &'a File,
    /// The new worktree's administrative directory,
    /// as the path a Git run is given.
    pub admin_path: &'a Path,
    /// The attributes the fence answered before the copies,
    /// and the rules it judged them by.
    pub fence: AttributeFence,
    /// The prototype's size rules;
    /// no rule at all is the shipped shape of this rung.
    pub rules: SizeRules,
}

/// What:
///  The tracked paths the source's index and the tree of the source's `HEAD` name,
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
    /// The flattened tree of the source's `HEAD`.
    tree: Vec<FlatEntry>,
}

/// What:
///  Read the source's index and the tree of its `HEAD`.
/// Why:
///   The split judges every tree path against the index entry and the file,
/// and the index build takes
///  the tree,
/// so one reading answers both,
/// and the tree is the source's `HEAD`'s because that is what the split and the index name;
/// the ladder's `read-tree -m -u` carries a differing new `HEAD` in afterwards.
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
    // The write step and the index ask nothing of the order,
    // but a record of what the rung did reads better in path order.
    writes.sort();
    return Ok(Split { clones, writes });
}

/// What:
///  The paths the source's index holds another object or mode for than the tree of `HEAD`,
/// asked of Git in the source itself;
/// the error says why the question could not be answered.
/// Why:
///   The split decides which paths are staged from the index it read and the tree it flattened,
/// and that one
///  comparison is the only thing between a staged path and a clone;
/// Git's own listing is a second,
///  differently built answer for the class where a mistake is worst,
/// and the rung refuses when the two disagree
///  rather than clone the person's bytes into a worktree that must hold the commit's.
/// The listing is against the source's `HEAD`,
///  because that is the tree the split and the new index are built from;
/// `read-tree -m -u` carries the difference to a new `HEAD` afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = stagedPaths(context, source); // throws the reason Git could not answer
/// ```
fn staged_paths(context: &GitContext, source: &SnapshotSource) -> Result<Vec<Vec<u8>>, String> {
    let mut request: GitRequest = GitRequest::new(source.root.as_path(), &["--no-optional-locks", "diff-index", "--cached", "--name-only", "-z", source.clean.head_commit.as_str()]);
    request.without_prefix = true;
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git diff-index --cached --name-only` could not start ({error})")),
    };
    if output.code != Some(0) {
        return Err(format!("`git diff-index --cached --name-only` ended with {:?}", output.code));
    }
    return Ok(output.stdout.split(|byte: &u8| { return *byte == 0; }).filter(|path| { return !path.is_empty(); }).map(<[u8]>::to_vec).collect());
}

/// What:
///  The first path among the clones that Git's own listing names as staged,
/// in words,
/// or nothing
///  when the two answers agree.
/// Why:
///   A staged path holds another blob than the tree names,
/// so a clone of it would put the person's bytes where
///  the commit's belong,
/// and no later step sees it:
/// the new index records the stat data of the file the clone
///  made,
/// so `git status` trusts it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = aStagedPathAmongTheClones(staged, entries, clones) ?? undefined;
/// ```
fn a_staged_path_among_the_clones(staged: &[Vec<u8>], entries: &[crate::worktree_clean::StatEntry], clones: &[usize]) -> Option<String> {
    for at in clones {
        let Some(entry) = entries.get(*at) else { continue };
        if staged.iter().any(|path| { return *path == entry.path; }) {
            let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
            return Some(format!("{} is staged in the source and would have been cloned", crate::worktree_snapshot_source::plain(shown.as_str())));
        }
    }
    return None;
}

/// What:
///  The stat fields the new index records:
/// those of the files the clones made,
/// with a link's entry keeping the
///  fields its source recorded,
/// and none at all for a path Git writes,
/// so nothing records a path Git has not written.
/// Why:
///   An entry whose stat data matches the file is exactly what makes Git skip it,
/// so a path the
///  index may not vouch for yet keeps the zero stat data the tree's own entry holds;
/// the index then takes the
///  worktree's name only after Git wrote those paths (`worktree_clone_index`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = withoutTheWritten(statsOfTheClones({ made, split, entries }), written);
/// ```
fn the_recorded_stats(made: HashMap<Vec<u8>, StatFields>, split: &Split, entries: &[crate::worktree_clean::StatEntry], written: &HashSet<Vec<u8>>) -> HashMap<Vec<u8>, StatFields> {
    let mut stats: HashMap<Vec<u8>, StatFields> = made;
    for at in &split.clones {
        let Some(entry) = entries.get(*at) else { continue };
        if entry.mode & MODE_TYPE == MODE_SYMLINK {
            stats.insert(entry.path.clone(), fields_the_source_recorded(entry.stored));
        }
    }
    stats.retain(|path: &Vec<u8>, _fields: &mut StatFields| { return !written.contains(path); });
    return stats;
}

/// What:
///  The paths Git writes:
/// the paths a change reaches,
/// and the paths the conversion fence takes because a clone
///  would keep bytes Git's checkout would write others for,
/// in path order.
/// Why:
///  Both lists are sorted by path,
/// and the index, the tree and both steps read them in that order;
/// a path on both lists is written once,
/// which is what the fence's own answer for a path a change also reaches needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = [...new Set([...split.writes, ...fence.gitWrites])].sort(compareBytes);
/// ```
fn the_written_paths(split: &Split, fence: &Fence) -> Vec<Vec<u8>> {
    let mut written: HashSet<Vec<u8>> = split.writes.iter().cloned().collect();
    for path in &fence.git_writes {
        written.insert(path.clone());
    }
    let mut ordered: Vec<Vec<u8>> = written.into_iter().collect();
    ordered.sort();
    return ordered;
}

/// What:
///  Clone the unchanged tracked paths of a source with changes into the destination,
/// write the index built from
///  the tree of the source's `HEAD` under the fence's waiting name,
/// let the fence answer beside the copies,
/// and answer the
///  entries the clones made,
/// the fence's decision widened by the changed paths,
/// and the waiting index.
/// Why:
///   See the module description.
/// The order is the fence's own (`worktree_clone_conversion_beside_the_clone`):
/// the attributes and the decline were
///  decided before anything was copied,
/// the remaining reads run beside the copies,
/// the index is written under a name of its own with no
///  stat data for a path Git still has to write,
/// and it takes the worktree's name only after that write,
/// which the
///  ladder's serving step does.
/// Nothing is removed on a failure:
/// the entries the clones created stay and are counted by the
///  caller,
/// and the checkout that completes the worktree overwrites the tracked ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { created, decided, pending } = cloneTheUnchanged({ context, source, destination, adminPath, fence, rules }); // throws the reason
/// ```
pub(crate) fn clone_the_unchanged(work: PartialClone<'_>) -> Result<(PartCloned, Fence, PendingIndex), String> {
    let PartialClone { context, source, destination, admin, admin_path, fence, rules }: PartialClone<'_> = work;
    let read_entered: bool = begin_step("partRead");
    let read_outcome = read_tracked(source);
    end_step(read_entered);
    let tracked: Tracked = read_outcome?;
    let classify_entered: bool = begin_step("partClassify");
    let classify_outcome = split_the_tree(tracked.entries.as_slice(), tracked.indexed.as_slice(), tracked.tree.as_slice(), source.root.as_path(), source.clean.index_seconds);
    end_step(classify_entered);
    let settled: Split = classify_outcome.map_err(|refusal| { return refusal.reason; })?;
    let split: Split = apply_the_size_rules(settled, tracked.entries.as_slice(), &rules)?;
    let staged_entered: bool = begin_step("partStaged");
    let staged_outcome = staged_paths(context, source).and_then(|staged| {
        return match a_staged_path_among_the_clones(staged.as_slice(), tracked.entries.as_slice(), split.clones.as_slice()) {
            Some(refusal) => Err(refusal),
            None => Ok(()),
        };
    });
    end_step(staged_entered);
    staged_outcome?;
    let cloned_count: usize = split.clones.len();
    let beside_entered: bool = begin_step("conversionBesideTheClone");
    let (cloned, answered) = clone_these_bytes_beside(source, split.cloned_entries(tracked.entries.as_slice()), destination, fence_beside_the_clone(source, context, fence))?;
    end_step(beside_entered);
    // A fence's reason, or a pool that would not run it, ends the rung here before any index exists: Git's own checkout
    // writes the worktree.
    let decided: Fence = match answered {
        Some(Ok(decided)) => decided,
        Some(Err(reason)) => return Err(reason),
        None => return Err(String::from("the fence beside the copies answered nothing")),
    };
    let written: Vec<Vec<u8>> = the_written_paths(&split, &decided);
    let written_count: usize = written.len();
    let set: HashSet<Vec<u8>> = written.iter().cloned().collect();
    let recorded: HashMap<Vec<u8>, StatFields> = the_recorded_stats(cloned.stats().clone(), &split, tracked.entries.as_slice(), &set);
    let index_entered: bool = begin_step("partIndex");
    let seed: Result<Vec<u8>, String> = index_of_the_tree(tracked.tree.as_slice(), &recorded, source.clean.hash_length);
    let index_outcome = seed.and_then(|bytes| { return install_the_index_beside_the_write((admin, admin_path), bytes.as_slice(), &recorded, &set, source.clean.hash_length); });
    end_step(index_entered);
    let pending: PendingIndex = index_outcome?;
    debug(TAG, format!("cloned {cloned_count} of {} tracked paths and had Git write {written_count} of them", tracked.tree.len()).as_str());
    let part: PartCloned = PartCloned { cloned: cloned_count, written: written_count, created: cloned.into_created() };
    return Ok((part, Fence { git_writes: written, carried: decided.carried }, pending));
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
/// It is a control's answer,
/// not a step of the rung:
/// the shipped rung's guarantee is the index it wrote and the fence's write,
/// and this reads the result the
///  way the measurement harness does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheWorktree({ context, destination, newCommit }); // throws the reason it is not Git's result
/// ```
#[cfg(test)]
pub fn prove_the_worktree(context: &GitContext, destination: &Path, admin_path: &Path, new_commit: &str) -> Result<(), String> {
    let (listed_code, listed) = ask_in_the_worktree(context, destination, admin_path, &["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=all", "--no-renames"])?;
    if listed_code != Some(0) {
        return Err(format!("`git status` ended with {listed_code:?} in the new worktree"));
    }
    if let Some(first) = listed.split(|byte: &u8| { return *byte == 0; }).find(|entry| { return !entry.is_empty(); }) {
        let shown: String = String::from_utf8_lossy(&first[3.min(first.len())..]).into_owned();
        return Err(format!("the new worktree is not what Git's checkout makes: {} differs", crate::worktree_snapshot_source::plain(shown.as_str())));
    }
    let (compared_code, _quiet) = ask_in_the_worktree(context, destination, admin_path, &["--no-optional-locks", "diff-index", "--cached", "--quiet", new_commit])?;
    if compared_code != Some(0) {
        return Err(String::from("the new worktree's index does not equal the tree of the new HEAD"));
    }
    return Ok(());
}

/// What:
///  Run one captured Git query in the new worktree and answer its status and standard output,
/// or the reason it
///  could not be run at all.
/// Why:
///   The proof asks Git twice,
/// and both runs name the worktree's own Git directory and work tree,
/// so neither
///  can read the source's.
/// A run that reports a failure through its status is an answer,
/// not an error:
/// a quiet difference
///  check says "they differ" with status 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, stdout } = askInTheWorktree({ context, destination, adminPath, words }); // throws the reason it did not start
/// ```
#[cfg(test)]
fn ask_in_the_worktree(context: &GitContext, destination: &Path, admin_path: &Path, words: &[&str]) -> Result<(Option<i32>, Vec<u8>), String> {
    let request: GitRequest = GitRequest {
        cwd: destination.to_path_buf(),
        arguments: words.iter().copied().map(std::ffi::OsString::from).collect(),
        without_prefix: true,
        environment: crate::worktree_ladder_target::checkout_environment(admin_path, destination),
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("`git {}` could not start ({error})", words.join(" "))),
    };
    return Ok((output.code, output.stdout));
}

/// Partial-clone controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_partial_tests.rs"]
mod tests;

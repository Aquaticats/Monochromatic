//! What:
//!  The fence over a clone rung's bytes:
//! which tracked paths the clones may carry,
//! which paths Git must write,
//! the attributes each decision reads,
//! and the one Git run that writes the rest.
//! Why:
//!  Cloning a source's working files and installing the source's index
//! makes a worktree that every check calls clean,
//!  even where Git's checkout would have written other bytes
//! (`package/git-policy/cli/SPEC.md`,
//! rung 2;
//! GitHub issue 6323).
//! The clone may carry a path only where the checkout's conversion of its blob is the blob itself,
//! with the source's file proven to hold that blob:
//! `worktree_clone_conversion_rules::decide` decides that from Git's own attributes,
//! `core.autocrlf`,
//! `core.eol` and the recorded sizes.
//! The attributes come from one `git check-attr --all`,
//! so every source Git reads is answered by Git,
//! macros and precedence included;
//! sizes come from one `git ls-tree -r -l`,
//! for every path the fence may carry,
//! because a path with no rule can still hold an earlier rule's form;
//! and the paths Git must write go through one `git checkout-index` in the destination,
//! which applies the destination's own conversion.
//! The source's bytes must be the blob's,
//! so an untracked or ignored `.gitattributes` above a tracked path refuses the rung:
//! Git read it when the files were added,
//! the new worktree will not have it,
//! and no answer Git gives today can prove what it said then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fence = fenceTheClone(source, context); // throws the reason the rung cannot be proven
//! if (fence.gitWrites.length > 0) writeThroughGit(destination, admin, context, fence.gitWrites);
//! ```

use crate::diagnostic_log::debug;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_clean::{StatEntry, stored_stats};
use crate::worktree_clone_conversion_rules::{ConversionRules, FenceVerdict, PathAttributes, apply_attribute, checkout_converts, conversion_rules_from_config, decide, needs_blob_size};
use crate::worktree_ladder_target::{checkout_environment, checkout_environment_with_index};
use crate::worktree_snapshot_source::SnapshotSource;
/// The change a rung cannot serve,
/// and why,
/// re-exported so every caller of this fence names it here.
pub use crate::worktree_change_a_rung_cannot_serve::{ChangeNotServed, WhyNotServed};
/// How many paths a rule of their own converts,
/// and the share that declines the rung,
/// re-exported so every caller of this fence names them here.
pub use crate::worktree_clone_conversion_decline::{DECLINE_PATHS_FLOOR, DECLINE_SHARE_DIVISOR, converting_paths, declines};
use crate::worktree_clone_conversion_decline::{MODE_SYMLINK, MODE_TYPE_MASK};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCloneConversionFence";

/// The name of the attributes file whose untracked presence refuses the rung.
const ATTRIBUTES_NAME: &str = ".gitattributes";

/// The paths beyond which the fence logs no more reasons,
/// because the count says the rest and a line per path would bury it.
const WRITTEN_PATHS_REPORTED: usize = 8;

/// What:
///  What one clone rung's fence decided.
/// Why:
///   The caller clones what it may carry and hands the rest to one Git run,
/// whose paths it also names in a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fence = { gitWrites: Buffer[]; carried: number };
/// ```
pub struct Fence {
    /// The paths Git writes,
    /// in index order.
    pub git_writes: Vec<Vec<u8>>,
    /// How many tracked paths the clones may carry.
    pub carried: usize,
}

/// What:
///  One Git run's failure as the reason the rung ends,
/// with Git's own words.
/// Why:
///   The rung's answer is a reason string,
/// and a run that cannot start or that fails says which run and what Git said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = (what, output) => `${what} ended with ${output.code} (${output.stderr.trim()})`;
/// ```
fn run_reason(what: &str, output: &GitOutput) -> String {
    return format!("{what} ended with {:?} ({})", output.code, String::from_utf8_lossy(output.stderr.as_slice()).trim());
}

/// What:
///  Run one Git command of the fence,
/// with optional standard input and environment,
/// and give its output.
/// Why:
///   Every read of the fence is one step of the add's own record,
/// and a run that cannot start answers a reason rather than a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runIn(context, cwd, arguments, input, environment): GitOutput;
/// ```
fn run_in(context: &GitContext, cwd: &Path, arguments: &[OsString], input: Option<Vec<u8>>, environment: Vec<(OsString, OsString)>) -> Result<GitOutput, String> {
    let request: GitRequest = GitRequest {
        cwd: cwd.to_path_buf(),
        arguments: arguments.to_vec(),
        without_prefix: true,
        environment,
        input,
        ..GitRequest::default()
    };
    return run_git(context, &request).map_err(|error| return format!("a Git run of the fence could not start ({error})"));
}

/// What:
///  A command line's words as the owned strings a request carries.
/// Why:
///   One request takes an owned argument a caller built (`--prefix`),
/// so every request of the fence is built the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (list: string[]) => list.map((word) => word);
/// ```
fn words(list: &[&str]) -> Vec<OsString> {
    return list.iter().map(|word| return OsString::from(word)).collect();
}

/// What:
///  The first directory above a tracked path that holds an attributes file the index does not track,
/// as the reason the rung ends.
/// Why:
///   Git reads such a file for the paths beneath it,
/// the new worktree will not have it,
/// and the source's files may have been made under a rule that is gone
///  (the gate review's finding L3).
/// Only the ancestors of tracked paths are looked at,
/// so the answer costs no walk of the worktree's ignored directories;
/// the root is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstUntrackedAttributes(entries, root): string | undefined;
/// ```
fn first_untracked_attributes(entries: &[StatEntry], root: &Path) -> Option<String> {
    let mut tracked: HashSet<Vec<u8>> = HashSet::new();
    for entry in entries {
        if is_attributes_path(entry.path.as_slice()) {
            tracked.insert(entry.path.clone());
        }
    }
    let mut looked: HashSet<Vec<u8>> = HashSet::new();
    for entry in entries {
        let mut directory: Vec<u8> = Vec::new();
        let mut boundary: usize = 0;
        loop {
            if looked.insert(directory.clone()) && let Some(reason) = attributes_file_at(root, directory.as_slice(), &tracked) {
                return Some(reason);
            }
            let Some(next) = entry.path[boundary..].iter().position(|byte| return *byte == b'/') else { break };
            boundary += next + 1;
            directory = entry.path[..boundary - 1].to_vec();
        }
    }
    return None;
}

/// Whether a path names an attributes file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAttributes = (path) => path === '.gitattributes' || path.endsWith('/.gitattributes');
/// ```
fn is_attributes_path(path: &[u8]) -> bool {
    return path == ATTRIBUTES_NAME.as_bytes() || path.ends_with(format!("/{ATTRIBUTES_NAME}").as_bytes());
}

/// What:
///  Whether the directory below `root` holds an attributes file the index does not track,
/// named as the reason the rung ends.
/// Why:
///   A tracked one is the tree's and is read by both worktrees;
/// an untracked one is the source's alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributesFileAt(root, directory, tracked): string | undefined;
/// ```
fn attributes_file_at(root: &Path, directory: &[u8], tracked: &HashSet<Vec<u8>>) -> Option<String> {
    let mut path: Vec<u8> = directory.to_vec();
    if !path.is_empty() {
        path.push(b'/');
    }
    path.extend_from_slice(ATTRIBUTES_NAME.as_bytes());
    if tracked.contains(path.as_slice()) {
        return None;
    }
    let on_disk: PathBuf = root.join(OsStr::from_bytes(path.as_slice()));
    if std::fs::symlink_metadata(on_disk.as_path()).is_err() {
        return None;
    }
    return Some(format!(
        "{} is in the source and not in its index, so the rules Git read when its tracked files were added cannot be known",
        String::from_utf8_lossy(path.as_slice())
    ));
}

/// What:
///  One `git check-attr --all -z --stdin` over every tracked path,
/// as the attributes of each path.
/// Why:
///   Git resolves which attributes file applies,
/// in which order,
/// and what a macro sets,
/// so the fence never matches a pattern itself;
/// a path Git does not answer for keeps the defaults,
/// which the configuration then decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributesOf(context, root, entries): Map<string, PathAttributes>;
/// ```
fn attributes_of(context: &GitContext, root: &Path, entries: &[StatEntry]) -> Result<HashMap<Vec<u8>, PathAttributes>, String> {
    let mut input: Vec<u8> = Vec::new();
    for entry in entries {
        input.extend_from_slice(entry.path.as_slice());
        input.push(0);
    }
    let output: GitOutput = run_in(context, root, &words(&["check-attr", "--all", "-z", "--stdin"]), Some(input), Vec::new())?;
    if output.code != Some(0) {
        return Err(format!("the attributes of the source cannot be read: {}", run_reason("`git check-attr`", &output)));
    }
    let mut attributes: HashMap<Vec<u8>, PathAttributes> = HashMap::new();
    let mut fields = output.stdout.split(|byte| return *byte == 0);
    loop {
        let (Some(path), Some(name), Some(value)) = (fields.next(), fields.next(), fields.next()) else { break };
        if path.is_empty() {
            break;
        }
        let held: &mut PathAttributes = attributes.entry(path.to_vec()).or_default();
        apply_attribute(held, name, value);
    }
    return Ok(attributes);
}

/// What:
///  The blob size of every path of the commit's tree,
/// as one `git ls-tree -r -l -z` reports them.
/// Why:
///   The size is what proves that a file whose line-ending rule only strips carriage returns still holds its blob's bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blobSizes(context, root, commit): Map<string, number>;
/// ```
fn blob_sizes(context: &GitContext, root: &Path, commit: &str) -> Result<HashMap<Vec<u8>, u64>, String> {
    let output: GitOutput = run_in(context, root, &words(&["ls-tree", "-r", "-l", "-z", commit]), None, Vec::new())?;
    if output.code != Some(0) {
        return Err(format!("the sizes of the committed blobs cannot be read: {}", run_reason("`git ls-tree`", &output)));
    }
    let mut sizes: HashMap<Vec<u8>, u64> = HashMap::new();
    for record in output.stdout.split(|byte| return *byte == 0) {
        // `mode SP type SP object SP size TAB path`
        let Some(tab) = record.iter().position(|byte| return *byte == b'\t') else { continue };
        // The size is right-aligned, so the words between the spaces hold empty ones that are not fields.
        let mut fields = record[..tab].split(|byte| return *byte == b' ').filter(|word| return !word.is_empty());
        let (Some(_mode), Some(_kind), Some(_object), Some(size)) = (fields.next(), fields.next(), fields.next(), fields.next()) else { continue };
        let Ok(size_text) = std::str::from_utf8(size) else { continue };
        let Ok(size_value) = size_text.parse::<u64>() else { continue };
        sizes.insert(record[tab + 1..].to_vec(), size_value);
    }
    return Ok(sizes);
}

/// What:
///  Refuse a source with an untracked or ignored `.gitattributes` above a tracked path,
///  with the reason the rung cannot be proven.
/// Why:
///   Git read that file when the source's files were added and the new worktree will not have it,
/// so no answer Git gives today
///  can prove what it said then.
/// The walk costs no Git run and no walk of the ignored directories,
/// so the rung takes it before it copies anything,
///  beside the other cheap refusals;
/// the fence's own reads then run where the shape is,
///  before or beside the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseUntrackedAttributes(source); // throws the reason
/// ```
pub fn refuse_untracked_attributes(source: &SnapshotSource) -> Result<(), String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    return match first_untracked_attributes(entries.as_slice(), source.root.as_path()) {
        Some(reason) => Err(reason),
        None => Ok(()),
    };
}

/// What:
///  The half of the fence the attributes alone decide:
/// the tracked entries,
/// the attributes Git answers for each of them,
/// the conversion rules,
///  and nothing that needs a blob.
/// Why:
///   This is the half that a decline needs,
/// and the decline must be decided before the first copy
///  (`doc/planning/cli-git-clone-rung-conversion-fence.md`,
/// option B's first step):
/// its two reads are `git check-attr` and the count over its answer,
///  so a source above the quarter costs the add one Git checkout and this query and no copy at all.
/// The rest of the fence needs `git ls-tree`,
/// which is the expensive read,
///  so it stays where the copies are (`decide_the_paths`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeFence = { entries: StatEntry[]; attributes: Map<string, PathAttributes>; rules: ConversionRules };
/// ```
#[derive(Clone)]
pub struct AttributeFence {
    /// The tracked entries of the source's frozen index,
    /// in its order.
    entries: Vec<StatEntry>,
    /// The attributes Git answered for each path.
    attributes: HashMap<Vec<u8>, PathAttributes>,
    /// The conversion rules the answer is judged by.
    rules: ConversionRules,
    /// Whether any changed path could change this fence's answer:
    /// a path a rule of its own converts,
    ///  or a link whose copy no index entry can vouch for.
    a_changed_path_could_matter: bool,
}

impl AttributeFence {
    /// What:
    ///  Whether a changed path could change this fence's answer at all,
    ///  so one read of the target tree could end the rung.
    /// Why:
    ///   The skip's read is one Git run,
    /// and in a repository no rule converts and no link is tracked no changed path can be one the rung cannot serve,
    /// so the read buys nothing and is not paid.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const matters = (read) => read.converting > 0 || read.hasALink;
    /// ```
    pub fn a_changed_path_could_matter(&self) -> bool {
        return self.a_changed_path_could_matter;
    }

    /// What:
    ///  The first of `changed` a clone of this source cannot serve,
    ///  in the source's index order,
    ///  and why,
    ///  or nothing when every changed path is one the clones make and vouch for.
    /// Why:
    ///   This is the half of the fence that is decided before the copies,
    /// so it is what lets the ladder see a target tree its tree difference would refuse before it claims anything:
    /// a path here whose entry the fence installs without stat data,
    ///  or whose link the copies make again,
    ///  is a path `git read-tree -m -u` refuses.
    /// A path the source's index does not hold has no entry to refuse,
    ///  so the search runs over the entries and asks which of them changed,
    ///  not over the changed paths and their attributes,
    ///  because a path Git answers nothing for has no attribute entry at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const firstNotServed = (read, changed) => read.entries.find((entry) => changed.has(entry.path) && (isLink(entry) || checkoutConverts(read.attributes.get(entry.path), read.rules)));
    /// ```
    pub fn first_change_it_cannot_serve(&self, changed: &[Vec<u8>]) -> Option<ChangeNotServed> {
        if changed.is_empty() {
            return None;
        }
        let changed_paths: HashSet<&[u8]> = changed.iter().map(|path| return path.as_slice()).collect();
        for entry in &self.entries {
            if !changed_paths.contains(entry.path.as_slice()) {
                continue;
            }
            if entry.mode & MODE_TYPE_MASK == MODE_SYMLINK {
                return Some(ChangeNotServed { path: entry.path.clone(), why: WhyNotServed::ALinkIsCopied });
            }
            let held: PathAttributes = self.attributes.get(entry.path.as_slice()).cloned().unwrap_or_default();
            if checkout_converts(&held, &self.rules) {
                return Some(ChangeNotServed { path: entry.path.clone(), why: WhyNotServed::TheFenceWritesIt });
            }
        }
        return None;
    }
}

/// What:
///  Read the attributes of every tracked path and decide the decline,
/// before anything is copied.
/// Why:
///   The cheap refusal comes first (no Git run),
/// then one `git check-attr` and the count of the paths a rule of
///  their own would convert;
/// a source above the quarter ends the rung here,
/// which is what makes a declined add one Git checkout plus this query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceTheAttributes(source, context): AttributeFence; // throws the reason the rung cannot be proven
/// ```
pub fn fence_the_attributes(source: &SnapshotSource, context: &GitContext) -> Result<AttributeFence, String> {
    refuse_untracked_attributes(source)?;
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let rules: ConversionRules = conversion_rules_from_config(source.config.entries());
    let attributes: HashMap<Vec<u8>, PathAttributes> = attributes_of(context, source.root.as_path(), entries.as_slice())?;
    // What:
    //  The decline rule: how many paths a rule of their own would convert is known from the attributes alone, before any
    //  blob is read and before anything is cloned.
    // Why:
    //  Writing many paths through Git makes the fence slower than `main`'s own checkout (the converting source measured
    //  1.50 times `main` without the rule, `measure-r5-crlf-*`),
    //  and the rule is decided here, before the add's record is claimed and before real Git runs, so a declining source
    //  pays only these reads and Git's own single checkout (measured at 1.67 to 1.75 of `main` when the rule was decided
    //  after the copies, and at 1.06 to 1.09 when it was decided before them).
    #[cfg(feature = "measurement-routes")]
    let measure_decline: bool = std::env::var("MONOCHROMATIC_CLONE_FENCE").unwrap_or_default() != "nodecline";
    #[cfg(not(feature = "measurement-routes"))]
    let measure_decline: bool = true;
    let converting: usize = converting_paths(entries.as_slice(), &attributes, &rules);
    if measure_decline && declines(converting, entries.len()) {
        return Err(format!(
            "a line-ending rule, a filter, an ident or an encoding can change {converting} of {} paths, more than a quarter of them, so this rung declines and Git's checkout writes the worktree",
            entries.len()
        ));
    }
    // A link is never converted by its path's rules, so the two questions are counted separately: a changed link is a path
    //  the copies cannot make Git's own entry, and one converting path is what the fence hands Git.
    let a_changed_path_could_matter: bool = converting > 0 || entries.iter().any(|entry| return entry.mode & MODE_TYPE_MASK == MODE_SYMLINK);
    return Ok(AttributeFence { entries, attributes, rules, a_changed_path_could_matter });
}

/// What:
///  Decide every tracked path the attributes left:
/// what the clones may carry and what Git must write.
/// Why:
///   This is the half that costs `git ls-tree`,
/// so it runs where the copies are,
/// and a control can price it apart from
///  the reads the decline needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideThePaths(source, context, read): Fence; // throws the reason the rung cannot be proven
/// ```
pub fn decide_the_paths(source: &SnapshotSource, context: &GitContext, read: AttributeFence) -> Result<Fence, String> {
    let AttributeFence { entries, attributes, rules, a_changed_path_could_matter: _matters }: AttributeFence = read;
    // One `git ls-tree` for every path whose checkout can write the blob unchanged, which is every path this fence may
    // carry: the blob's size is the proof the file holds it, and a file can hold another rule's form even when today it
    // has no rule at all (an unset `core.autocrlf`, a tracked attributes file a later commit deleted).
    let sizes: Option<HashMap<Vec<u8>, u64>> = if entries.iter().any(|entry| {
        let held: PathAttributes = attributes.get(entry.path.as_slice()).cloned().unwrap_or_default();
        return entry.mode & MODE_TYPE_MASK != MODE_SYMLINK && needs_blob_size(&held, &rules);
    }) {
        Some(blob_sizes(context, source.root.as_path(), source.clean.head_commit.as_str())?)
    } else {
        None
    };
    let mut git_writes: Vec<Vec<u8>> = Vec::new();
    let mut carried: usize = 0;
    for entry in &entries {
        let held: PathAttributes = attributes.get(entry.path.as_slice()).cloned().unwrap_or_default();
        let is_link: bool = entry.mode & MODE_TYPE_MASK == MODE_SYMLINK;
        let file_size: u64 = u64::from(entry.stored.size);
        let listed: Option<u64> = sizes.as_ref().and_then(|known| return known.get(entry.path.as_slice()).copied());
        match decide(&held, &rules, is_link, file_size, listed) {
            FenceVerdict::Carry => carried += 1,
            FenceVerdict::WriteThroughGit(reason) => {
                if git_writes.len() < WRITTEN_PATHS_REPORTED {
                    debug(TAG, fence_sentence(entry.path.as_slice(), reason.as_str()).as_str());
                }
                git_writes.push(entry.path.clone());
            }
        }
    }
    debug(TAG, format!("the fence carries {carried} paths and leaves {} to Git", git_writes.len()).as_str());
    return Ok(Fence { git_writes, carried });
}

/// What:
///  The first of `changed` the whole fence hands to Git,
///  in the source's index order,
///  or nothing when every changed path is one the clones may carry.
/// Why:
///   The snapshot rung decides both halves of the fence before its snapshot,
/// so `git_writes` is the exact set of paths its index installs without stat data,
///  which is the set `git read-tree -m -u` refuses when the target tree changes one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstWrittenChange = (fence, changed) => fence.gitWrites.find((path) => changed.has(path));
/// ```
pub fn first_written_change(fence: &Fence, changed: &[Vec<u8>]) -> Option<ChangeNotServed> {
    if changed.is_empty() {
        return None;
    }
    let changed_paths: HashSet<&[u8]> = changed.iter().map(|path| return path.as_slice()).collect();
    for path in &fence.git_writes {
        if changed_paths.contains(path.as_slice()) {
            return Some(ChangeNotServed { path: path.clone(), why: WhyNotServed::TheFenceWritesIt });
        }
    }
    return None;
}

/// What:
///  Both halves of the fence,
/// one after the other,
/// for a rung that decides it before its copy (rung 1's snapshot).
/// Why:
///   The snapshot's index is installed by the snapshot itself,
/// so the written paths must be known before it runs,
///  and a snapshot rung copies everything anyway:
/// there is no second thread for the reads to run beside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fence = decideThePaths(source, context, fenceTheAttributes(source, context));
/// ```
pub fn fence_the_clone(source: &SnapshotSource, context: &GitContext) -> Result<Fence, String> {
    return decide_the_paths(source, context, fence_the_attributes(source, context)?);
}

/// What:
///  One path and the rule that makes a clone of its source file unprovable,
/// as a log line.
/// Why:
///   The reason names the rule,
/// so a report can be checked against `convert.c` without a second run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentence = (path, reason) => `${path}: ${reason}`;
/// ```
fn fence_sentence(path: &[u8], reason: &str) -> String {
    return format!("{}: {reason}", String::from_utf8_lossy(path));
}

/// What:
///  The paths of the index at `index`,
///  as the set a write step intersects a fence's answer with.
/// Why:
///   The tree difference can remove a path the new `HEAD` lacks,
/// and `git checkout-index` fails the whole rung on a path its index does not hold
///  ("is not in the cache"),
///  so both write steps ask the index they are pointed at which of the fenced paths are still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsTheIndexHolds(index, hashLength): Set<string>; // throws the reason
/// ```
fn paths_the_index_holds(index: &Path, hash_length: usize) -> Result<HashSet<Vec<u8>>, String> {
    let bytes: Vec<u8> = std::fs::read(index).map_err(|error| return format!("the new worktree's index cannot be read ({error})"))?;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(entries) = parse_index(bytes.as_slice(), format) else {
        return Err(String::from("the new worktree's index has a layout the reader does not know"));
    };
    return Ok(entries.iter().map(|entry| return entry.path.clone()).collect());
}

/// What:
///  Zero the instance data of the index entries for `paths`,
/// with one `git update-index -z --index-info`,
/// so the entry no longer vouches for whatever the rung put in the work tree,
/// and answer the paths the destination's index actually holds.
/// Why:
///   `git checkout-index` in place writes a path only when its entry's stat data no longer matches the file
/// (`entry.c`,
///  `checkout_entry_ca`,
/// `if (!changed) return 0;`),
/// and both rungs can put bytes in the work tree that the entry's own fields still describe:
/// the clone rung installs the source's index over a copy made here,
/// and a btrfs snapshot keeps the source's inode and times,
/// so the copied file *is* the file the source's index recorded.
/// A zeroed entry matches nothing (`read-tree` leaves entries this way),
/// so Git rewrites it and the first status hashes it once and finds it clean.
/// The intersection is a correctness matter,
/// not a saving:
/// `git update-index --index-info` *adds* an entry a path lacks,
/// so a fenced path the new `HEAD` does not carry would be injected into the destination's index,
/// and `git checkout-index` would then fail the whole rung ("is not in the cache").
/// This is option A′'s write step;
///  the fence beside the clone installs its zeroes with the index itself and needs only the checkout below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function zeroTheEntries(admin, destination, context, paths, hashLength); // throws the reason
/// ```
fn zero_the_entries(admin: &Path, destination: &Path, context: &GitContext, paths: &[Vec<u8>], hash_length: usize) -> Result<Vec<Vec<u8>>, String> {
    let index_path: PathBuf = admin.join("index");
    let bytes: Vec<u8> = std::fs::read(index_path.as_path()).map_err(|error| return format!("the new worktree's index cannot be read ({error})"))?;
    let format: ObjectFormat = if hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let Some(entries) = parse_index(bytes.as_slice(), format) else {
        return Err(String::from("the new worktree's index has a layout the reader does not know"));
    };
    let wanted: HashSet<&[u8]> = paths.iter().map(|path| return path.as_slice()).collect();
    let mut input: Vec<u8> = Vec::new();
    let mut zeroed: Vec<Vec<u8>> = Vec::new();
    for entry in &entries {
        if !wanted.contains(entry.path.as_slice()) || entry.mode & MODE_TYPE_MASK == MODE_SYMLINK {
            continue;
        }
        let object: String = crate::repo_read_object::to_hex(entry.id.as_slice());
        input.extend_from_slice(format!("{:o} {object}\t", entry.mode).as_bytes());
        input.extend_from_slice(entry.path.as_slice());
        input.push(0);
        zeroed.push(entry.path.clone());
    }
    if input.is_empty() {
        return Ok(zeroed);
    }
    let output: GitOutput = run_in(context, destination, words(&["update-index", "-z", "--index-info"]).as_slice(), Some(input), checkout_environment(admin, destination))?;
    return if output.code == Some(0) { Ok(zeroed) } else { Err(run_reason("`git update-index --index-info`", &output)) };
}

/// What:
///  Write the paths Git must write,
/// in the destination,
/// through one `git checkout-index --force --stdin`.
/// Why:
///   Git owns the conversion,
/// so asking it for these bytes is the only answer that needs no model of a filter,
///  an encoding or an ending;
/// the runs name the destination's Git directory and work tree,
/// as the ladder's other runs do,
/// so the destination's own settings decide.
/// The entries are zeroed first (`zero_the_entries`),
/// because an entry that still matches the file is left alone,
/// and only the paths the destination's index holds are sent,
/// because one it lacks would be injected into the index and then fail the run.
/// `-u` records the stat data of each file Git writes,
/// so the new worktree's first `git status` compares instead of rehashing every fenced path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeThroughGit(admin, destination, context, paths, hashLength); // throws the reason
/// ```
pub fn write_through_git(admin: &Path, destination: &Path, context: &GitContext, paths: &[Vec<u8>], hash_length: usize) -> Result<(), String> {
    let zeroed: Vec<Vec<u8>> = zero_the_entries(admin, destination, context, paths, hash_length)?;
    let index: PathBuf = admin.join("index");
    return checkout_paths(admin, destination, context, zeroed.as_slice(), index.as_path());
}

/// What:
///  Run the one `git checkout-index --force -u --stdin` of `paths`,
/// against the index at `index`.
/// Why:
///   Both write steps end in the same run,
/// and each has already decided which paths its index holds,
///  so the run itself asks nothing else;
/// `GIT_INDEX_FILE` names the index,
/// so a rung whose index awaits the worktree's own name records its `-u`
///  stat data in the file that will become that index,
/// and the worktree itself holds no index while the paths are written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutPaths(admin, destination, context, paths, index); // throws the reason
/// ```
fn checkout_paths(admin: &Path, destination: &Path, context: &GitContext, paths: &[Vec<u8>], index: &Path) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let output: GitOutput = run_in(context, destination, words(&["checkout-index", "--force", "-u", "-z", "--stdin"]).as_slice(), Some(input), checkout_environment_with_index((admin, destination), Some(index)))?;
    return if output.code == Some(0) { Ok(()) } else { Err(run_reason("`git checkout-index`", &output)) };
}

/// What:
///  Write the paths Git must write through one `git checkout-index --force --stdin`,
/// without zeroing anything first,
/// and only for the paths the destination's index holds.
/// Why:
///   This is the fence beside the clone's write step:
/// the seed the index was installed from holds the written paths' stat data already zeroed
/// (`worktree_clone_conversion_beside_the_clone::seed_without_stat`),
///  so a zeroing run would only rewrite zeroes;
/// the intersection stays,
/// because the tree difference can have removed a path the new `HEAD` lacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexThroughGit(admin, destination, context, paths, hashLength); // throws the reason
/// ```
pub fn checkout_index_through_git(admin: &Path, destination: &Path, context: &GitContext, paths: &[Vec<u8>], hash_length: usize) -> Result<(), String> {
    let index: PathBuf = admin.join("index");
    return checkout_index_through_the_index_at((admin, destination, context), index.as_path(), paths, hash_length);
}

/// What:
///  `checkout_index_through_git` with the index named,
///  for a rung whose index awaits the worktree's own name
/// (`worktree_clone_index::PendingIndex`).
/// Why:
///   Git is pointed at that file with `GIT_INDEX_FILE`,
/// so its `-u` records the stat data of the bytes it wrote in the file that will become the worktree's index,
///  and the worktree holds no index at all until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutIndexThroughTheIndexAt({ admin, destination, context }, pendingPath, paths, hashLength); // throws the reason
/// ```
pub fn checkout_index_through_the_index_at((admin, destination, context): (&Path, &Path, &GitContext), index: &Path, paths: &[Vec<u8>], hash_length: usize) -> Result<(), String> {
    let held: HashSet<Vec<u8>> = paths_the_index_holds(index, hash_length)?;
    let present: Vec<Vec<u8>> = paths.iter().filter(|path| return held.contains(path.as_slice())).cloned().collect();
    return checkout_paths(admin, destination, context, present.as_slice(), index);
}

#[cfg(test)]
#[path = "worktree_clone_conversion_fence_tests.rs"]
mod tests;

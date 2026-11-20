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
//! (`package/git-policy/cli/SPEC.md`, rung 2;
//! GitHub issue 6323).
//! The clone may carry a path only where the checkout's conversion of its blob is the blob itself,
//! with the source's file proven to hold that blob:
//! `worktree_clone_conversion_rules::decide` decides that from Git's own attributes,
//! `core.autocrlf`, `core.eol` and the recorded sizes.
//! The attributes come from one `git check-attr --all`,
//! so every source Git reads is answered by Git,
//! macros and precedence included;
//! sizes come from one `git ls-tree -r -l`, and only when a line-ending rule may strip carriage returns;
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
use crate::config_snapshot::ConfigEntry;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_clean::{StatEntry, stored_stats};
use crate::worktree_clone_conversion_rules::{ConversionRules, FenceVerdict, PathAttributes, apply_attribute, conversion_rules_from_config, decide, needs_blob_size};
use crate::worktree_ladder_target::checkout_environment;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCloneConversionFence";

/// The mode masks that name an entry's type,
/// and the type a symbolic link has.
const MODE_TYPE_MASK: u32 = 0o170000;
const MODE_SYMLINK: u32 = 0o120000;

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
        let mut fields = record[..tab].split(|byte| return *byte == b' ');
        let (Some(_mode), Some(_kind), Some(_object), Some(size)) = (fields.next(), fields.next(), fields.next(), fields.next()) else { continue };
        let Ok(size_text) = std::str::from_utf8(size) else { continue };
        let Ok(size_value) = size_text.parse::<u64>() else { continue };
        sizes.insert(record[tab + 1..].to_vec(), size_value);
    }
    return Ok(sizes);
}

/// The configuration entries the fence reads its settings from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configOf = (source) => source.config.entries();
/// ```
fn config_of(source: &SnapshotSource) -> &[ConfigEntry] {
    return source.config.entries();
}

/// What:
///  Decide every tracked path of the clean source:
/// what the clones may carry and what Git must write.
/// Why:
///   The answer is the fence,
/// and every read it needs is taken here,
/// so the caller only decides what to do with each path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceTheClone(source, context): Fence; // throws the reason the rung cannot be proven
/// ```
pub fn fence_the_clone(source: &SnapshotSource, context: &GitContext) -> Result<Fence, String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    if let Some(reason) = first_untracked_attributes(entries.as_slice(), source.root.as_path()) {
        return Err(reason);
    }
    let rules: ConversionRules = conversion_rules_from_config(config_of(source));
    let attributes: HashMap<Vec<u8>, PathAttributes> = attributes_of(context, source.root.as_path(), entries.as_slice())?;
    let mut sizes: Option<HashMap<Vec<u8>, u64>> = None;
    let mut git_writes: Vec<Vec<u8>> = Vec::new();
    let mut carried: usize = 0;
    for entry in &entries {
        let held: PathAttributes = attributes.get(entry.path.as_slice()).cloned().unwrap_or_default();
        let is_link: bool = entry.mode & MODE_TYPE_MASK == MODE_SYMLINK;
        let file_size: u64 = u64::from(entry.stored.size);
        let mut verdict: FenceVerdict = decide(&held, &rules, is_link, file_size, None);
        if !is_link && matches!(verdict, FenceVerdict::WriteThroughGit(_)) && needs_blob_size(&held, &rules) {
            if sizes.is_none() {
                sizes = Some(blob_sizes(context, source.root.as_path(), source.clean.head_commit.as_str())?);
            }
            let listed: Option<u64> = sizes.as_ref().and_then(|known| return known.get(entry.path.as_slice()).copied());
            verdict = decide(&held, &rules, is_link, file_size, listed);
        }
        match verdict {
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
///  Write the paths Git must write,
/// in the destination,
/// through one `git checkout-index --force --stdin`.
/// Why:
///   Git owns the conversion,
/// so asking it for these bytes is the only answer that needs no model of a filter, an encoding or an ending;
/// the run names the destination's Git directory and work tree,
/// as the ladder's other runs do,
/// so the destination's own settings decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeThroughGit(destination, admin, context, paths); // throws the reason
/// ```
pub fn write_through_git(destination: &Path, admin: &Path, context: &GitContext, paths: &[Vec<u8>]) -> Result<(), String> {
    let mut input: Vec<u8> = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    // What:
    //  The destination, written as `git checkout-index`'s `--prefix`, ending with the separator it needs.
    // Why:
    //  A path given `--prefix` is written through the conversion without the check Git makes of an in-place entry:
    //  `checkout-index` in place skips a path whose stat data matches its index entry (`entry.c`, `checkout_entry_ca`,
    //  the `if (!changed) return 0;`), and the installed index vouches for exactly the bytes that are wrong.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const prefix = `--prefix=${destination}/`;
    // ```
    let mut prefix: OsString = OsString::from("--prefix=");
    prefix.push(destination.as_os_str());
    prefix.push("/");
    crate::diagnostic_log::debug(TAG, format!("writing {} paths through Git in {}", paths.len(), destination.display()).as_str());
    let output: GitOutput = run_in(context, destination, words(&["checkout-index", "--force", "-z", "--stdin"]).into_iter().chain([prefix]).collect::<Vec<OsString>>().as_slice(), Some(input), checkout_environment(admin, destination))?;
    crate::diagnostic_log::debug(TAG, format!("checkout-index code {:?} stderr {}", output.code, String::from_utf8_lossy(output.stderr.as_slice()).trim()).as_str());
    return if output.code == Some(0) { Ok(()) } else { Err(run_reason("`git checkout-index`", &output)) };
}

#[cfg(test)]
#[path = "worktree_clone_conversion_fence_tests.rs"]
mod tests;

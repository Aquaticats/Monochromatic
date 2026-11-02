//! What:
//!  What the ladder does after real Git returned: the snapshot rung's steps (read the new `HEAD`, snapshot, apply the
//!  tree difference, swap into place, print Git's checkout line), and the fall back to Git's own `reset --hard` when a
//!  step cannot finish.
//! Why:
//!  Every step before the swap leaves the destination exactly as Git's `--no-checkout` run made it, so a failure there
//!  destroys the stage and completes the worktree the way Git would have. The line that Git's own checkout would have
//!  printed is printed here, so the person cannot tell the rungs apart except by the time they take. No hook runs on
//!  any rung: every real-Git start of the wrapper switches hooks off (`hooks_off.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await ladder.finish({ git, token, runGit, context, out });
//! ```

use super::diagnostic_log::{debug, warn};
use super::landed_summary::short_name;
use super::repo_read_layout::ObjectFormat;
use super::repo_read_object::from_hex;
use super::repo_read_odb::ObjectDatabase;
use super::repo_read_tree::{CommitFields, FlatEntry, flatten_tree, parse_commit, subject_of};
use super::transaction_git::GitContext;
use super::worktree_clean::head_commit;
use super::worktree_copy::{GitRunner, GitStatus, GitStep};
use super::worktree_add_plan::AddPlan;
use super::worktree_copy_stage_claim::StageClaim;
use super::worktree_ladder_run::{Ladder, LadderResult, Rung, discard_stage};
use super::worktree_snapshot::{put_in_place, remove_leftover, take_snapshot};
use super::worktree_snapshot_source::SnapshotSource;
use super::worktree_subvolume::settle_subvolume_destination;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  The administrative directory a freshly made `.git` pointer names.
/// Why:
///   The snapshot's index goes there, and `read-tree` runs against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function adminOf(pointer: Uint8Array, destination: string): string;
/// ```
fn admin_of(pointer: &[u8], destination: &Path) -> Result<PathBuf, String> {
    use std::os::unix::ffi::OsStrExt;
    let line: &[u8] = pointer.strip_suffix(b"\n").unwrap_or(pointer);
    let named: &[u8] = line.strip_prefix(b"gitdir: ").ok_or_else(|| { return String::from("Git's .git file has no gitdir line"); })?;
    return std::fs::canonicalize(destination.join(std::ffi::OsStr::from_bytes(named))).map_err(|error| { return format!("the administrative directory is unreadable ({error})"); });
}

/// What:
///  The `.gitattributes` files of a tree as path and object name.
/// Why:
///   Unchanged paths keep the source's bytes, so the conversion rules of the two trees must agree for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributeFiles(tree: FlatEntry[]): [Uint8Array, Uint8Array][];
/// ```
fn attribute_files(tree: &[FlatEntry]) -> Vec<(&[u8], &[u8])> {
    return tree
        .iter()
        .filter(|entry: &&FlatEntry| { return entry.path == b".gitattributes" || entry.path.ends_with(b"/.gitattributes"); })
        .map(|entry: &FlatEntry| { return (entry.path.as_slice(), entry.id.as_slice()); })
        .collect();
}

/// What:
///  Whether the two trees carry the same `.gitattributes` files.
/// Why:
///   A different rule could change how a path both trees share is written, which only Git's checkout gets right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributesAgree(database: ObjectDatabase, from: Uint8Array, to: Uint8Array, format: ObjectFormat): boolean;
/// ```
fn attributes_agree(database: &ObjectDatabase, from: &[u8], to: &[u8], format: ObjectFormat) -> Result<(), String> {
    let (Some(old), Some(new)) = (flatten_tree(database, from, format), flatten_tree(database, to, format)) else { return Err(String::from("a tree is unreadable")) };
    return if attribute_files(old.as_slice()) == attribute_files(new.as_slice()) { Ok(()) } else { Err(String::from("the .gitattributes files differ between the source and the new HEAD")) };
}

/// What:
///  Run one Git command through the lifecycle's runner and require status zero.
/// Why:
///   Every Git run of the ladder inherits the caller's streams and reports only a status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runStep(runGit, token, args, env);
/// ```
fn run_step(run_git: &mut GitRunner, token: Option<&str>, arguments: &[OsString], environment: &[(OsString, OsString)]) -> Result<GitStatus, String> {
    return run_git(token, &GitStep::Other { arguments, environment }).map_err(|error| { return error.0; });
}

/// What:
///  The environment Git gives the commands of its own checkout: its git directory and work tree.
/// Why:
///   The ladder's runs name the same two so that they see the new worktree whatever the caller's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const env = { GIT_DIR: gitDir, GIT_WORK_TREE: workTree };
/// ```
fn checkout_environment(git_dir: &Path, work_tree: &Path) -> Vec<(OsString, OsString)> {
    return vec![(OsString::from("GIT_DIR"), git_dir.as_os_str().to_os_string()), (OsString::from("GIT_WORK_TREE"), work_tree.as_os_str().to_os_string())];
}

/// What:
///  Everything before the swap and the swap itself; the error is the reason the rung could not finish.
/// Why:
///   The caller destroys the stage on an error and completes the worktree with Git's own checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveSnapshot(args): Promise<string>; // the checkout line; throws a reason
/// ```
fn serve_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext) -> Result<String, String> {
    let destination: &Path = plan.destination.as_path();
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).map_err(|error| { return format!("Git's .git file is unreadable ({error})"); })?;
    let admin: PathBuf = admin_of(pointer.as_slice(), destination)?;
    if std::fs::read_dir(destination).map_err(|error| { return format!("the destination is unreadable ({error})"); })?.count() != 1 {
        return Err(String::from("the destination holds more than Git's .git file"));
    }
    let format: ObjectFormat = if source.clean.hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let target: String = head_commit(admin.as_path(), source.common_dir.as_path(), format).ok_or_else(|| { return String::from("the new HEAD is not a plain ref to a commit"); })?;
    let database: ObjectDatabase = ObjectDatabase::open(source.common_dir.as_path(), None, format).ok_or_else(|| { return String::from("the object database is not plain"); })?;
    let name: Vec<u8> = from_hex(target.as_bytes(), format).ok_or_else(|| { return String::from("the new HEAD is not an object name"); })?;
    let raw = database.read(name.as_slice()).ok_or_else(|| { return String::from("the new HEAD commit is unreadable"); })?;
    let fields: CommitFields = parse_commit(raw.bytes.as_slice(), format).ok_or_else(|| { return String::from("the new HEAD commit has a layout the reader does not know"); })?;
    let differs: bool = fields.tree != source.clean.head_tree;
    if differs {
        attributes_agree(&database, source.clean.head_tree.as_slice(), fields.tree.as_slice(), format)?;
    }
    let line: String = if plan.quiet {
        String::new()
    } else {
        let subject: Vec<u8> = subject_of(fields.message.as_slice()).ok_or_else(|| { return String::from("the subject of the new HEAD needs Git to print"); })?;
        format!("HEAD is now at {} {}\n", short_name(context, destination, source.common_dir.join("objects").as_path(), Some(&source.config), target.as_str()), String::from_utf8_lossy(subject.as_slice()))
    };
    take_snapshot(source, plan, stage, admin.as_path(), pointer.as_slice())?;
    if differs {
        let arguments: Vec<OsString> = ["-C"].iter().map(OsString::from).chain([stage.as_os_str().to_os_string()]).chain(["read-tree", "-m", "-u"].iter().map(OsString::from)).chain([OsString::from(source.clean.head_commit.as_str()), OsString::from(target.as_str())]).collect();
        let status: GitStatus = run_step(run_git, token, arguments.as_slice(), checkout_environment(admin.as_path(), stage).as_slice())?;
        if !status.succeeded() {
            return Err(format!("`git read-tree -m -u` ended with {:?}", status.code));
        }
    }
    let mode: u32 = std::fs::metadata(source.root.as_path()).map_err(|error| { return format!("the source's root is unreadable ({error})"); })?.mode() & 0o7777;
    put_in_place(stage, destination, mode)?;
    if let Err(reason) = remove_leftover(stage) {
        warn(TAG, reason.as_str());
    }
    return Ok(line);
}

/// What:
///  Complete the worktree the way Git does after a refused rung: `git reset --hard --no-recurse-submodules`.
/// Why:
///   Real Git ran with `--no-checkout`, so the destination holds only its `.git` file; this is Git's own checkout step.
///  Git's `post-checkout` hook is not run: every real-Git start of the wrapper switches hooks off (`hooks_off.rs`), so
///  the Git run of a forwarded `git worktree add` never ran it either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkoutLikeGit(args): Promise<GitStatus>;
/// ```
fn checkout_like_git(plan: &AddPlan, token: Option<&str>, run_git: &mut GitRunner) -> GitStatus {
    let destination: &Path = plan.destination.as_path();
    let mut arguments: Vec<OsString> = ["-C"].iter().map(OsString::from).chain([destination.as_os_str().to_os_string()]).chain(["reset", "--hard", "--no-recurse-submodules"].iter().map(OsString::from)).collect();
    if plan.quiet {
        arguments.push(OsString::from("--quiet"));
    }
    let environment: Vec<(OsString, OsString)> = checkout_environment(destination.join(".git").as_path(), destination);
    let status: GitStatus = match run_step(run_git, token, arguments.as_slice(), environment.as_slice()) {
        Ok(status) => status,
        Err(reason) => {
            warn(TAG, format!("Git's checkout of {} could not start ({reason})", destination.display()).as_str());
            return GitStatus { code: Some(1) };
        }
    };
    if !status.succeeded() {
        return status;
    }
    return status;
}

/// Release the claim on a stage name, saying so when that fails.
fn release_claim(claim: StageClaim) {
    if let Err(error) = claim.release() {
        warn(TAG, format!("the stage claim could not be released ({})", error.0).as_str());
    }
}

impl Ladder {
    /// What:
    ///  Settle the ladder after real Git returned `git`: serve the snapshot rung or fall back to Git's checkout, or settle
    ///  the empty subvolume of the Git rung.
    /// Why:
    ///   The result carries the status the person should see and the destination whose ignored state a snapshot carried.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const result = await ladder.finish({ git, token, runGit, context, out });
    /// ```
    pub fn finish(self, git: GitStatus, token: Option<&str>, run_git: &mut GitRunner, context: &GitContext, out: &mut dyn Write) -> LadderResult {
        let (source, claim, plan) = match (self.rung, self.plan) {
            (Rung::Snapshot(source, claim), Some(plan)) => (source, claim, plan),
            (Rung::Snapshot(_, claim), None) => {
                release_claim(claim);
                return LadderResult { git, carried: None };
            }
            (Rung::Git(subvolume), _) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_subvolume_destination(created);
                }
                return LadderResult { git, carried: None };
            }
        };
        let stage: PathBuf = claim.container().to_path_buf();
        if !git.succeeded() {
            release_claim(claim);
            return LadderResult { git, carried: None };
        }
        let result: LadderResult = match serve_snapshot(&source, &plan, stage.as_path(), token, run_git, context) {
            Ok(line) => {
                debug(TAG, format!("the snapshot served {}", plan.destination.display()).as_str());
                // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
                let _written = out.write_all(line.as_bytes());
                LadderResult { git, carried: Some(plan.destination.clone()) }
            }
            Err(reason) => {
                debug(TAG, format!("the snapshot rung ended: {reason}; Git checks the worktree out").as_str());
                discard_stage(stage.as_path());
                LadderResult { git: checkout_like_git(&plan, token, run_git), carried: None }
            }
        };
        release_claim(claim);
        return result;
    }
}

/// Ladder finish controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_finish_tests.rs"]
mod tests;

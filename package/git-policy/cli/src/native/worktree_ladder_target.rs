//! What:
//!  What both rungs that fill a worktree themselves need to know about the target of a `git worktree add` once real Git
//!  ran without a checkout: the administrative directory, Git's pointer, the commit and tree `HEAD` names, whether the
//!  tree differs from the source's, the line Git's checkout would print, and the step that applies the difference.
//! Why:
//!  The snapshot and clone rungs differ in how the source's files reach the destination and agree on everything else, so
//!  the shared reading of Git's result and the single `git read-tree -m -u` step live in one place. The first version keeps
//!  Git for the two-tree update because it owns the conversion filters (`doc/planning/cli-git-btrfs-worktree.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const target = prepareTarget(source, plan, context); // throws a reason
//! await applyDifference(source, target, workTree, runGit);
//! ```

use super::landed_summary::short_name;
use super::repo_read_layout::ObjectFormat;
use super::repo_read_object::from_hex;
use super::repo_read_odb::ObjectDatabase;
use super::repo_read_tree::{CommitFields, FlatEntry, flatten_tree, parse_commit, subject_of};
use super::transaction_git::GitContext;
use super::worktree_add_plan::AddPlan;
use super::worktree_clean::head_commit;
use super::worktree_copy::{GitRunner, GitStatus, GitStep};
use super::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The target of one `git worktree add` as real Git left it.
/// Why:
///   The rungs read these facts once and use them in turn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { admin: string; pointer: Uint8Array; commit: string; differs: boolean; line: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// The administrative directory of the new worktree.
    pub admin: PathBuf,
    /// The bytes of the `.git` file Git wrote in the destination.
    pub pointer: Vec<u8>,
    /// The commit the new `HEAD` names, as hexadecimal.
    pub commit: String,
    /// Whether the tree of that commit differs from the source's.
    pub differs: bool,
    /// The line Git's checkout prints (empty for `--quiet`).
    pub line: String,
}

/// What:
///  The administrative directory a freshly made `.git` pointer names.
/// Why:
///   The snapshot's index goes there, and `read-tree` runs against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function adminOf(pointer: Uint8Array, destination: string): string;
/// ```
pub(super) fn admin_of(pointer: &[u8], destination: &Path) -> Result<PathBuf, String> {
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
pub(super) fn run_step(run_git: &mut GitRunner, token: Option<&str>, arguments: &[OsString], environment: &[(OsString, OsString)]) -> Result<GitStatus, String> {
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
pub(super) fn checkout_environment(git_dir: &Path, work_tree: &Path) -> Vec<(OsString, OsString)> {
    return vec![(OsString::from("GIT_DIR"), git_dir.as_os_str().to_os_string()), (OsString::from("GIT_WORK_TREE"), work_tree.as_os_str().to_os_string())];
}

/// What:
///  Read the result of the `--no-checkout` run: the pointer, the administrative directory, the new `HEAD` and the line.
///  The error is the reason the rungs cannot serve the command.
/// Why:
///   A destination that holds more than Git's pointer, a `HEAD` the readers cannot resolve, a commit whose subject needs
///  Git to print, or `.gitattributes` files that differ between the two trees leave the checkout to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTarget(source: SnapshotSource, plan: AddPlan, context: GitContext): Target; // throws a reason
/// ```
pub fn prepare_target(source: &SnapshotSource, plan: &AddPlan, context: &GitContext) -> Result<Target, String> {
    let destination: &Path = plan.destination.as_path();
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).map_err(|error| { return format!("Git's .git file is unreadable ({error})"); })?;
    let admin: PathBuf = admin_of(pointer.as_slice(), destination)?;
    if std::fs::read_dir(destination).map_err(|error| { return format!("the destination is unreadable ({error})"); })?.count() != 1 {
        return Err(String::from("the destination holds more than Git's .git file"));
    }
    let format: ObjectFormat = if source.clean.hash_length == 32 { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
    let commit: String = head_commit(admin.as_path(), source.common_dir.as_path(), format).ok_or_else(|| { return String::from("the new HEAD is not a plain ref to a commit"); })?;
    let database: ObjectDatabase = ObjectDatabase::open(source.common_dir.as_path(), None, format).ok_or_else(|| { return String::from("the object database is not plain"); })?;
    let name: Vec<u8> = from_hex(commit.as_bytes(), format).ok_or_else(|| { return String::from("the new HEAD is not an object name"); })?;
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
        format!("HEAD is now at {} {}\n", short_name(context, destination, source.common_dir.join("objects").as_path(), Some(&source.config), commit.as_str()), String::from_utf8_lossy(subject.as_slice()))
    };
    return Ok(Target { admin, pointer, commit, differs, line });
}

/// What:
///  Apply the difference between the source's tree and the new `HEAD` to `work_tree`, whose index is the source's:
///  nothing when the trees are equal, else one `git read-tree -m -u`.
/// Why:
///   Git owns the conversion filters, so the paths that differ are written by Git and the unchanged ones keep the bytes
///  the source had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await applyDifference(source, target, workTree, token, runGit); // throws a reason
/// ```
pub fn apply_difference(source: &SnapshotSource, target: &Target, work_tree: &Path, token: Option<&str>, run_git: &mut GitRunner) -> Result<(), String> {
    if !target.differs {
        return Ok(());
    }
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        work_tree.as_os_str().to_os_string(),
        OsString::from("read-tree"),
        OsString::from("-m"),
        OsString::from("-u"),
        OsString::from(source.clean.head_commit.as_str()),
        OsString::from(target.commit.as_str()),
    ];
    let status: GitStatus = run_step(run_git, token, arguments.as_slice(), checkout_environment(target.admin.as_path(), work_tree).as_slice())?;
    return if status.succeeded() { Ok(()) } else { Err(format!("`git read-tree -m -u` ended with {:?}", status.code)) };
}

/// Ladder target controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_target_tests.rs"]
mod tests;

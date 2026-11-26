//! What:
//!  What both rungs that fill a worktree themselves need to know about the target of a `git worktree add` once real Git
//!  ran without a checkout:
//! the administrative directory,
//! Git's pointer,
//! the commit and tree `HEAD` names,
//! the line Git's checkout would print,
//! and the step that applies the difference.
//! Why:
//!  The snapshot and clone rungs differ in how the source's files reach the destination and agree on everything else,
//! so
//!  the shared reading of Git's result and the single `git read-tree -m -u` step live in one place.
//! The first version keeps
//!  Git for the two-tree update because it owns the conversion filters (`doc/planning/cli-git-btrfs-worktree.md`).
//! The checks that the two worktrees convert files alike are a file of their own because this one is over the line
//!  limit:
//! `worktree_ladder_target_conversion_agreement.rs` holds them,
//! and this file asks them,
//!  the same way a caller always has.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const target = prepareTarget(source, plan, context); // throws a reason
//! await applyDifference(source, target, workTree, runGit);
//! ```

use crate::config_snapshot::read_config_snapshot;
use crate::forwarding::NoChildRecorded;
use crate::fs_beneath::handle_path;
use crate::landed_summary::short_name;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::from_hex;
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{
    parse_commit,
    subject_of,
};
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{
    Bound,
    path_identity,
};
use crate::worktree_clean::head_commit;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
    GitStep,
};
#[cfg(test)]
use crate::worktree_ladder_target_conversion_agreement::{
    LFS_FILTER,
    filters_named,
};
use crate::worktree_ladder_target_conversion_agreement::{
    attributes_agree,
    conversion_agrees,
    external_conversion_refusal,
    untracked_attribute_texts,
};
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The target of one `git worktree add` as real Git left it.
/// Why:
///   The rungs read these facts once and use them in turn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { admin: string; pointer: Uint8Array; commit: string; differs: boolean; line: string; index?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// The administrative directory of the new worktree.
    pub admin: PathBuf,
    /// The index file the restore reads and writes when it is not the
    /// worktree's own name yet,
    /// which the ZFS clone route's fill needs
    /// (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
    /// "The restore" step 5 and F41).
    pub index: Option<PathBuf>,
    /// The bytes of the `.git` file Git wrote in the destination.
    pub pointer: Vec<u8>,
    /// The commit the new `HEAD` names,
    /// as hexadecimal.
    pub commit: String,
    /// Whether the tree of that commit differs from the source's.
    pub differs: bool,
    /// The line Git's checkout prints (empty for `--quiet`).
    pub line: String,
}

/// What:
///  Run one Git command through the lifecycle's runner and require status zero.
/// Why:
///   Every Git run of the ladder inherits the caller's streams and reports only a status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runStep(runGit, args, env);
/// ```
pub(crate) fn run_step(
    run_git: &mut dyn GitRunner,
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
) -> Result<GitStatus, String> {
    // A step of the ladder runs after the add's binding is recorded, so its Git needs no name in a record.
    return match run_git.run(
        &GitStep::Other {
            arguments,
            environment,
        },
        &mut NoChildRecorded,
    ) {
        Ok(status) => Ok(status),
        Err(error) => Err(error.0),
    };
}

/// What:
///  The environment Git gives the commands of its own checkout:
/// its git directory and work tree.
/// Why:
///   The ladder's runs name the same two so that they see the new worktree whatever the caller's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const env = { GIT_DIR: gitDir, GIT_WORK_TREE: workTree };
/// ```
pub(crate) fn checkout_environment(
    git_dir: &Path,
    work_tree: &Path,
) -> Vec<(OsString, OsString)> {
    return checkout_environment_with_index((git_dir, work_tree), None);
}

/// What:
///  `checkout_environment` with the index file named,
///  for a run that must work on an index the worktree does not hold under its own name yet.
/// Why:
///   A Git run given `GIT_INDEX_FILE` reads and writes that file and leaves the worktree's `index` alone,
///  which is what a rung whose index awaits the paths Git must write needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const env = { GIT_DIR: gitDir, GIT_WORK_TREE: workTree, GIT_INDEX_FILE: index };
/// ```
pub(crate) fn checkout_environment_with_index(
    (git_dir, work_tree): (&Path, &Path),
    index: Option<&Path>,
) -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = vec![
        (
            OsString::from("GIT_DIR"),
            git_dir.as_os_str().to_os_string(),
        ),
        (
            OsString::from("GIT_WORK_TREE"),
            work_tree.as_os_str().to_os_string(),
        ),
    ];
    if let Some(named) = index {
        environment.push((
            OsString::from("GIT_INDEX_FILE"),
            named.as_os_str().to_os_string(),
        ));
    }
    return environment;
}

/// What:
///  The database the target's own HEAD and tree are read from.
/// Why:
///  They are the new worktree's own objects,
/// so they come from the repository's own store,
///  as `main`'s reader read them;
///  a store or a replacement base the command's environment names is not what they are,
///  and where the repository's own store lacks them this answers `None` and the rung leaves the add to Git.
///  One function,
/// so the one place the rung opens the database is the place a control can ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const database = openObjectDatabaseOfTheRepositoryAlone({ commonDir, format });
/// ```
fn the_targets_database(
    common_dir: &Path,
    format: ObjectFormat,
) -> Option<ObjectDatabase> {
    return ObjectDatabase::open_of_the_repository_alone(common_dir, None, format);
}

/// What:
///  Read the result of the `--no-checkout` run:
/// the pointer,
/// the administrative directory,
/// the new `HEAD` and the line.
///  The error is the reason the rungs cannot serve the command.
/// Why:
///   A destination that holds more than Git's pointer,
/// a `HEAD` the readers cannot resolve,
/// a commit whose subject needs
///  Git to print,
/// or `.gitattributes` files that differ between the two trees leave the checkout to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTarget(source: SnapshotSource, plan: AddPlan, context: GitContext): Target; // throws a reason
/// ```
pub fn prepare_target(
    source: &SnapshotSource,
    plan: &AddPlan,
    context: &GitContext,
    bound: &Bound,
) -> Result<Target, String> {
    let destination: &Path = plan.destination.as_path();
    if bound.binding.destination != destination {
        return Err(String::from("the add is bound to another destination"));
    }
    let pointer: Vec<u8> = bound.binding.pointer.clone();
    let admin: PathBuf = bound.binding.admin.clone();
    // The destination as bound must still be the one directory Git made, holding nothing but Git's pointer.
    if path_identity(destination) != Some(bound.binding.destination_id) {
        return Err(String::from(
            "the destination is no longer the directory Git made",
        ));
    }
    let held_listing = match std::fs::read_dir(handle_path(&bound.destination)) {
        Ok(listing) => listing,
        Err(error) => return Err(format!("the destination is unreadable ({error})")),
    };
    let held_entries: usize = held_listing.count();
    if held_entries != 1 {
        return Err(String::from(
            "the destination holds more than Git's .git file",
        ));
    }
    let Some(target_config) = read_config_snapshot(context, destination) else {
        return Err(String::from(
            "the new worktree's configuration could not be read",
        ));
    };
    conversion_agrees(source.config.entries(), target_config.entries())?;
    let format: ObjectFormat = if source.clean.hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    let Some(commit) = head_commit(admin.as_path(), source.common_dir.as_path(), format) else {
        return Err(String::from("the new HEAD is not a plain ref to a commit"));
    };
    let Some(database) = the_targets_database(source.common_dir.as_path(), format) else {
        return Err(String::from("the object database is not plain"));
    };
    let Some(name) = from_hex(commit.as_bytes(), format) else {
        return Err(String::from("the new HEAD is not an object name"));
    };
    let Some(raw) = database.read(name.as_slice()) else {
        return Err(String::from("the new HEAD commit is unreadable"));
    };
    let Some(fields) = parse_commit(raw.bytes.as_slice(), format) else {
        return Err(String::from(
            "the new HEAD commit has a layout the reader does not know",
        ));
    };
    let differs: bool = fields.tree != source.clean.head_tree;
    if differs {
        attributes_agree(
            &database,
            source.clean.head_tree.as_slice(),
            fields.tree.as_slice(),
            format,
        )?;
    }
    // The tracked attributes files are the same in both trees by now, so the source's stand for both.
    let mut attribute_texts: Vec<Vec<u8>> =
        untracked_attribute_texts(source, target_config.entries());
    for (path, id) in &source.clean.attribute_blobs {
        let Some(blob) = database.read(id.as_slice()) else {
            return Err(format!("{} is unreadable", String::from_utf8_lossy(path)));
        };
        attribute_texts.push(blob.bytes.clone());
    }
    if let Some(reason) =
        external_conversion_refusal(target_config.entries(), attribute_texts.as_slice())
    {
        return Err(reason);
    }
    let line: String = if plan.quiet {
        String::new()
    } else {
        let Some(subject) = subject_of(fields.message.as_slice()) else {
            return Err(String::from(
                "the subject of the new HEAD needs Git to print",
            ));
        };
        format!(
            "HEAD is now at {} {}\n",
            short_name(
                context,
                destination,
                source.common_dir.join("objects").as_path(),
                Some(&source.config),
                commit.as_str()
            ),
            String::from_utf8_lossy(subject.as_slice())
        )
    };
    return Ok(Target {
        admin,
        index: None,
        pointer,
        commit,
        differs,
        line,
    });
}

impl Target {
    /// The index file the restore's three runs read and write:
    /// the one this target names,
    /// or the worktree's own.
    /// Why:
    ///  Every site that named `<admin>/index` reads this,
    /// so a route that writes the index under a private name first changes one
    /// place and no run reads a name the worktree does not hold.
    pub fn index_file(&self) -> PathBuf {
        return match self.index.clone() {
            Some(named) => named,
            None => self.admin.join("index"),
        };
    }
}

/// What:
///  Apply the difference between the source's tree and the new `HEAD` to `work_tree`,
/// whose index is the source's:
///  nothing when the trees are equal,
/// else one `git read-tree -m -u`.
/// Why:
///   Git owns the conversion filters,
/// so the paths that differ are written by Git and the unchanged ones keep the bytes
///  the source had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await applyDifference(source, target, workTree, runGit); // throws a reason
/// ```
pub fn apply_difference(
    source: &SnapshotSource,
    target: &Target,
    work_tree: &Path,
    run_git: &mut dyn GitRunner,
) -> Result<(), String> {
    return apply_difference_with_index((source, target, work_tree, run_git), None);
}

/// What:
///  `apply_difference` against the index file `index` when it is named,
///  rather than the worktree's own.
/// Why:
///   The clone rung writes its index under its own name until the paths Git must write are written
/// (`worktree_clone_index::PendingIndex`),
/// so the difference is applied to that file and the worktree's `index` appears only once the write is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await applyDifferenceWithIndex({ source, target, workTree, runGit }, pendingIndex); // throws a reason
/// ```
pub fn apply_difference_with_index(
    (source, target, work_tree, run_git): (&SnapshotSource, &Target, &Path, &mut dyn GitRunner),
    index: Option<&Path>,
) -> Result<(), String> {
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
    let status: GitStatus = run_step(
        run_git,
        arguments.as_slice(),
        checkout_environment_with_index((target.admin.as_path(), work_tree), index).as_slice(),
    )?;
    return if status.succeeded() {
        Ok(())
    } else {
        Err(format!(
            "`git read-tree -m -u` ended with {:?}",
            status.code
        ))
    };
}

/// Ladder target controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_target_tests.rs"]
mod tests;

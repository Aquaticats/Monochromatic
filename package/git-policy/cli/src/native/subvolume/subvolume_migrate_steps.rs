//! What:
//!  The steps of a migration that change the checkout:
//! create the new subvolume and the nested ones,
//! hold the index locks,
//! run the last delta and exchange the two directories,
//! record which directory the exchange installed,
//! and finish with the catch-up,
//! the index rewrite and the verification.
//! Why:
//!  Every step is idempotent and skipped when the journal says it finished,
//! so a rerun after a kill at any point resumes;
//! the command that drives them is `subvolume_migrate.rs`,
//! the options and the verdict it returns are `subvolume_migrate_options.rs`,
//! and the lines a person reads are `subvolume_migrate_words.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createSubvolumes(run, options); await exchangeTrees(run); await finish(run, options);
//! ```

use super::subvolume_migrate_options::MigrateOptions;
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::{
    IndexLock,
    linked_worktrees_inside,
};
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use crate::subvolume_lock::{
    acquire,
    lock_paths,
    release_all,
    token_of,
};
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight::{
    INSTALLED_ROOT,
    installed_identity,
};
use crate::subvolume_run::{
    INDEX_LOCK,
    Run,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

/// Whether a path is the root of a subvolume,
/// false when it cannot be asked.
pub(crate) fn is_subvolume(
    platform: &dyn Platform,
    path: &Path,
) -> bool {
    return platform.is_subvolume_root(path).unwrap_or(false);
}

/// What:
///  The create and nested steps:
/// the new subvolume,
/// then one nested subvolume per linked worktree inside the checkout.
/// Why:
///   A later snapshot of the root then holds an empty stub for each worktree instead of a full copy;
/// the copy fills them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createSubvolumes(run, options);
/// ```
pub(crate) fn create_subvolumes(
    run: &mut Run<'_>,
    options: MigrateOptions,
) -> Result<(), MigrateError> {
    if !run.journal.is_done("create") {
        run.journal.mark("create", "started")?;
        if !run.layout.new_path.exists() {
            run.platform
                .create_subvolume(run.layout.new_path.as_path())?;
        }
        // The identity of the stage this step made or adopted rides with the step's mark, so one write carries both: a journal
        // that says the step finished also names the subvolume, and the preflight lets through only an empty subvolume when
        // the identity is not recorded yet.
        let Some(identity) =
            crate::worktree_convert_phases::identity_of_path(run.layout.new_path.as_path())
        else {
            return Err(MigrateError::Journal(format!(
                "{} was not there after it was created",
                run.layout.new_path.display()
            )));
        };
        run.journal.record_stage_identity(identity);
        run.journal.mark("create", "done")?;
    }
    run.checkpoint("after-create")?;
    if options.nested_worktrees && !run.journal.is_done("nested") {
        for linked in linked_worktrees_inside(run.layout.root.as_path()) {
            let Ok(relative) = linked.worktree.strip_prefix(run.layout.root.as_path()) else {
                continue;
            };
            let target: PathBuf = run.layout.new_path.join(relative);
            if let Some(parent) = target.parent() {
                MigrateError::from_io_result(
                    std::fs::create_dir_all(parent),
                    "create the directory",
                    parent,
                )?;
            }
            if !target.exists() {
                run.platform.create_subvolume(target.as_path())?;
            }
        }
        run.journal.mark("nested", "done")?;
    }
    return run.checkpoint("after-nested");
}

/// What:
///  The exchange step:
/// hold the index lock,
/// run the last delta,
/// swap the two directories,
/// rename the old tree aside.
/// Why:
///   The lock keeps Git's index writers out while the last delta runs and the swap happens.
/// It is Git's own `index.lock`
///  (nothing else excludes Git),
/// one per index,
/// created with `O_EXCL`,
/// never waited for,
/// and held only from the last delta
///  to the end of the index rewrite;
/// see `subvolume_lock.rs`.
/// After a kill,
/// `exchange`
///  recorded as started with the checkout already a subvolume means only the rename or the record is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await exchangeTrees(run);
/// ```
pub(crate) fn exchange_trees(run: &mut Run<'_>) -> Result<(), MigrateError> {
    if run.journal.is_done("exchange") {
        return Ok(());
    }
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    if run.journal.has_begun("exchange") && is_subvolume(run.platform, root.as_path()) {
        if new_path.exists() && !old_path.exists() {
            MigrateError::from_io_result(
                std::fs::rename(&new_path, &old_path),
                "rename",
                new_path.as_path(),
            )?;
        }
        record_installed(run);
        return run.journal.mark("exchange", "done");
    }
    let token: String = token_of(&run.journal);
    acquire_all(root.as_path(), token.as_str())?;
    run.journal.mark("lock", "started")?;
    run.line(format!("holding {} (and the lock of every linked worktree's index) so no Git command writes an index during the last delta", root.join(INDEX_LOCK).display()).as_str());
    run.checkpoint("after-lock")?;
    run.pass(
        "last delta (under the lock)",
        root.as_path(),
        new_path.as_path(),
        true,
        false,
    )?;
    // A worktree administered since the locks were taken has no lock yet; the new tree holds the same locks before the exchange.
    acquire_all(root.as_path(), token.as_str())?;
    acquire_all(new_path.as_path(), token.as_str())?;
    run.journal.mark("exchange", "started")?;
    run.checkpoint("before-exchange")?;
    run.platform.exchange(root.as_path(), new_path.as_path())?;
    run.checkpoint("after-exchange")?;
    MigrateError::from_io_result(
        std::fs::rename(&new_path, &old_path),
        "rename",
        new_path.as_path(),
    )?;
    run.checkpoint("after-rename")?;
    run.line(format!("exchanged; the old tree is now {}", old_path.display()).as_str());
    record_installed(run);
    return run.journal.mark("exchange", "done");
}

/// What:
///  Record which directory the exchange installed at the checkout,
/// once,
/// with the step that installed it.
/// Why:
///   A later `--remove-old` or `--rollback` tells the installed checkout from one that replaced it by hand with this identity,
/// and a migration stopped after the exchange but before verification is exactly the state a person is most likely to
///  rearrange:
/// recording it only at verification leaves every step in between unprotected.
/// The first record wins,
/// so a
///  directory that took the checkout's place after the install is never blessed with its own identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordInstalled(run); // journal.values.installedRoot ??= identityOf(run.layout.root)
/// ```
fn record_installed(run: &mut Run<'_>) {
    if run.journal.value(INSTALLED_ROOT).is_some() {
        return;
    }
    if let Some(identity) = installed_identity(run.layout.root.as_path()) {
        run.journal.set_value(INSTALLED_ROOT, identity.as_str());
    }
}

/// What:
///  Create the migration's lock of every index in a tree.
/// Why:
///   A lock held by another Git command stops the migration with the file named;
/// its own lock from an earlier run is
///  accepted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const lock of lockPaths(tree)) await acquire(lock, token);
/// ```
fn acquire_all(
    tree: &Path,
    token: &str,
) -> Result<(), MigrateError> {
    for lock in lock_paths(tree) {
        acquire(lock.as_path(), token)?;
    }
    return Ok(());
}

/// What:
///  Remove the migration's locks that travelled with the old tree.
/// Why:
///   The old tree is kept as a plain directory;
/// a lock left in it would block every Git command there,
/// and only the
///  locks that hold this migration's token are removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await releaseAll(oldPath, token);
/// ```
fn drop_old_locks(run: &mut Run<'_>) -> Result<(), MigrateError> {
    let token: String = token_of(&run.journal);
    let released: usize = release_all(run.layout.old_path.as_path(), token.as_str())?;
    if released > 0 {
        run.line(
            format!(
                "removed {released} index lock(s) from {}",
                run.layout.old_path.display()
            )
            .as_str(),
        );
    }
    return Ok(());
}

/// What:
///  The steps after the exchange:
/// catch-up,
/// index rewrite,
/// verification.
/// Why:
///   Writers that held descriptors in the old tree may have written after the swap;
/// the index rewrite makes the first
///  `git status` cheap;
/// the verification proves the checkout is a subvolume root with a readable `.git/HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await finish(run, options);
/// ```
pub(crate) fn finish(
    run: &mut Run<'_>,
    options: MigrateOptions,
) -> Result<(), MigrateError> {
    let root: PathBuf = run.layout.root.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    drop_old_locks(run)?;
    let token: String = token_of(&run.journal);
    if !run.journal.is_done("catchup") {
        run.pass(
            "catch-up (files newer in the old tree)",
            old_path.as_path(),
            root.as_path(),
            false,
            true,
        )?;
        run.journal.mark("catchup", "done")?;
    }
    run.checkpoint("after-catchup")?;
    if options.refresh_indexes && !run.journal.is_done("indexes") {
        let started: Instant = Instant::now();
        let mut lines: Vec<String> = Vec::new();
        // The refresh reports each index into the list the lines are collected in; they are said even when the refresh
        //  stops, because each of them is a fact about an index that was looked at.
        let refreshed: Result<usize, MigrateError> = refresh_all_indexes(
            root.as_path(),
            IndexLock::Held(token.as_str().into()),
            std::slice::from_ref(&old_path),
            run.platform.flushes_to_disk(),
            &mut lines,
        );
        for text in &lines {
            run.line(text.as_str());
        }
        let count: usize = refreshed?;
        run.line(
            format!(
                "index stat rewrite: {count} index file(s), {:.2} s",
                started.elapsed().as_secs_f64()
            )
            .as_str(),
        );
        run.journal.mark("indexes", "done")?;
    }
    run.checkpoint("after-indexes")?;
    // The new checkout was locked from the exchange until now; Git commands may use it again.
    release_all(root.as_path(), token.as_str())?;
    if !is_subvolume(run.platform, root.as_path()) {
        return Err(MigrateError::Journal(format!(
            "{} is not a subvolume root after the exchange; run with --rollback to restore the original tree",
            root.display()
        )));
    }
    if !root.join(".git").join("HEAD").exists() {
        return Err(MigrateError::Journal(format!(
            "{} has no .git/HEAD after the exchange; run with --rollback to restore the original tree",
            root.display()
        )));
    }
    // A journal that somehow reached verification with no installed identity gets one now; the first record is the install's,
    // and a later step never blesses a directory that took the checkout's place.
    record_installed(run);
    return run.journal.mark("verify", "done");
}

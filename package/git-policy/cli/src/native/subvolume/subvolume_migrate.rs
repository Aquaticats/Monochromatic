//! What:
//!  Convert a main checkout into a btrfs subvolume at the same path,
//! in process,
//! resumably.
//! Why:
//!  The steps are those of the proven script (`doc/handover/cli-git-subvolume-migration-script.md`),
//! without any
//!  program started:
//! create `<root>.subvol-new` as a subvolume;
//! create the linked worktrees inside it as nested
//!  subvolumes;
//! copy the tree with cloned data;
//! repeat the copy as a delta;
//! hold `.git/index.lock`,
//! run the last
//!  delta and swap the two directories in one atomic step;
//! rename the old tree aside;
//! carry over files newer in the
//!  old tree;
//! rewrite the Git index stat fields;
//! verify.
//! Each step is recorded in the journal (`subvolume_journal.rs`),
//!  so a rerun after a kill at any point resumes:
//! a finished step is skipped and an unfinished one is repeated,
//! and all
//!  of them are idempotent.
//! The old tree is never deleted here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = await migrate({ platform, root, options, interrupt, say });
//! ```

use crate::process_use::{UseTarget, processes_using};
use crate::subvolume_arguments::MIGRATE_RISK_NOTICE;
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_index_files::{IndexLock, linked_worktrees_inside};
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use crate::subvolume_lock::{acquire, lock_paths, release_all, token_of};
use crate::subvolume_platform::Platform;
use crate::subvolume_preflight::{INSTALLED_ROOT, Layout, Refusal, installed_identity, layout_of, migrate_refusals};
use crate::subvolume_run::{INDEX_LOCK, Run};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What:
///  What the person asked for the migration to do.
/// Why:
///   The two switches are the script's `--no-nested-worktrees` and `--no-refresh-indexes`;
/// the dry run previews.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MigrateOptions = { dryRun: boolean; nestedWorktrees: boolean; refreshIndexes: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrateOptions {
    /// Preview only:
    /// nothing is created,
    /// written or journaled.
    pub dry_run: bool,
    /// Create the linked worktrees inside the checkout as nested subvolumes.
    pub nested_worktrees: bool,
    /// Rewrite the Git index stat fields after the exchange.
    pub refresh_indexes: bool,
}

/// What:
///  How a migration command ended,
/// when it did not fail.
/// Why:
///   The command maps each to an exit status and the closing text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'migrated' } | { kind: 'already' } | { kind: 'dry-run' } | { kind: 'refused'; refusals: Refusal[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The checkout is now a subvolume and verified.
    Migrated,
    /// The checkout already was one (with a finished journal or none).
    AlreadyDone,
    /// The preview was printed.
    DryRun,
    /// Nothing was changed:
    /// these conditions stand in the way.
    Refused(Vec<Refusal>),
}

/// Whether a path is the root of a subvolume,
/// false when it cannot be asked.
fn is_subvolume(platform: &dyn Platform, path: &Path) -> bool {
    return platform.is_subvolume_root(path).unwrap_or(false);
}

/// What:
///  Print the facts the preview and the run begin with:
/// the mount,
/// the option,
/// the processes that will stay behind.
/// Why:
///   A process whose working directory is inside the checkout keeps writing into the old tree after the exchange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// announce(run, options);
/// ```
fn announce(run: &mut Run<'_>, options: MigrateOptions) {
    let root: PathBuf = run.layout.root.clone();
    let removal: bool = run.platform.removal_allowed(root.as_path());
    run.line(format!("checkout {}; new subvolume {}; old tree will be {}", root.display(), run.layout.new_path.display(), run.layout.old_path.display()).as_str());
    run.line(format!("user_subvol_rm_allowed: {}", if removal { "active" } else { "absent (a subvolume is then removed by emptying it and rmdir; see git monochromatic-git-policy-cli doctor)" }).as_str());
    let real: PathBuf = std::fs::canonicalize(root.as_path()).unwrap_or(root);
    let scan = processes_using(&UseTarget::Inside(real.as_path()));
    if scan.users.is_empty() {
        run.line("no other process has its working directory inside the checkout");
    } else {
        run.line(format!("WARNING: {} process(es) have a working directory inside the checkout and will stay in the OLD tree after the exchange; restart them (or `cd \"$PWD\"` in a shell) afterwards:", scan.users.len()).as_str());
        for user in &scan.users {
            run.line(format!("  pid {} ({})", user.pid, user.command.as_deref().unwrap_or("name unreadable")).as_str());
        }
    }
    let nested: usize = if options.nested_worktrees { linked_worktrees_inside(run.layout.root.as_path()).len() } else { 0 };
    run.line(format!("linked worktrees inside the checkout that become nested subvolumes: {nested}").as_str());
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
fn create_subvolumes(run: &mut Run<'_>, options: MigrateOptions) -> Result<(), MigrateError> {
    if !run.journal.is_done("create") {
        run.journal.mark("create", "started")?;
        if !run.layout.new_path.exists() {
            run.platform.create_subvolume(run.layout.new_path.as_path())?;
        }
        // The identity of the stage this step made or adopted rides with the step's mark, so one write carries both: a journal
        // that says the step finished also names the subvolume, and the preflight lets through only an empty subvolume when
        // the identity is not recorded yet.
        let Some(identity) = crate::worktree_convert_phases::identity_of_path(run.layout.new_path.as_path()) else {
            return Err(MigrateError::Journal(format!("{} was not there after it was created", run.layout.new_path.display())));
        };
        run.journal.record_stage_identity(identity);
        run.journal.mark("create", "done")?;
    }
    run.checkpoint("after-create")?;
    if options.nested_worktrees && !run.journal.is_done("nested") {
        for linked in linked_worktrees_inside(run.layout.root.as_path()) {
            let Ok(relative) = linked.worktree.strip_prefix(run.layout.root.as_path()) else { continue };
            let target: PathBuf = run.layout.new_path.join(relative);
            if let Some(parent) = target.parent() {
                MigrateError::from_io_result(std::fs::create_dir_all(parent), "create the directory", parent)?;
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
fn exchange_trees(run: &mut Run<'_>) -> Result<(), MigrateError> {
    if run.journal.is_done("exchange") {
        return Ok(());
    }
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    if run.journal.has_begun("exchange") && is_subvolume(run.platform, root.as_path()) {
        if new_path.exists() && !old_path.exists() {
            MigrateError::from_io_result(std::fs::rename(&new_path, &old_path), "rename", new_path.as_path())?;
        }
        record_installed(run);
        return run.journal.mark("exchange", "done");
    }
    let token: String = token_of(&run.journal);
    acquire_all(root.as_path(), token.as_str())?;
    run.journal.mark("lock", "started")?;
    run.line(format!("holding {} (and the lock of every linked worktree's index) so no Git command writes an index during the last delta", root.join(INDEX_LOCK).display()).as_str());
    run.checkpoint("after-lock")?;
    run.pass("last delta (under the lock)", root.as_path(), new_path.as_path(), true, false)?;
    // A worktree administered since the locks were taken has no lock yet; the new tree holds the same locks before the exchange.
    acquire_all(root.as_path(), token.as_str())?;
    acquire_all(new_path.as_path(), token.as_str())?;
    run.journal.mark("exchange", "started")?;
    run.checkpoint("before-exchange")?;
    run.platform.exchange(root.as_path(), new_path.as_path())?;
    run.checkpoint("after-exchange")?;
    MigrateError::from_io_result(std::fs::rename(&new_path, &old_path), "rename", new_path.as_path())?;
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
fn acquire_all(tree: &Path, token: &str) -> Result<(), MigrateError> {
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
        run.line(format!("removed {released} index lock(s) from {}", run.layout.old_path.display()).as_str());
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
fn finish(run: &mut Run<'_>, options: MigrateOptions) -> Result<(), MigrateError> {
    let root: PathBuf = run.layout.root.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    drop_old_locks(run)?;
    let token: String = token_of(&run.journal);
    if !run.journal.is_done("catchup") {
        run.pass("catch-up (files newer in the old tree)", old_path.as_path(), root.as_path(), false, true)?;
        run.journal.mark("catchup", "done")?;
    }
    run.checkpoint("after-catchup")?;
    if options.refresh_indexes && !run.journal.is_done("indexes") {
        let started: Instant = Instant::now();
        let mut lines: Vec<String> = Vec::new();
        // The refresh reports each index into the list the lines are collected in; they are said even when the refresh
        //  stops, because each of them is a fact about an index that was looked at.
        let refreshed: Result<usize, MigrateError> = refresh_all_indexes(root.as_path(), IndexLock::Held(token.as_str().into()), std::slice::from_ref(&old_path), &mut lines);
        for text in &lines {
            run.line(text.as_str());
        }
        let count: usize = refreshed?;
        run.line(format!("index stat rewrite: {count} index file(s), {:.2} s", started.elapsed().as_secs_f64()).as_str());
        run.journal.mark("indexes", "done")?;
    }
    run.checkpoint("after-indexes")?;
    // The new checkout was locked from the exchange until now; Git commands may use it again.
    release_all(root.as_path(), token.as_str())?;
    if !is_subvolume(run.platform, root.as_path()) {
        return Err(MigrateError::Journal(format!("{} is not a subvolume root after the exchange; run with --rollback to restore the original tree", root.display())));
    }
    if !root.join(".git").join("HEAD").exists() {
        return Err(MigrateError::Journal(format!("{} has no .git/HEAD after the exchange; run with --rollback to restore the original tree", root.display())));
    }
    // A journal that somehow reached verification with no installed identity gets one now; the first record is the install's,
    // and a later step never blesses a directory that took the checkout's place.
    record_installed(run);
    return run.journal.mark("verify", "done");
}

/// What:
///  The closing text of a finished migration.
/// Why:
///   It states what is left to do and every way to undo or finish:
/// restart sessions,
/// check Git,
/// delete the old tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// closing(run);
/// ```
fn closing(run: &mut Run<'_>) {
    let layout: &Layout = run.layout;
    let (root, old_path) = (layout.root.display().to_string(), layout.old_path.display().to_string());
    run.line(format!("{root} is now a btrfs subvolume (root inode 256).").as_str());
    run.line(format!("The original tree is kept at {old_path}. Check the result with `git status` and `git fsck`, then delete it with `git monochromatic-git-policy-cli migrate-to-subvolume --remove-old` (or undo everything with `--rollback`).").as_str());
    run.line("Shells, editors and agent sessions that had their working directory inside the checkout still use the old tree; restart them, or run `cd \"$PWD\"` in a shell.");
    if !run.unreadable.is_empty() {
        let count: usize = run.unreadable.len();
        let list: String = run.layout.unreadable_path.display().to_string();
        run.line(format!("{count} file(s) this user may not read were not copied and exist only in the old tree; they are listed in {list}. Copy them with the needed privilege before deleting the old tree.").as_str());
    }
}

/// What:
///  Run the whole migration command:
/// preflight,
/// preview or steps.
///  `say` receives progress lines as they happen.
/// Why:
///   A refusal returns before anything is created.
/// A finished migration or a plain subvolume with no journal returns
///  `AlreadyDone`.
/// Any failure after the first step leaves the journal so a rerun resumes,
/// and the error says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = await migrate({ platform, root, options, interrupt, say });
/// ```
pub fn migrate(
    platform: &dyn Platform,
    root: &Path,
    options: MigrateOptions,
    interrupt: &Interrupt,
    say: &mut dyn FnMut(&str),
) -> Result<Verdict, MigrateError> {
    let Some(layout) = layout_of(root) else {
        return Err(MigrateError::Journal(format!("{} has no parent directory to hold the new subvolume", root.display())));
    };
    let mut run: Run<'_> = Run::open(platform, &layout, interrupt, say)?;
    if is_subvolume(platform, root) && (run.journal.is_done("verify") || !run.journal.is_persisted()) {
        run.line(format!("{} is already a btrfs subvolume; nothing to do.", root.display()).as_str());
        if run.journal.is_done("verify") && layout.old_path.exists() {
            run.line(format!("The original tree is still at {}; delete it with `git monochromatic-git-policy-cli migrate-to-subvolume --remove-old`.", layout.old_path.display()).as_str());
        }
        return Ok(Verdict::AlreadyDone);
    }
    let refusals: Vec<Refusal> = migrate_refusals(platform, &layout, &run.journal);
    if !refusals.is_empty() {
        return Ok(Verdict::Refused(refusals));
    }
    // The accepted risk of another program changing the same files is said once, before the first step that changes the
    // checkout; a refusal above and a preview change nothing, so neither says it.
    if !options.dry_run {
        run.line(MIGRATE_RISK_NOTICE);
    }
    announce(&mut run, options);
    if options.dry_run {
        run.line("plan: create the new subvolume and the nested ones; copy with cloned data; repeat as a delta; hold .git/index.lock, run the last delta, exchange the directories atomically; carry over newer files; rewrite the index stat fields; verify.");
        run.line("dry run complete; nothing was changed. Run the command without --dry-run to migrate.");
        return Ok(Verdict::DryRun);
    }
    let started: Instant = Instant::now();
    create_subvolumes(&mut run, options)?;
    if !run.journal.is_done("copy") {
        run.journal.mark("copy", "started")?;
        run.pass("copy", layout.root.as_path(), layout.new_path.as_path(), false, false)?;
        run.checkpoint("after-copy")?;
        run.journal.mark("copy", "done")?;
    }
    if !run.journal.is_done("delta1") && !run.journal.has_begun("exchange") {
        run.pass("first delta", layout.root.as_path(), layout.new_path.as_path(), true, false)?;
        run.journal.mark("delta1", "done")?;
    }
    run.checkpoint("after-delta1")?;
    exchange_trees(&mut run)?;
    finish(&mut run, options)?;
    run.line(format!("migration took {:.2} s", started.elapsed().as_secs_f64()).as_str());
    closing(&mut run);
    return Ok(Verdict::Migrated);
}

/// Migration controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_migrate_tests.rs"]
mod tests;

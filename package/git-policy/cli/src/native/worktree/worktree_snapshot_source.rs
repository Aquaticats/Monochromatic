//! What:
//!  Whether the creating worktree can be the source of a snapshot for a new worktree,
//! decided before real Git runs,
//! and
//!  everything the snapshot needs to know about it.
//! Why:
//!  A snapshot copies the whole subvolume,
//! so the source must be a subvolume root on the same filesystem as the
//!  destination,
//! clean,
//! and free of every feature whose result would differ from Git's checkout:
//! a sparse checkout,
//! a
//!  worktree-specific configuration,
//! a submodule,
//! and a nested linked worktree that is an ordinary directory (a
//!  snapshot carries its files,
//! and removing them one by one costs more than the checkout saves).
//! Each refusal returns
//!  its reason in words;
//! the caller logs it and falls to the next rung without a word to the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = await chooseSnapshotSource({ context, identity, plan }); // throws the reason it cannot be one
//! ```

use crate::add_timing::{
    begin_work,
    end_work,
};
use crate::config_snapshot::ConfigSnapshot;
use crate::diagnostic_log::debug;
use crate::fs_kind::is_subvolume_root;
use crate::lazy_runtime::run_on_runtime;
use crate::transaction_git::GitContext;
use crate::worktree_clean::{
    CleanRefusal,
    CleanSource,
    check_clean_awaiting,
    dirty_source,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_identity::WorktreeIdentity;
pub use crate::worktree_snapshot_source_changed_paths::{
    ChangedPaths,
    changed_paths,
    plain,
};
use crate::worktree_snapshot_source_cheap_refusals::worktree_of;
#[cfg(test)]
pub(crate) use crate::worktree_snapshot_source_cheap_refusals::{
    CloneRungQuestions,
    clone_refusal_asked,
};
pub use crate::worktree_snapshot_source_cheap_refusals::{
    clone_refusal,
    snapshot_refusal,
};
use crate::worktree_snapshot_source_reads::{
    clean_check_of_source,
    configuration_refusal,
    read_configuration_timed,
    read_configuration_timed_awaiting,
};
pub use crate::worktree_snapshot_source_refusal::{
    Refusal,
    RefusalKind,
};
use crate::worktree_snapshot_source_refusal::{
    status_failed,
    unclean,
};
use crate::worktree_trash_ignore_rule::{
    trash_location_is_ignored,
    trash_location_is_ignored_awaiting,
};
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

const TAG: &str = "worktreeSnapshotSource";

/// What:
///  The source a snapshot is taken from and what the later steps need to know about it.
/// Why:
///   The checks ran once before Git;
/// the steps after Git use their results instead of asking again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotSource = { root: string; gitDir: string; commonDir: string; clean: CleanSource; nested: string[]; config: ConfigSnapshot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotSource {
    /// The subvolume root of the creating worktree.
    pub root: PathBuf,
    /// Its Git directory:
    /// its own for a linked worktree,
    /// the common one for a main checkout.
    pub git_dir: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// What its `HEAD` names,
    /// proven clean.
    pub clean: CleanSource,
    /// Registered linked worktrees whose roots lie inside the source;
    /// the snapshot holds an empty stub for each.
    pub nested: Vec<PathBuf>,
    /// The configuration the checks read,
    /// which names the abbreviation length of the line Git's checkout prints.
    pub config: ConfigSnapshot,
    /// Whether Git ignores the place a snapshot sets its left-out directories aside;
    /// false when it does not,
    ///  and then the leave-out removes in the foreground.
    pub trash_ignored: bool,
    /// The uncommitted changes of a source that is not clean,
    /// when it is snapshotted anyway and restored to its `HEAD`;
    ///  nothing for a clean source.
    pub dirty: Option<ChangedPaths>,
}

/// What:
///  The roots of the linked worktrees registered in a repository,
/// read from their `gitdir` files.
/// Why:
///   Git lists the same paths;
/// reading them here costs no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function registeredRoots(commonDir: string): Promise<string[]>;
/// ```
pub fn registered_roots(common_dir: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else {
        return Vec::new();
    };
    let mut roots: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let Ok(recorded) = std::fs::read(entry.path().join("gitdir")) else {
            continue;
        };
        let line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
        let pointer: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(line));
        if let Some(root) = pointer.parent() {
            roots.push(root.to_path_buf());
        }
    }
    return roots;
}

/// What:
///  The registered worktrees inside the source that are not empty stubs,
/// if any.
/// Why:
///   A nested worktree that is a subvolume appears in a snapshot as an empty directory;
/// an ordinary directory would be
///  carried with all its files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedOrdinaryWorktree(nested: string[]): string | undefined;
/// ```
fn nested_ordinary_worktree(nested: &[PathBuf]) -> Option<PathBuf> {
    for path in nested {
        if path.exists() && !is_subvolume_root(path.as_path()).unwrap_or(false) {
            return Some(path.clone());
        }
    }
    return None;
}

/// What:
///  The registered worktrees inside the source whose recorded path is not their real path,
/// if any.
/// Why:
///   A recorded path reached through a link names a directory the snapshot carries as a link,
/// not as a stub;
/// the
///  snapshot rung leaves such a source to the next rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedThroughLink(nested: string[]): string | undefined;
/// ```
fn nested_through_link(nested: &[PathBuf]) -> Option<PathBuf> {
    for path in nested {
        if path.exists() && std::fs::canonicalize(path.as_path()).ok().as_ref() != Some(path) {
            return Some(path.clone());
        }
    }
    return None;
}

/// What:
///  Whether the place left-out directories are set aside is ignored in this source,
/// for every kind of source,
///  timed as the work `trashIgnored`.
/// Why:
///   The answer decides whether another command's stage container is renamed into a trash in one step
/// or removed in the
///  foreground,
/// and a linked worktree has the same ignore rules as the checkout it was made from.
/// The
///  repository's own exclude line is written before the question is asked,
/// so the answer does not depend on what the
///  target commit ignores:
/// an add whose target commit ignores nothing under `.claude/` still gets a trash it can set aside
///  at once,
/// and the standing trash is still unstageable (`worktree_trash_exclude_line.rs`).
/// A line that cannot be
///  written leaves the question to the ignore rules the repository already has,
/// and a `false` answer removes the directories
///  in the foreground,
/// which is the safe shape the person already had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); ensureTrashIsExcluded(commonDir); const ignored = trashLocationIsIgnored(context, root); endWork('trashIgnored', began);
/// ```
fn trash_location_ignored_timed(
    (context, root, _git_dir, common_dir): (&GitContext, &Path, &Path, &Path)
) -> bool {
    let began: std::time::Instant = begin_work();
    let ignored: bool = ignore_line_written(common_dir) && trash_location_is_ignored(context, root);
    end_work("trashIgnored", began);
    return ignored;
}

/// What:
///  Write the repository's exclude line for the trash place,
/// and say whether it is there.
/// Why:
///   The question that follows decides whether a trash is made at all,
/// so a repository whose exclude file cannot be written
///  keeps the foreground removal rather than a trash nothing keeps out of the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignoreLineWritten = (commonDir) => { try { ensureTrashIsExcluded(commonDir); return true; } catch (reason) { debug(reason); return false; } };
/// ```
fn ignore_line_written(common_dir: &Path) -> bool {
    return match crate::worktree_trash_exclude_line::ensure_trash_is_excluded(common_dir) {
        Ok(_added) => true,
        Err(reason) => {
            debug(TAG, reason.as_str());
            false
        }
    };
}

/// What:
///  `trash_location_ignored_timed` with the Git child awaited on the runtime.
/// Why:
///   The question is one of the three the source check asks on a task of the runtime,
/// beside the configuration read and the
///  cleanliness check;
/// the run's bound and quiet window are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork(); const ignored = await trashLocationIsIgnoredAwaiting(context, root); endWork('trashIgnored', began);
/// ```
async fn trash_location_ignored_timed_awaiting(
    (context, root, _git_dir, common_dir): (&GitContext, &Path, &Path, &Path)
) -> bool {
    let began: std::time::Instant = begin_work();
    // The write is a few bytes of one file, small enough to do on this task rather than as a job of the pool, and the
    //  question below must not be asked before it.
    let ignored: bool =
        ignore_line_written(common_dir) && trash_location_is_ignored_awaiting(context, root).await;
    end_work("trashIgnored", began);
    return ignored;
}

/// What:
///  Check the creating worktree for everything a copy of its files needs,
/// from the cheapest check to the dearest,
/// and
///  collect what the later steps use;
/// the refusal is the error.
/// Why:
///   The dear checks (the clean check starts one Git run) happen once for both rungs,
/// and only after a cheap check let a
///  rung through.
/// The kind of the refusal lets the ladder tell the person the cause in fitting words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceRefusal(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a Refusal
/// ```
pub fn source_refusal(
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Result<SnapshotSource, Refusal> {
    return checked_source(context, identity, false);
}

/// What:
///  `source_refusal`,
/// except that a main checkout with only ordinary uncommitted changes is a source too,
/// carrying the list of
///  its changes;
/// every other refusal stays.
/// Why:
///   The ladder snapshots such a source and restores the copy to the new worktree's `HEAD`;
/// the clone rung takes the same source and clones the paths its proof settles while Git writes the rest,
/// and the other
///  callers of the clean answer see clean sources only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceOrDirty(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a Refusal
/// ```
pub fn source_or_dirty(
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Result<SnapshotSource, Refusal> {
    return checked_source(context, identity, true);
}

/// What the trash-location read and the cleanliness check found.
type TrashAndCleanVerdicts = (bool, Result<CleanSource, CleanRefusal>);

/// What:
///  `read whether the trash location is ignored` and `check the cleanliness of the source` on a task of the runtime:
/// the trash question awaited
///  through its child and the cleanliness check awaited through its own two waits.
/// Why:
///   The two read different things and write nothing;
/// the verdicts are read in the order the checks have always had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [ignored, checked] = await Promise.all([trashLocationIgnored(args), cleanCheckOfSource(args)]);
/// ```
async fn check_trash_and_cleanliness_awaiting(
    (context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)
) -> TrashAndCleanVerdicts {
    return tokio::join!(
        trash_location_ignored_timed_awaiting((context, root, git_dir, common_dir)),
        check_clean_awaiting(context, root, git_dir, common_dir)
    );
}

/// What:
///  The three checks the source needs before it can be copied:
/// the configuration read,
/// the ignore rule of the trash
///  location and the cleanliness of the files,
/// each a wait of this task where Git is asked.
/// Why:
///   The three read different things and write nothing,
/// so they run beside each other;
/// the filesystem proof inside the
///  cleanliness check is a job of the blocking pool (`worktree_clean.rs`),
/// so no walk of the source holds the Git waits open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [config, [ignored, checked]] = await Promise.all([readConfiguration(args), checkTrashAndCleanliness(args)]);
/// ```
async fn the_checks_of_the_source(
    state: (&GitContext, &Path, &Path, &Path)
) -> (Option<ConfigSnapshot>, TrashAndCleanVerdicts) {
    return tokio::join!(
        read_configuration_timed_awaiting((state.0, state.1)),
        check_trash_and_cleanliness_awaiting(state)
    );
}

/// What:
///  `the_checks_of_the_source` one check after another on the calling thread,
/// for the state where no runtime can be built.
/// Why:
///   The owner's rule is "Commits should never fail.",
/// so an add whose checks cannot share the runtime still gets the same three
///  answers and the same refusals;
/// nothing runs beside anything,
/// which is the only difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const config = readConfiguration(args); const ignored = trashLocationIgnored(args); const checked = checkClean(args);
/// ```
fn the_checks_of_the_source_in_turn(
    (context, root, git_dir, common_dir): (&GitContext, &Path, &Path, &Path)
) -> (Option<ConfigSnapshot>, TrashAndCleanVerdicts) {
    let read_config: Option<ConfigSnapshot> = read_configuration_timed((context, root));
    let trash_ignored: bool = trash_location_ignored_timed((context, root, git_dir, common_dir));
    let checked: Result<CleanSource, CleanRefusal> =
        clean_check_of_source((context, root, git_dir, common_dir));
    return (read_config, (trash_ignored, checked));
}

/// The checks of `source_refusal`,
/// and of `source_or_dirty` when `allow_dirty` is set.
fn checked_source(
    context: &GitContext,
    identity: &WorktreeIdentity,
    allow_dirty: bool,
) -> Result<SnapshotSource, Refusal> {
    let (root, git_dir, common_dir): (&PathBuf, &PathBuf, &PathBuf) =
        worktree_of(identity).map_err(Refusal::unsupported)?;
    if git_dir.join("info/sparse-checkout").exists()
        || common_dir.join("info/sparse-checkout").exists()
    {
        return Err(Refusal::unsupported("the source has a sparse checkout"));
    }
    let mut nested: Vec<PathBuf> = Vec::new();
    for path in registered_roots(common_dir.as_path()) {
        // A record that is inside the source by its text is kept as written, so a link inside the source is still caught.
        if path.starts_with(root) {
            if path != *root {
                nested.push(path);
            }
            continue;
        }
        // A record written before the repository moved behind a link spells the old name and names a worktree inside the source.
        let real: PathBuf = std::fs::canonicalize(path.as_path()).unwrap_or(path);
        if real != *root && real.starts_with(root) {
            nested.push(real);
        }
    }
    // The configuration, the ignore rule of the place left-out directories are set aside at and the cleanliness of the files
    //  read different things and write nothing, so they run side by side on the process's runtime, or one after another on this
    //  thread where no runtime can be built; the verdicts are read in the order the checks have always had.
    let state: (&GitContext, &Path, &Path, &Path) = (
        context,
        root.as_path(),
        git_dir.as_path(),
        common_dir.as_path(),
    );
    let (read_config, (trash_ignored, checked)): (Option<ConfigSnapshot>, TrashAndCleanVerdicts) =
        match run_on_runtime(the_checks_of_the_source(state)) {
            Ok(verdicts) => verdicts,
            Err(refusal) => {
                crate::diagnostic_log::debug(
                    "checkedSource",
                    format!("the checks of the source run one after another ({refusal})").as_str(),
                );
                the_checks_of_the_source_in_turn(state)
            }
        };
    let Some(snapshot) = read_config else {
        return Err(Refusal::unsupported(
            "the Git configuration could not be read",
        ));
    };
    if let Some(reason) = configuration_refusal(snapshot.entries()) {
        return Err(Refusal::unsupported(reason));
    }
    let (clean, dirty): (CleanSource, Option<ChangedPaths>) = match checked {
        Ok(found) => (found, None),
        Err(why) if allow_dirty && restorable_here(identity) && why.kind.restorable() => {
            let found =
                dirty_source(git_dir.as_path(), common_dir.as_path(), why).map_err(unclean)?;
            // A source whose files Git finds equal to the commit is snapshotted too: the stat data was all that kept it from being clean.
            let changed: ChangedPaths =
                changed_paths(context, root.as_path()).map_err(status_failed)?;
            if changed.attributes_changed {
                return Err(Refusal::unsupported(
                    "a .gitattributes file is among the uncommitted changes, and its rules could change how the restore writes other files",
                ));
            }
            crate::diagnostic_log::debug("sourceOrDirty", format!("{} is snapshotted with {} uncommitted paths, the first {:?}; the restore follows ({})", root.display(), changed.count, changed.first, found.why).as_str());
            (found.source, Some(changed))
        }
        Err(why) => return Err(unclean(why)),
    };
    return Ok(SnapshotSource {
        root: root.clone(),
        git_dir: git_dir.clone(),
        common_dir: common_dir.clone(),
        clean,
        nested,
        config: snapshot,
        trash_ignored,
        dirty,
    });
}

/// What:
///  Whether a source that is not clean can be copied and restored:
/// the main checkout,
/// whose index lies inside the snapshot.
/// Why:
///   The restore starts from the index the snapshot carries,
/// which is one instant with the files;
/// a linked worktree's index
///  lies outside its directory and is not in the snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restorableHere(identity: WorktreeIdentity): boolean;
/// ```
fn restorable_here(identity: &WorktreeIdentity) -> bool {
    return matches!(identity, WorktreeIdentity::MainWorktree { worktree_root, git_dir, .. } if *git_dir == worktree_root.join(".git"));
}

/// What:
///  Check the creating worktree for everything a copy of its files needs;
/// the reason is the error.
/// Why:
///   Callers that only log or compare the reason ask for the words;
/// the ladder asks `source_refusal` for the kind too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commonSource(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn common_source(
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Result<SnapshotSource, String> {
    return match source_refusal(context, identity) {
        Ok(source) => Ok(source),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What:
///  The reason a nested worktree that is an ordinary directory blocks the snapshot.
/// Why:
///   The person is told which worktree to convert and with what.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the worktree ${path} inside the source is an ordinary directory; ...`
/// ```
fn ordinary_worktree_reason(path: PathBuf) -> String {
    return format!(
        "the worktree {} inside the source is an ordinary directory; `git monochromatic-git-policy-cli migrate-to-subvolume` makes it a subvolume",
        path.display()
    );
}

/// What:
///  The registered worktree inside the source that would be carried into a snapshot with all its files,
/// if any.
/// Why:
///   Only the snapshot rung cares:
/// a nested worktree that is a subvolume is an empty stub in a snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotBlocker(source: SnapshotSource): string | undefined;
/// ```
pub fn snapshot_blocker(source: &SnapshotSource) -> Option<String> {
    if let Some(path) = nested_through_link(source.nested.as_slice()) {
        return Some(format!(
            "the worktree {} inside the source is reached through a symbolic link",
            path.display()
        ));
    }
    // A snapshot of the main checkout sets the nested ordinary worktrees aside (`worktree_trash.rs`), so they do not block it.
    if source.git_dir == source.common_dir {
        return None;
    }
    return nested_ordinary_worktree(source.nested.as_slice()).map(ordinary_worktree_reason);
}

/// What:
///  Decide whether the creating worktree of a planned add can be the snapshot source:
/// the cheap refusals,
/// the shared
///  checks,
/// then what only a snapshot minds.
/// Why:
///   The ladder asks the three in turn so the clone rung can share the dear checks;
/// this is the whole question for the
///  snapshot alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function chooseSnapshotSource(args: { context: GitContext; identity: WorktreeIdentity; plan: AddPlan }): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn choose_snapshot_source(
    context: &GitContext,
    identity: &WorktreeIdentity,
    plan: &AddPlan,
) -> Result<SnapshotSource, String> {
    if let Some(reason) = snapshot_refusal(identity, plan) {
        return Err(reason);
    }
    let source: SnapshotSource = common_source(context, identity)?;
    return match snapshot_blocker(&source) {
        Some(reason) => Err(reason),
        None => Ok(source),
    };
}

/// Source selection controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_source_tests.rs"]
mod tests;

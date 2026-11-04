//! What:
//!  Whether the creating worktree can be the source of a snapshot for a new worktree, decided before real Git runs, and
//!  everything the snapshot needs to know about it.
//! Why:
//!  A snapshot copies the whole subvolume, so the source must be a subvolume root on the same filesystem as the
//!  destination, clean, and free of every feature whose result would differ from Git's checkout: a sparse checkout, a
//!  worktree-specific configuration, a submodule, and a nested linked worktree that is an ordinary directory (a
//!  snapshot carries its files, and removing them one by one costs more than the checkout saves). Each refusal returns
//!  its reason in words; the caller logs it and falls to the next rung without a word to the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = await chooseSnapshotSource({ context, identity, plan }); // throws the reason it cannot be one
//! ```

use crate::scoped_start::start_or_run;
use crate::config_snapshot::{ConfigEntry, ConfigSnapshot, read_config_snapshot};
use crate::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use crate::fs_mount_options::read_mount_facts;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clean::{CleanRefusal, CleanRefusalKind, CleanSource, check_clean, dirty_source};
use crate::worktree_identity::WorktreeIdentity;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What:
///  The source a snapshot is taken from and what the later steps need to know about it.
/// Why:
///   The checks ran once before Git; the steps after Git use their results instead of asking again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotSource = { root: string; gitDir: string; commonDir: string; clean: CleanSource; nested: string[]; config: ConfigSnapshot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotSource {
    /// The subvolume root of the creating worktree.
    pub root: PathBuf,
    /// Its Git directory: its own for a linked worktree, the common one for a main checkout.
    pub git_dir: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// What its `HEAD` names, proven clean.
    pub clean: CleanSource,
    /// Registered linked worktrees whose roots lie inside the source; the snapshot holds an empty stub for each.
    pub nested: Vec<PathBuf>,
    /// The configuration the checks read, which names the abbreviation length of the line Git's checkout prints.
    pub config: ConfigSnapshot,
    /// Whether Git ignores the place a snapshot of the main checkout sets its left-out directories aside; false for a linked
    ///  worktree, which sets nothing aside.
    pub trash_ignored: bool,
    /// The uncommitted changes of a source that is not clean, when it is snapshotted anyway and restored to its `HEAD`;
    ///  nothing for a clean source.
    pub dirty: Option<ChangedPaths>,
}

/// What:
///  What Git's status lists as changed or untracked in a worktree.
/// Why:
///   The person is told how many paths and one name; the restore refuses a changed `.gitattributes` file, whose rules could
///  change how the restore writes other files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChangedPaths = { count: number; first?: string; attributesChanged: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedPaths {
    /// How many paths `git status` lists; none when Git finds every file equal to its commit by content although the stat data
    ///  of the index cannot vouch for it (files as new as the index, or touched without a change).
    pub count: usize,
    /// The first of them, with control characters shown as `?`; nothing when `count` is zero.
    pub first: Option<String>,
    /// Whether a `.gitattributes` file is changed or untracked.
    pub attributes_changed: bool,
}

/// What:
///  The text with every control character replaced by `?`.
/// Why:
///   A path in a sentence on a terminal or in the journal must not carry an escape sequence or start a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/\p{Cc}/gu, '?')
/// ```
pub fn plain(text: &str) -> String {
    return text.chars().map(|character: char| { return if character.is_control() { '?' } else { character }; }).collect();
}

impl ChangedPaths {
    /// Whether Git lists any change; false for a worktree whose files equal its commit by content.
    pub fn has_changes(&self) -> bool {
        return self.count > 0;
    }
}

/// What:
///  The paths `git status` lists as changed or untracked in the worktree at `root`, counted, with the first, and whether a
///  `.gitattributes` file is among them (also one inside an untracked directory); or the words that say which Git run failed
///  and how.
/// Why:
///   Only Git's own status tells a real change from a stale record of file times. The read takes no optional lock, so it never
///  disturbs the checkout a person works in. A listing with nothing in it is an answer (every file equals its commit by content),
///  never a failure: the stat data of an index that was just written, or of a file that was touched, proves nothing, and Git
///  settles those by content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = await changedPaths(context, root); // throws the reason Git could not say
/// ```
pub fn changed_paths(context: &GitContext, root: &Path) -> Result<ChangedPaths, String> {
    let ask = |arguments: &[&str]| -> Result<Vec<u8>, String> {
        let words: String = arguments.iter().skip(1).copied().take(1).collect::<Vec<&str>>().join(" ");
        let request: GitRequest = GitRequest { cwd: root.to_path_buf(), arguments: arguments.iter().map(|word: &&str| { return (*word).into(); }).collect(), without_prefix: true, ..GitRequest::default() };
        return match run_git(context, &request) {
            Ok(found) if found.code == Some(0) => Ok(found.stdout),
            Ok(found) => {
                let said: String = String::from_utf8_lossy(found.stderr.as_slice()).trim().to_string();
                let code: String = found.code.map_or_else(|| { return String::from("was stopped by a signal"); }, |number: i32| { return format!("ended with exit code {number}"); });
                crate::diagnostic_log::debug("changedPaths", format!("Git could not list the changes (`git {words}` {code}: {said})").as_str());
                Err(if said.is_empty() { format!("`git {words}` {code} and said nothing") } else { format!("`git {words}` {code}: {}", plain(said.as_str())) })
            }
            Err(error) => {
                crate::diagnostic_log::debug("changedPaths", format!("Git could not list the changes ({error})").as_str());
                Err(format!("`git {words}` could not be started ({error})"))
            }
        };
    };
    let listing: Vec<u8> = ask(&["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=normal", "--no-renames"])?;
    let untracked_attributes: Vec<u8> = ask(&["--no-optional-locks", "ls-files", "--others", "--exclude-standard", "-z", "--", ":(glob)**/.gitattributes"])?;
    // Each status entry is two status letters, a space, then the path.
    let paths: Vec<&[u8]> = listing.split(|byte: &u8| { return *byte == 0; }).filter(|entry: &&[u8]| { return !entry.is_empty(); }).filter_map(|entry: &[u8]| { return entry.get(3..); }).collect();
    let attributes_changed: bool = !untracked_attributes.is_empty() || paths.iter().any(|path: &&[u8]| { return *path == b".gitattributes" || path.ends_with(b"/.gitattributes"); });
    return Ok(ChangedPaths { count: paths.len(), first: paths.first().map(|path: &&[u8]| { return plain(String::from_utf8_lossy(path).as_ref()); }), attributes_changed });
}

/// What:
///  The roots of the linked worktrees registered in a repository, read from their `gitdir` files.
/// Why:
///   Git lists the same paths; reading them here costs no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function registeredRoots(commonDir: string): Promise<string[]>;
/// ```
pub fn registered_roots(common_dir: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else { return Vec::new() };
    let mut roots: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let Ok(recorded) = std::fs::read(entry.path().join("gitdir")) else { continue };
        let line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
        let pointer: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(line));
        if let Some(root) = pointer.parent() {
            roots.push(root.to_path_buf());
        }
    }
    return roots;
}

/// The configuration key that turns a sparse checkout on; Git copies the patterns into a new worktree, a snapshot cannot.
const SPARSE_KEY: &str = "core.sparsecheckout";

/// What:
///  The reason the configuration rules the source out, or nothing.
/// Why:
///   Git copies the filtered worktree configuration and the sparse-checkout patterns into a new worktree, and conversion
///  settings that live only in a worktree would disagree between the two worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configurationRefusal(entries: ConfigEntry[]): string | undefined;
/// ```
fn configuration_refusal(entries: &[ConfigEntry]) -> Option<String> {
    for entry in entries {
        if entry.key == SPARSE_KEY {
            return Some(String::from("core.sparseCheckout is set"));
        }
        if entry.scope == "worktree" {
            return Some(format!("the worktree-specific configuration sets {}", entry.key));
        }
    }
    return None;
}

/// What:
///  Whether the mounts that hold two paths are one btrfs filesystem.
/// Why:
///   A snapshot and a clone cross subvolumes of one filesystem only; the mount table names the device, which is the
///  same for every subvolume of a filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onOneBtrfs(first: string, second: string): boolean;
/// ```
fn on_one_btrfs(first: &Path, second: &Path) -> bool {
    if filesystem_kind(first) != Ok(FilesystemKind::Btrfs) || filesystem_kind(second) != Ok(FilesystemKind::Btrfs) {
        return false;
    }
    let (Ok(Some(one)), Ok(Some(other))) = (read_mount_facts(first), read_mount_facts(second)) else { return false };
    return one.source == other.source;
}

/// What:
///  The registered worktrees inside the source that are not empty stubs, if any.
/// Why:
///   A nested worktree that is a subvolume appears in a snapshot as an empty directory; an ordinary directory would be
///  carried with all its files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedOrdinaryWorktree(nested: string[]): string | undefined;
/// ```
fn nested_ordinary_worktree(nested: &[PathBuf]) -> Option<PathBuf> {
    return nested
        .iter()
        .find(|path: &&PathBuf| { return path.exists() && !is_subvolume_root(path.as_path()).unwrap_or(false); })
        .cloned();
}

/// What:
///  The registered worktrees inside the source whose recorded path is not their real path, if any.
/// Why:
///   A recorded path reached through a link names a directory the snapshot carries as a link, not as a stub; the
///  snapshot rung leaves such a source to the next rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedThroughLink(nested: string[]): string | undefined;
/// ```
fn nested_through_link(nested: &[PathBuf]) -> Option<PathBuf> {
    return nested
        .iter()
        .find(|path: &&PathBuf| { return path.exists() && std::fs::canonicalize(path.as_path()).ok().as_ref() != Some(*path); })
        .cloned();
}

/// What:
///  The root, Git directory and common directory of the creating worktree, or why there is none.
/// Why:
///   A bare repository and a location outside any worktree have no files to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeOf(identity: WorktreeIdentity): { root: string; gitDir: string; commonDir: string }; // throws a reason
/// ```
fn worktree_of(identity: &WorktreeIdentity) -> Result<(&PathBuf, &PathBuf, &PathBuf), String> {
    return match identity {
        WorktreeIdentity::MainWorktree { worktree_root, git_dir, common_dir } | WorktreeIdentity::LinkedWorktree { worktree_root, git_dir, common_dir } => Ok((worktree_root, git_dir, common_dir)),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => Err(String::from("there is no worktree to copy from")),
    };
}

/// What:
///  The cheap reasons the snapshot rung cannot serve a planned add: the source is no subvolume root, or it and the
///  destination are not on one btrfs filesystem.
/// Why:
///   These need two `statfs` calls and one inode number, so they run before the dear checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotRefusal(identity: WorktreeIdentity, plan: AddPlan): string | undefined;
/// ```
pub fn snapshot_refusal(identity: &WorktreeIdentity, plan: &AddPlan) -> Option<String> {
    let (root, _git_dir, _common_dir): (&PathBuf, &PathBuf, &PathBuf) = match worktree_of(identity) {
        Ok(found) => found,
        Err(reason) => return Some(reason),
    };
    let Some(parent) = plan.destination.parent() else { return Some(String::from("the destination has no parent directory")) };
    if !is_subvolume_root(root.as_path()).unwrap_or(false) {
        return Some(format!("{} is not a btrfs subvolume", root.display()));
    }
    if !on_one_btrfs(root.as_path(), parent) {
        return Some(format!("{} and {} are not on one btrfs filesystem", root.display(), parent.display()));
    }
    return None;
}

/// What:
///  Whether the filesystem of `path` may share data blocks between files: btrfs, XFS (reflink is a per-filesystem
///  feature that the first clone reveals) and overlayfs (a clone inside the overlay reaches the layers' filesystem).
/// Why:
///   Other filesystems never clone, so the clone rung is not tried there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayClone(path: string): boolean;
/// ```
fn may_clone(path: &Path) -> bool {
    if matches!(filesystem_kind(path), Ok(FilesystemKind::Btrfs | FilesystemKind::Overlayfs)) {
        return true;
    }
    return crate::fs_kind::is_xfs(path).unwrap_or(false);
}

/// What:
///  The cheap reasons the clone rung cannot serve a planned add: either side is on a filesystem that never clones, or the
///  two are not in one clone domain (one btrfs filesystem, or one device elsewhere).
/// Why:
///   A clone crosses subvolumes of one btrfs filesystem and nothing else; an overlay or XFS is one device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneRefusal(identity: WorktreeIdentity, plan: AddPlan): string | undefined;
/// ```
pub fn clone_refusal(identity: &WorktreeIdentity, plan: &AddPlan) -> Option<String> {
    let (root, _git_dir, _common_dir): (&PathBuf, &PathBuf, &PathBuf) = match worktree_of(identity) {
        Ok(found) => found,
        Err(reason) => return Some(reason),
    };
    let Some(parent) = plan.destination.parent() else { return Some(String::from("the destination has no parent directory")) };
    if !may_clone(root.as_path()) || !may_clone(parent) {
        return Some(String::from("the filesystem does not share data blocks between files"));
    }
    let one_domain: bool = if filesystem_kind(root.as_path()) == Ok(FilesystemKind::Btrfs) {
        on_one_btrfs(root.as_path(), parent)
    } else {
        std::fs::metadata(root.as_path()).map(|found: std::fs::Metadata| { return found.dev(); }).ok() == std::fs::metadata(parent).map(|found: std::fs::Metadata| { return found.dev(); }).ok()
    };
    return if one_domain { None } else { Some(format!("{} and {} are on different filesystems", root.display(), parent.display())) };
}

/// Which kind of check refused a source, so the person can be told the cause in words that fit it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalKind {
    /// The source could not be proven clean, for the kind of reason the clean check gives.
    Unclean(CleanRefusalKind),
    /// A feature a copy of the files does not reproduce, or a state the checks cannot read.
    Unsupported,
    /// The stat data could not vouch for the files, and the Git run that settles it by content failed.
    StatusFailed,
}

/// What:
///  Why a source cannot be copied, and which kind of check said so.
/// Why:
///   The reason is for the log and for the sentence the person reads; the kind lets the sentence be chosen without reading
///  the reason's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { reason: string; kind: 'unclean' | 'unsupported' | 'status-failed' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// The reason, in words.
    pub reason: String,
    /// The kind of check that refused.
    pub kind: RefusalKind,
}

impl Refusal {
    /// A refusal by a feature or a state the checks cannot read.
    fn unsupported(reason: impl Into<String>) -> Refusal {
        return Refusal { reason: reason.into(), kind: RefusalKind::Unsupported };
    }
}

/// What:
///  Check the creating worktree for everything a copy of its files needs, from the cheapest check to the dearest, and
///  collect what the later steps use; the refusal is the error.
/// Why:
///   The dear checks (the clean check starts one Git run) happen once for both rungs, and only after a cheap check let a
///  rung through. The kind of the refusal lets the ladder tell the person the cause in fitting words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceRefusal(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a Refusal
/// ```
pub fn source_refusal(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, Refusal> {
    return checked_source(context, identity, false);
}

/// What:
///  `source_refusal`, except that a main checkout with only ordinary uncommitted changes is a source too, carrying the list of
///  its changes; every other refusal stays.
/// Why:
///   The ladder snapshots such a source and restores the copy to the new worktree's `HEAD`; the clone rung and the other
///  callers of the clean answer never see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceOrDirty(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a Refusal
/// ```
pub fn source_or_dirty(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, Refusal> {
    return checked_source(context, identity, true);
}

/// The checks of `source_refusal`, and of `source_or_dirty` when `allow_dirty` is set.
fn checked_source(context: &GitContext, identity: &WorktreeIdentity, allow_dirty: bool) -> Result<SnapshotSource, Refusal> {
    let (root, git_dir, common_dir): (&PathBuf, &PathBuf, &PathBuf) = worktree_of(identity).map_err(Refusal::unsupported)?;
    if git_dir.join("info/sparse-checkout").exists() || common_dir.join("info/sparse-checkout").exists() {
        return Err(Refusal::unsupported("the source has a sparse checkout"));
    }
    let nested: Vec<PathBuf> = registered_roots(common_dir.as_path()).into_iter().filter(|path: &PathBuf| { return path != root && path.starts_with(root); }).collect();
    // The configuration, the cleanliness of the files and (for a main checkout) the ignore rule for the place left-out directories
    //  are set aside read different things and write nothing, so they run side by side; the verdicts are read in the order the
    //  checks have always had.
    let (read_config, checked, trash_ignored): (Option<ConfigSnapshot>, Result<CleanSource, CleanRefusal>, bool) = std::thread::scope(|scope| {
        let config = start_or_run(scope, || { return crate::add_timing::work("readConfig", || { return read_config_snapshot(context, root.as_path()); }); });
        let trash = start_or_run(scope, || { return crate::add_timing::work("trashIgnored", || { return git_dir == common_dir && crate::worktree_trash::trash_location_is_ignored(context, root.as_path()); }); });
        let checked = check_clean(context, root.as_path(), git_dir.as_path(), common_dir.as_path());
        return (config.join().unwrap_or(None), checked, trash.join().unwrap_or(false));
    });
    let snapshot: ConfigSnapshot = read_config.ok_or_else(|| { return Refusal::unsupported("the Git configuration could not be read"); })?;
    if let Some(reason) = configuration_refusal(snapshot.entries()) {
        return Err(Refusal::unsupported(reason));
    }
    let unclean = |why: CleanRefusal| -> Refusal { return Refusal { reason: why.reason, kind: RefusalKind::Unclean(why.kind) }; };
    let (clean, dirty): (CleanSource, Option<ChangedPaths>) = match checked {
        Ok(found) => (found, None),
        Err(why) if allow_dirty && restorable_here(identity) && why.kind.restorable() => {
            let found = dirty_source(git_dir.as_path(), common_dir.as_path(), why).map_err(unclean)?;
            // A source whose files Git finds equal to the commit is snapshotted too: the stat data was all that kept it from being clean.
            let changed: ChangedPaths = changed_paths(context, root.as_path())
                .map_err(|reason: String| { return Refusal { reason, kind: RefusalKind::StatusFailed }; })?;
            if changed.attributes_changed {
                return Err(Refusal::unsupported("a .gitattributes file is among the uncommitted changes, and its rules could change how the restore writes other files"));
            }
            crate::diagnostic_log::debug("sourceOrDirty", format!("{} is snapshotted with {} uncommitted paths, the first {:?}; the restore follows ({})", root.display(), changed.count, changed.first, found.why).as_str());
            (found.source, Some(changed))
        }
        Err(why) => return Err(unclean(why)),
    };
    return Ok(SnapshotSource { root: root.clone(), git_dir: git_dir.clone(), common_dir: common_dir.clone(), clean, nested, config: snapshot, trash_ignored, dirty });
}

/// What:
///  Whether a source that is not clean can be copied and restored: the main checkout, whose index lies inside the snapshot.
/// Why:
///   The restore starts from the index the snapshot carries, which is one instant with the files; a linked worktree's index
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
///  Check the creating worktree for everything a copy of its files needs; the reason is the error.
/// Why:
///   Callers that only log or compare the reason ask for the words; the ladder asks `source_refusal` for the kind too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commonSource(context: GitContext, identity: WorktreeIdentity): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn common_source(context: &GitContext, identity: &WorktreeIdentity) -> Result<SnapshotSource, String> {
    return source_refusal(context, identity).map_err(|refusal: Refusal| { return refusal.reason; });
}

/// What:
///  The registered worktree inside the source that would be carried into a snapshot with all its files, if any.
/// Why:
///   Only the snapshot rung cares: a nested worktree that is a subvolume is an empty stub in a snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotBlocker(source: SnapshotSource): string | undefined;
/// ```
pub fn snapshot_blocker(source: &SnapshotSource) -> Option<String> {
    if let Some(path) = nested_through_link(source.nested.as_slice()) {
        return Some(format!("the worktree {} inside the source is reached through a symbolic link", path.display()));
    }
    // A snapshot of the main checkout sets the nested ordinary worktrees aside (`worktree_trash.rs`), so they do not block it.
    if source.git_dir == source.common_dir {
        return None;
    }
    return nested_ordinary_worktree(source.nested.as_slice()).map(|path: PathBuf| { return format!("the worktree {} inside the source is an ordinary directory; `git cli-git migrate-to-subvolume` makes it a subvolume", path.display()); });
}

/// What:
///  Decide whether the creating worktree of a planned add can be the snapshot source: the cheap refusals, the shared
///  checks, then what only a snapshot minds.
/// Why:
///   The ladder asks the three in turn so the clone rung can share the dear checks; this is the whole question for the
///  snapshot alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function chooseSnapshotSource(args: { context: GitContext; identity: WorktreeIdentity; plan: AddPlan }): Promise<SnapshotSource>; // throws a reason
/// ```
pub fn choose_snapshot_source(context: &GitContext, identity: &WorktreeIdentity, plan: &AddPlan) -> Result<SnapshotSource, String> {
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

//! What:
//!  Convert one registered linked worktree that is an ordinary directory into a btrfs subvolume at the same path,
//!  resumably: quiesce it, copy it with cloned data into `<path>.cli-git-converting`, prove the copy equals the original,
//!  swap the two with one atomic exchange, refresh the index stat fields, and remove the old tree only after the swap is
//!  verified.
//! Why:
//!  A snapshot of the main checkout shows a nested worktree that is a subvolume as an empty directory, and one that is an
//!  ordinary directory as a full copy that must then be removed (0.5 s each). Converting each ordinary worktree once makes
//!  every later snapshot skip it for nothing. The registration is untouched: the path and the bytes of the `.git` file do
//!  not change, so the administrative directory's back-pointer stays valid. A journal in the administrative directory
//!  records the phase, so a rerun completes the conversion in place; nothing is deleted that the journal does not prove
//!  this command created.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = await convertWorktree({ platform, path, options, interrupt, say });
//! ```

use crate::fs_beneath::{ObjectId, identity_at, open_root};
use crate::process_use::{UseTarget, processes_using};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_index_files::{IndexLock, RefreshOptions, RefreshStatus, refresh_index_file_report, repository_hash_length};
use crate::subvolume_lock::{acquire, is_ours};
use crate::subvolume_platform::Platform;
use crate::subvolume_sync::{SyncOptions, SyncStats, sync_tree};
use crate::worktree_clean::check_clean_tracked_by_content;
use crate::single_owner_claim::{ClaimOutcome, OwnerClaim, live_owner_of, take_claim};
use crate::worktree_convert_journal::{CLAIM_PREFIX, Journal, is_recorded_object, legacy_lock_is_held, read_plain_file, read_plain_text};
pub use crate::worktree_convert_journal::{JOURNAL_NAME, Phase};
use crate::worktree_convert_remove::{Removal, remove_recorded_old_tree};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The most bytes of a `.git` pointer or a registration file that are read.
const GIT_FILE_LIMIT: u64 = 1 << 16;

/// The suffix of the sibling subvolume the copy is built in.
pub const CONVERTING_SUFFIX: &str = ".cli-git-converting";

/// What:
///  What a run did with one worktree.
/// Why:
///   The command prints one line per worktree and maps the verdicts to its exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = 'converted' | 'resumed' | 'would-convert' | 'already-subvolume' | 'abandoned' | { skipped: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The conversion ran from the start to the end.
    Converted,
    /// An unfinished conversion was completed.
    Resumed,
    /// A dry run found nothing in the way.
    WouldConvert,
    /// The path is a subvolume already.
    AlreadySubvolume,
    /// An unfinished conversion was undone before its swap.
    Abandoned,
    /// Something stands in the way; each reason is a sentence.
    Skipped(Vec<String>),
}

/// What:
///  What the command asks of one worktree.
/// Why:
///   The dry run and the abandon are modes of the same pass over the same facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertOptions = { dryRun: boolean; abandon: boolean; commandLineAttributes: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConvertOptions {
    /// List what would happen and change nothing.
    pub dry_run: bool,
    /// Undo an unfinished conversion that has not reached its swap.
    pub abandon: bool,
    /// The invocation chose an attribute source on the command line (`--attr-source`, `-c attr.tree`), which the worktree
    ///  files read here do not model.
    pub command_line_attributes: bool,
}

/// What:
///  The facts about a worktree path that every mode needs.
/// Why:
///   They are read once, from the `.git` file and the administrative directory, without starting Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Candidate = { path: string; admin: string; common: string; gitFile: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The canonical worktree root.
    pub path: PathBuf,
    /// The worktree's administrative directory.
    pub admin: PathBuf,
    /// The common Git directory.
    pub common: PathBuf,
    /// The exact bytes of the worktree's `.git` file.
    pub git_file: Vec<u8>,
}

/// The sibling path the copy is built at.
pub fn converting_path(path: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = path.as_os_str().to_os_string();
    name.push(CONVERTING_SUFFIX);
    return PathBuf::from(name);
}

/// What:
///  Read the registration of a worktree path: its `.git` file, its administrative directory, and the back-pointer.
/// Why:
///   Only a linked worktree whose administrative directory names this very path back is converted; anything else (the main
///  checkout, a submodule, a stale registration) is refused with the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inspect(path: string): Candidate; // throws the reason
/// ```
pub fn inspect(path: &Path) -> Result<Candidate, String> {
    let real: PathBuf = std::fs::canonicalize(path).map_err(|error: std::io::Error| { return format!("{} cannot be resolved ({error})", path.display()); })?;
    let git_path: PathBuf = real.join(".git");
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(&git_path).map_err(|_| { return format!("{} has no .git entry, so it is not a worktree", real.display()); })?;
    if metadata.is_dir() {
        return Err(format!("{} holds a .git directory: it is a main checkout or a repository, not a linked worktree", real.display()));
    }
    let git_file: Vec<u8> = read_plain_file(&git_path, GIT_FILE_LIMIT).map_err(|error: std::io::Error| { return format!("{} is unreadable ({error})", git_path.display()); })?;
    let text: &str = std::str::from_utf8(git_file.as_slice()).map_err(|_| { return format!("{} is not text", git_path.display()); })?;
    let Some(named) = text.trim().strip_prefix("gitdir:") else {
        return Err(format!("{} has no gitdir line", git_path.display()));
    };
    let admin: PathBuf = std::fs::canonicalize(real.join(named.trim())).map_err(|error: std::io::Error| { return format!("the administrative directory {} cannot be resolved ({error})", named.trim()); })?;
    let recorded: Vec<u8> = read_plain_file(&admin.join("gitdir"), GIT_FILE_LIMIT).map_err(|error: std::io::Error| { return format!("{} has no gitdir file ({error})", admin.display()); })?;
    if Path::new(std::ffi::OsStr::from_bytes(recorded.trim_ascii())) != git_path.as_path() {
        return Err(format!("{} does not name {} back, so the registration is stale or points elsewhere", admin.display(), git_path.display()));
    }
    let common_text: String = read_plain_text(&admin.join("commondir")).map_err(|error: std::io::Error| { return format!("{} has no commondir ({error})", admin.display()); })?;
    let common: PathBuf = std::fs::canonicalize(admin.join(common_text.trim())).map_err(|error: std::io::Error| { return format!("the common directory cannot be resolved ({error})"); })?;
    return Ok(Candidate { path: real, admin, common, git_file });
}

/// What kind of reason stands in the way, so a caller exempts what it owns by kind and identity, never by the words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalKind {
    /// The worktree is locked with `git worktree lock`.
    Locked,
    /// An `index.lock` exists.
    IndexLock,
    /// Something exists at the converting name.
    ConvertingName,
    /// The filesystem is not btrfs or cannot be examined.
    Filesystem,
    /// A worktree is registered inside this one.
    Nested,
    /// A tracked file or the state of `HEAD` is not provably clean.
    NotQuiet,
    /// A process uses the worktree, or the process table cannot be read.
    Process,
    /// Another command of this program owns the worktree or the checkout around it.
    OtherCommand,
}

/// What an interrupted conversion of this command owns at this moment, established by identity before the refusals are made.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Owned {
    /// The `index.lock` holds this conversion's token.
    pub index_lock: bool,
    /// The object at the converting name is the subvolume whose identity the journal recorded.
    pub stage: bool,
}

/// One reason a worktree cannot be converted now: its kind and the sentence for the person.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// What kind of reason it is.
    pub kind: RefusalKind,
    /// The sentence.
    pub text: String,
}

/// What:
///  The directories of worktrees registered in the common directory `common` that lie inside `inside`, not `inside` itself.
/// Why:
///   A listing or a registration that cannot be read is an error and not the absence of a nested worktree; only a registration
///  with no `gitdir` file, or one whose directory is gone, is skipped, because Git would prune it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registeredInside(common: string, inside: string): string[]; // throws the reason a registration cannot be read
/// ```
fn registered_inside(common: &Path, inside: &Path) -> Result<Vec<PathBuf>, String> {
    let directory: PathBuf = common.join("worktrees");
    let names = match std::fs::read_dir(&directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the registered worktrees of {} cannot be listed ({error}), so a worktree inside this one cannot be ruled out", common.display())),
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for item in names {
        let name: std::fs::DirEntry = item.map_err(|error: std::io::Error| { return format!("the registered worktrees of {} cannot be listed ({error}), so a worktree inside this one cannot be ruled out", common.display()); })?;
        let recorded: Vec<u8> = match read_plain_file(&name.path().join("gitdir"), GIT_FILE_LIMIT) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("the registration {} cannot be read ({error}), so a worktree inside this one cannot be ruled out", name.path().display())),
        };
        // The recorded path is raw bytes: a directory named with bytes that are not UTF-8 is a valid path on Linux.
        let Some(worktree) = Path::new(std::ffi::OsStr::from_bytes(recorded.trim_ascii())).parent() else { continue };
        if worktree.starts_with(inside) && worktree != inside && worktree.is_dir() {
            found.push(worktree.to_path_buf());
        }
    }
    found.sort();
    return Ok(found);
}

/// The sentence that refuses a conversion while a migration of the main checkout that holds the worktree is running, or
///  nothing when none is (or the common directory is not a main checkout's `.git`).
fn migration_of_main_checkout(common: &Path) -> Option<String> {
    let root: &Path = common.parent().filter(|_parent: &&Path| { return common.file_name() == Some(std::ffi::OsStr::new(".git")); })?;
    let layout = crate::subvolume_preflight::layout_of(root)?;
    return match live_owner_of(layout.parent.as_path(), layout.claim_prefix.as_os_str()) {
        Ok(None) => None,
        Ok(Some(owner)) => Some(format!("process {} is migrating the main checkout {} to a subvolume (it owns {}); wait for it to finish, then run this command again", owner.pid, root.display(), owner.path.display())),
        Err(error) => Some(format!("whether the main checkout {} is being migrated cannot be decided ({error})", root.display())),
    };
}

/// What:
///  Every reason a worktree cannot be converted now, with its kind; empty means it can.
/// Why:
///   The dry run prints them all, and a real run refuses on the first call with the whole list. The registrations are read
///  from the common directory itself, so a bare layout is checked like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusals(platform, candidate): Refusal[];
/// ```
pub fn refusals(platform: &dyn Platform, candidate: &Candidate, owned: Owned, command_line_attributes: bool) -> Vec<Refusal> {
    let mut reasons: Vec<Refusal> = Vec::new();
    let mut refuse = |kind: RefusalKind, text: String| { reasons.push(Refusal { kind, text }); };
    if candidate.admin.join("locked").exists() {
        refuse(RefusalKind::Locked, format!("the worktree is locked ({} exists); unlock it with `git worktree unlock` first", candidate.admin.join("locked").display()));
    }
    if !owned.index_lock && candidate.admin.join("index.lock").exists() {
        refuse(RefusalKind::IndexLock, format!("{} exists: a Git command is writing the index", candidate.admin.join("index.lock").display()));
    }
    if let Some(text) = migration_of_main_checkout(candidate.common.as_path()) {
        refuse(RefusalKind::OtherCommand, text);
    }
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if !owned.stage && std::fs::symlink_metadata(&converting).is_ok() {
        refuse(RefusalKind::ConvertingName, format!("{} exists and no journal of this command owns it, so it is left alone; remove it yourself once you know what it is", converting.display()));
    }
    match platform.filesystem_facts(candidate.path.as_path()) {
        Ok((kind, _device)) if kind == crate::fs_kind::FilesystemKind::Btrfs => {}
        Ok(_) => refuse(RefusalKind::Filesystem, format!("{} is not on btrfs", candidate.path.display())),
        Err(error) => refuse(RefusalKind::Filesystem, format!("the filesystem of {} cannot be examined ({error})", candidate.path.display())),
    }
    match registered_inside(candidate.common.as_path(), candidate.path.as_path()) {
        Ok(nested_worktrees) => {
            for nested in nested_worktrees {
                refuse(RefusalKind::Nested, format!("the worktree {} is registered inside it; convert or move that one first", nested.display()));
            }
        }
        Err(reason) => refuse(RefusalKind::Nested, reason),
    }
    let mut environment = platform.conversion_environment();
    if command_line_attributes && environment.alternate_attribute_source.is_none() {
        environment.alternate_attribute_source = Some(String::from("--attr-source or -c attr.tree was given on the command line"));
    }
    if let Err(reason) = check_clean_tracked_by_content(candidate.path.as_path(), candidate.admin.as_path(), candidate.common.as_path(), &environment, owned.index_lock) {
        refuse(RefusalKind::NotQuiet, format!("it is not quiet and unmodified: {reason}"));
    }
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    if let Some(why) = scan.unavailable.as_ref() {
        refuse(RefusalKind::Process, format!("the process table cannot be read ({why}), so no process can be ruled out"));
    }
    for user in &scan.users {
        refuse(RefusalKind::Process, format!("process {} ({}) has its working directory or an open file inside it", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    return reasons;
}

/// A token unique to this run, written to the index lock so that only this command removes it.
fn new_token() -> String {
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed: std::time::Duration| { return elapsed.as_nanos(); });
    return format!("cli-git convert-worktree-to-subvolume pid={} at={nanos}", std::process::id());
}

/// What:
///  Everything one conversion needs besides the platform.
/// Why:
///   The phases share the same facts, so they travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Job = { candidate: Candidate; converting: string; journal: Journal };
/// ```
struct Job<'a> {
    /// The worktree.
    candidate: &'a Candidate,
    /// The sibling the copy is built at.
    converting: PathBuf,
    /// The journal, updated in step with the phases.
    journal: Journal,
}

impl Job<'_> {
    /// Record a phase durably.
    fn reach(&mut self, phase: Phase) -> Result<(), MigrateError> {
        self.journal.phase = phase;
        return self.journal.save(self.candidate.admin.as_path());
    }

    /// The directory that holds the worktree and its converting sibling, held open.
    fn parent(&self) -> Result<std::fs::File, MigrateError> {
        return held_parent(self.candidate.path.as_path());
    }

    /// The name of the converting sibling in the parent directory.
    fn converting_name(&self) -> &std::ffi::OsStr {
        return self.converting.file_name().unwrap_or_default();
    }

    /// The identity of the object at the converting name, if there is one.
    fn stage_identity(&self) -> Result<Option<ObjectId>, MigrateError> {
        return stage_identity_of(self.candidate.path.as_path());
    }
}

/// The identity of the object at `path`, a link not followed, or nothing when it cannot be examined.
fn identity_of_path(path: &Path) -> Option<ObjectId> {
    let parent: std::fs::File = held_parent(path).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// The identity, with the birth time, of the directory at `path`.
fn stage_free_identity(path: &Path) -> Result<ObjectId, MigrateError> {
    let parent: std::fs::File = held_parent(path)?;
    return identity_at(&parent, path.file_name().unwrap_or_default()).map_err(|error: std::io::Error| { return MigrateError::io("examine", path, &error); });
}

/// The directory that holds `path`, held open so that names below it mean the same objects for as long as it is held.
fn held_parent(path: &Path) -> Result<std::fs::File, MigrateError> {
    let parent: &Path = path.parent().ok_or_else(|| { return MigrateError::Journal(format!("{} has no parent directory", path.display())); })?;
    return open_root(parent).map_err(|error: std::io::Error| { return MigrateError::io("hold the directory of the worktree", parent, &error); });
}

/// The identity of the object at the converting name of `path`, or nothing when the name is free.
fn stage_identity_of(path: &Path) -> Result<Option<ObjectId>, MigrateError> {
    let converting: PathBuf = converting_path(path);
    let parent: std::fs::File = held_parent(path)?;
    return match identity_at(&parent, converting.file_name().unwrap_or_default()) {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(MigrateError::io("examine", converting.as_path(), &error)),
    };
}

/// What:
///  Make sure the converting subvolume exists and is the one this command made: create it and record its identity, or find
///  the recorded identity at the name.
/// Why:
///   A directory at the converting name that the journal has no identity for is foreign, whether someone made it or this
///  command was killed between creating it and recording it; it is never synced into, exchanged or destroyed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureStage(platform, job); // throws when the name holds anything the journal did not record creating
/// ```
fn ensure_stage(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt) -> Result<(), MigrateError> {
    match (job.stage_identity()?, job.journal.new_identity) {
        (Some(found), Some(recorded)) if found == recorded => return Ok(()),
        (Some(_), _) => {
            return Err(MigrateError::Journal(format!(
                "{} exists but the journal does not record this conversion creating it, so it is left alone; remove it yourself once you know what it is, or run the command with --abandon",
                job.converting.display()
            )));
        }
        (None, Some(_)) => {
            return Err(MigrateError::Journal(format!(
                "{} was created by this conversion and is gone; run the command with --abandon and start again",
                job.converting.display()
            )));
        }
        (None, None) => {}
    }
    platform.create_subvolume(job.converting.as_path())?;
    interrupt.check("stage-unrecorded")?;
    let made: ObjectId = job.stage_identity()?.ok_or_else(|| { return MigrateError::Journal(format!("{} was not there after it was created", job.converting.display())); })?;
    job.journal.new_identity = Some(made);
    return job.journal.save(job.candidate.admin.as_path());
}

/// Whether a sync pass changed anything: it copied, deleted, recreated or lost a file or link.
fn pass_changed(stats: &SyncStats) -> bool {
    return stats.files_copied > 0 || stats.deleted > 0 || stats.vanished > 0 || stats.symlinks > 0 || stats.fifos > 0;
}

/// Fail with the unreadable files a pass could not copy, because a copy that leaves them out would lose them at the swap.
fn require_complete(stats: &SyncStats, path: &Path) -> Result<(), MigrateError> {
    if let Some(first) = stats.unreadable.first() {
        return Err(MigrateError::Journal(format!(
            "{} holds {} file(s) this user cannot read, the first being {}; they would be lost at the swap, so nothing was swapped",
            path.display(),
            stats.unreadable.len(),
            first.display()
        )));
    }
    return Ok(());
}

/// Take Git's `index.lock` of the worktree with this conversion's token, or accept the one an earlier run of it took.
fn hold_index_lock(job: &Job<'_>) -> Result<(), MigrateError> {
    return acquire(job.candidate.admin.join("index.lock").as_path(), job.journal.token.as_str());
}

/// Remove Git's `index.lock` of the worktree when it holds this conversion's token; a lock that holds other text is a Git
///  command's and stays.
fn release_index_lock(job: &Job<'_>) -> Result<(), MigrateError> {
    let lock: PathBuf = job.candidate.admin.join("index.lock");
    if !is_ours(lock.as_path(), job.journal.token.as_str()) {
        return Ok(());
    }
    return match std::fs::remove_file(&lock) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(MigrateError::io("release the index lock", lock.as_path(), &error)),
    };
}

/// What:
///  The copy phase: create the subvolume if needed, copy with cloned data, and prove the copy equals the original.
/// Why:
///   It runs again on every resume before the swap, whatever the journal says: `Synced` only records that the copy once
///  equalled the original, and a writer may have acted since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyPhase(platform, job, interrupt, say);
/// ```
fn copy_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<(), MigrateError> {
    if job.journal.phase < Phase::Creating {
        job.reach(Phase::Creating)?;
    }
    interrupt.check("journaled")?;
    ensure_stage(platform, job, interrupt)?;
    interrupt.check("created")?;
    let first: SyncStats = sync_tree(platform, job.candidate.path.as_path(), job.converting.as_path(), SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None }, interrupt)?;
    require_complete(&first, job.candidate.path.as_path())?;
    say(format!("  copied {} entries ({} files, {} cloned)", first.entries, first.files_copied, first.cloned).as_str());
    // Git's `index.lock` is taken only now: the long first copy ran without it, and the lock covers the span in which a Git
    // writer could invalidate the proof, the swap or the index rewrite.
    hold_index_lock(job)?;
    // The proof: a second pass that must find nothing to copy or delete, so the copy equals the original at this instant.
    let second: SyncStats = sync_tree(platform, job.candidate.path.as_path(), job.converting.as_path(), SyncOptions { delete: true, update_only: false, excluded: &[], created_since: None }, interrupt)?;
    if let Err(error) = require_complete(&second, job.candidate.path.as_path()) {
        release_index_lock(job)?;
        return Err(error);
    }
    if pass_changed(&second) {
        // Nothing is swapped, so nothing needs the lock any more; Git commands may write the index again.
        release_index_lock(job)?;
        return Err(MigrateError::Journal(format!(
            "{} changed while it was copied ({} file(s) copied and {} removed by the proving pass); nothing was swapped, run the command again",
            job.candidate.path.display(),
            second.files_copied,
            second.deleted
        )));
    }
    interrupt.check("verified")?;
    job.reach(Phase::Synced)?;
    interrupt.check("synced")?;
    return Ok(());
}

/// What:
///  Which of the two directories holds the old tree: the worktree path (the swap has not happened) or the converting name
///  (it has).
/// Why:
///   The swap is one atomic exchange and the journal is written after it, so a kill between the two leaves the journal one phase
///  behind; the identity the journal recorded for the old root says where the old tree is now, so a rerun neither exchanges back
///  nor loses track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whereIsTheOldTree(journal, path, converting): 'at-path' | 'at-converting' | 'nowhere';
/// ```
fn old_tree_is_at(journal: &Journal, path: &Path, converting: &Path) -> Where {
    let is_old = |candidate: &Path| -> bool {
        return identity_of_path(candidate).is_some_and(|found: ObjectId| { return is_recorded_object(&journal.old_identity, &found); });
    };
    if is_old(path) {
        return Where::AtPath;
    }
    if is_old(converting) {
        return Where::AtConverting;
    }
    return Where::Nowhere;
}

/// Where the old tree is, relative to the swap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Where {
    /// At the worktree path: the swap has not happened.
    AtPath,
    /// At the converting name: the swap has happened.
    AtConverting,
    /// At neither: something else replaced it.
    Nowhere,
}

/// The swap phase: exchange the two directories (unless a killed run already did), then record it.
fn swap_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt) -> Result<(), MigrateError> {
    match old_tree_is_at(&job.journal, job.candidate.path.as_path(), job.converting.as_path()) {
        Where::AtPath => {
            ensure_stage(platform, job, interrupt)?;
            let refused: Vec<String> = refusals_after_lock(platform, job.candidate);
            if !refused.is_empty() {
                return Err(MigrateError::Journal(format!("it can no longer be swapped: {}", refused.join("; "))));
            }
            platform.exchange(job.candidate.path.as_path(), job.converting.as_path())?;
        }
        Where::AtConverting => {}
        Where::Nowhere => {
            return Err(MigrateError::Journal(format!(
                "neither {} nor {} is the directory this conversion recorded, so nothing is swapped or removed",
                job.candidate.path.display(),
                job.converting.display()
            )));
        }
    }
    interrupt.check("exchanged")?;
    job.reach(Phase::Exchanged)?;
    return Ok(());
}

/// The reasons that still bind just before the swap: a process that appeared, a lock that was taken, a file that changed.
fn refusals_after_lock(_platform: &dyn Platform, candidate: &Candidate) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    for user in &scan.users {
        reasons.push(format!("process {} ({}) is using the worktree", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    if let Some(why) = scan.unavailable.as_ref() {
        reasons.push(format!("the process table cannot be read ({why})"));
    }
    return reasons;
}

/// The checks after the swap: the path is the subvolume this command created and its `.git` file holds the bytes it held before.
fn verify_swap(platform: &dyn Platform, job: &Job<'_>) -> Result<(), MigrateError> {
    let path: &Path = job.candidate.path.as_path();
    if !platform.is_subvolume_root(path)? {
        return Err(MigrateError::Journal(format!("{} is not a subvolume root after the swap; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    let parent: std::fs::File = job.parent()?;
    let installed: ObjectId = identity_at(&parent, path.file_name().unwrap_or_default()).map_err(|error: std::io::Error| { return MigrateError::io("examine", path, &error); })?;
    if job.journal.new_identity != Some(installed) {
        return Err(MigrateError::Journal(format!("{} is not the subvolume this conversion created; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    let now: Vec<u8> = read_plain_file(&path.join(".git"), GIT_FILE_LIMIT).map_err(|error: std::io::Error| { return MigrateError::io("read", path.join(".git").as_path(), &error); })?;
    if now != job.journal.git_file {
        return Err(MigrateError::Journal(format!("the .git file of {} differs after the swap; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    return Ok(());
}

/// The index phase: rewrite the stat fields of the index to the files of the subvolume, so the first status stays fast.
/// A refresh that did not install the new index stops the conversion with the old tree kept, and the journal stays at the swap.
fn index_phase(job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<(), MigrateError> {
    let references: Vec<PathBuf> = vec![job.converting.clone()];
    let report = refresh_index_file_report(
        job.candidate.admin.join("index").as_path(),
        job.candidate.path.as_path(),
        &RefreshOptions { lock: IndexLock::Held(job.journal.token.as_str()), references: references.as_slice(), hash_length: repository_hash_length(job.candidate.common.as_path()) },
    );
    say(format!("  index: {}", report.line).as_str());
    if report.status == RefreshStatus::Skipped {
        return Err(MigrateError::Journal(format!(
            "the index of {} was not refreshed ({}); the old tree is kept at {}; fix that and run the command again",
            job.candidate.path.display(),
            report.line,
            job.converting.display()
        )));
    }
    interrupt.check("refreshed")?;
    job.reach(Phase::Refreshed)?;
    // The rewrite is the last thing that needs the lock; the removal of the old tree can take long and writes no index.
    return release_index_lock(job);
}

/// What:
///  The removal phase: keep the old tree when a process still holds it or when the installed tree does not hold all of it, and
///  otherwise delete it, but only when it is the very directory the journal recorded.
/// Why:
///   The old tree sits at the converting name from the exchange on, so nothing opens it by a name that anything uses; a process
///  that already held it open is found in the process table. The comparison with the installed tree is read-only and copies
///  nothing, so an edit made to the installed worktree is never overwritten; a difference keeps the old tree and is reported,
///  and a person decides which version is wanted. Anything that leaves the old tree in place is an incomplete conversion: the
///  journal stays and the run says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removalPhase(platform, job, interrupt, say);
/// ```
fn removal_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt) -> Result<(), MigrateError> {
    let parent: std::fs::File = job.parent()?;
    let scan = processes_using(&UseTarget::Inside(job.converting.as_path()));
    let mut holders: Vec<String> = scan.users.iter().map(|user| { return format!("process {} ({})", user.pid, user.command.as_deref().unwrap_or("unknown")); }).collect();
    if let Some(why) = scan.unavailable.as_ref() {
        holders.push(format!("the process table cannot be read ({why})"));
    }
    if !holders.is_empty() {
        return Err(MigrateError::Journal(format!("the old tree at {} is kept because {}; stop that and run the command again", job.converting.display(), holders.join("; "))));
    }
    match remove_recorded_old_tree(platform, &parent, job.converting_name(), job.journal.old_identity, job.candidate.path.as_path())? {
        Removal::Removed | Removal::Absent => {}
        Removal::Kept(why) => {
            return Err(MigrateError::Journal(format!("the old tree was not removed: {why}; the journal is kept, so put things right and run the command again")));
        }
        Removal::Differs(why) => {
            return Err(MigrateError::Journal(format!(
                "the old tree at {} is kept, and nothing was copied from it: {why}; settle which version is wanted by hand, then run the command again to remove it",
                job.converting.display()
            )));
        }
    }
    interrupt.check("removed")?;
    job.reach(Phase::Removed)?;
    return Ok(());
}

/// Release the lock and drop the journal, the last step of a finished conversion.
fn finish(job: &Job<'_>) -> Result<(), MigrateError> {
    release_index_lock(job)?;
    let journal: PathBuf = job.candidate.admin.join(JOURNAL_NAME);
    return std::fs::remove_file(&journal).map_err(|error: std::io::Error| { return MigrateError::io("remove the conversion journal", journal.as_path(), &error); });
}

/// Undo an unfinished conversion that has not reached its swap: destroy the subvolume this command created (the one whose
///  identity the journal recorded; anything else at the name is left where it is), release the lock and drop the journal.
///  Past the swap there is nothing to undo, and the command says to run it again.
fn abandon(platform: &dyn Platform, candidate: &Candidate, journal: Journal, say: &mut dyn FnMut(&str)) -> Result<Verdict, MigrateError> {
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if journal.phase >= Phase::Exchanged {
        say("  the swap already happened; run the command again to finish it (there is nothing to undo)");
        return Ok(Verdict::Skipped(vec![String::from("past the swap")]));
    }
    if let Some(found) = stage_identity_of(candidate.path.as_path())? {
        if journal.new_identity == Some(found) {
            if !platform.is_subvolume_root(converting.as_path())? {
                return Err(MigrateError::Journal(format!("{} is not a subvolume root, so it is not what this conversion created and is left alone", converting.display())));
            }
            platform.destroy_subvolume(converting.as_path())?;
        } else {
            say(format!("  {} is not recorded as made by this conversion, so it was left alone", converting.display()).as_str());
        }
    }
    let job: Job<'_> = Job { candidate, converting, journal };
    finish(&job)?;
    return Ok(Verdict::Abandoned);
}

/// What:
///  Convert, resume, abandon or preview one worktree.
/// Why:
///   One function holds the phase order, so the controls exercise the real sequence; the platform and the interrupt are
///  injected so the controls run it on any filesystem and kill it at every phase. A real run holds the conversion lock from
///  before it reads the journal until it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = await convertWorktree({ platform, path, options, interrupt, say });
/// ```
pub fn convert_worktree(platform: &dyn Platform, path: &Path, options: ConvertOptions, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<Verdict, MigrateError> {
    let candidate: Candidate = match inspect(path) {
        Ok(found) => found,
        Err(reason) => return Ok(Verdict::Skipped(vec![reason])),
    };
    // A dry run changes nothing, so it needs no claim; every other mode owns the conversion to the end. The claim is taken
    // before the journal is read, so two commands never read, write or resume one journal.
    let _claim: Option<OwnerClaim> = if options.dry_run {
        None
    } else {
        match take_claim(candidate.admin.as_path(), std::ffi::OsStr::new(CLAIM_PREFIX))? {
            ClaimOutcome::Held(held) => {
                if legacy_lock_is_held(candidate.admin.as_path())? {
                    return Ok(Verdict::Skipped(vec![format!(
                        "another conversion of {} is running (a command of release 0.5.20 or older holds {}), so this one did not start",
                        candidate.path.display(),
                        candidate.admin.join(crate::worktree_convert_journal::LOCK_NAME).display()
                    )]));
                }
                Some(held)
            }
            ClaimOutcome::Busy(owner) => {
                return Ok(Verdict::Skipped(vec![format!(
                    "another conversion of {} is running (process {} owns it through {}), so this one did not start",
                    candidate.path.display(),
                    owner.pid,
                    owner.path.display()
                )]));
            }
        }
    };
    let mut journal: Option<Journal> = Journal::load(candidate.admin.as_path())?;
    // A journal belongs to the path the conversion began at; a worktree moved since then is refused before anything changes.
    if let Some(found) = journal.as_ref().filter(|found: &&Journal| { return found.path.as_slice() != candidate.path.as_os_str().as_bytes(); }) {
        return Err(MigrateError::Journal(format!(
            "an unfinished conversion began at {}, but the worktree is at {} now; it was moved while the conversion was unfinished, so nothing was changed; move it back and run the command again",
            String::from_utf8_lossy(found.path.as_slice()),
            candidate.path.display()
        )));
    }
    // A kill between the exchange and its journal line leaves the journal one phase behind; bring it up to date in memory in
    // every mode, and on disk when the run changes anything.
    if let Some(found) = journal.as_mut().filter(|found: &&mut Journal| { return found.phase == Phase::Synced; }) {
        let converting: PathBuf = converting_path(candidate.path.as_path());
        if old_tree_is_at(found, candidate.path.as_path(), converting.as_path()) == Where::AtConverting {
            found.phase = Phase::Exchanged;
            if !options.dry_run {
                found.save(candidate.admin.as_path())?;
            }
        }
    }
    if options.abandon {
        return match journal {
            Some(found) => abandon(platform, &candidate, found, say),
            None => Ok(Verdict::Skipped(vec![String::from("no conversion is in progress here, so there is nothing to abandon")])),
        };
    }
    if journal.is_none() && platform.is_subvolume_root(candidate.path.as_path())? {
        return Ok(Verdict::AlreadySubvolume);
    }
    let resuming: bool = journal.is_some();
    // A conversion that has not reached its swap is checked again like a fresh one; past the swap the old path is gone.
    let must_be_quiet: bool = journal.as_ref().is_none_or(|found: &Journal| { return found.phase < Phase::Exchanged; });
    if must_be_quiet {
        // Only what this command owns is exempt: the index lock that holds its token, and the stage whose identity it recorded.
        let owned: Owned = match journal.as_ref() {
            Some(found) => Owned {
                index_lock: is_ours(candidate.admin.join("index.lock").as_path(), found.token.as_str()),
                stage: found.new_identity.is_some() && stage_identity_of(candidate.path.as_path())? == found.new_identity,
            },
            None => Owned::default(),
        };
        let reasons: Vec<Refusal> = refusals(platform, &candidate, owned, options.command_line_attributes);
        if !reasons.is_empty() {
            return Ok(Verdict::Skipped(reasons.into_iter().map(|reason: Refusal| { return reason.text; }).collect()));
        }
    }
    if options.dry_run {
        return Ok(if resuming { Verdict::Resumed } else { Verdict::WouldConvert });
    }
    let old: ObjectId = stage_free_identity(candidate.path.as_path())?;
    let token: String = journal.as_ref().map_or_else(new_token, |found: &Journal| { return found.token.clone(); });
    let started: Journal = journal.unwrap_or(Journal {
        phase: Phase::Started,
        token,
        path: candidate.path.as_os_str().as_bytes().to_vec(),
        old_identity: old,
        git_file: candidate.git_file.clone(),
        new_identity: None,
    });
    let converting: PathBuf = converting_path(candidate.path.as_path());
    let mut job: Job<'_> = Job { candidate: &candidate, converting, journal: started };
    if !resuming {
        // The journal first, made exclusively: the lock and the subvolume exist only after the journal names them.
        job.journal.create_new(candidate.admin.as_path())?;
    }
    if job.journal.phase < Phase::Exchanged {
        copy_phase(platform, &mut job, interrupt, say)?;
        swap_phase(platform, &mut job, interrupt)?;
    }
    verify_swap(platform, &job)?;
    if job.journal.phase < Phase::Refreshed {
        // A run that stopped after the swap still owes the index rewrite, which needs the lock; a release up to 0.5.20 left it
        // in place from the start of the conversion, and taking it again is accepted when it holds this token.
        hold_index_lock(&job)?;
        index_phase(&mut job, interrupt, say)?;
    }
    if job.journal.phase < Phase::Removed {
        removal_phase(platform, &mut job, interrupt)?;
    }
    finish(&job)?;
    return Ok(if resuming { Verdict::Resumed } else { Verdict::Converted });
}

/// Conversion controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_tests.rs"]
mod tests;

/// The cleanliness proof of the conversion by content stays out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_content_tests.rs"]
mod content_tests;

/// The controls for who owns a conversion and how long the index lock is held stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_ownership_tests.rs"]
mod ownership_tests;

/// The controls for what keeps a foreign or late change from being lost: identities, the lock, the journal's files, the pinned
///  removal and the refusals by kind.
#[cfg(test)]
#[path = "worktree_convert_safety_tests.rs"]
mod safety_tests;

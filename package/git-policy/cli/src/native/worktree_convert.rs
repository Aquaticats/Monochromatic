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

use super::process_use::{UseTarget, processes_using};
use super::subvolume_error::{Interrupt, MigrateError};
use super::subvolume_index_files::{IndexLock, RefreshOptions, linked_worktrees_inside, refresh_index_file_with, repository_hash_length};
use super::subvolume_lock::{acquire, is_ours};
use super::subvolume_platform::Platform;
use super::subvolume_remove::remove_tree;
use super::subvolume_sync::{SyncOptions, SyncStats, sync_tree};
use super::worktree_clean::check_clean_tracked;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The suffix of the sibling subvolume the copy is built in.
pub const CONVERTING_SUFFIX: &str = ".cli-git-converting";

/// The name of the journal inside the worktree's administrative directory.
pub const JOURNAL_NAME: &str = "cli-git-convert";

/// The first line of every journal.
const JOURNAL_HEADER: &str = "cli-git-convert v1";

/// What:
///  The phase a conversion has reached, in order.
/// Why:
///   A rerun reads it to skip the finished phases and to decide whether an abandon is still possible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'started' | 'creating' | 'synced' | 'exchanged' | 'refreshed' | 'removed';
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Phase {
    /// The journal and the index lock exist; nothing else.
    Started,
    /// The converting subvolume is being created, or exists.
    Creating,
    /// The copy was made and proved equal to the original.
    Synced,
    /// The path names the subvolume; the old tree sits at the converting name.
    Exchanged,
    /// The index stat fields describe the files of the subvolume.
    Refreshed,
    /// The old tree is gone; only the journal and the lock remain.
    Removed,
}

impl Phase {
    /// The name written to the journal.
    fn name(self) -> &'static str {
        return match self {
            Phase::Started => "started",
            Phase::Creating => "creating",
            Phase::Synced => "synced",
            Phase::Exchanged => "exchanged",
            Phase::Refreshed => "refreshed",
            Phase::Removed => "removed",
        };
    }

    /// The phase a journal line names.
    fn parse(text: &str) -> Option<Phase> {
        return [Phase::Started, Phase::Creating, Phase::Synced, Phase::Exchanged, Phase::Refreshed, Phase::Removed]
            .into_iter()
            .find(|phase: &Phase| { return phase.name() == text; });
    }
}

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
/// type ConvertOptions = { dryRun: boolean; abandon: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConvertOptions {
    /// List what would happen and change nothing.
    pub dry_run: bool,
    /// Undo an unfinished conversion that has not reached its swap.
    pub abandon: bool,
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
    let git_file: Vec<u8> = std::fs::read(&git_path).map_err(|error: std::io::Error| { return format!("{} is unreadable ({error})", git_path.display()); })?;
    let text: &str = std::str::from_utf8(git_file.as_slice()).map_err(|_| { return format!("{} is not text", git_path.display()); })?;
    let Some(named) = text.trim().strip_prefix("gitdir:") else {
        return Err(format!("{} has no gitdir line", git_path.display()));
    };
    let admin: PathBuf = std::fs::canonicalize(real.join(named.trim())).map_err(|error: std::io::Error| { return format!("the administrative directory {} cannot be resolved ({error})", named.trim()); })?;
    let recorded: Vec<u8> = std::fs::read(admin.join("gitdir")).map_err(|error: std::io::Error| { return format!("{} has no gitdir file ({error})", admin.display()); })?;
    if Path::new(std::ffi::OsStr::from_bytes(recorded.trim_ascii())) != git_path.as_path() {
        return Err(format!("{} does not name {} back, so the registration is stale or points elsewhere", admin.display(), git_path.display()));
    }
    let common_text: String = std::fs::read_to_string(admin.join("commondir")).map_err(|error: std::io::Error| { return format!("{} has no commondir ({error})", admin.display()); })?;
    let common: PathBuf = std::fs::canonicalize(admin.join(common_text.trim())).map_err(|error: std::io::Error| { return format!("the common directory cannot be resolved ({error})"); })?;
    return Ok(Candidate { path: real, admin, common, git_file });
}

/// What:
///  Every reason a worktree cannot be converted now, as sentences; empty means it can.
/// Why:
///   The dry run prints them all, and a real run refuses on the first call with the whole list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusals(platform, candidate): string[];
/// ```
pub fn refusals(platform: &dyn Platform, candidate: &Candidate) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    if candidate.admin.join("locked").exists() {
        reasons.push(format!("the worktree is locked ({} exists); unlock it with `git worktree unlock` first", candidate.admin.join("locked").display()));
    }
    if candidate.admin.join("index.lock").exists() {
        reasons.push(format!("{} exists: a Git command is writing the index", candidate.admin.join("index.lock").display()));
    }
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if std::fs::symlink_metadata(&converting).is_ok() {
        reasons.push(format!("{} exists and no journal of this command owns it, so it is left alone; remove it yourself once you know what it is", converting.display()));
    }
    match platform.filesystem_facts(candidate.path.as_path()) {
        Ok((kind, _device)) if kind == super::fs_kind::FilesystemKind::Btrfs => {}
        Ok(_) => reasons.push(format!("{} is not on btrfs", candidate.path.display())),
        Err(error) => reasons.push(format!("the filesystem of {} cannot be examined ({error})", candidate.path.display())),
    }
    if let Some(root) = candidate.common.parent().filter(|_parent: &&Path| { return candidate.common.file_name().is_some_and(|name: &std::ffi::OsStr| { return name == ".git"; }); }) {
        for nested in linked_worktrees_inside(root) {
            if nested.worktree != candidate.path && nested.worktree.starts_with(candidate.path.as_path()) {
                reasons.push(format!("the worktree {} is registered inside it; convert or move that one first", nested.worktree.display()));
            }
        }
    }
    if let Err(reason) = check_clean_tracked(candidate.path.as_path(), candidate.admin.as_path(), candidate.common.as_path()) {
        reasons.push(format!("it is not quiet and unmodified: {reason}"));
    }
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    if let Some(why) = scan.unavailable.as_ref() {
        reasons.push(format!("the process table cannot be read ({why}), so no process can be ruled out"));
    }
    for user in &scan.users {
        reasons.push(format!("process {} ({}) has its working directory or an open file inside it", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    return reasons;
}

/// What:
///  The journal of one conversion, kept in the worktree's administrative directory.
/// Why:
///   It is the only proof that the converting subvolume and the lock are this command's, and it names the identity of the
///  old root, which is what the final removal checks before it deletes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertJournal = { phase: Phase; token: string; path: string; oldDevice: bigint; oldInode: bigint; gitFile: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct Journal {
    /// The phase reached.
    phase: Phase,
    /// The token the index lock holds.
    token: String,
    /// The worktree path as raw bytes.
    path: Vec<u8>,
    /// Device of the old root.
    old_device: u64,
    /// Inode of the old root.
    old_inode: u64,
    /// The exact bytes of the `.git` file.
    git_file: Vec<u8>,
}

/// The lowercase hexadecimal form of some bytes.
fn hex(bytes: &[u8]) -> String {
    return bytes.iter().map(|byte: &u8| { return format!("{byte:02x}"); }).collect();
}

/// The bytes a hexadecimal string names.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    return (0..text.len()).step_by(2).map(|at: usize| { return u8::from_str_radix(text.get(at..at + 2)?, 16).ok(); }).collect();
}

impl Journal {
    /// The journal text.
    fn encode(&self) -> String {
        return format!(
            "{JOURNAL_HEADER}\nphase={}\ntoken={}\npath={}\nold_device={}\nold_inode={}\ngit_file={}\n",
            self.phase.name(),
            self.token,
            hex(self.path.as_slice()),
            self.old_device,
            self.old_inode,
            hex(self.git_file.as_slice())
        );
    }

    /// The journal a text describes.
    fn decode(text: &str) -> Result<Journal, MigrateError> {
        let bad = || { return MigrateError::Journal(String::from("the conversion journal has a shape this command does not know; nothing was changed")); };
        let mut lines = text.lines();
        if lines.next() != Some(JOURNAL_HEADER) {
            return Err(bad());
        }
        let field = |name: &str| -> Option<String> { return text.lines().find_map(|line: &str| { return line.strip_prefix(name).and_then(|rest: &str| { return rest.strip_prefix('='); }).map(String::from); }); };
        return Ok(Journal {
            phase: Phase::parse(field("phase").ok_or_else(bad)?.as_str()).ok_or_else(bad)?,
            token: field("token").ok_or_else(bad)?,
            path: unhex(field("path").ok_or_else(bad)?.as_str()).ok_or_else(bad)?,
            old_device: field("old_device").and_then(|digits: String| { return digits.parse().ok(); }).ok_or_else(bad)?,
            old_inode: field("old_inode").and_then(|digits: String| { return digits.parse().ok(); }).ok_or_else(bad)?,
            git_file: unhex(field("git_file").ok_or_else(bad)?.as_str()).ok_or_else(bad)?,
        });
    }

    /// Write the journal beside its place, synchronize it, and rename it into place.
    fn save(&self, admin: &Path) -> Result<(), MigrateError> {
        let target: PathBuf = admin.join(JOURNAL_NAME);
        let temporary: PathBuf = admin.join(format!("{JOURNAL_NAME}.tmp"));
        let written: std::io::Result<()> = (|| {
            let mut file: std::fs::File = std::fs::File::create(&temporary)?;
            file.write_all(self.encode().as_bytes())?;
            file.sync_all()?;
            std::fs::rename(&temporary, &target)?;
            return std::fs::File::open(admin).and_then(|directory: std::fs::File| { return directory.sync_all(); });
        })();
        return written.map_err(|error: std::io::Error| { return MigrateError::io("write the conversion journal", target.as_path(), &error); });
    }

    /// The journal of an administrative directory, if one exists.
    fn load(admin: &Path) -> Result<Option<Journal>, MigrateError> {
        let path: PathBuf = admin.join(JOURNAL_NAME);
        match std::fs::read_to_string(&path) {
            Ok(text) => return Journal::decode(text.as_str()).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(MigrateError::io("read the conversion journal", path.as_path(), &error)),
        }
    }
}

/// A token unique to this run, written to the index lock so that only this command removes it.
fn new_token() -> String {
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed: std::time::Duration| { return elapsed.as_nanos(); });
    return format!("cli-git convert-worktree-to-subvolume pid={} at={nanos}", std::process::id());
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
}

/// The copy phase: create the subvolume if needed, copy with cloned data, and prove the copy equals the original.
fn copy_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<(), MigrateError> {
    if job.journal.phase < Phase::Creating {
        job.reach(Phase::Creating)?;
    }
    interrupt.check("journaled")?;
    if std::fs::symlink_metadata(&job.converting).is_err() {
        platform.create_subvolume(job.converting.as_path())?;
    }
    interrupt.check("created")?;
    let first: SyncStats = sync_tree(platform, job.candidate.path.as_path(), job.converting.as_path(), SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None }, interrupt)?;
    require_complete(&first, job.candidate.path.as_path())?;
    say(format!("  copied {} entries ({} files, {} cloned)", first.entries, first.files_copied, first.cloned).as_str());
    // The proof: a second pass that must find nothing to copy or delete, so the copy equals the original at this instant.
    let second: SyncStats = sync_tree(platform, job.candidate.path.as_path(), job.converting.as_path(), SyncOptions { delete: true, update_only: false, excluded: &[], created_since: None }, interrupt)?;
    require_complete(&second, job.candidate.path.as_path())?;
    if pass_changed(&second) {
        return Err(MigrateError::Journal(format!(
            "{} changed while it was copied ({} file(s) copied and {} removed by the proving pass); nothing was swapped, run the command again",
            job.candidate.path.display(),
            second.files_copied,
            second.deleted
        )));
    }
    interrupt.check("verified")?;
    job.reach(Phase::Synced)?;
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
        return std::fs::symlink_metadata(candidate).is_ok_and(|metadata: std::fs::Metadata| { return metadata.dev() == journal.old_device && metadata.ino() == journal.old_inode; });
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

/// The checks after the swap: the path is a subvolume root and its `.git` file holds the bytes it held before.
fn verify_swap(platform: &dyn Platform, job: &Job<'_>) -> Result<(), MigrateError> {
    let path: &Path = job.candidate.path.as_path();
    if !platform.is_subvolume_root(path)? {
        return Err(MigrateError::Journal(format!("{} is not a subvolume root after the swap; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    let now: Vec<u8> = std::fs::read(path.join(".git")).map_err(|error: std::io::Error| { return MigrateError::io("read", path.join(".git").as_path(), &error); })?;
    if now != job.journal.git_file {
        return Err(MigrateError::Journal(format!("the .git file of {} differs after the swap; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    return Ok(());
}

/// The index phase: rewrite the stat fields of the index to the files of the subvolume, so the first status stays fast.
fn index_phase(job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<(), MigrateError> {
    let references: Vec<PathBuf> = vec![job.converting.clone()];
    let report: String = refresh_index_file_with(
        job.candidate.admin.join("index").as_path(),
        job.candidate.path.as_path(),
        &RefreshOptions { lock: IndexLock::Held(job.journal.token.as_str()), references: references.as_slice(), hash_length: repository_hash_length(job.candidate.common.as_path()) },
    );
    say(format!("  index: {report}").as_str());
    interrupt.check("refreshed")?;
    job.reach(Phase::Refreshed)?;
    return Ok(());
}

/// The removal phase: delete the old tree, but only when it is the very directory the journal recorded.
fn removal_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<(), MigrateError> {
    match std::fs::symlink_metadata(&job.converting) {
        Ok(metadata) => {
            let is_old: bool = metadata.dev() == job.journal.old_device && metadata.ino() == job.journal.old_inode && !platform.is_subvolume_root(job.converting.as_path())?;
            if is_old {
                remove_tree(job.converting.as_path()).map_err(|error: std::io::Error| { return MigrateError::io("remove the old tree", job.converting.as_path(), &error); })?;
            } else {
                say(format!("  {} is not the old tree this conversion recorded, so it was left alone", job.converting.display()).as_str());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(MigrateError::io("examine", job.converting.as_path(), &error)),
    }
    interrupt.check("removed")?;
    job.reach(Phase::Removed)?;
    return Ok(());
}

/// Release the lock and drop the journal, the last step of a finished conversion.
fn finish(job: &Job<'_>) -> Result<(), MigrateError> {
    let lock: PathBuf = job.candidate.admin.join("index.lock");
    if is_ours(lock.as_path(), job.journal.token.as_str()) {
        std::fs::remove_file(&lock).map_err(|error: std::io::Error| { return MigrateError::io("release the index lock", lock.as_path(), &error); })?;
    }
    let journal: PathBuf = job.candidate.admin.join(JOURNAL_NAME);
    return std::fs::remove_file(&journal).map_err(|error: std::io::Error| { return MigrateError::io("remove the conversion journal", journal.as_path(), &error); });
}

/// Undo an unfinished conversion that has not reached its swap: destroy the subvolume this command created, release the
///  lock and drop the journal. Past the swap there is nothing to undo, and the command says to run it again.
fn abandon(platform: &dyn Platform, candidate: &Candidate, journal: Journal, say: &mut dyn FnMut(&str)) -> Result<Verdict, MigrateError> {
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if journal.phase >= Phase::Exchanged {
        say("  the swap already happened; run the command again to finish it (there is nothing to undo)");
        return Ok(Verdict::Skipped(vec![String::from("past the swap")]));
    }
    if journal.phase >= Phase::Creating && std::fs::symlink_metadata(&converting).is_ok() {
        if !platform.is_subvolume_root(converting.as_path())? {
            return Err(MigrateError::Journal(format!("{} is not a subvolume root, so it is not what this conversion created and is left alone", converting.display())));
        }
        platform.destroy_subvolume(converting.as_path())?;
    }
    let job: Job<'_> = Job { candidate, converting, journal };
    finish(&job)?;
    return Ok(Verdict::Abandoned);
}

/// What:
///  Convert, resume, abandon or preview one worktree.
/// Why:
///   One function holds the phase order, so the controls exercise the real sequence; the platform and the interrupt are
///  injected so the controls run it on any filesystem and kill it at every phase.
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
    let mut journal: Option<Journal> = Journal::load(candidate.admin.as_path())?;
    // A kill between the exchange and its journal line leaves the journal one phase behind; bring it up to date first.
    if let Some(found) = journal.as_mut().filter(|found: &&mut Journal| { return found.phase == Phase::Synced; }) {
        let converting: PathBuf = converting_path(candidate.path.as_path());
        if old_tree_is_at(found, candidate.path.as_path(), converting.as_path()) == Where::AtConverting && !options.dry_run {
            found.phase = Phase::Exchanged;
            found.save(candidate.admin.as_path())?;
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
        let mut reasons: Vec<String> = refusals(platform, &candidate);
        if resuming {
            // The lock and the converting subvolume of the journal are this command's own.
            let token: &str = journal.as_ref().map_or("", |found: &Journal| { return found.token.as_str(); });
            reasons.retain(|reason: &String| { return !(reason.contains("index.lock") && is_ours(candidate.admin.join("index.lock").as_path(), token)) && !reason.contains(CONVERTING_SUFFIX); });
        }
        if !reasons.is_empty() {
            return Ok(Verdict::Skipped(reasons));
        }
    }
    if options.dry_run {
        return Ok(if resuming { Verdict::Resumed } else { Verdict::WouldConvert });
    }
    let old: std::fs::Metadata = std::fs::symlink_metadata(&candidate.path).map_err(|error: std::io::Error| { return MigrateError::io("examine", candidate.path.as_path(), &error); })?;
    let token: String = journal.as_ref().map_or_else(new_token, |found: &Journal| { return found.token.clone(); });
    let started: Journal = journal.unwrap_or(Journal {
        phase: Phase::Started,
        token,
        path: candidate.path.as_os_str().as_bytes().to_vec(),
        old_device: old.dev(),
        old_inode: old.ino(),
        git_file: candidate.git_file.clone(),
    });
    let converting: PathBuf = converting_path(candidate.path.as_path());
    let mut job: Job<'_> = Job { candidate: &candidate, converting, journal: started };
    if !resuming {
        // The journal first: the lock and the subvolume exist only after the journal names them.
        job.journal.save(candidate.admin.as_path())?;
    }
    acquire(candidate.admin.join("index.lock").as_path(), job.journal.token.as_str())?;
    if job.journal.phase < Phase::Synced {
        copy_phase(platform, &mut job, interrupt, say)?;
    }
    if job.journal.phase < Phase::Exchanged {
        swap_phase(platform, &mut job, interrupt)?;
    }
    verify_swap(platform, &job)?;
    if job.journal.phase < Phase::Refreshed {
        index_phase(&mut job, interrupt, say)?;
    }
    if job.journal.phase < Phase::Removed {
        removal_phase(platform, &mut job, interrupt, say)?;
    }
    finish(&job)?;
    return Ok(if resuming { Verdict::Resumed } else { Verdict::Converted });
}

/// Conversion controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_tests.rs"]
mod tests;

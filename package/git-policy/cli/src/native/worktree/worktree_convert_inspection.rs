//! What:
//!  The first look at a worktree a conversion might change:
//! what its `.git` file names,
//! and every reason the conversion must
//!  not start.
//! Why:
//!  A conversion swaps a whole directory,
//! so it starts only when the worktree is registered as the administrative directory
//!  says,
//! is quiet,
//! is unmodified,
//! and is on btrfs;
//! each reason that stops it is reported,
//! not only the first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const candidate = inspect(path);
//! const reasons = refusals(platform, candidate, owned, commandLineAttributes); // every reason the conversion must not start
//! ```

use crate::process_use::{UseTarget, processes_using};
use crate::single_owner_claim::live_owner_of;
use crate::subvolume_platform::Platform;
use crate::worktree_clean::check_clean_tracked_by_content;
use crate::plain_file::{read_plain_file, read_plain_text};
use crate::worktree_path_failure_message::{path_failure, subject_failure};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The most bytes of a `.git` pointer or a registration file that are read.
pub const GIT_FILE_LIMIT: u64 = 1 << 16;

/// The suffix of the sibling subvolume the copy is built in.
pub const CONVERTING_SUFFIX: &str = ".cli-git-converting";

/// What:
///  The facts about a worktree path that every mode needs.
/// Why:
///   They are read once,
/// from the `.git` file and the administrative directory,
/// without starting Git.
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
///  Read the registration of a worktree path:
/// its `.git` file,
/// its administrative directory,
/// and the back-pointer.
/// Why:
///   Only a linked worktree whose administrative directory names this very path back is converted;
/// anything else (the main
///  checkout,
/// a submodule,
/// a stale registration) is refused with the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inspect(path: string): Candidate; // throws the reason
/// ```
pub fn inspect(path: &Path) -> Result<Candidate, String> {
    let real: PathBuf = path_failure(std::fs::canonicalize(path), path, "cannot be resolved")?;
    let git_path: PathBuf = real.join(".git");
    let Ok(metadata) = std::fs::symlink_metadata(&git_path) else {
        return Err(format!("{} has no .git entry, so it is not a worktree", real.display()));
    };
    if metadata.is_dir() {
        return Err(format!("{} holds a .git directory: it is a main checkout or a repository, not a linked worktree", real.display()));
    }
    let git_file: Vec<u8> = path_failure(read_plain_file(&git_path, GIT_FILE_LIMIT), git_path.as_path(), "is unreadable")?;
    let Ok(text) = std::str::from_utf8(git_file.as_slice()) else {
        return Err(format!("{} is not text", git_path.display()));
    };
    let Some(named) = text.trim().strip_prefix("gitdir:") else {
        return Err(format!("{} has no gitdir line", git_path.display()));
    };
    let admin: PathBuf = match std::fs::canonicalize(real.join(named.trim())) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("the administrative directory {} cannot be resolved ({error})", named.trim())),
    };
    let recorded: Vec<u8> = path_failure(read_plain_file(&admin.join("gitdir"), GIT_FILE_LIMIT), admin.as_path(), "has no gitdir file")?;
    // The registration may spell this worktree through a symbolic link or relative to itself; the question is which file it names.
    let recorded_path: PathBuf = recorded_path(admin.as_path(), recorded.trim_ascii());
    if !crate::real_git_candidate::same_file(recorded_path.as_path(), git_path.as_path()) {
        return Err(format!("{} does not name {} back (it names {}), so the registration is stale or points elsewhere", admin.join("gitdir").display(), git_path.display(), recorded_path.display()));
    }
    let common_text: String = path_failure(read_plain_text(&admin.join("commondir")), admin.as_path(), "has no commondir")?;
    let common: PathBuf = subject_failure(std::fs::canonicalize(admin.join(common_text.trim())), &"the common directory", "cannot be resolved")?;
    return Ok(Candidate { path: real, admin, common, git_file });
}

/// What:
///  The path a registration's `gitdir` file names,
/// absolute whether the record is absolute or written relative to the registration.
/// Why:
///   Git writes the record relative to the registration's directory under `--relative-paths` and `worktree.useRelativePaths`;
/// both spellings name the same file,
/// so the record is resolved before it is compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = isAbsolute(recorded) ? recorded : join(admin, recorded);
/// ```
fn recorded_path(admin: &Path, recorded: &[u8]) -> PathBuf {
    let named: &Path = Path::new(std::ffi::OsStr::from_bytes(recorded));
    return if named.is_absolute() { named.to_path_buf() } else { admin.join(named) };
}

/// What kind of reason stands in the way,
/// so a caller exempts what it owns by kind and identity,
/// never by the words.
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
    /// A process uses the worktree,
    /// or the process table cannot be read.
    Process,
    /// Another command of this program owns the worktree or the checkout around it.
    OtherCommand,
}

/// What an interrupted conversion of this command owns at this moment,
/// established by identity before the refusals are made.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Owned {
    /// The `index.lock` holds this conversion's token.
    pub index_lock: bool,
    /// The object at the converting name is the subvolume whose identity the journal recorded.
    pub stage: bool,
}

/// One reason a worktree cannot be converted now:
/// its kind and the sentence for the person.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// What kind of reason it is.
    pub kind: RefusalKind,
    /// The sentence.
    pub text: String,
}

/// What:
///  The directories of worktrees registered in the common directory `common` that lie inside `inside`,
/// not `inside` itself.
/// Why:
///   A listing or a registration that cannot be read is an error and not the absence of a nested worktree;
/// only a registration
///  with no `gitdir` file,
/// or one whose directory is gone,
/// is skipped,
/// because Git would prune it.
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
        let name: std::fs::DirEntry = match item {
            Ok(entry) => entry,
            Err(error) => return Err(format!("the registered worktrees of {} cannot be listed ({error}), so a worktree inside this one cannot be ruled out", common.display())),
        };
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

/// The sentence that refuses a conversion while a migration of the main checkout that holds the worktree is running,
/// or
///  nothing when none is (or the common directory is not a main checkout's `.git`).
fn migration_of_main_checkout(common: &Path) -> Option<String> {
    if common.file_name() != Some(std::ffi::OsStr::new(".git")) {
        return None;
    }
    let root: &Path = common.parent()?;
    let layout = crate::subvolume_preflight::layout_of(root)?;
    return match live_owner_of(layout.parent.as_path(), layout.claim_prefix.as_os_str()) {
        Ok(None) => None,
        Ok(Some(owner)) => Some(format!("process {} is migrating the main checkout {} to a subvolume (it owns {}); wait for it to finish, then run this command again", owner.pid, root.display(), owner.path.display())),
        Err(error) => Some(format!("whether the main checkout {} is being migrated cannot be decided ({error})", root.display())),
    };
}

/// What:
///  Add one refusal of `kind` with its `text` to the list.
/// Why:
///   Every check of the conversion's start adds its refusal the same way,
/// so all of them are reported at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reasons.push({ kind, text })
/// ```
fn refuse(reasons: &mut Vec<Refusal>, kind: RefusalKind, text: String) {
    reasons.push(Refusal { kind, text });
}

/// What:
///  Every reason a worktree cannot be converted now,
/// with its kind;
/// empty means it can.
/// Why:
///   The dry run prints them all,
/// and a real run refuses on the first call with the whole list.
/// The registrations are read
///  from the common directory itself,
/// so a bare layout is checked like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusals(platform, candidate): Refusal[];
/// ```
pub fn refusals(platform: &dyn Platform, candidate: &Candidate, owned: Owned, command_line_attributes: bool) -> Vec<Refusal> {
    let mut reasons: Vec<Refusal> = Vec::new();
    if candidate.admin.join("locked").exists() {
        refuse(&mut reasons, RefusalKind::Locked, format!("the worktree is locked ({} exists); unlock it with `git worktree unlock` first", candidate.admin.join("locked").display()));
    }
    if !owned.index_lock && candidate.admin.join("index.lock").exists() {
        refuse(&mut reasons, RefusalKind::IndexLock, format!("{} exists: a Git command is writing the index", candidate.admin.join("index.lock").display()));
    }
    if let Some(text) = migration_of_main_checkout(candidate.common.as_path()) {
        refuse(&mut reasons, RefusalKind::OtherCommand, text);
    }
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if !owned.stage && std::fs::symlink_metadata(&converting).is_ok() {
        refuse(&mut reasons, RefusalKind::ConvertingName, format!("{} exists and no journal of this command owns it, so it is left alone; remove it yourself once you know what it is", converting.display()));
    }
    match platform.filesystem_facts(candidate.path.as_path()) {
        Ok((kind, _device)) if kind == crate::fs_kind::FilesystemKind::Btrfs => {}
        Ok(_) => refuse(&mut reasons, RefusalKind::Filesystem, format!("{} is not on btrfs", candidate.path.display())),
        Err(error) => refuse(&mut reasons, RefusalKind::Filesystem, format!("the filesystem of {} cannot be examined ({error})", candidate.path.display())),
    }
    match registered_inside(candidate.common.as_path(), candidate.path.as_path()) {
        Ok(nested_worktrees) => {
            for nested in nested_worktrees {
                refuse(&mut reasons, RefusalKind::Nested, format!("the worktree {} is registered inside it; convert or move that one first", nested.display()));
            }
        }
        Err(reason) => refuse(&mut reasons, RefusalKind::Nested, reason),
    }
    let mut environment = platform.conversion_environment();
    if command_line_attributes && environment.alternate_attribute_source.is_none() {
        environment.alternate_attribute_source = Some(String::from("--attr-source or -c attr.tree was given on the command line"));
    }
    if let Err(reason) = check_clean_tracked_by_content(candidate.path.as_path(), candidate.admin.as_path(), candidate.common.as_path(), &environment, owned.index_lock) {
        refuse(&mut reasons, RefusalKind::NotQuiet, format!("it is not quiet and unmodified: {reason}"));
    }
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    if let Some(why) = scan.unavailable.as_ref() {
        refuse(&mut reasons, RefusalKind::Process, format!("the process table cannot be read ({why}), so no process can be ruled out"));
    }
    for user in &scan.users {
        refuse(&mut reasons, RefusalKind::Process, format!("process {} ({}) has its working directory or an open file inside it", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    return reasons;
}

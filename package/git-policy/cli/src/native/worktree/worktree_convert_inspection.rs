//! What:
//!  The first look at a worktree a conversion might change:
//! the sibling name the copy is built at,
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
//! const reasons = refusals(platform, candidate, owned, commandLineAttributes); // every reason the conversion must not start
//! ```

/// Import the bounded read of one plain file,
/// which is how a registration is read.
use crate::plain_file::read_plain_file;
use crate::process_use::{
    UseTarget,
    processes_using,
};
use crate::single_owner_claim::live_owner_of;
use crate::subvolume_platform::Platform;
use crate::worktree_clean::check_clean_tracked_by_content;
/// Import the facts of the worktree this look is taken at,
/// and hand them on under this module's own name,
/// because every caller that reaches them here keeps that path.
pub use crate::worktree_convert_candidate::{
    Candidate,
    GIT_FILE_LIMIT,
    inspect,
};
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// The suffix of the sibling subvolume the copy is built in.
pub const CONVERTING_SUFFIX: &str = ".cli-git-converting";

/// The sibling path the copy is built at.
pub fn converting_path(path: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = path.as_os_str().to_os_string();
    name.push(CONVERTING_SUFFIX);
    return PathBuf::from(name);
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
fn registered_inside(
    common: &Path,
    inside: &Path,
) -> Result<Vec<PathBuf>, String> {
    let directory: PathBuf = common.join("worktrees");
    let names = match std::fs::read_dir(&directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "the registered worktrees of {} cannot be listed ({error}), so a worktree inside this one cannot be ruled out",
                common.display()
            ));
        }
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for item in names {
        let name: std::fs::DirEntry = match item {
            Ok(entry) => entry,
            Err(error) => {
                return Err(format!(
                    "the registered worktrees of {} cannot be listed ({error}), so a worktree inside this one cannot be ruled out",
                    common.display()
                ));
            }
        };
        let recorded: Vec<u8> = match read_plain_file(&name.path().join("gitdir"), GIT_FILE_LIMIT) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "the registration {} cannot be read ({error}), so a worktree inside this one cannot be ruled out",
                    name.path().display()
                ));
            }
        };
        // The recorded path is raw bytes: a directory named with bytes that are not UTF-8 is a valid path on Linux.
        let Some(worktree) = Path::new(std::ffi::OsStr::from_bytes(recorded.trim_ascii())).parent()
        else {
            continue;
        };
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
        Ok(Some(owner)) => Some(format!(
            "process {} is migrating the main checkout {} to a subvolume (it owns {}); wait for it to finish, then run this command again",
            owner.pid,
            root.display(),
            owner.path.display()
        )),
        Err(error) => Some(format!(
            "whether the main checkout {} is being migrated cannot be decided ({error})",
            root.display()
        )),
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
fn refuse(
    reasons: &mut Vec<Refusal>,
    kind: RefusalKind,
    text: String,
) {
    reasons.push(Refusal {
        kind,
        text,
    });
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
pub fn refusals(
    platform: &dyn Platform,
    candidate: &Candidate,
    owned: Owned,
    command_line_attributes: bool,
) -> Vec<Refusal> {
    let mut reasons: Vec<Refusal> = Vec::new();
    if candidate.admin.join("locked").exists() {
        refuse(
            &mut reasons,
            RefusalKind::Locked,
            format!(
                "the worktree is locked ({} exists); unlock it with `git worktree unlock` first",
                candidate.admin.join("locked").display()
            ),
        );
    }
    if !owned.index_lock && candidate.admin.join("index.lock").exists() {
        refuse(
            &mut reasons,
            RefusalKind::IndexLock,
            format!(
                "{} exists: a Git command is writing the index",
                candidate.admin.join("index.lock").display()
            ),
        );
    }
    if let Some(text) = migration_of_main_checkout(candidate.common.as_path()) {
        refuse(&mut reasons, RefusalKind::OtherCommand, text);
    }
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if !owned.stage && std::fs::symlink_metadata(&converting).is_ok() {
        refuse(
            &mut reasons,
            RefusalKind::ConvertingName,
            format!(
                "{} exists and no journal of this command owns it, so it is left alone; remove it yourself once you know what it is",
                converting.display()
            ),
        );
    }
    match platform.filesystem_facts(candidate.path.as_path()) {
        Ok((kind, _device)) if kind == crate::fs_kind::FilesystemKind::Btrfs => {}
        Ok(_) => refuse(
            &mut reasons,
            RefusalKind::Filesystem,
            format!("{} is not on btrfs", candidate.path.display()),
        ),
        Err(error) => refuse(
            &mut reasons,
            RefusalKind::Filesystem,
            format!(
                "the filesystem of {} cannot be examined ({error})",
                candidate.path.display()
            ),
        ),
    }
    match registered_inside(candidate.common.as_path(), candidate.path.as_path()) {
        Ok(nested_worktrees) => {
            for nested in nested_worktrees {
                refuse(
                    &mut reasons,
                    RefusalKind::Nested,
                    format!(
                        "the worktree {} is registered inside it; convert or move that one first",
                        nested.display()
                    ),
                );
            }
        }
        Err(reason) => refuse(&mut reasons, RefusalKind::Nested, reason),
    }
    let mut environment = platform.conversion_environment();
    if command_line_attributes && environment.alternate_attribute_source.is_none() {
        environment.alternate_attribute_source = Some(String::from(
            "--attr-source or -c attr.tree was given on the command line",
        ));
    }
    if let Err(reason) = check_clean_tracked_by_content(
        candidate.path.as_path(),
        candidate.admin.as_path(),
        candidate.common.as_path(),
        &environment,
        owned.index_lock,
    ) {
        refuse(
            &mut reasons,
            RefusalKind::NotQuiet,
            format!("it is not quiet and unmodified: {reason}"),
        );
    }
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    if let Some(why) = scan.unavailable.as_ref() {
        refuse(
            &mut reasons,
            RefusalKind::Process,
            format!("the process table cannot be read ({why}), so no process can be ruled out"),
        );
    }
    for user in &scan.users {
        refuse(
            &mut reasons,
            RefusalKind::Process,
            format!(
                "process {} ({}) has its working directory or an open file inside it",
                user.pid,
                user.command.as_deref().unwrap_or("unknown")
            ),
        );
    }
    return reasons;
}

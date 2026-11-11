//! What:
//!  Whether a directory is one of this repository's linked worktrees,
//!  proved from the pointer and the registration Git wrote there.
//! Why:
//!  A record outlives the worktree it was written for:
//!  a service that died holds a claim until it runs out,
//!  and a person may remove the worktree and put their own directory at that path meanwhile.
//!  Two commands then rewrite state in that directory
//!  (the record of the worktree the wrapper made,
//!  and the delayed relink of a new worktree's copied package state),
//!  so both ask this first.
//!  Three facts prove it,
//!  and a `.git` pointer alone is none of them:
//!  the pointer must name a directory directly below this repository's `worktrees`,
//!  that registration's `gitdir` back-link must name the pointer back,
//!  and its `commondir` must be this repository.
//!  A copy of an old pointer in a replacement directory fails the back-link,
//!  a registration copied in from another repository fails `commondir`,
//!  and a `.git` that is no regular file
//!  (a named pipe waits inside `open(2)` for a writer that may never come)
//!  is refused at once instead of holding a command
//!  (`SPEC.md`,
//!  "Every wait has a bound").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const admin = registrationOf(commonDir, worktree); // throws: not this repository, or undecided for now
//! ```

use crate::hooks_path_values::without_trailing_newlines;
use crate::private_storage::{ReadRefusal, read_regular_file};
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// The most bytes one record of a registration may hold.
/// Why:
///  a pointer is one line, `gitdir: <path>`,
///  and the longest path this system allows is 4096 bytes,
///  so a longer file is not a record Git wrote.
const RECORD_LIMIT: u64 = 4096;

/// What:
///  Why a directory is not one of this repository's registrations,
///  or why that could not be decided now.
/// Why:
///   The two answers are different work for the caller:
/// a directory that is no worktree of this repository is left exactly as it is and the work that named it is done,
/// while a proof that could not be read for a reason that may pass
/// (a permission,
/// an input-output error,
/// a descriptor table that is full)
/// keeps the work queued to be tried again,
/// because nothing was decided about that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegistrationRefusal = { kind: 'not-this-repository'; reason: string } | { kind: 'undecided'; reason: string };
/// ```
#[derive(Debug)]
pub(crate) enum RegistrationRefusal {
    /// The directory is no worktree of this repository, whatever it holds.
    NotThisRepository(String),
    /// Nothing could be decided about the directory now.
    Undecided(String),
}

impl RegistrationRefusal {
    /// The words the person reads.
    pub(crate) fn into_words(self) -> String {
        return match self {
            RegistrationRefusal::NotThisRepository(reason) => reason,
            RegistrationRefusal::Undecided(reason) => reason,
        };
    }
}

/// What:
///  The bytes of one record of a registration,
///  read without waiting for a writer and without following a link to a record.
/// Why:
///   A registration is a handful of small files Git writes,
/// and a name that holds a named pipe would wait inside `open(2)` for a writer that may never come.
/// A missing record and a record that is no regular file are this directory not being a registration Git wrote,
/// while a read that failed another way may pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = readRecord(path, "the worktree's pointer"); // throws the refusal
/// ```
pub(crate) fn read_record(path: &Path, what: &str) -> Result<Vec<u8>, RegistrationRefusal> {
    #[cfg(test)]
    if a_control_made_the_record_unreadable() {
        return Err(RegistrationRefusal::Undecided(format!("{what} ({}) cannot be read now (a control's own answer)", path.display())));
    }
    return match read_regular_file(path, RECORD_LIMIT) {
        Ok(bytes) => Ok(bytes),
        Err(ReadRefusal::Missing) => Err(RegistrationRefusal::NotThisRepository(format!("{what} ({}) does not exist", path.display()))),
        Err(ReadRefusal::NotRegular) => {
            return Err(RegistrationRefusal::NotThisRepository(format!("{what} ({}) is no regular file, and Git writes no such record", path.display())))
        }
        Err(ReadRefusal::TooLarge) => {
            return Err(RegistrationRefusal::NotThisRepository(format!("{what} ({}) holds more than {RECORD_LIMIT} bytes, and Git writes no such record", path.display())))
        }
        Err(ReadRefusal::Io(error)) => Err(RegistrationRefusal::Undecided(format!("{what} ({}) cannot be read ({error})", path.display()))),
    };
}

/// What:
///  Whether an error says the path is not there,
///  whatever the system called it.
/// Why:
///   A directory that is gone,
/// a component that is a file and a link that loops all name no registration of this repository,
/// while a permission or an input-output error may pass and is a different answer for the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absent = (error) => ['ENOENT', 'ENOTDIR', 'ELOOP'].includes(error.code);
/// ```
pub(crate) fn is_absent(error: &std::io::Error) -> bool {
    return matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory) || error.raw_os_error() == Some(libc::ELOOP);
}

#[cfg(test)]
thread_local! {
    /// A control makes the next record read answer that nothing could be decided now.
    pub(crate) static THE_RECORD_READ_ANSWERS_UNDECIDED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// What:
///  Whether a control has made the next record read answer that nothing could be decided now.
/// Why:
///   A read the system refuses (a permission, an input-output error) is the shape of that answer,
///  and the gate runs this suite as root,
///  where no mode makes a file unreadable,
///  so the control makes the read answer instead and the product's own mapping of the answer is what it proves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theControlAnswersUndecided = () => theRecordReadAnswersUndecided;
/// ```
#[cfg(test)]
fn a_control_made_the_record_unreadable() -> bool {
    return THE_RECORD_READ_ANSWERS_UNDECIDED.with(std::cell::Cell::get);
}

/// What:
///  A path of a registration resolved,
///  with what is not there kept apart from what could not be decided now.
/// Why:
///   A path that is gone names no registration of this repository,
/// while a path the system refused to resolve for another reason may be there a moment later,
/// and the work that named it is then tried again instead of dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolved = resolveRegistrationPath(path); // throws the refusal
/// ```
fn resolve(path: &Path) -> Result<PathBuf, RegistrationRefusal> {
    return match std::fs::canonicalize(path) {
        Ok(resolved) => Ok(resolved),
        Err(error) if is_absent(&error) => Err(RegistrationRefusal::NotThisRepository(format!("{} does not exist", path.display()))),
        Err(error) => Err(RegistrationRefusal::Undecided(format!("{} cannot be resolved ({error})", path.display()))),
    };
}

/// What:
///  The path one record of the registration `admin` holds,
///  resolved against the registration when it is written relative to it.
/// Why:
///   Git writes the worktree's own pointer as an absolute path and `../..` for the common directory,
/// and a registration another command wrote may hold either spelling,
/// so both are read as the files they name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = namedInTheRegistration(admin, 'commondir', "the registration's common directory");
/// ```
fn named_in_the_registration(admin: &Path, name: &str, what: &str) -> Result<PathBuf, RegistrationRefusal> {
    let record: PathBuf = admin.join(name);
    let bytes: Vec<u8> = read_record(record.as_path(), what)?;
    let named: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(without_trailing_newlines(bytes.as_slice())));
    return Ok(if named.is_absolute() { named } else { admin.join(named) });
}

/// What:
///  The registration the pointer bytes of `worktree` name,
///  proved to belong to that worktree and to this repository.
/// Why:
///   The pointer alone proves nothing:
/// a person may remove a worktree,
/// keep a copy of the pointer it had,
/// and put their own directory at that path.
/// The registration Git writes beside a worktree names the worktree's own pointer in its `gitdir` back-link
/// and the repository it belongs to in its `commondir`,
/// so both are read and compared before a caller rewrites any state there.
/// A caller that addresses the worktree through a directory held open reads the pointer through that directory
/// and hands the bytes in,
/// so the proof and its words still name the recorded path.
/// Only the trailing line breaks Git's own reader takes are taken,
///  so a directory whose name ends in a space is the registration the pointer names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const admin = registrationOfPointer(commonDir, worktree, pointerBytes); // throws the reason
/// ```
pub(crate) fn registration_of_pointer(common_dir: &Path, worktree: &Path, pointer: &[u8]) -> Result<PathBuf, RegistrationRefusal> {
    let pointer_path: PathBuf = worktree.join(".git");
    let line: &[u8] = without_trailing_newlines(pointer);
    let Some(named) = line.strip_prefix(b"gitdir: ") else {
        return Err(RegistrationRefusal::NotThisRepository(format!("{} does not start with `gitdir: `", pointer_path.display())));
    };
    let named_path: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(named));
    let admin: PathBuf = resolve(named_path.as_path())?;
    let worktrees: PathBuf = resolve(common_dir.join("worktrees").as_path())?;
    if admin.parent() != Some(worktrees.as_path()) {
        return Err(RegistrationRefusal::NotThisRepository(format!("{} is not one of the registrations in {}", admin.display(), worktrees.display())));
    }
    let back_link: PathBuf = named_in_the_registration(admin.as_path(), "gitdir", "the registration's back-link")?;
    if !crate::real_git_candidate::same_file(back_link.as_path(), pointer_path.as_path()) {
        return Err(RegistrationRefusal::NotThisRepository(format!(
            "the registration {} names {} as its worktree and not {}, so {} is no worktree it belongs to",
            admin.display(),
            back_link.display(),
            pointer_path.display(),
            worktree.display()
        )));
    }
    let common_record: PathBuf = named_in_the_registration(admin.as_path(), "commondir", "the registration's common directory")?;
    if !crate::real_git_candidate::same_file(common_record.as_path(), common_dir) {
        return Err(RegistrationRefusal::NotThisRepository(format!(
            "the registration {} belongs to {} and not to {}",
            admin.display(),
            common_record.display(),
            common_dir.display()
        )));
    }
    return Ok(admin);
}

/// What:
///  The administrative directory of the worktree at `worktree`,
///  proved to be one of this repository's registrations.
/// Why:
///   The record of a wrapper-made worktree must land beside that worktree's registration,
///  and the tools of a delayed relink rewrite the link layout of this repository's worktree at the recorded path,
///  so a directory whose pointer names anything else is not one this repository's wrapper may change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const admin = registrationOf(commonDir, worktree); // throws the reason
/// ```
pub(crate) fn registration_of(common_dir: &Path, worktree: &Path) -> Result<PathBuf, RegistrationRefusal> {
    let pointer: Vec<u8> = read_record(worktree.join(".git").as_path(), "the worktree's pointer")?;
    return registration_of_pointer(common_dir, worktree, pointer.as_slice());
}

/// Registration controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_registration_proof_tests.rs"]
mod tests;

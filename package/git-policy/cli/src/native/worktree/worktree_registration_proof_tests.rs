//! What:
//!  Controls for the proof that a directory is one of this repository's linked worktrees.
//! Why:
//!   Two commands rewrite state in the directory a record names,
//!  so a proof that a copy of an old pointer or a registration of another repository satisfies
//!  would change a directory that record was never written for,
//!  and a read that waits for a writer of a named pipe would hold the command for ever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(registrationOf(commonDir, worktree)).toEqual(admin);
//! ```

use crate::test_support::{fixture, git, repository};
use crate::worktree_registration_proof::{RegistrationRefusal, registration_of};
use std::path::PathBuf;
use std::time::Duration;

/// How long a control waits for a proof that must answer without waiting for anything,
/// before it releases whatever the proof is waiting for and reports.
const BOUND: Duration = Duration::from_secs(5);

/// A repository and one linked worktree of it,
/// made by real Git.
struct Added {
    /// The repository's common Git directory.
    common: PathBuf,
    /// The worktree.
    worktree: PathBuf,
}

/// What:
///  A repository with one linked worktree.
/// Why:
///   The proof reads the registration Git just wrote,
/// so the fixture is Git's own output and not a directory this control built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = aRepositoryWithOneWorktree(name);
/// ```
fn a_repository_with_one_worktree(name: &str) -> Added {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let worktree: PathBuf = root.join("worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    return Added { common: main.join(".git"), worktree };
}

/// The one registration below a common directory.
fn only_registration(common: &std::path::Path) -> PathBuf {
    let found: Vec<PathBuf> = std::fs::read_dir(common.join("worktrees"))
        .expect("the registrations")
        .flatten()
        .map(|entry| { return entry.path() })
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    return found.into_iter().next().expect("one registration");
}

/// Make a named pipe at `path`, which blocks whoever opens it for reading until a writer comes.
fn make_a_named_pipe(path: &std::path::Path) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "mkfifo {}", path.display());
}

/// What:
///  The registration Git wrote for a worktree satisfies the proof.
/// Why:
///   The proof must accept what Git writes,
/// or an add stops recording its own worktree and a relink never runs.
#[test]
fn the_registration_git_wrote_satisfies_the_proof() {
    let added: Added = a_repository_with_one_worktree("proof-git-registration");
    let admin: PathBuf = only_registration(added.common.as_path());
    let proved: Result<PathBuf, RegistrationRefusal> = registration_of(added.common.as_path(), added.worktree.as_path());
    assert_eq!(proved.expect("Git's own registration"), admin);
}

/// What:
///  A directory whose pointer names a registration that does not point back at it is no worktree of the repository.
/// Why:
///   A person may remove a worktree, put their own directory at that path,
///  and leave a copy of the old pointer there.
///  Only the registration's `gitdir` back-link, which Git writes beside the registration and which names the
///  worktree's own pointer, tells that copy from the worktree the registration belongs to.
#[test]
fn a_pointer_without_its_registrations_back_link_is_no_worktree() {
    let added: Added = a_repository_with_one_worktree("proof-no-back-link");
    let admin: PathBuf = only_registration(added.common.as_path());
    std::fs::remove_file(admin.join("gitdir")).expect("the back-link is removed");
    let proved: Result<PathBuf, RegistrationRefusal> = registration_of(added.common.as_path(), added.worktree.as_path());
    assert!(matches!(proved, Err(RegistrationRefusal::NotThisRepository(_))), "a registration with no back-link answered for this directory: {proved:?}");
}

/// What:
///  A directory whose registration names another worktree is no worktree of this one.
/// Why:
///   A registration is a directory below this repository's `worktrees`,
///  so a pointer a person copied from another worktree still names one of them;
///  the back-link is what says which worktree that registration belongs to.
#[test]
fn a_registration_that_names_another_worktree_is_no_worktree_of_this_one() {
    let added: Added = a_repository_with_one_worktree("proof-other-worktree");
    let admin: PathBuf = only_registration(added.common.as_path());
    let elsewhere: PathBuf = added.worktree.join("somewhere-else");
    std::fs::create_dir_all(elsewhere.as_path()).expect("another directory");
    std::fs::write(admin.join("gitdir"), format!("{}\n", elsewhere.join(".git").display())).expect("a back-link to another directory");
    let proved: Result<PathBuf, RegistrationRefusal> = registration_of(added.common.as_path(), added.worktree.as_path());
    assert!(matches!(proved, Err(RegistrationRefusal::NotThisRepository(_))), "a back-link to another directory answered for this one: {proved:?}");
}

/// What:
///  A pointer whose registration belongs to another repository is no worktree of this one.
/// Why:
///   A registration copied in beside this repository's own would otherwise make its worktree look like one of
///  this repository's, and the registration's `commondir` is what says which repository it belongs to.
#[test]
fn a_registration_of_another_repository_is_no_worktree_of_this_one() {
    let added: Added = a_repository_with_one_worktree("proof-other-repository");
    let admin: PathBuf = only_registration(added.common.as_path());
    std::fs::write(admin.join("commondir"), b"../elsewhere\n").expect("a common directory that is not this repository's");
    let proved: Result<PathBuf, RegistrationRefusal> = registration_of(added.common.as_path(), added.worktree.as_path());
    assert!(matches!(proved, Err(RegistrationRefusal::NotThisRepository(_))), "a registration of another repository answered for this directory: {proved:?}");
}

/// What:
///  A pointer that is a named pipe is refused without waiting for a writer.
/// Why:
///   `std::fs::read` waits inside `open(2)` for a writer of a pipe,
///  so a malformed or replaced worktree pointer would stop the command that reads it for ever,
///  with nothing said and no bound
///  (`SPEC.md`,
///  "Every wait has a bound").
#[test]
fn a_pointer_that_is_a_named_pipe_is_refused_without_waiting() {
    use std::io::Write as _;
    let added: Added = a_repository_with_one_worktree("proof-pointer-pipe");
    let pointer: PathBuf = added.worktree.join(".git");
    let old_pointer: Vec<u8> = std::fs::read(pointer.as_path()).expect("the old pointer");
    std::fs::remove_file(pointer.as_path()).expect("the regular pointer is removed");
    make_a_named_pipe(pointer.as_path());
    let (sender, receiver) = std::sync::mpsc::channel::<Result<PathBuf, RegistrationRefusal>>();
    let asked_common: PathBuf = added.common.clone();
    let asked_worktree: PathBuf = added.worktree.clone();
    let worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _ = sender.send(registration_of(asked_common.as_path(), asked_worktree.as_path()));
    });
    let answered: Result<Result<PathBuf, RegistrationRefusal>, std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(BOUND);
    if answered.is_err() {
        // Release the reader so this control reports what it found instead of leaving a thread waiting for ever.
        let mut writer: std::fs::File = std::fs::OpenOptions::new().write(true).open(pointer.as_path()).expect("release the reader");
        writer.write_all(old_pointer.as_slice()).expect("write the pointer");
        drop(writer);
    }
    let proved: Result<PathBuf, RegistrationRefusal> = match answered {
        Ok(found) => found,
        Err(_) => receiver.recv_timeout(BOUND).expect("the released reader lets the read finish"),
    };
    worker.join().expect("join the read");
    assert!(matches!(proved, Err(RegistrationRefusal::NotThisRepository(_))), "a pointer that is no regular file answered as a registration: {proved:?}");
}

/// What:
///  An error that says the path is not there is kept apart from one that may pass.
/// Why:
///   A directory that is no worktree of this repository is left as it is and the work that named it is done,
/// while a permission or an input-output error keeps that work queued for another try.
/// The gate runs this suite as root, where no file is unreadable for its mode,
/// so the answers are asked of the classification directly.
#[test]
fn an_absent_path_is_not_the_same_answer_as_a_read_that_may_pass() {
    for kind in [std::io::ErrorKind::NotFound, std::io::ErrorKind::NotADirectory] {
        assert!(crate::worktree_registration_proof::is_absent(&std::io::Error::from(kind)), "{kind:?} names no path");
    }
    assert!(crate::worktree_registration_proof::is_absent(&std::io::Error::from_raw_os_error(libc::ELOOP)), "a link that loops names no path");
    for code in [libc::EACCES, libc::EIO, libc::EMFILE, libc::ENOSPC, libc::ESTALE] {
        assert!(
            !crate::worktree_registration_proof::is_absent(&std::io::Error::from_raw_os_error(code)),
            "{code} may pass"
        );
    }
}

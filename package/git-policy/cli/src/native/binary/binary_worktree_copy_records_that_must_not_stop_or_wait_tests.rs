//! What:
//!  What a stage, a claim or a record of an interrupted ignored-state copy may do to the command that meets it:
//!  nothing that stops it and nothing that holds it.
//! Why:
//!  Every forwarded command in a linked worktree reads these records before Git runs,
//! so a folder this command may not list,
//!  a record that is a named pipe,
//!  and a claim a backup tool gave a second name
//!  are each a way to stop or hold every command of a repository for ever.
//! A folder a command cannot judge is named and the command goes on;
//! a record is read through the bounded plain-file reader;
//! a claim is judged by what it says and not by how many names it has.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const state of [unlistable, pipeAtIntended, pipeAtKept, twoNames]) { expect(runWrapped(wrappedGit, ['status']).status).toBe(0); }
//! ```

/// The shared fixtures and the bounded process helpers.
use super::support::{Fixture, Observed, fixture, remove, repository, run_wrapped, stderr_of};
use git_policy_cli::worktree_copy_journal::journal_root;
use git_policy_cli::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry};
use git_policy_cli::worktree_copy_stage_intended_entries::{INTENDED_NAME, kept_record_beside, record_intended_entries};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The folder a copy fills inside its stage,
/// as the wrapper writes it.
const PAYLOAD_FOLDER: &str = "payload";

/// A repository with one linked worktree,
/// and the common directory the records live under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { fixture, one, common } = aLinkedWorktree('...');
/// ```
fn a_linked_worktree(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let added: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "one", "../one"]);
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let one: PathBuf = std::fs::canonicalize(fixture.root.join("one")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    return (fixture, one, common);
}

/// A dead copy's claim beside a destination,
/// its stage folder holding one payload entry the copy recorded,
/// and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { claim, container } = aDeadCopysStage(common, worktrees);
/// ```
fn a_dead_copy_stage(common: &Path, worktrees: &Path, name: &str) -> (PathBuf, PathBuf) {
    let destination: PathBuf = worktrees.join(name);
    std::fs::create_dir_all(destination.as_path()).expect("destination");
    let identifier: String = format!("{}-0000-4000-8000-{:012}", name.len(), std::process::id());
    let container: PathBuf = worktrees.join(format!(".cli-git-worktree-copy-{identifier}"));
    std::fs::create_dir(container.as_path()).expect("the stage");
    std::fs::set_permissions(container.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private stage");
    let payload: PathBuf = container.join(PAYLOAD_FOLDER);
    std::fs::create_dir_all(payload.join("ignored").as_path()).expect("the payload");
    std::fs::write(payload.join("ignored/index.js"), b"x\n").expect("a copied entry");
    let staged: WorktreeCopyEntry = WorktreeCopyEntry { kind: EntryKind::File, mode: 0o644, relative_path: b"ignored/index.js".to_vec() };
    record_intended_entries(container.as_path(), [staged].as_slice()).expect("the record of what the payload holds");
    let folder: PathBuf = journal_root(common);
    std::fs::create_dir_all(folder.as_path()).expect("the journal root");
    let parent: PathBuf = folder.parent().expect("the parent").to_path_buf();
    std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private parent");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private journal root");
    let claim_file: PathBuf = folder.join(format!("{identifier}.stage"));
    let text: String = format!("{}\ncontainerHex={}\n", container.display(), hex_of(container.as_path()));
    std::fs::write(claim_file.as_path(), text.as_bytes()).expect("the claim of a dead copy");
    std::fs::set_permissions(claim_file.as_path(), std::fs::Permissions::from_mode(0o600)).expect("private claim");
    return (claim_file, container);
}

/// The bytes of a path as the lowercase hexadecimal a claim's container line holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexOf = (path) => Buffer.from(path).toString('hex')
/// ```
fn hex_of(path: &Path) -> String {
    use std::os::unix::ffi::OsStrExt as _;
    let mut hex: String = String::with_capacity(path.as_os_str().as_bytes().len() * 2);
    for byte in path.as_os_str().as_bytes() {
        hex.push_str(format!("{byte:02x}").as_str());
    }
    return hex;
}

/// A stage folder this account owns and may not list is named and the command goes on,
/// on that command and on the next one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // chmod 000 the stage; expect(run('status').status).toBe(0); expect(stderr).toContain('Permission denied');
/// ```
#[test]
fn a_stage_folder_this_account_cannot_list_keeps_the_command_going() {
    let (fixture, one, common) = a_linked_worktree("copy-unlistable-stage");
    let worktrees: PathBuf = common.parent().expect("the repository's parent").to_path_buf();
    let (_claim_file, container): (PathBuf, PathBuf) = a_dead_copy_stage(common.as_path(), worktrees.as_path(), "topic");
    std::fs::set_permissions(container.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a folder that cannot be listed");
    let first: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    let said: String = stderr_of(&first);
    // A command that cannot judge the folder names it and goes on, and nothing inside it is touched.
    assert_eq!(first.code, Some(0), "the command goes on: {said}");
    assert!(said.contains("was not changed") && said.contains("Permission denied"), "the line says what could not be read: {said}");
    std::fs::set_permissions(container.as_path(), std::fs::Permissions::from_mode(0o700)).expect("listable again");
    assert!(container.join(PAYLOAD_FOLDER).join("ignored/index.js").is_file(), "nothing of the folder that could not be judged was removed");
    // The next command can list it, and settles it as it settles any folder whose payload is the copy's own.
    let second: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    assert_eq!(second.code, Some(0), "the next command goes on too: {}", stderr_of(&second));
    assert!(!container.exists(), "the folder settles once it can be judged: {}", stderr_of(&second));
    remove(&fixture);
}

/// A named pipe where a record of a copy is read never holds a command:
/// at the record of what the payload holds,
/// at the record of why a stage was kept,
/// and at the record of the object a stage's creation made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // mkfifo the record; expect(run('status') finished in under a second and exited 0)
/// ```
#[test]
fn a_named_pipe_where_a_record_is_read_never_holds_a_command() {
    let (fixture, one, common) = a_linked_worktree("copy-pipe-records");
    let worktrees: PathBuf = common.parent().expect("the repository's parent").to_path_buf();
    let (claim_file, container): (PathBuf, PathBuf) = a_dead_copy_stage(common.as_path(), worktrees.as_path(), "topic");
    // The record of what the payload holds, as a pipe: the stage cannot be judged, so it is kept and named.
    std::fs::remove_file(container.join(INTENDED_NAME).as_path()).expect("the record goes");
    let pipe: std::ffi::CString = std::ffi::CString::new(container.join(INTENDED_NAME).as_os_str().as_encoded_bytes()).expect("a path without NUL");
    // SAFETY: the path is a NUL-terminated string that outlives the call and names a name inside the control's own stage.
    assert_eq!(unsafe { libc::mkfifo(pipe.as_ptr(), 0o600) }, 0, "a named pipe at the record's name");
    let at_the_record: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    assert_eq!(at_the_record.code, Some(0), "a pipe at the record of the payload does not hold the command: {}", stderr_of(&at_the_record));
    assert!(container.exists(), "the stage whose payload cannot be judged is kept");
    // The record of why a stage was kept, as a pipe beside the claim.
    let kept: PathBuf = kept_record_beside(claim_file.as_path());
    std::fs::remove_file(kept.as_path()).ok();
    let kept_pipe: std::ffi::CString = std::ffi::CString::new(kept.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    // SAFETY: the path is a NUL-terminated string that outlives the call and names a name inside the control's own journal root.
    assert_eq!(unsafe { libc::mkfifo(kept_pipe.as_ptr(), 0o600) }, 0, "a named pipe at the record of why a stage was kept");
    let at_the_kept: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    assert_eq!(at_the_kept.code, Some(0), "a pipe at the kept record does not hold the command: {}", stderr_of(&at_the_kept));
    // The record of the object a stage's creation made, as a pipe beside the claim.
    let made: PathBuf = claim_file.with_extension("made");
    let made_pipe: std::ffi::CString = std::ffi::CString::new(made.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    // SAFETY: the path is a NUL-terminated string that outlives the call and names a name inside the control's own journal root.
    assert_eq!(unsafe { libc::mkfifo(made_pipe.as_ptr(), 0o600) }, 0, "a named pipe at the record of the object made");
    let at_the_made: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    assert_eq!(at_the_made.code, Some(0), "a pipe at the record of the object made does not hold the command: {}", stderr_of(&at_the_made));
    for name in [kept.as_path(), made.as_path()] {
        std::fs::remove_file(name).expect("the control's pipe");
    }
    std::fs::remove_file(container.join(INTENDED_NAME).as_path()).expect("the control's pipe");
    std::fs::remove_dir_all(container.as_path()).expect("the control's stage");
    remove(&fixture);
}

/// A running add's claim that gains a second name is still that add's:
/// its stage is not reaped under a live process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // link the live claim to a second name; expect(stage).toExist(); expect(stderr).toSayNothingAboutRemoving)
/// ```
#[test]
fn a_claims_second_name_does_not_give_a_live_adds_stage_away() {
    let (fixture, one, common) = a_linked_worktree("copy-claim-two-names");
    let worktrees: PathBuf = common.parent().expect("the repository's parent").to_path_buf();
    let (claim_file, container): (PathBuf, PathBuf) = a_dead_copy_stage(common.as_path(), worktrees.as_path(), "topic");
    // The claim names a process that is running, as a live add's does: this control starts it and kills it by number.
    let mut owner: std::process::Child = std::process::Command::new("sleep").arg("60").spawn().expect("a live process");
    let pid: i64 = i64::from(owner.id());
    let birth: String = git_policy_cli::process::process_identity::process_birth_identity(pid).expect("readable").expect("the process runs");
    let token: String = format!("{pid}~{birth}~");
    let claim: &Path = claim_file.as_path();
    let text: String = std::fs::read_to_string(claim).expect("the claim");
    std::fs::write(claim, format!("{text}owner={token}\n").as_bytes()).expect("a claim of a live owner");
    std::fs::set_permissions(claim, std::fs::Permissions::from_mode(0o600)).expect("private claim");
    // The second name a backup tool gives it.
    let second: PathBuf = claim.with_extension("stage.backup");
    std::fs::hard_link(claim, second.as_path()).expect("a second name for the claim");
    let observed: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    let alive: bool = owner.try_wait().expect("the child is readable").is_none();
    let said: String = stderr_of(&observed);
    owner.kill().expect("the control's process is ended");
    owner.wait().expect("and reaped");
    assert_eq!(observed.code, Some(0), "the command goes on: {said}");
    assert!(alive, "the owner was running throughout");
    assert!(container.exists(), "a live add's stage is not reaped because its claim has two names: {said}");
    std::fs::remove_file(second.as_path()).expect("the control's second name");
    std::fs::remove_file(claim).expect("the control's claim");
    std::fs::remove_dir_all(container.as_path()).expect("the control's stage");
    remove(&fixture);
}

/// A directory that could not be listed is examined again once it can be,
/// so the reason a stage was kept is not believed after its cause is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // chmod 000 a payload directory; run; chmod 755; run; expect(the stage settled)
/// ```
#[test]
fn a_listing_that_failed_is_examined_again_once_it_can_be_made() {
    let (fixture, one, common) = a_linked_worktree("copy-listing-failed");
    let worktrees: PathBuf = common.parent().expect("the repository's parent").to_path_buf();
    let (_claim_file, container): (PathBuf, PathBuf) = a_dead_copy_stage(common.as_path(), worktrees.as_path(), "topic");
    let payload_directory: PathBuf = container.join(PAYLOAD_FOLDER).join("ignored");
    let outside: PathBuf = worktrees.join("a-person's-file");
    std::fs::write(outside.as_path(), b"a person's only copy").expect("a person's file");
    std::fs::set_permissions(payload_directory.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a directory that cannot be listed");
    let first: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    assert_eq!(first.code, Some(0), "the command goes on: {}", stderr_of(&first));
    assert!(stderr_of(&first).contains("was not changed"), "the stage is kept and named: {}", stderr_of(&first));
    assert!(container.exists(), "the stage stands");
    // The person fixes what stopped the look, and the next command examines the payload again and settles the stage.
    std::fs::set_permissions(payload_directory.as_path(), std::fs::Permissions::from_mode(0o700)).expect("listable again");
    let second: Observed = run_wrapped(&fixture, one.as_path(), &["status", "--short"]);
    assert_eq!(second.code, Some(0), "the next command goes on: {}", stderr_of(&second));
    assert!(!container.exists(), "the stage settles once the directory can be listed: {}", stderr_of(&second));
    assert!(outside.is_file(), "nothing of the person's in the fixture was touched");
    remove(&fixture);
}

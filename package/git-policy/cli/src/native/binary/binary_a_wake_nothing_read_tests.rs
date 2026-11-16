//! What:
//!  A wake no service read, through the built executable:
//!  the `git worktree add` that hands a relink over,
//!  and what the add does when the service at its socket greets and reads nothing of the wake.
//! Why:
//!   A hand-over whose answer was the write's own return left a new worktree whose copied links still named the source,
//!  and the line the person read said the commit service would relink it after the command returned
//!  (the review of 2026-11-13,
//!  "Out-of-scope observation").
//!  The record was durable,
//!  so nothing was lost,
//!  but the promise was kept by a later round and never by the command,
//!  and a worktree whose links name the source resolves the wrong packages without an error.
//!  These controls run the add against a stand-in service of the test's own,
//!  which greets and then reads nothing,
//!  so no control needs a loaded host to make a service stop reading.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const add = run(wrappedGit, ['worktree', 'add', '-b', 'one', '../one'], { XDG_RUNTIME_DIR: runtime }); expect(pnpmRecord()).toHaveLength(1);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, Observed, executable, fixture, git, observe_without_waiting_for_the_after_commit_work, repository, stderr_of, wrapped};
use git_policy_cli::service_address_of_a_worktree::socket_address_of;
use git_policy_cli::service_instance::instance_of;
use git_policy_cli::service_protocol::{PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_service};
use git_policy_cli::service_server::Bind;
use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The words of the line an add prints for work it handed over and did not do itself.
const HANDED_OVER_WORDS: &str = "because the commit service runs it after this command returns";

/// The words of the line an add prints for a relink it made at the new location itself.
const RELINKED_HERE_WORDS: &str = "work at its new location";

/// A repository whose tracked files hold a lockfile and whose ignored directory holds a `node_modules` to copy,
/// which is what makes the add hand a relink over.
fn repository_with_node_modules(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"node_modules/\n").expect("gitignore");
    std::fs::write(repo.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "pnpm-lock.yaml"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=lockfile"]);
    std::fs::create_dir_all(repo.join("node_modules/dep")).expect("node_modules");
    std::fs::write(repo.join("node_modules/dep/index.js"), b"module.exports = 1;\n").expect("dependency");
    return repo;
}

/// A fixture `pnpm` that appends the directory it ran in to a record and exits 0,
/// so a control can say whether pnpm ran and where.
fn fake_pnpm(fixture: &Fixture) -> PathBuf {
    let record: PathBuf = fixture.root.join("pnpm-record");
    executable(
        fixture.root.join("bin/pnpm").as_path(),
        format!("#!/bin/sh\necho \"$* | $(pwd -P)\" >> '{}'\nexit 0\n", record.display()).as_bytes(),
    );
    return record;
}

/// The lines the fixture's pnpm recorded, empty when it never ran.
fn recorded(record: &Path) -> Vec<String> {
    return std::fs::read_to_string(record).map(|text: String| { return text.lines().map(str::to_owned).collect(); }).unwrap_or_default();
}

/// What:
///  A stand-in service at `address` that greets every connection and reads nothing of it,
///  which is a service that has stopped reading inside its own process.
/// Why:
///   The state is held by the stand-in and not by this host's load,
///  so the control fails for the defect and never for a busy machine.
///  The wrapper reaches the address for a short runtime directory through the socket file a person's session keeps,
///  and no session bus is in the command's environment,
///  so nothing tries to start a real service beside the stand-in.
fn a_service_that_greets_and_reads_nothing(address: &Bind) -> std::thread::JoinHandle<()> {
    let Bind::Path(path) = address else {
        panic!("a short runtime directory keeps the socket file");
    };
    let service: UnixListener = UnixListener::bind(path).expect("the stand-in binds the wrapper's address");
    let _set = service.set_nonblocking(true);
    return std::thread::spawn(move || {
        let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) });
        greeting.push('\n');
        let until: std::time::Instant = std::time::Instant::now() + STAND_IN_PATIENCE;
        while std::time::Instant::now() < until {
            let Ok((stream, _address)) = service.accept() else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            (&stream).write_all(greeting.as_bytes()).expect("greet the client");
            // Held open and never read, which is what a service inside a stopped read leaves the client to find.
            std::thread::sleep(HELD_OPEN);
        }
    });
}

/// How long the stand-in holds each connection it greeted, and how long it serves at all.
const HELD_OPEN: Duration = Duration::from_secs(20);
const STAND_IN_PATIENCE: Duration = Duration::from_secs(25);

/// What:
///  Add a worktree through the built wrapper with a service at its address that greets and reads nothing,
///  and ask what the add did with the relink it had to hand over.
/// Why:
///   The worktree is usable only once its copied links name the new location,
///  and the add's own line says what it did with them:
///  the relink is either the service's work or the add's own.
fn add_while_the_service_reads_nothing(name: &str) -> (Observed, Vec<String>, Fixture) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fake_pnpm(&fixture);
    let runtime: PathBuf = super::support::short_runtime_directory(&fixture);
    let instance: String = instance_of(repo.join(".git").as_path());
    let address: Bind = socket_address_of(runtime.as_path(), instance.as_str());
    let stand_in: std::thread::JoinHandle<()> = a_service_that_greets_and_reads_nothing(&address);
    // The add's own answer is what this control reads, and a record the add hands over is not the service's to settle within
    //  this run: a control that waited for it would wait on the state it is asking about.
    let created: Observed = observe_without_waiting_for_the_after_commit_work(
        wrapped(&fixture).env("XDG_RUNTIME_DIR", runtime.as_path()).current_dir(repo.as_path()).args(["worktree", "add", "-b", "one", "../one"]),
        b"",
    );
    // The stand-in serves for its own bound, and a control that waited for it would be as long as that bound.
    drop(stand_in);
    return (created, recorded(record.as_path()), fixture);
}

/// What:
///  A service that greets and reads nothing of the wake leaves the relink to the add,
///  which relinks the worktree itself and says so.
/// Why:
///   The record is durable,
///  so nothing is lost either way;
///  what the person was promised is that the links work at the new location before their next command there,
///  and a wake the service did not read is no promise of that.
///  A worktree whose `node_modules` still names the source resolves the source's packages,
///  which is wrong without an error,
///  so the add does the work it cannot hand over.
///  The add still succeeds and still says what it did:
///  a repair never fails an add,
///  and a person who reads the line must be able to tell a relink that happened from one that waits.
#[test]
fn a_wake_the_service_never_read_answers_with_the_add_relinking_the_worktree_itself() {
    let (created, ran, fixture) = add_while_the_service_reads_nothing("wake-not-read-relink");
    let printed: String = stderr_of(&created);
    assert_eq!(created.code, Some(0), "{printed}");
    assert!(!printed.contains(HANDED_OVER_WORDS), "the add may not say the commit service relinks a worktree no service was told about:\n{printed}");
    assert_eq!(ran.len(), 1, "the add relinked the worktree itself, so pnpm ran once here: {ran:?}\n{printed}");
    assert!(ran[0].ends_with(format!("| {}", fixture.root.join("one").display()).as_str()), "and it ran in the new worktree: {ran:?}");
    assert!(printed.contains(RELINKED_HERE_WORDS), "the add says the relink was made at its new location:\n{printed}");
    remove_settled(&fixture);
}

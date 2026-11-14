//! Controls of the note that names an instance's service:
//! its text,
//! its reading,
//! and the birth identity that tells a reused number apart.

use super::*;

/// A folder of this test's own under the build directory.
fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("pid-note-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// What:
///  A claimed note names this process,
///  and removing it leaves nothing.
/// Why:
///  The note is the only record of which process serves an instance,
/// and a note that is claimed and then removed must leave neither a claim nor a file.
#[test]
fn the_note_names_the_process_that_claimed_it() {
    let root = folder("written");
    let note = path(root.as_path(), "3f9a");
    claim(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("the note is claimed");
    let written = read(note.as_path()).expect("the note reads back");
    assert_eq!(written.pid, i32::try_from(std::process::id()).expect("a process number"));
    assert!(is_still_the_process(&written), "the identity the note holds is this process's");
    remove(note.as_path());
    assert!(read(note.as_path()).is_none(), "a removed note names nothing");
}

/// A note whose identity is another birth names a process that is gone,
/// although its number is this one's.
#[test]
fn a_note_with_another_birth_names_nothing() {
    let root = folder("reused");
    let note = path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), format!("{} linux:000000\n", std::process::id())).expect("a note of another birth");
    let written = read(note.as_path()).expect("the note reads back");
    assert!(!is_still_the_process(&written), "the same number with another birth is another process");
}

/// A text that is not a note names nothing,
/// so a file someone else put there is never a reason to signal a process.
#[test]
fn text_that_is_not_a_note_names_nothing() {
    assert!(parse("").is_none());
    assert!(parse("nonsense\n").is_none());
    assert!(parse("12 linux:99\n").is_some());
}

/// The environment variable that marks the child of the two-claimer control.
const CLAIMER_CHILD_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PID_NOTE_CLAIMER_CHILD";

/// The note the child claims.
const CLAIMER_NOTE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PID_NOTE_TO_CLAIM";

/// The file the child leaves once it has looked and before it publishes.
const CLAIMER_LOOKED_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PID_NOTE_LOOKED";

/// The file whose appearance lets the child publish.
const CLAIMER_RELEASE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PID_NOTE_RELEASE";

/// The file the child leaves its outcome in.
const CLAIMER_OUTCOME_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PID_NOTE_OUTCOME";

/// How long the parent and the child wait for each other before they give up.
const CLAIMER_PATIENCE: std::time::Duration = std::time::Duration::from_secs(30);

/// What the child of the two-claimer control does:
/// claim the note,
/// held between its look and its publish until the parent has claimed it first.
fn claim_as_the_second_process() {
    let note: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os(CLAIMER_NOTE_VARIABLE).expect("the note to claim"));
    let looked: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os(CLAIMER_LOOKED_VARIABLE).expect("where to say it looked"));
    let release: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os(CLAIMER_RELEASE_VARIABLE).expect("what lets it publish"));
    let outcome: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os(CLAIMER_OUTCOME_VARIABLE).expect("where its outcome goes"));
    a_control_runs_before_the_publish(move || {
        std::fs::write(&looked, b"").expect("the parent is told this claimer has looked");
        let began: std::time::Instant = std::time::Instant::now();
        while !release.exists() && began.elapsed() < CLAIMER_PATIENCE {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });
    let claimed: std::io::Result<()> = claim(note.as_path(), i32::try_from(std::process::id()).expect("a process number"));
    let said: &str = match claimed {
        Ok(()) => "claimed",
        Err(_refused) => "refused",
    };
    std::fs::write(&outcome, said.as_bytes()).expect("the child leaves its outcome");
}

/// Waits,
/// bounded,
/// until the condition holds.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: std::time::Instant = std::time::Instant::now();
    while !condition() {
        assert!(began.elapsed() < CLAIMER_PATIENCE, "never happened: {what}");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// What:
///  Two processes that claim one instance leave exactly one owner.
/// Why:
///  The fifth review of issue 6293 (finding 2):
/// a look,
/// a write and a read back were not one step,
/// so two services could both read a note that named nobody,
/// each write its own name after the other's,
/// and both read their own writing back.
/// The order is the code's own seam:
/// the second claimer is held between its look and its publish until the first has claimed the instance.
#[test]
fn two_claimers_at_once_leave_one_owner() {
    if std::env::var_os(CLAIMER_CHILD_VARIABLE).is_some() {
        return claim_as_the_second_process();
    }
    let root = folder("two-claimers");
    let note = path(root.as_path(), "3f9a");
    let looked: std::path::PathBuf = root.join("looked");
    let release: std::path::PathBuf = root.join("release");
    let outcome: std::path::PathBuf = root.join("outcome");
    let mut child: std::process::Child = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"))
        .args([
            "--exact",
            "service::service_server_pid_note_of_an_instance::tests::two_claimers_at_once_leave_one_owner",
            "--nocapture",
        ])
        .env(CLAIMER_CHILD_VARIABLE, "1")
        .env(CLAIMER_NOTE_VARIABLE, note.as_os_str())
        .env(CLAIMER_LOOKED_VARIABLE, looked.as_os_str())
        .env(CLAIMER_RELEASE_VARIABLE, release.as_os_str())
        .env(CLAIMER_OUTCOME_VARIABLE, outcome.as_os_str())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the second claimer starts");
    wait_until("the second claimer looked at the note", &|| { return looked.exists(); });
    // The first claimer takes the instance while the second is held between its look and its publish.
    claim(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("the first claimer takes the instance");
    std::fs::write(&release, b"").expect("the second claimer may publish now");
    let began: std::time::Instant = std::time::Instant::now();
    while child.try_wait().expect("the second claimer's state").is_none() && began.elapsed() < CLAIMER_PATIENCE {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let _killed = child.kill();
    let _reaped = child.wait();
    let said: String = std::fs::read_to_string(&outcome).expect("the second claimer left its outcome");
    assert_eq!(said, "refused", "the second claimer does not believe it holds an instance the first does");
    let held: ServerProcess = read(note.as_path()).expect("the note still names its owner");
    assert_eq!(held.pid, i32::try_from(std::process::id()).expect("a process number"), "the note still names the first claimer");
    assert!(is_still_the_process(&held), "and that process is this one");
}

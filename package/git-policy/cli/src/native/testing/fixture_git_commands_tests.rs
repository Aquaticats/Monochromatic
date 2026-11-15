//! What:
//!  The controls of the bounded real-Git query in `fixture_git_commands.rs`:
//!  a descendant that holds one of the query's pipes,
//! and endings that are the query's own.
//! Why:
//!  The bound is the helper's own promise, so two things must hold:
//! a pipe some other process still holds cannot hold the answer past the bound,
//! and no status a query comes back with is read as the bound's doing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test("a descendant holding the pipe", () => { const answer = runUnderBound(setsidSleep, 1); expect(answer.heldOpen).toBe(true); });
//! ```

/// The bounded query under test and the grace it answers within.
use super::{FIXTURE_GIT_BOUND, PIPE_GRACE, QueryAnswer, QueryEnding, REAPING, arm_the_ending_seam, disarm_the_ending_seam, real_git_within};
/// The disposable fixture directory and its removal.
use crate::test_support::{fixture, remove};
/// `OsString` is owned operating-system text, which a query's arguments are.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// The clock a control measures its answer with.
use std::time::{Duration, Instant};
/// `signal` reads which signal ended a query.
use std::os::unix::process::ExitStatusExt;

/// The shell every fixture program of these controls is run by.
const SHELL: &str = "/bin/sh";

/// One executable script in the fixture's directory.
fn script(root: &Path, name: &str, body: &str) -> PathBuf {
    let path: PathBuf = root.join(name);
    std::fs::write(path.as_path(), body).expect("the fixture script");
    let mut permissions: std::fs::Permissions = std::fs::metadata(path.as_path()).expect("the script's metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o700);
    std::fs::set_permissions(path.as_path(), permissions).expect("the script runs");
    return path;
}

/// The arguments that run `source` through the shell.
fn shell_arguments(source: &str) -> Vec<OsString> {
    return vec![OsString::from("-c"), OsString::from(source)];
}

/// A descendant in a session of its own that holds the query's output pipe
/// cannot hold the answer past the bound, and the answer says what it left.
///  The descendant sleeps longer than the answer may take and shorter than the run's watchdog,
/// so a helper that waited for it fails this control instead of hanging the run.
///  No control sends it a signal: a number read back from a file is not a process this run started,
/// and the fixture's own teardown owns the directory the descendant works in.
#[test]
fn a_query_whose_descendant_holds_its_pipe_answers_at_its_bound_and_says_so() {
    let root: PathBuf = fixture("real-git-within-descendant");
    let query: PathBuf = script(
        root.as_path(),
        "query-with-a-descendant",
        "#!/bin/sh\n/usr/bin/setsid /bin/sleep 4 &\nwait\n",
    );
    let bound: Duration = Duration::from_secs(1);
    let began: Instant = Instant::now();
    let answer: QueryAnswer = real_git_within(query.as_path(), Some(root.as_path()), &[], &[], None, bound);
    let elapsed: Duration = began.elapsed();
    assert_eq!(
        answer.ending,
        QueryEnding::AtTheBound,
        "the one-second bound ended a query that never ends by itself, after {elapsed:?}"
    );
    assert!(answer.held_open, "the descendant held the query's pipe, so the answer must say so: {answer:?}");
    assert!(
        elapsed < bound + PIPE_GRACE + Duration::from_millis(500),
        "the answer waited past its bound of {bound:?} and its grace of {PIPE_GRACE:?}: {elapsed:?}"
    );
    remove(root.as_path());
}

/// An input larger than a pipe holds, written to a query that never reads it,
/// does not hold the answer past the bound:
///  the bound ends the query,
/// the write that the ending broke is part of that ending,
/// and the answer says so.
///  The bound is long enough that the machine's speed cannot be what fails this control:
/// the defect it guards against never returns at all,
/// and the answer it asks for arrives within the bound, the reaping window and five seconds of slack
/// whatever else the host is doing.
#[test]
fn an_input_a_query_never_reads_does_not_hold_the_answer_past_its_bound() {
    let root: PathBuf = fixture("real-git-within-stdin");
    let bound: Duration = Duration::from_secs(2);
    // A pipe holds 64 KiB by default, so this input fills one many times over.
    let input: Vec<u8> = vec![b'x'; 1024 * 1024];
    let words: Vec<OsString> = shell_arguments("exec /bin/sleep 60");
    let began: Instant = Instant::now();
    let answer: QueryAnswer = real_git_within(Path::new(SHELL), Some(root.as_path()), &[], words.as_slice(), Some(input.as_slice()), bound);
    let elapsed: Duration = began.elapsed();
    assert_eq!(
        answer.ending,
        QueryEnding::AtTheBound,
        "a query that never reads its input and never ends by itself is ended by the bound; the answer came back after {elapsed:?}"
    );
    assert!(!answer.held_open, "nothing of this query outlives its ending: {answer:?}");
    assert!(
        elapsed < bound + REAPING + Duration::from_secs(5),
        "the answer waited past its bound of {bound:?}, its reaping window of {REAPING:?} and five seconds of slack: {elapsed:?}"
    );
    remove(root.as_path());
}

/// The query's own exit in the window between the bound's end and the signal
/// is the query's ending, and this helper reports it as its own.
///  The seam places that exit there: the query ends by itself with a status of its own while the seam waits for it
/// without reaping it, and the signal then reaches a query that has already ended.
///  Nothing in this control is a race: the seam returns only once the kernel says the query has exited,
/// so the machine's speed cannot decide the answer.
#[test]
fn a_query_that_ends_by_itself_before_the_signal_is_its_own_ending() {
    let root: PathBuf = fixture("real-git-within-ending-window");
    let bound: Duration = Duration::from_millis(100);
    // The query ends by itself a second after it starts, so the bound has certainly passed when the seam runs.
    let words: Vec<OsString> = shell_arguments("exec /bin/sleep 1");
    arm_the_ending_seam(|query: libc::pid_t| {
        let mut information: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: `waitid` writes the one `siginfo_t` it is given for the child this test started,
        // and `WNOWAIT` leaves that child unreaped for the helper's own wait.
        let waited: i32 = unsafe { libc::waitid(libc::P_PID, query as libc::id_t, &mut information, libc::WEXITED | libc::WNOWAIT) };
        assert_eq!(waited, 0, "the query's own ending is waited for");
    });
    let began: Instant = Instant::now();
    let answer: QueryAnswer = real_git_within(Path::new(SHELL), Some(root.as_path()), &[], words.as_slice(), None, bound);
    let elapsed: Duration = began.elapsed();
    assert!(!disarm_the_ending_seam(), "the query ended before the bound, so the seam never ran");
    assert_eq!(
        answer.ending,
        QueryEnding::Itself,
        "the query ended by itself with its own status, so the ending is its own: {answer:?} after {elapsed:?}"
    );
    assert_eq!(answer.output.status.code(), Some(0), "the query's own status: {answer:?}");
    remove(root.as_path());
}

/// A query that answered and left its input to a descendant answers with that said,
/// and no answer the query gave is turned into a panic.
///  The query forks a descendant in a session of its own that inherits the three standard descriptors and sleeps;
/// the query itself leaves with status 0 while the writer of a one-mebibyte input is still blocked,
/// because no process reads that input and the descendant keeps the read end open.
#[test]
fn a_query_that_leaves_its_input_to_a_descendant_answers_with_that_said() {
    let root: PathBuf = fixture("real-git-within-input-held");
    let query: PathBuf = script(
        root.as_path(),
        "query-that-leaves-its-input",
        // The explicit redirections are what make the background descendant keep the three descriptors:
        // a shell gives a background job the null device unless the command redirects for itself.
        "#!/bin/sh\n/usr/bin/setsid /bin/sleep 8 0<&0 1>&1 2>&2 &\nexit 0\n",
    );
    // A pipe holds 64 KiB by default, so this input fills one many times over.
    let input: Vec<u8> = vec![b'x'; 1024 * 1024];
    let began: Instant = Instant::now();
    let answer: QueryAnswer = real_git_within(query.as_path(), Some(root.as_path()), &[], &[], Some(input.as_slice()), FIXTURE_GIT_BOUND);
    let elapsed: Duration = began.elapsed();
    assert_eq!(answer.ending, QueryEnding::Itself, "the query left with status 0 of its own: {answer:?}");
    assert_eq!(answer.output.status.code(), Some(0), "the query's own status: {answer:?}");
    assert!(answer.input_held, "a descendant of the query still holds its input, so the answer must say so: {answer:?}");
    assert!(answer.held_open, "the same descendant holds the output pipes: {answer:?}");
    assert!(
        elapsed < PIPE_GRACE + Duration::from_secs(5),
        "the answer waited past its grace of {PIPE_GRACE:?} and five seconds of slack: {elapsed:?}"
    );
    remove(root.as_path());
}

/// A query that ends by itself keeps its own ending:
///  an exit status of 124 or 137,
///  and a signal's death,
/// are none of them this helper's bound.
#[test]
fn a_query_that_ends_by_itself_keeps_its_own_ending() {
    let root: PathBuf = fixture("real-git-within-endings");
    let cases: [(&str, Option<i32>, Option<i32>); 3] = [
        ("exit 124", Some(124), None),
        ("exit 137", Some(137), None),
        ("kill -KILL $$", None, Some(libc::SIGKILL)),
    ];
    for (source, code, signal) in cases {
        let words: Vec<OsString> = shell_arguments(source);
        let began: Instant = Instant::now();
        let answer: QueryAnswer = real_git_within(Path::new(SHELL), Some(root.as_path()), &[], words.as_slice(), None, FIXTURE_GIT_BOUND);
        assert_eq!(
            answer.ending,
            QueryEnding::Itself,
            "{source}: the query ended by itself, long before the bound of {} s, which the answer reports instead",
            FIXTURE_GIT_BOUND.as_secs()
        );
        assert_eq!(answer.output.status.code(), code, "{source}");
        assert_eq!(answer.output.status.signal(), signal, "{source}");
        assert!(began.elapsed() < FIXTURE_GIT_BOUND, "{source}: the answer waited past the bound");
    }
    remove(root.as_path());
}

/// One query that writes, and ends, says what it wrote:
///  the bound does not turn its output into nothing.
#[test]
fn a_query_that_ends_by_itself_keeps_its_output() {
    let root: PathBuf = fixture("real-git-within-output");
    let words: Vec<OsString> = shell_arguments("printf 'answered\\n'");
    let answer: QueryAnswer = real_git_within(Path::new(SHELL), Some(root.as_path()), &[], words.as_slice(), None, FIXTURE_GIT_BOUND);
    assert_eq!(answer.ending, QueryEnding::Itself, "the query ends by itself");
    assert!(!answer.held_open, "nothing of this query outlives it");
    assert_eq!(String::from_utf8_lossy(&answer.output.stdout).as_ref(), "answered\n");
    assert_eq!(answer.output.status.code(), Some(0));
    remove(root.as_path());
}

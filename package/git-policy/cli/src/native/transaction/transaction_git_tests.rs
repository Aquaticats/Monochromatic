//! Controls for the transaction Git runner against real Git.

use super::*;
use crate::test_support::{ActionSink, SharedSink, executable, fixture, git_context, git_text, remove, repository, while_test_lives};
use crate::child_class::ChildClass;
use crate::wait_ceilings::{OBJECT_STEP_CEILING, PLUMBING_STEP_CEILING};
use crate::wait_progress::Cadence;

/// A function-like value with the right shape is a source of watch terms in these controls:
/// the shipped caller writes a small struct.
impl<Terms: Fn() -> WatchTerms> WatchTermsSource for Terms {
    fn terms(&self) -> WatchTerms {
        return self();
    }
}

/// Requests carry their arguments and nothing else by default.
#[test]
fn requests_start_empty() {
    let request: GitRequest = GitRequest::new(Path::new("/repo"), &["status", "--short"]);
    assert_eq!(request.cwd, PathBuf::from("/repo"));
    assert_eq!(
        request.arguments,
        vec![OsString::from("status"), OsString::from("--short")]
    );
    assert!(!request.without_prefix);
    assert_eq!(request.index, None);
    assert_eq!(request.object_directory, None);
    assert!(request.environment.is_empty());
    assert!(request.unset.is_empty());
    assert_eq!(request.input, None);
    assert_eq!(
        command_text(&[OsString::from("update-ref"), OsString::from("-d")]),
        "git update-ref -d"
    );
    assert_eq!(command_text(&[]), "git");
}

/// Output,
///  exit codes and input reach the caller;
///  failures name the command.
#[test]
fn runs_capture_output_and_feed_input() {
    let root: PathBuf = fixture("transaction-git-run");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let mut hashing: GitRequest =
        GitRequest::new(repo.as_path(), &["hash-object", "-w", "--stdin"]);
    hashing.input = Some(b"content\n".to_vec());
    let hashed: GitOutput = run_git_checked(&context, &hashing).expect("hash-object");
    assert!(hashed.succeeded());
    assert_eq!(
        String::from_utf8_lossy(hashed.stdout.as_slice()).trim(),
        "d95f3ad14dee633a758d2e331151e950dd13e4ed"
    );
    // Large input never deadlocks against Git's output.
    let mut large: GitRequest = GitRequest::new(repo.as_path(), &["hash-object", "--stdin"]);
    large.input = Some(vec![b'x'; 4 * 1024 * 1024]);
    assert!(run_git_checked(&context, &large).is_ok());
    let missing: GitRequest = GitRequest::new(
        repo.as_path(),
        &["rev-parse", "--verify", "--quiet", "refs/heads/none"],
    );
    let output: GitOutput = run_git(&context, &missing).expect("rev-parse runs");
    assert_eq!(output.code, Some(1));
    assert!(!output.succeeded());
    let failure: GitFailure = run_git_checked(
        &context,
        &GitRequest::new(
            repo.as_path(),
            &["cat-file", "-t", "0000000000000000000000000000000000000001"],
        ),
    )
    .expect_err("missing object");
    assert!(
        failure
            .to_string()
            .starts_with("git cat-file -t 0000000000000000000000000000000000000001 failed: fatal:"),
        "{failure}"
    );
    let unstartable: GitContext = GitContext {
        real_git: root.join("no-git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    let start: GitFailure =
        run_git_checked(&unstartable, &GitRequest::new(repo.as_path(), &["status"]))
            .expect_err("no Git");
    assert!(
        start.0.starts_with("git status could not start: "),
        "{start}"
    );
    remove(root.as_path());
}

/// The global prefix,
///  the overlay,
///  removals,
///  additions,
///  index and store reach Git in order.
#[test]
fn environment_and_prefix_reach_git() {
    let root: PathBuf = fixture("transaction-git-environment");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-c"), OsString::from("cli.probe=prefixed")];
    let prefixed: GitOutput = run_git_checked(
        &context,
        &GitRequest::new(repo.as_path(), &["config", "cli.probe"]),
    )
    .expect("prefix");
    assert_eq!(
        String::from_utf8_lossy(prefixed.stdout.as_slice()),
        "prefixed\n"
    );
    let mut bare: GitRequest = GitRequest::new(repo.as_path(), &["config", "cli.probe"]);
    bare.without_prefix = true;
    assert_eq!(run_git(&context, &bare).expect("unprefixed").code, Some(1));
    // `GIT_CONFIG_PARAMETERS` set by the overlay, then removed, then added back by the request.
    context.overlay.push((
        OsString::from("GIT_CONFIG_PARAMETERS"),
        OsString::from("'cli.probe'='overlay'"),
    ));
    let mut removed: GitRequest = GitRequest::new(repo.as_path(), &["config", "cli.probe"]);
    removed.without_prefix = true;
    removed.unset = vec![OsString::from("GIT_CONFIG_PARAMETERS")];
    assert_eq!(run_git(&context, &removed).expect("removed").code, Some(1));
    removed.environment = vec![(
        OsString::from("GIT_CONFIG_PARAMETERS"),
        OsString::from("'cli.probe'='request'"),
    )];
    let added: GitOutput = run_git_checked(&context, &removed).expect("added");
    assert_eq!(
        String::from_utf8_lossy(added.stdout.as_slice()),
        "request\n"
    );
    // A private index and object store receive the writes.
    let private_index: PathBuf = root.join("private.index");
    let store: PathBuf = root.join("objects");
    std::fs::create_dir(&store).expect("store");
    let mut update: GitRequest = GitRequest::new(
        repo.as_path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "100644,d95f3ad14dee633a758d2e331151e950dd13e4ed,new.txt",
        ],
    );
    update.without_prefix = true;
    update.index = Some(private_index.clone());
    run_git_checked(&context, &update).expect("private index write");
    assert!(private_index.exists());
    assert_eq!(git_text(repo.as_path(), &["ls-files"]), "");
    let mut write: GitRequest = GitRequest::new(repo.as_path(), &["hash-object", "-w", "--stdin"]);
    write.without_prefix = true;
    write.object_directory = Some(store.clone());
    write.input = Some(b"private\n".to_vec());
    let written: GitOutput = run_git_checked(&context, &write).expect("private object");
    let oid: String = String::from_utf8_lossy(written.stdout.as_slice())
        .trim()
        .to_string();
    assert!(store.join(&oid[..2]).join(&oid[2..]).exists());
    let unknown: GitOutput = run_git(
        &context,
        &GitRequest::new(repo.as_path(), &["cat-file", "-e", oid.as_str()]),
    )
    .expect("cat-file runs");
    assert!(!unknown.succeeded());
    assert_eq!(unknown.error_text(), "");
    remove(root.as_path());
}

/// Error text is trimmed and lossily decoded.
#[test]
fn error_text_is_trimmed() {
    let output: GitOutput = GitOutput {
        code: Some(128),
        stdout: Vec::new(),
        stderr: b"  fatal: \xff broken \n".to_vec(),
    };
    assert_eq!(output.error_text(), "fatal: \u{fffd} broken");
    assert!(
        !GitOutput {
            code: None,
            stdout: Vec::new(),
            stderr: Vec::new()
        }
        .succeeded()
    );
}

/// Input files are private and already unlinked.
#[cfg(unix)]
#[test]
fn input_files_leave_nothing_behind() {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let root: PathBuf = fixture("transaction-git-input");
    let mut file: std::fs::File = input_file(b"bytes", Some(root.as_path())).expect("input file");
    let metadata: std::fs::Metadata = file.metadata().expect("metadata");
    assert_eq!(metadata.nlink(), 0);
    assert_eq!(metadata.mode() & 0o777, 0o600);
    let mut read: Vec<u8> = Vec::new();
    file.read_to_end(&mut read).expect("read");
    assert_eq!(read, b"bytes");
    let left: Vec<std::ffi::OsString> = std::fs::read_dir(root.join("monochromatic-git-policy-cli-scratch"))
        .expect("the repository scratch directory exists")
        .map(|entry| { return entry.expect("entry").file_name(); })
        .collect();
    assert!(left.is_empty(), "nothing is left in the scratch directory: {left:?}");
    remove(root.as_path());
}

/// A request that carries input needs a repository:
/// without one the failure names the defect and writes nothing.
#[test]
fn input_without_a_repository_is_refused_naming_the_defect() {
    let error: std::io::Error = input_file(b"bytes", None).expect_err("no repository");
    assert!(error.to_string().contains("no repository scratch directory"), "{error}");
}

/// A common directory that does not exist is an error,
/// and the input file is never made elsewhere.
#[test]
fn input_in_a_missing_common_directory_fails_without_a_fallback() {
    let root: PathBuf = fixture("transaction-git-input-missing");
    let missing: PathBuf = root.join("no-such-common-directory");
    assert!(input_file(b"bytes", Some(missing.as_path())).is_err());
    assert!(!missing.exists());
    remove(root.as_path());
}

/// A fake Git that prints where its standard input lives,
/// which is the input file's name before it was unlinked.
#[cfg(target_os = "linux")]
const FAKE_GIT_STDIN_PATH: &[u8] = b"#!/bin/sh\nreadlink /proc/$$/fd/0\n";

/// The input a command gets is a file in the repository scratch directory of its context,
/// and is gone afterwards.
#[cfg(target_os = "linux")]
#[test]
fn a_command_reads_its_input_from_the_repository_scratch_directory() {
    let root: PathBuf = fixture("transaction-git-input-location");
    let program: PathBuf = root.join("fake-git");
    executable(program.as_path(), FAKE_GIT_STDIN_PATH);
    let common: PathBuf = root.join("common");
    std::fs::create_dir(&common).expect("common directory");
    let context: GitContext = GitContext {
        real_git: program,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: Some(common.clone()),
    };
    let mut request: GitRequest = GitRequest::new(root.as_path(), &["hash-object", "--stdin"]);
    request.input = Some(b"12345".to_vec());
    let output: GitOutput = run_git(&context, &request).expect("the fake runs");
    let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    let expected: String = format!("{}/monochromatic-git-policy-cli-scratch/cli-git-input-", common.display());
    assert!(named.starts_with(expected.as_str()), "{named} should start with {expected}");
    assert_eq!(std::fs::read_dir(common.join("monochromatic-git-policy-cli-scratch")).expect("scratch").count(), 0);
    remove(root.as_path());
}

/// A fake Git that counts its runs,
/// refuses the first `FAKE_REFUSALS` of them the way a host out of threads does,
///  and otherwise prints how many input bytes it read.
const FAKE_GIT: &[u8] = b"#!/bin/sh\n\
count=$(cat \"$FAKE_COUNT\" 2>/dev/null || echo 0)\n\
count=$((count + 1))\n\
echo $count > \"$FAKE_COUNT\"\n\
bytes=$(wc -c | tr -d ' ')\n\
if [ \"$count\" -le \"$FAKE_REFUSALS\" ]; then\n\
  echo 'fatal: unable to create thread: Resource temporarily unavailable' >&2\n\
  exit 128\n\
fi\n\
if [ -n \"$FAKE_OTHER_FAILURE\" ]; then\n\
  echo 'fatal: bad object' >&2\n\
  exit 128\n\
fi\n\
echo \"read $bytes\"\n";

/// A context running the fake Git,
/// a request for `arguments` with the fake's counters,
/// and the count file.
fn fake_run(root: &Path, arguments: &[&str], refusals: u32, other_failure: bool) -> (GitContext, GitRequest, PathBuf) {
    let program: PathBuf = root.join("fake-git");
    executable(program.as_path(), FAKE_GIT);
    let count: PathBuf = root.join("count");
    let context: GitContext = GitContext {
        real_git: program,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: Some(root.to_path_buf()),
    };
    let mut request: GitRequest = GitRequest::new(root, arguments);
    request.input = Some(b"12345".to_vec());
    request.environment = vec![
        (OsString::from("FAKE_COUNT"), count.clone().into_os_string()),
        (OsString::from("FAKE_REFUSALS"), OsString::from(refusals.to_string())),
        (OsString::from("FAKE_OTHER_FAILURE"), OsString::from(if other_failure { "1" } else { "" })),
    ];
    return (context, request, count);
}

/// How many times the fake Git ran.
fn runs_of(count: &Path) -> u32 {
    return std::fs::read_to_string(count).map_or(0, |text| { return text.trim().parse().unwrap_or(0); });
}

/// A pause that records nothing and waits for nothing,
/// so the controls do not sleep.
fn no_pause(_wait: Duration) {}

/// A fake Git that runs until the file `FAKE_RELEASE` names exists,
/// and then answers;
/// it also ends when the test that started
///  it is gone,
/// so a failed test leaves no script waiting for a release nobody writes.
fn slow_git(root: &Path) -> String {
    return format!("#!/bin/sh\nwhile [ ! -e \"$FAKE_RELEASE\" ] && {}; do sleep 0.02; done\necho slow\n", while_test_lives(root));
}

/// A context running the slow fake Git and a request whose release file is `release`.
fn slow_run(root: &Path, release: &Path) -> (GitContext, GitRequest) {
    let program: PathBuf = root.join("slow-git");
    executable(program.as_path(), slow_git(root).as_bytes());
    // The request sends no input, so it needs no scratch directory.
    let context: GitContext =
        GitContext { real_git: program, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: None };
    let mut request: GitRequest = GitRequest::new(root, &["add", "--", "a.md"]);
    request.environment = vec![(OsString::from("FAKE_RELEASE"), release.as_os_str().to_os_string())];
    return (context, request);
}

/// How long the control below waits for the fake Git to be gone,
/// so a script that never ends fails it and never holds it.
const SCRIPT_GONE_PATIENCE: Duration = Duration::from_secs(20);

/// How often the control below looks at the fake Git.
const SCRIPT_GONE_LOOK: Duration = Duration::from_millis(50);

/// What:
///  A fake Git that waits for a release file ends by itself once the test that started it is gone,
/// however the test ended.
/// Why:
///   Product code starts these scripts,
/// so the test cannot kill them with a guard of its own;
/// one that waited for a release a
///  failed test never wrote ran on after the test,
/// in a fixture that was already removed.
/// The script watches its test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pid = await startThatThrows(() => { startFakeGit({ release }); throw new Error('the test fails'); });
/// await waitUntil(() => !isRunning(pid));
/// ```
#[test]
fn a_fake_git_waiting_for_a_release_ends_when_its_test_is_gone() {
    let (sender, receiver) = std::sync::mpsc::channel::<u32>();
    let handle = std::thread::spawn(move || {
        let root: PathBuf = fixture("transaction-git-script-dies");
        let release: PathBuf = root.join("release");
        let (context, request) = slow_run(root.as_path(), release.as_path());
        let mut command: std::process::Command = std::process::Command::new(&context.real_git);
        command.envs(request.environment.iter().map(|(name, value)| { return (name, value); })).current_dir(&root);
        let mut child: std::process::Child = command.spawn().expect("the fake Git starts");
        // The premise: the script is inside its wait loop when the test fails. Until the interpreter has started it, the process
        //  is still being executed and a removed fixture would take the script file from it, which is not the leak under test.
        let waiting = |number: u32| -> bool {
            return std::fs::read_to_string(format!("/proc/{number}/stat")).is_ok_and(|text| { return text.contains(") S "); });
        };
        let started: std::time::Instant = std::time::Instant::now();
        while !waiting(child.id()) {
            assert!(started.elapsed() < SCRIPT_GONE_PATIENCE, "the fake Git never reached its wait loop");
            std::thread::sleep(SCRIPT_GONE_LOOK);
        }
        sender.send(child.id()).expect("send the pid");
        // A look that does not wait: the script is still running when the test fails, as in the leak this control is for.
        let _still_running = child.try_wait();
        panic!("the test fails before it releases the fake Git");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let pid: u32 = receiver.recv().expect("the pid");
    // A zombie that nothing reaped (the test thread dropped its handle) has exited and runs nothing.
    let runs = |number: u32| { return std::fs::read_to_string(format!("/proc/{number}/stat")).is_ok_and(|text| { return !text.contains(") Z "); }); };
    let began: std::time::Instant = std::time::Instant::now();
    while runs(pid) && began.elapsed() < SCRIPT_GONE_PATIENCE {
        std::thread::sleep(SCRIPT_GONE_LOOK);
    }
    let left: bool = runs(pid);
    if left {
        // Tidy up the leak this control exists to catch, so a red run leaves nothing behind either.
        // SAFETY: `kill` takes a pid and a signal and touches no memory.
        unsafe { libc::kill(libc::pid_t::try_from(pid).expect("pid"), libc::SIGKILL) };
    }
    assert!(!left, "the fake Git of process {pid} outlived the test that started it");
}

/// Terms of a limit,
/// a quick cadence and the given sink.
fn quick_terms(limit: Duration, notices: Box<dyn std::io::Write + Send>) -> WatchTerms {
    return WatchTerms {
        limit,
        cadence: Cadence { grace: Duration::from_millis(20), interval: Duration::from_millis(20) },
        notices,
    };
}

/// The production entry runs a query under the query bound and any other step under the bulk bound:
///  the terms come from the request's own subcommand.
#[test]
fn the_terms_of_a_request_follow_its_subcommand() {
    let query: GitRequest = GitRequest::new(Path::new("."), &["rev-parse", "HEAD"]);
    let bulk: GitRequest = GitRequest::new(Path::new("."), &["pack-objects", "--stdout"]);
    assert_eq!(terms_for_request(&query).limit, ChildClass::Plumbing.limit());
    assert_eq!(terms_for_request(&bulk).limit, ChildClass::Object.limit());
}

/// A Git step that runs long is announced by name while it lasts,
/// and its output still comes back.
#[test]
fn a_git_step_that_runs_long_is_announced_by_name() {
    let root: PathBuf = fixture("transaction-git-slow");
    let release: PathBuf = root.join("release");
    let (context, request) = slow_run(root.as_path(), release.as_path());
    let sink: SharedSink = SharedSink::default();
    // The third notice releases the stand-in, so the step ends on that event, whatever the host's speed.
    let output: GitOutput = run_git_once_with(&context, &request, &|| {
        let release_file: PathBuf = release.clone();
        let counted: ActionSink = ActionSink::new(&sink, 3, move || {
            std::fs::write(&release_file, b"").expect("release the fake Git");
        });
        return quick_terms(Duration::from_secs(60), Box::new(counted));
    })
    .expect("finished");
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "slow\n");
    let text: String = sink.text();
    assert!(text.lines().count() >= 3, "{text}");
    assert!(
        text.starts_with(format!("monochromatic-git-policy-cli: still waiting for git add -- a.md, {} (process ", class_of(request.arguments.as_slice()).label()).as_str()),
        "{text}"
    );
    remove(root.as_path());
}

/// The quiet window of the controls below:
/// past the sampling interval of the progress meter.
const QUIET_WINDOW: Duration = Duration::from_millis(1200);

/// A fake Git that counts its runs in `FAKE_COUNT` and then sits silent for `FAKE_SILENT_SECONDS` before it answers `late`.
const SILENT_GIT: &[u8] = b"#!/bin/sh\n\
count=$(cat \"$FAKE_COUNT\" 2>/dev/null || echo 0)\n\
echo $((count + 1)) > \"$FAKE_COUNT\"\n\
if [ \"$count\" -lt \"$FAKE_HANGS\" ]; then sleep \"$FAKE_SILENT_SECONDS\"; fi\n\
echo late\n";

/// A context running the silent fake Git,
/// and a request for `arguments` that makes it sit silent in its first `hangs` runs.
fn silent_run(root: &Path, arguments: &[&str], hangs: u32, silent_seconds: u64) -> (GitContext, GitRequest, PathBuf) {
    let program: PathBuf = root.join("silent-git");
    executable(program.as_path(), SILENT_GIT);
    let count: PathBuf = root.join("count");
    let context: GitContext = GitContext { real_git: program, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: None };
    let mut request: GitRequest = GitRequest::new(root, arguments);
    request.environment = vec![
        (OsString::from("FAKE_COUNT"), count.clone().into_os_string()),
        (OsString::from("FAKE_HANGS"), OsString::from(hangs.to_string())),
        (OsString::from("FAKE_SILENT_SECONDS"), OsString::from(silent_seconds.to_string())),
    ];
    return (context, request, count);
}

/// How long the silent fake sits silent when nothing ends it:
/// far past the window.
const FAR_PAST_THE_WINDOW_SECONDS: u64 = 30;

/// A Git step that shows no sign of work for the window is ended and its step is run again,
/// and the run again is the answer.
#[test]
fn a_silent_git_step_is_ended_after_the_quiet_window_and_redone() {
    let root: PathBuf = fixture("transaction-git-redo");
    let (context, request, count) = silent_run(root.as_path(), &["rev-parse", "HEAD"], 1, FAR_PAST_THE_WINDOW_SECONDS);
    let sink: SharedSink = SharedSink::default();
    let output: GitOutput = run_git_once_with(&context, &request, &|| {
        return quick_terms(QUIET_WINDOW, Box::new(sink.clone()));
    })
    .expect("the step is redone and answers");
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "late\n");
    assert_eq!(runs_of(count.as_path()), 2, "ended once, redone once");
    remove(root.as_path());
}

/// A step that is silent again in its redo fails with the quiet-time sentence,
/// after exactly the ended run and the redo.
#[test]
fn a_step_silent_in_its_redo_too_fails_naming_the_quiet_window() {
    let root: PathBuf = fixture("transaction-git-stuck");
    let (context, request, count) = silent_run(root.as_path(), &["rev-parse", "HEAD"], u32::MAX, FAR_PAST_THE_WINDOW_SECONDS);
    let sink: SharedSink = SharedSink::default();
    let failure: std::io::Error = run_git_once_with(&context, &request, &|| {
        return quick_terms(QUIET_WINDOW, Box::new(sink.clone()));
    })
    .expect_err("a step silent in its redo is reported");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        failure.to_string().starts_with(format!("git rev-parse HEAD, {} showed no sign of work for 1 seconds and was stopped", class_of(request.arguments.as_slice()).label()).as_str()),
        "{failure}"
    );
    assert_eq!(runs_of(count.as_path()), 2, "the run and its redo");
    // The notice that is due goes out before the stall is judged, so an ended step was announced at least once.
    assert!(sink.text().lines().count() >= 1, "{}", sink.text());
    remove(root.as_path());
}

/// A step that may not be run twice (a reference update) is announced while it is silent and never ended:
/// it runs once and
///  its answer comes back,
/// however long it was silent.
#[test]
fn a_silent_writer_is_waited_for_and_never_ended() {
    let root: PathBuf = fixture("transaction-git-writer");
    let silent_seconds: u64 = 3;
    let (context, request, count) = silent_run(root.as_path(), &["update-ref", "refs/heads/main", "abc"], 1, silent_seconds);
    let sink: SharedSink = SharedSink::default();
    let started: std::time::Instant = std::time::Instant::now();
    let output: GitOutput = run_git_once_with(&context, &request, &|| {
        return quick_terms(QUIET_WINDOW, Box::new(sink.clone()));
    })
    .expect("a writer is waited for");
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "late\n");
    assert!(started.elapsed() >= Duration::from_secs(silent_seconds), "it was silent for {silent_seconds} seconds: {:?}", started.elapsed());
    assert_eq!(runs_of(count.as_path()), 1, "a writer is not run twice");
    assert!(sink.text().lines().count() >= 1, "the silence was announced: {}", sink.text());
    remove(root.as_path());
}

/// A timed-out step reaches the caller of the checked run as its own failure text.
#[test]
fn the_checked_run_reports_a_bound_without_saying_the_step_failed_to_start() {
    let arguments: Vec<OsString> = vec![OsString::from("add")];
    let timed_out = checked_from(
        Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "git add did not finish")),
        arguments.as_slice(),
    )
    .expect_err("a bound is a failure");
    assert_eq!(timed_out.0, "git add did not finish");
    let other = checked_from(Err(std::io::Error::other("no such file")), arguments.as_slice())
        .expect_err("a start failure is a failure");
    assert_eq!(other.0, "git add could not start: no such file");
}

/// A rerunnable command that Git reports as refused is run again with a fresh input file until it works.
#[test]
fn a_rerunnable_command_refused_inside_git_is_run_again_with_its_input() {
    let root: PathBuf = fixture("transaction-git-rerun");
    let (context, request, count) = fake_run(root.as_path(), &["rev-parse", "HEAD"], 2, false);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("the third run works");
    assert_eq!(output.code, Some(0));
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "read 5\n", "the input file was fresh");
    assert_eq!(runs_of(count.as_path()), 3);
    remove(root.as_path());
}

/// A command that may not be run twice runs once,
/// whatever Git says.
#[test]
fn a_writer_that_may_not_be_rerun_runs_once() {
    let root: PathBuf = fixture("transaction-git-once");
    let (context, request, count) = fake_run(root.as_path(), &["update-ref", "refs/heads/main", "abc"], 5, false);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("the run is reported");
    assert_eq!(output.code, Some(128));
    assert!(output.error_text().contains("Resource temporarily unavailable"));
    assert_eq!(runs_of(count.as_path()), 1);
    remove(root.as_path());
}

/// A failure that is not a refusal is not run again.
#[test]
fn another_failure_is_not_run_again() {
    let root: PathBuf = fixture("transaction-git-other");
    let (context, request, count) = fake_run(root.as_path(), &["rev-parse", "HEAD"], 0, true);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("the run is reported");
    assert_eq!(output.error_text(), "fatal: bad object");
    assert_eq!(runs_of(count.as_path()), 1);
    remove(root.as_path());
}

/// A refusal that outlasts the budget returns Git's own last output,
/// after a bounded number of runs.
#[test]
fn a_refusal_past_the_budget_returns_gits_last_output() {
    let root: PathBuf = fixture("transaction-git-budget");
    let (context, request, count) = fake_run(root.as_path(), &["rev-parse", "HEAD"], 1000, false);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("Git's last output");
    assert_eq!(output.code, Some(128));
    assert!(output.error_text().contains("unable to create thread"));
    assert_eq!(runs_of(count.as_path()), 15, "the same bound as a refused start");
    remove(root.as_path());
}

/// The allow-list:
/// reads and idempotent writers may be rerun,
/// other writers and unrelated shapes may not.
#[test]
fn only_harmless_commands_may_be_rerun() {
    let may = |arguments: &[&str], private_index: bool| -> bool {
        let mut request: GitRequest = GitRequest::new(Path::new("/repo"), arguments);
        if private_index {
            request.index = Some(PathBuf::from("/private.index"));
        }
        return may_be_rerun(&request);
    };
    assert!(may(&["rev-parse", "HEAD"], false));
    assert!(may(&["-c", "core.hooksPath=/x", "pack-objects", "--revs"], false));
    assert!(may(&["--git-dir=/shadow", "-C", "/dir", "index-pack", "--keep=m"], false));
    assert!(may(&["config", "--get", "a.b"], false));
    assert!(may(&["config", "--file", "config", "--null", "--list"], false));
    assert!(may(&["remote", "get-url", "origin"], false));
    assert!(may(&["add", "--all", "--", "a.txt"], true));
    assert!(may(&["reset", "--quiet", "abc", "--", "a.txt"], true));
    assert!(!may(&["add", "--all", "--", "a.txt"], false), "the real index is not rerun");
    assert!(!may(&["config", "a.b", "c"], false), "a config write is not a read");
    assert!(!may(&["remote", "add", "origin", "url"], false));
    assert!(!may(&["update-ref", "refs/heads/main", "abc"], false));
    assert!(!may(&["commit", "--message=x"], false));
    assert!(!may(&["push", "origin"], false));
    assert!(!may(&[], false));
    assert!(!may(&["-c", "a=b"], false));
}

/// The owner's criterion (2026-11-04):
/// a step keeps its ceiling exactly when it can run twice.
/// Each form below was cut at every
/// mutating system call,
/// with SIGKILL and with SIGTERM,
/// then run a second time,
/// and its end state compared with an uncut run's
/// (`doc/planning/cli-git-no-unbounded-waits.md`,
/// "The cut experiment").
/// A form that came out identical every time ends at the
/// ceiling of its class;
/// the others are never ended:
/// the commit service's landing thread waits for them,
/// and any other command
/// stops waiting at its bound plus the short extra wait and leaves them running.
#[test]
fn a_step_keeps_its_ceiling_exactly_when_the_cut_experiment_ran_it_twice_to_the_same_state() {
    let bound = |arguments: &[&str], private_index: bool| -> StepBound {
        let mut request: GitRequest = GitRequest::new(Path::new("/repo"), arguments);
        if private_index {
            request.index = Some(PathBuf::from("/private.index"));
        }
        return step_bound(&request);
    };
    let ends = |arguments: &[&str]| { return matches!(bound(arguments, false), StepBound::EndAt(_)); };
    // Identical after a second run at every cut.
    for same in [
        &["stash", "list", "--format=%H"][..],
        &["clean", "--force", "-d"],
        &["read-tree", "--reset", "-u", "abc"],
        &["update-index", "--refresh"],
        &["update-index", "-z", "--index-info"],
        &["update-index", "--add", "--cacheinfo", "100644,abc,x"],
        &["add", "--update", "--", ":/"],
        &["add", "--all"],
        &["config", "--local", "a.b", "c"],
        &["checkout-index", "--all", "--force"],
        &["restore", "--staged", "--worktree", "--source=HEAD", "--", "a.txt"],
        &["reset", "--quiet", "abc", "--", "a.txt"],
    ] {
        assert!(ends(same), "{same:?} ran twice to the same state and keeps its ceiling");
    }
    // Different after a second run at some cut, or not in the experiment.
    for differs in [
        // A write of a side ref cannot be shown to run twice, with the protections on its argv or without (sol-1 round 4, P1 B): a reflog
        //  another Git creates between any check and the write is appended to twice by a cut write and its rerun.
        &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"][..],
        &["-c", "core.logAllRefUpdates=false", "update-ref", "-d", "--no-deref", "refs/worktree/cli-git-service/1"],
        &["update-ref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["update-ref", "-d", "refs/worktree/cli-git-service/1"],
        &["update-ref", "--stdin"],
        // The import is a writer of a reference whatever its stream says here: the experiment found a marks lock it left that the
        //  second run could not pass.
        &["-c", "core.logAllRefUpdates=false", "fast-import", "--done", "--export-marks=marks-1"],
        &["fast-import", "--done"],
        &["update-ref", "refs/heads/main", "new", "old"],
        &["update-ref", "refs/heads/main", "new"],
        &["update-ref", "-d", "refs/heads/old"],
        &["update-ref", "refs/cli-git/kept/1", "new", "old"],
        &["stash", "store", "-m", "message", "abc"],
        &["stash", "push"],
        &["merge", "--ff-only", "--no-edit", "abc"],
        &["reset", "--hard", "--no-recurse-submodules", "abc"],
        &["reset", "--soft", "abc"],
        &["read-tree", "-u", "-m", "a", "b"],
        &["checkout", "--quiet", "other"],
        &["switch", "--quiet", "other"],
    ] {
        assert!(!ends(differs), "{differs:?} came out different after a second run and has no ceiling");
    }
    // What:
    //  The word-level second pass's line merge keeps its ceiling,
    //  and the form that writes a file has none.
    // Why:
    //  `git merge-file --stdout --object-id` reads three object names and answers on standard output,
    //  so it changes nothing and may be run twice;
    //  `git merge-file` without `--stdout` writes the merged bytes into the first file it was given,
    //  a file of the person's,
    //  and is not a read.
    //  The pass needs the first form to end at a ceiling:
    //  a line merge that is never ended and cannot be ended answers nothing at all,
    //  and it is the answer the pass parses
    //  (`doc/planning/cli-git-no-unbounded-waits.md`,
    //  "The cut experiment",
    //  and "Stop only on no progress").
    // The same index changes on a private index, the readers and the object writers are not in question.
    assert!(ends(&["merge-file", "--stdout", "--object-id", "--zdiff3", "-L", "ours", "-L", "base", "-L", "theirs", "aaaa", "bbbb", "cccc"]));
    assert!(!ends(&["merge-file", "a.md", "base.md", "theirs.md"]), "the in-place form writes the first file, so it has no ceiling");
    assert_eq!(bound(&["add", "--all"], true), StepBound::EndAt(OBJECT_STEP_CEILING));
    assert_eq!(bound(&["rev-parse", "HEAD"], false), StepBound::EndAt(PLUMBING_STEP_CEILING));
    assert_eq!(bound(&["merge-base", "a", "b"], false), StepBound::EndAt(PLUMBING_STEP_CEILING));
    assert_eq!(bound(&["log", "-1"], false), StepBound::EndAt(OBJECT_STEP_CEILING));
    assert_eq!(bound(&["commit-tree", "abc", "-m", "x"], false), StepBound::EndAt(OBJECT_STEP_CEILING));
}

/// A step that cannot be run twice is left running by the command that started it,
/// which stops waiting at the bound of the step's
/// class plus the short extra wait;
/// on the commit service's landing thread,
/// whose clients are told through the protocol,
/// it is
/// waited for without a bound.
/// The control shortens the bound of the class for the thread that runs it,
/// and never the extra wait
/// of production.
#[test]
fn a_step_that_cannot_run_twice_is_left_running_by_a_command_and_waited_for_by_the_landing_thread() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let merge: GitRequest = GitRequest::new(Path::new("/repo"), &["merge", "--ff-only", "abc"]);
        assert_eq!(step_bound(&merge), StepBound::LeaveRunning { after: OBJECT_STEP_CEILING, extra: crate::wait_ceilings::SHORT_EXTRA_WAIT });
        let reference: GitRequest = GitRequest::new(Path::new("/repo"), &["update-ref", "--stdin"]);
        assert_eq!(step_bound(&reference), StepBound::LeaveRunning { after: PLUMBING_STEP_CEILING, extra: crate::wait_ceilings::SHORT_EXTRA_WAIT });
        let shortened: Duration = Duration::from_secs(3);
        STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(shortened)); });
        let ends: StepBound = step_bound(&GitRequest::new(Path::new("/repo"), &["rev-parse", "HEAD"]));
        let leaves: StepBound = step_bound(&merge);
        STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
        assert_eq!(ends, StepBound::EndAt(shortened));
        assert_eq!(leaves, StepBound::LeaveRunning { after: shortened, extra: crate::wait_ceilings::SHORT_EXTRA_WAIT });
        // The landing thread is the one that has a record of what it is inside.
        let board = crate::service_busy::LandingBoard::new(fixture("transaction-git-landing-thread").join("journal"), Duration::from_secs(60));
        crate::service_busy_step_record_on_this_thread::use_on_this_thread(board);
        assert_eq!(step_bound(&reference), StepBound::Unbounded);
        assert_eq!(step_bound(&GitRequest::new(Path::new("/repo"), &["rev-parse", "HEAD"])), StepBound::EndAt(PLUMBING_STEP_CEILING));
        // What:
        //  The second pass's line merge ends at its class ceiling on the landing thread too.
        // Why:
        //  The commit replay reaches the pass inside the landing's own scope,
        //  and a `merge-file` that is neither rerunnable nor endable is waited for
        //  without a bound there,
        //  so every landing job behind it stays queued for good.
        let line_merge: GitRequest = GitRequest::new(
            Path::new("/repo"),
            &["merge-file", "--stdout", "--object-id", "--zdiff3", "-L", "ours", "-L", "base", "-L", "theirs", "aaaa", "bbbb", "cccc"],
        );
        assert_eq!(step_bound(&line_merge), StepBound::EndAt(OBJECT_STEP_CEILING));
    });
}

/// A Git that cannot be started at all is the caller's failure with its own kind,
/// not a refusal to retry and not a
///  failure of another kind:
/// the error of a rerunnable command comes back as the operating system gave it.
#[test]
fn a_start_failure_keeps_its_kind_through_the_rerun_loop() {
    let root: PathBuf = fixture("transaction-git-missing");
    let context: GitContext =
        GitContext { real_git: root.join("no-such-git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: None };
    let request: GitRequest = GitRequest::new(root.as_path(), &["rev-parse", "HEAD"]);
    assert!(may_be_rerun(&request), "the command is one the loop may run again");
    let failure: std::io::Error =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect_err("Git cannot start");
    assert_eq!(failure.kind(), std::io::ErrorKind::NotFound, "{failure}");
    remove(root.as_path());
}

/// A refused run followed by a run that cannot start reports the start failure,
/// not the stale output of the refused
///  run:
/// the output of an earlier run answers only when the budget ran out on refusals.
///  The fake Git refuses once and removes itself,
/// so the second start finds no program.
#[test]
fn a_refused_run_followed_by_a_start_failure_reports_the_start_failure() {
    let root: PathBuf = fixture("transaction-git-refused-then-gone");
    let program: PathBuf = root.join("vanishing-git");
    executable(
        program.as_path(),
        b"#!/bin/sh\necho 'fatal: unable to create thread: Resource temporarily unavailable' >&2\nrm -- \"$0\"\nexit 128\n",
    );
    let context: GitContext = GitContext { real_git: program, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: None };
    let request: GitRequest = GitRequest::new(root.as_path(), &["rev-parse", "HEAD"]);
    let failure: std::io::Error = run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause)
        .expect_err("the second start finds no program");
    assert_eq!(failure.kind(), std::io::ErrorKind::NotFound, "{failure}");
    remove(root.as_path());
}

/// Only Git's own words for a refused process or thread count as a refusal.
#[test]
fn only_a_refusal_by_the_system_counts() {
    let failure = |text: &str, code: Option<i32>| -> GitOutput {
        return GitOutput { code, stdout: Vec::new(), stderr: text.as_bytes().to_vec() };
    };
    assert!(refused_by_the_system(&failure("fatal: unable to create thread: Resource temporarily unavailable", Some(128))));
    assert!(refused_by_the_system(&failure("fatal: cannot fork() for pack-objects: Resource temporarily unavailable", None)));
    assert!(!refused_by_the_system(&failure("Resource temporarily unavailable", Some(0))), "a success is not a refusal");
    assert!(!refused_by_the_system(&failure("fatal: bad object", Some(128))));
}

/// No hook of the person runs below a step the wrapper starts:
/// every watched start carries the hooks directory that never
///  exists and the switch of every hook event,
/// `reference-transaction` included,
/// so the stop of a quiet step can never end a
///  person's hook and the watch needs no way to tell a hook from Git.
#[test]
fn a_watched_step_starts_with_every_hook_switched_off() {
    // The executable switches this on before any start; the unit controls leave it off and each test has its own process.
    crate::hooks_off::activate();
    let context: GitContext = git_context();
    let request: GitRequest = GitRequest::new(Path::new("."), &["update-ref", "refs/heads/main", "abc"]);
    let command: Command = build_command(&context, &request);
    let arguments: Vec<String> = command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
    assert!(arguments.iter().any(|argument| { return argument.starts_with("core.hooksPath=/dev/null/"); }), "{arguments:?}");
    assert!(crate::hooks_off::HOOK_EVENTS.contains(&"reference-transaction"));
    for event in crate::hooks_off::switched_off_events() {
        assert!(arguments.contains(&format!("hook.{event}.enabled=false")), "{event} in {arguments:?}");
    }
}

/// A person's merge whose fast-forward does not return:
/// `run_git` hands back the error that says the result is not known,
/// naming the
/// step and what to check,
/// at the bound of the class plus the extra wait (shortened for the control),
/// and the step is left running
/// until it ends by itself.
#[test]
fn a_merge_that_does_not_return_gives_the_not_known_error_and_is_left_running() {
    let root: PathBuf = fixture("transaction-git-merge-not-known");
    let program: PathBuf = root.join("slow-merge-git");
    let started: PathBuf = root.join("started");
    let finished: PathBuf = root.join("finished");
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\n: > '{started}'\nwhile [ ! -e '{release}' ] && {alive}; do sleep 0.05; done\n: > '{finished}'\n",
            started = started.display(),
            release = root.join("release").display(),
            finished = finished.display(),
            alive = while_test_lives(root.as_path())
        )
        .as_bytes(),
    );
    let context: GitContext = GitContext { real_git: program, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(root.clone()) };
    let request: GitRequest = GitRequest::new(root.as_path(), &["merge", "--ff-only", "--no-edit", "abc"]);
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(Duration::from_millis(500))); });
    EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(Some(Duration::from_millis(500))); });
    let began: std::time::Instant = std::time::Instant::now();
    let failure = run_git(&context, &request).expect_err("the command stopped waiting");
    let waited: Duration = began.elapsed();
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    assert!(crate::child_watch::is_not_known(&failure), "{failure}");
    assert!(failure.to_string().contains("git merge --ff-only --no-edit abc") && failure.to_string().contains("git status") && failure.to_string().contains("left running"), "{failure}");
    assert!(waited >= Duration::from_secs(1) && waited < Duration::from_secs(30), "{waited:?}");
    assert!(started.exists() && !finished.exists(), "the step was not ended");
    std::fs::write(root.join("release"), b"").expect("release");
    let waiting_since: std::time::Instant = std::time::Instant::now();
    while !finished.exists() {
        assert!(waiting_since.elapsed() < Duration::from_secs(30), "the step never finished");
        std::thread::sleep(Duration::from_millis(50));
    }
    remove(root.as_path());
}

/// Forms the cut experiment did not run get no ceiling (sol-1:
/// the classifier failed open).
/// Each is a form of a step the experiment
/// did run,
/// or a step it did not,
/// that a rule by the name of the step would have ended at a ceiling.
#[test]
fn a_form_the_cut_experiment_did_not_run_has_no_ceiling() {
    let ends = |arguments: &[&str]| -> bool { return matches!(step_bound(&GitRequest::new(Path::new("/repo"), arguments)), StepBound::EndAt(_)); };
    for unrun in [
        // `config --add` appends a second time when the first run had installed its update.
        &["config", "--add", "a.b", "c"][..],
        &["config", "--unset", "a.b"],
        &["config", "--replace-all", "a.b", "c"],
        // `stash -m list` is a push of a stash with the message "list".
        &["stash", "-m", "list"],
        &["stash", "list", "--stat"],
        &["stash", "push", "-m", "x"],
        // A reflog introduced on a side ref is the doubling that made a branch write different.
        &["update-ref", "--create-reflog", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["update-ref", "-m", "message", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        // The numbered ref is a plain ref only with `--no-deref`, and has no reflog only with the setting on the command line (the
        //  repository's own `core.logAllRefUpdates=always` would append a second entry in the second run).
        &["update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=false", "update-ref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=always", "update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=true", "update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=false", "update-ref", "-d", "refs/worktree/cli-git-service/1"],
        // The last setting of the key wins in Git, a key is not case-sensitive, and `--config-env` sets one too.
        &["-c", "core.logAllRefUpdates=false", "-c", "core.logAllRefUpdates=always", "update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=false", "-c", "core.logallrefupdates=true", "update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=false", "--config-env=core.logAllRefUpdates=LOGGING", "update-ref", "--no-deref", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", "--create-reflog", "refs/cli-git/kept/1", "0123456789abcdef0123456789abcdef01234567"],
        // A ref under the prefix is not any ref under it: the wrapper makes numbered ones, and `..` is not a number.
        &["update-ref", "refs/cli-git/../heads/main", "0123456789abcdef0123456789abcdef01234567"],
        &["update-ref", "refs/cli-git/kept/1/../../heads/x", "0123456789abcdef0123456789abcdef01234567"],
        &["update-ref", "refs/cli-git/kept/1", "refs/heads/main"],
        // Other forms of steps that have a proven form.
        &["clean", "--force", "-d", "-x"],
        &["clean", "--force", "-d", "--", "dir"],
        &["read-tree", "--reset", "-u", "a", "b"],
        &["add", "--intent-to-add", "--", "a.txt"],
        &["add", "--update", "--all"],
        &["checkout-index", "--all"],
        &["restore", "--staged", "a.txt"],
        &["reset", "--mixed", "abc"],
        &["update-index", "--chmod=+x", "a.txt"],
        // Steps nobody listed.
        &["gc", "--prune=now"],
        &["maintenance", "run"],
        &["repack", "-a", "-d"],
        &["worktree", "add", "x"],
        &["tag", "-f", "t", "abc"],
        &["branch", "-f", "b", "abc"],
        &[],
    ] {
        assert!(!ends(unrun), "{unrun:?} was not run in the experiment and has no ceiling");
    }
    // The forms that were run are exact: the same words with the proven options, and nothing else, keep the ceiling.
    for proven in [
        &["stash", "list", "--format=%H"][..],
        &["clean", "--force", "-d"],
        &["read-tree", "--reset", "-u", "abc"],
        &["config", "--local", "a.b", "c"],
        &["reset", "--quiet", "abc", "--", "a.txt"],
        &["restore", "--staged", "--worktree", "--source=HEAD", "--", "a.txt"],
        &["add", "--all"],
        &["add", "--update", "--", ":/"],
    ] {
        assert!(ends(proven), "{proven:?} is a form the experiment ran and keeps its ceiling");
    }
}

/// Which forms of `symbolic-ref` and `init` may be run again (sol-1 and sol-2 round 2):
/// the forms the wrapper runs,
/// and no other.
/// A
/// read changes nothing.
/// A write of a symbolic ref is run twice only in a shadow repository the wrapper made for one transaction
/// (the experiment:
/// identical after a second run at every cut).
/// `init` is only the shadow's creation,
/// which is private and thrown
/// away when it fails (the experiment:
/// different after a second run,
/// so a step ended there discards that transaction's shadow).
#[test]
fn symbolic_ref_and_init_are_run_again_only_in_the_forms_the_wrapper_runs() {
    let may = |arguments: &[&str], shadow: bool| -> bool {
        let mut request: GitRequest = GitRequest::new(Path::new("/repo"), arguments);
        request.without_prefix = shadow;
        return may_be_rerun(&request);
    };
    // Reads.
    assert!(may(&["symbolic-ref", "--quiet", "HEAD"], false));
    assert!(may(&["symbolic-ref", "--quiet", "--short", "HEAD"], false));
    // The wrapper's writes in a shadow repository.
    assert!(may(&["--git-dir=/shadow", "-c", "core.hooksPath=/x", "symbolic-ref", "HEAD", "refs/heads/main"], true));
    // A write in the repository of the person is a branch write.
    assert!(!may(&["symbolic-ref", "HEAD", "refs/heads/other"], false));
    // A delete, a message (it writes a reflog entry) and any other option are not forms the wrapper runs.
    assert!(!may(&["symbolic-ref", "--delete", "HEAD"], true));
    assert!(!may(&["symbolic-ref", "-m", "why", "HEAD", "refs/heads/main"], true));
    assert!(!may(&["symbolic-ref", "--delete", "--quiet", "HEAD"], false));
    // The shadow's creation is not run again in place: a cut `init` leaves a directory the second run fails in, and the shadow's creator
    //  discards the directory and starts a fresh one (`a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`).
    assert!(!may(&["init", "--bare", "--template=", "--object-format=sha1", "--ref-format=reftable", "/shadow"], true));
    // Any other `init` can relocate or re-initialize the repository of the person.
    assert!(!may(&["init", "--bare", "--template=", "/shadow"], false));
    assert!(!may(&["init", "--quiet", "--initial-branch=main"], true));
    assert!(!may(&["init", "--separate-git-dir=/elsewhere", "--bare"], false));
}

/// The shadow's creation keeps its ceiling only in the exact form the wrapper runs and only for a request on a shadow:
/// its creator
/// discards a cut start's directory and starts again in a fresh one,
/// which the experiment found equal to an uncut run (104 of 104).
/// Any other `init` can touch the repository of the person,
/// so it has no ceiling.
#[test]
fn only_the_shadow_creation_init_keeps_a_ceiling() {
    let ends = |arguments: &[&str], shadow: bool| -> bool {
        let mut request: GitRequest = GitRequest::new(Path::new("/repo"), arguments);
        request.without_prefix = shadow;
        return matches!(step_bound(&request), StepBound::EndAt(_));
    };
    for object_format in ["sha1", "sha256"] {
        let format: String = format!("--object-format={object_format}");
        assert!(ends(&["init", "--bare", "--template=", format.as_str(), "--ref-format=reftable", "/shadow"], true), "{object_format}");
    }
    assert!(!ends(&["init", "--bare", "--template=", "--object-format=sha1", "--ref-format=reftable", "/shadow"], false), "not a shadow request");
    assert!(!ends(&["init", "--bare", "--template=", "--object-format=sha1", "--ref-format=reftable", "--separate-git-dir=/x", "/shadow"], true));
    assert!(!ends(&["init", "--bare", "--template=/templates", "--object-format=sha1", "--ref-format=reftable", "/shadow"], true));
    assert!(!ends(&["init", "--bare", "--template=", "--object-format=sha1", "--ref-format=files", "/shadow"], true));
    assert!(!ends(&["init", "--quiet", "--initial-branch=main"], true));
}

/// A side-ref write on the landing thread is never ended and never waited for without a bound (owner:
/// "Never cut a branch write
/// (Recommended)" covers the branch write;
/// "nothing should ever wait forever or indefinitely" covers this):
/// the landing thread stops
/// waiting at the class ceiling plus the short extra wait,
/// with the not-known error,
/// and the step is left running.
/// The branch write
/// `update-ref --stdin` on the same thread is still waited for as long as it lives,
/// and its record tells the clients behind it.
#[test]
fn a_side_ref_write_that_hangs_on_the_landing_thread_is_left_running_after_a_bound() {
    let root: PathBuf = fixture("transaction-git-side-ref-landing");
    let program: PathBuf = root.join("hung-update-ref-git");
    let started: PathBuf = root.join("started");
    executable(
        program.as_path(),
        format!("#!/bin/sh\n: > '{started}'\nwhile [ ! -e '{release}' ] && {alive}; do sleep 0.05; done\n", started = started.display(), release = root.join("release").display(), alive = while_test_lives(root.as_path())).as_bytes(),
    );
    let context: GitContext = GitContext { real_git: program, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(root.clone()) };
    let (sender, receiver) = std::sync::mpsc::channel();
    let journal: PathBuf = root.join("journal");
    let work_root: PathBuf = root.clone();
    // The landing thread of the control: it has a record, and the bounds are shortened on it alone.
    let landing = std::thread::Builder::new().spawn(move || {
        STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(Duration::from_millis(400))); });
        EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(Some(Duration::from_millis(400))); });
        crate::landing_state_carried_by_its_task::in_a_landing(|| {
            crate::service_busy_step_record_on_this_thread::use_on_this_thread(crate::service_busy::LandingBoard::new(journal, Duration::from_secs(60)));
            let side: GitRequest = GitRequest::new(work_root.as_path(), &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", "refs/worktree/cli-git-service/7"]);
            let branch: GitRequest = GitRequest::new(work_root.as_path(), &["update-ref", "--stdin"]);
            assert_eq!(step_bound(&branch), StepBound::Unbounded, "the branch write is waited for on the landing thread");
            assert!(matches!(step_bound(&side), StepBound::LeaveRunning { .. }), "a side ref is not a branch: {:?}", step_bound(&side));
            let _sent = sender.send(run_git(&context, &side));
        });
    });
    assert!(landing.is_ok());
    let outcome = receiver.recv_timeout(Duration::from_secs(30)).expect("the landing thread came back from a hung side-ref write");
    let failure = outcome.expect_err("it stopped waiting");
    assert!(crate::child_watch::is_not_known(&failure), "{failure}");
    assert!(started.exists());
    std::fs::write(root.join("release"), b"").expect("release");
    remove(root.as_path());
}

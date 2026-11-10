//! Controls for the parent's run of the Git LFS upload.

use super::*;
use crate::test_support::{executable, fixture, remove};
use std::path::PathBuf;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// A plan for `origin main`.
fn plan() -> UploadPlan {
    return UploadPlan { remotes: words(&["origin"]), refs: words(&["main"]), everything: false, submodules: false, full_scan: false, objects: None };
}

/// A stand-in `git-lfs` that records its arguments,
/// directory,
/// `git` and configuration variables,
/// then exits.
fn stand_in(directory: &std::path::Path, record: &std::path::Path, status: i32) {
    std::fs::create_dir_all(directory).expect("bin directory");
    let source: String = format!(
        "#!/bin/sh\n{{ printf '%s\\n' \"$@\"; pwd; command -v git; /usr/bin/env; }} > '{record}'\nexit {status}\n",
        record = record.display()
    );
    executable(directory.join("git-lfs").as_path(), source.as_bytes());
}

/// Run the interactive upload with the given `PATH` directories,
/// prefix and status,
/// returning result and diagnostics.
fn run(root: &std::path::Path, path: &[&std::path::Path], global_prefix: &[&str], upload: &UploadPlan) -> (Option<i32>, String) {
    return run_with(root, path, global_prefix, upload, &[("XDG_STATE_HOME", root.join("state").to_str().expect("UTF-8"))]);
}

/// The same with the given environment variables besides `PATH`.
fn run_with(
    root: &std::path::Path,
    path: &[&std::path::Path],
    global_prefix: &[&str],
    upload: &UploadPlan,
    variables: &[(&str, &str)],
) -> (Option<i32>, String) {
    let real: PathBuf = root.join("real");
    std::fs::create_dir_all(&real).expect("real git directory");
    executable(real.join("git").as_path(), b"#!/bin/sh\n");
    let joined = std::env::join_paths(path).expect("PATH");
    let mut environment: Vec<(OsString, OsString)> = vec![(OsString::from("PATH"), joined)];
    environment.extend(variables.iter().map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    }));
    let mut diagnostics: Vec<u8> = Vec::new();
    let code = run_upload_interactive(
        upload,
        words(global_prefix).as_slice(),
        root,
        real.join("git").as_path(),
        environment.as_slice(),
        &[],
        &mut diagnostics,
    );
    return (code, String::from_utf8_lossy(diagnostics.as_slice()).into_owned());
}

/// A succeeding upload is silent,
/// gets the plan's arguments in the chosen directory,
/// and sees the real Git first.
#[test]
fn a_succeeding_upload_is_silent_and_sees_the_real_git_first() {
    let root = fixture("lfs-upload-ok");
    let record: PathBuf = root.join("record.txt");
    stand_in(root.join("bin").as_path(), record.as_path(), 0);
    let (code, diagnostics) = run(root.as_path(), &[root.join("bin").as_path()], &[], &plan());
    assert_eq!((code, diagnostics.as_str()), (None, ""));
    let seen: Vec<String> = std::fs::read_to_string(&record).expect("record").lines().map(str::to_owned).collect();
    let expected_arguments: Vec<String> = plan()
        .arguments(std::ffi::OsStr::new("origin"))
        .iter()
        .map(|word| {
            return word.to_string_lossy().into_owned();
        })
        .collect();
    let count: usize = expected_arguments.len();
    assert_eq!(seen[..count], expected_arguments[..]);
    assert_eq!(seen[count], root.to_string_lossy());
    assert_eq!(seen[count + 1], root.join("real/git").to_string_lossy(), "the real Git leads the child's PATH");
    remove(root.as_path());
}

/// The `git-lfs` the caller's `PATH` finds first runs,
/// even though the real Git's directory leads the child's `PATH`.
#[test]
fn the_program_is_the_one_the_callers_path_finds_first() {
    let root = fixture("lfs-upload-order");
    let (first, second) = (root.join("first"), root.join("second"));
    let (first_record, second_record) = (root.join("first.txt"), root.join("second.txt"));
    stand_in(first.as_path(), first_record.as_path(), 0);
    stand_in(second.as_path(), second_record.as_path(), 0);
    let real_directory_program: PathBuf = root.join("real");
    std::fs::create_dir_all(&real_directory_program).expect("real directory");
    stand_in(real_directory_program.as_path(), root.join("real.txt").as_path(), 0);
    let (code, _) = run(root.as_path(), &[first.as_path(), second.as_path()], &[], &plan());
    assert_eq!(code, None);
    assert!(first_record.exists(), "the caller's first match ran");
    assert!(!second_record.exists() && !root.join("real.txt").exists());
    remove(root.as_path());
}

/// The global options travel to the child:
/// `-c` as configuration pairs after the ones already set,
/// the repository
///  options as their variables.
#[test]
fn the_global_options_reach_the_child() {
    let root = fixture("lfs-upload-prefix");
    let record: PathBuf = root.join("record.txt");
    stand_in(root.join("bin").as_path(), record.as_path(), 0);
    let prefix: Vec<&str> = vec!["-c", "lfs.url=http://example.invalid/x", "-c", "flag", "--git-dir=/g", "--work-tree", "/w"];
    let (code, _) = run(root.as_path(), &[root.join("bin").as_path()], &prefix, &plan());
    assert_eq!(code, None);
    let seen: String = std::fs::read_to_string(&record).expect("record");
    for expected in [
        "GIT_CONFIG_COUNT=2",
        "GIT_CONFIG_KEY_0=lfs.url",
        "GIT_CONFIG_VALUE_0=http://example.invalid/x",
        "GIT_CONFIG_KEY_1=flag",
        "GIT_CONFIG_VALUE_1=true",
        "GIT_DIR=/g",
        "GIT_WORK_TREE=/w",
    ] {
        assert!(
            seen.lines().any(|line| {
                return line == expected;
            }),
            "{expected}: {seen}"
        );
    }
    remove(root.as_path());
}

/// Pairs the environment already holds come first,
/// so the global options win as they do in Git.
#[test]
fn the_variables_continue_after_the_environments_own_pairs() {
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_CONFIG_COUNT"), OsString::from("3"))];
    let variables = prefix_variables(words(&["-c", "a=b"]).as_slice(), environment.as_slice());
    let named = |name: &str| -> Option<String> {
        return variables
            .iter()
            .find(|(key, _)| {
                return key == name;
            })
            .map(|(_, value)| {
                return value.to_string_lossy().into_owned();
            });
    };
    assert_eq!(named("GIT_CONFIG_COUNT").as_deref(), Some("4"));
    assert_eq!(named("GIT_CONFIG_KEY_3").as_deref(), Some("a"));
    assert_eq!(named("GIT_CONFIG_VALUE_3").as_deref(), Some("b"));
    assert!(prefix_variables(&[], &[]).is_empty());
}

/// A failing upload stops with its own status and the explanation;
/// a missing program stops with status 1.
#[test]
fn a_failing_or_missing_upload_stops_with_the_explanation() {
    let root = fixture("lfs-upload-fail");
    let failing_status: i32 = 7;
    stand_in(root.join("bin").as_path(), root.join("record.txt").as_path(), failing_status);
    let (code, diagnostics) = run(root.as_path(), &[root.join("bin").as_path()], &[], &plan());
    assert_eq!(code, Some(failing_status));
    assert!(diagnostics.contains(&failed_message(&format!("exited with status {failing_status}"))), "{diagnostics}");
    let empty: PathBuf = root.join("empty");
    std::fs::create_dir(&empty).expect("empty directory");
    let (missing_code, missing) = run(root.as_path(), &[empty.as_path()], &[], &plan());
    assert_eq!((missing_code, missing), (Some(1), missing_program_message()));
    remove(root.as_path());
}

/// An upload ended by a signal says so by number,
/// and a signal sent to the wrapper reaches the child.
#[test]
fn an_upload_ended_by_a_signal_names_the_signal() {
    let root = fixture("lfs-upload-signal");
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir(&bin).expect("bin directory");
    executable(bin.join("git-lfs").as_path(), b"#!/bin/sh\nkill -TERM $$\nsleep 5\n");
    let (code, diagnostics) = run(root.as_path(), &[bin.as_path()], &[], &plan());
    assert!(code.is_some_and(|status| {
        return status != 0;
    }));
    assert!(diagnostics.contains(&format!("was ended by signal {}", libc::SIGTERM)), "{diagnostics}");
    remove(root.as_path());
}

/// A push that recurses into submodules gets the notice before the upload runs.
#[test]
fn a_submodule_push_gets_the_notice() {
    let root = fixture("lfs-upload-submodules");
    stand_in(root.join("bin").as_path(), root.join("record.txt").as_path(), 0);
    let marked: UploadPlan = UploadPlan { submodules: true, ..plan() };
    let (code, diagnostics) = run(root.as_path(), &[root.join("bin").as_path()], &[], &marked);
    assert_eq!((code, diagnostics), (None, submodule_notice()));
    remove(root.as_path());
}

/// A signal sent to the wrapper while an upload runs reaches the upload:
/// the child is gone,
/// and the status is the signal's.
///  (Each test runs in its own process,
/// so the process-wide relay of the guard affects only this test.)
#[test]
fn a_signal_sent_to_the_wrapper_stops_the_upload() {
    let root = fixture("lfs-upload-relay");
    let bin: PathBuf = root.join("bin");
    let pid_file: PathBuf = root.join("child.pid");
    std::fs::create_dir(&bin).expect("bin directory");
    executable(
        bin.join("git-lfs").as_path(),
        format!("#!/bin/sh\necho $$ > '{}'\nexec /usr/bin/sleep {HOLD_SECONDS}\n", pid_file.display()).as_bytes(),
    );
    let waiting_for_start: PathBuf = pid_file.clone();
    let sender = std::thread::spawn(move || {
        while std::fs::read_to_string(&waiting_for_start).unwrap_or_default().trim().is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // SAFETY: `kill` takes no pointers; the target is this test's own process, whose guard relays the signal.
        unsafe { libc::kill(libc::getpid(), libc::SIGTERM) };
    });
    let (code, diagnostics) = run(root.as_path(), &[bin.as_path()], &[], &plan());
    sender.join().expect("sender");
    assert_eq!(code, Some(128 + libc::SIGTERM), "{diagnostics}");
    assert!(diagnostics.contains(&format!("was ended by signal {}", libc::SIGTERM)), "{diagnostics}");
    let child: i32 = std::fs::read_to_string(&pid_file).expect("pid").trim().parse().expect("a pid");
    // SAFETY: signal 0 only tests whether the process exists.
    let alive: bool = unsafe { libc::kill(child, 0) } == 0;
    assert!(!alive, "the upload was still running after the signal");
    remove(root.as_path());
}

/// How long the upload would run if nothing stopped it,
/// in whole seconds;
/// far past the control.
const HOLD_SECONDS: u32 = 3_000;

/// The line `git-lfs` printed in the incident,
/// with the token the `lfs.url` carries.
const INCIDENT_LINE: &str = "batch response: Fatal error: Server error https://user:secret@lfs.example.invalid/objects/batch from HTTP 503";

/// A stand-in `git-lfs` that counts its runs in `counter` and,
/// for the first `failures` runs,
/// prints the text of
///  `message_file` on standard error and exits with `status`;
/// later runs succeed.
fn failing_stand_in(directory: &std::path::Path, counter: &std::path::Path, failures: u32, message: &str, status: i32) {
    std::fs::create_dir_all(directory).expect("bin directory");
    let message_file: PathBuf = directory.join("message.txt");
    std::fs::write(&message_file, format!("{message}\n")).expect("message");
    let source: String = format!(
        "#!/bin/sh\nn=$(/usr/bin/cat '{counter}' 2>/dev/null || echo 0)\nn=$((n + 1))\necho $n > '{counter}'\n\
         if [ \"$n\" -le {failures} ]; then /usr/bin/cat '{message_file}' >&2; exit {status}; fi\nexit 0\n",
        counter = counter.display(),
        message_file = message_file.display()
    );
    executable(directory.join("git-lfs").as_path(), source.as_bytes());
}

/// How many times the stand-in ran.
fn runs_of(counter: &std::path::Path) -> u32 {
    return std::fs::read_to_string(counter).map_or(0, |text| {
        return text.trim().parse().unwrap_or(0);
    });
}

/// A server error is not run again by the wrapper:
/// pick 12 of the shipped-limits audit (the coordinator's under the owner's
///  delegation,
/// open to the owner's veto) classes an HTTP 5xx as outside our control,
/// retried by the service's push retry plan
///  alone.
/// The upload runs once,
/// the program's status is kept,
/// the explanation follows,
/// and no retry is announced.
#[test]
fn a_server_error_ends_the_upload_at_once_with_the_explanation() {
    let root = fixture("lfs-upload-server-error-once");
    let counter: PathBuf = root.join("count.txt");
    failing_stand_in(root.join("bin").as_path(), counter.as_path(), 99, INCIDENT_LINE, 2);
    let state: String = root.join("state").to_string_lossy().into_owned();
    let variables = [("XDG_STATE_HOME", state.as_str())];
    let (code, diagnostics) = run_with(root.as_path(), &[root.join("bin").as_path()], &[], &plan(), &variables);
    assert_eq!(code, Some(2), "{diagnostics}");
    assert_eq!(runs_of(counter.as_path()), 1, "the wrapper does not run the upload again: {diagnostics}");
    assert!(!diagnostics.contains("retry 1"), "{diagnostics}");
    assert!(diagnostics.ends_with(&failed_message("exited with status 2")), "{diagnostics}");
    assert!(!diagnostics.contains("secret"), "{diagnostics}");
    remove(root.as_path());
}

/// A refused credential,
/// a missing object,
/// a failure with no known wording and a server error beside a missing object are
///  each run once,
/// as every other failure is:
/// the wrapper runs the upload again for none of them.
#[test]
fn every_failure_runs_the_upload_once() {
    let cases: [(&str, &str, i32); 4] = [
        ("auth", "batch response: Authorization error: https://u:secret@lfs.invalid/objects/batch", 2),
        ("missing", "Git LFS upload failed:\n  (missing) big.bin (abc)", 2),
        ("unknown", "something else went wrong", 1),
        ("mixed", "Server error https://u:secret@lfs.invalid/o from HTTP 503\n  (missing) big.bin (abc)", 2),
    ];
    for (name, message, status) in cases {
        let root = fixture(&format!("lfs-upload-once-{name}"));
        let counter: PathBuf = root.join("count.txt");
        failing_stand_in(root.join("bin").as_path(), counter.as_path(), 99, message, status);
        let state: String = root.join("state").to_string_lossy().into_owned();
        let variables = [("XDG_STATE_HOME", state.as_str())];
        let (code, diagnostics) = run_with(root.as_path(), &[root.join("bin").as_path()], &[], &plan(), &variables);
        assert_eq!(code, Some(status), "{name}: {diagnostics}");
        assert_eq!(runs_of(counter.as_path()), 1, "{name}: {diagnostics}");
        assert!(!diagnostics.contains("retry 1"), "{name}: {diagnostics}");
        assert!(!diagnostics.contains("secret"), "{name}: {diagnostics}");
        remove(root.as_path());
    }
}

/// A caller whose environment names no scratch location gets a refusal that says how to go on,
/// not an unmasked run.
#[test]
fn an_upload_that_cannot_capture_its_output_is_refused_with_the_ways_forward() {
    let root = fixture("lfs-upload-no-scratch");
    let counter: PathBuf = root.join("count.txt");
    failing_stand_in(root.join("bin").as_path(), counter.as_path(), 0, INCIDENT_LINE, 2);
    let (code, diagnostics) = run_with(root.as_path(), &[root.join("bin").as_path()], &[], &plan(), &[]);
    assert_eq!(code, Some(1));
    assert!(diagnostics.contains("GIT_LFS_SKIP_PUSH=1") && diagnostics.contains("XDG_STATE_HOME"), "{diagnostics}");
    assert_eq!(runs_of(counter.as_path()), 0, "nothing ran");
    remove(root.as_path());
}

/// The coordinator's Git for a fixture whose `PATH` finds `bin` first,
/// with the upload switched on.
fn push_git_through(bin: &std::path::Path, root: &std::path::Path) -> PushGit {
    let path: OsString = std::env::join_paths([bin, std::path::Path::new("/usr/bin"), std::path::Path::new("/bin")]).expect("a PATH");
    let mut environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), path),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
    ];
    environment.extend(crate::test_support::scratch_environment());
    return PushGit { real_git: root.join("real-git"), cwd: root.to_path_buf(), environment, lfs: true };
}

/// A stand-in `git-lfs` that answers `status` after a moment,
/// so a wait that held its thread is visible beside it.
fn slow_stand_in(directory: &std::path::Path, status: i32) {
    std::fs::create_dir_all(directory).expect("bin directory");
    let source: String = format!("#!/bin/sh\nsleep 0.2\nexit {status}\n");
    executable(directory.join("git-lfs").as_path(), source.as_bytes());
}

/// What:
///  The awaited upload answers what the synchronous upload answers,
/// and a task beside it keeps running while the program does.
/// Why:
///   The landing round's push uploads the objects of its commits as a task,
/// and a difference between the faces would be an upload that
///  reported another program's result,
/// or a round whose other directories stopped for the whole upload.
#[test]
fn the_awaited_upload_answers_what_the_synchronous_upload_answers() {
    for status in [0, 7] {
        let root: PathBuf = fixture(format!("lfs-upload-awaiting-{status}").as_str());
        std::fs::write(root.join("real-git"), b"#!/bin/sh\n").expect("stand-in real git");
        let bin: PathBuf = root.join("bin");
        slow_stand_in(bin.as_path(), status);
        let context: PushGit = push_git_through(bin.as_path(), root.as_path());
        let answer = run_upload_bounded(&context, &plan());
        let (awaited, ticks): (Result<Option<PushAttempt>, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(run_upload_bounded_awaiting(&context, &plan()));
        assert_eq!(format!("{awaited:?}"), format!("{answer:?}"), "{status}");
        assert!(ticks > 0, "the wait for the upload held the thread: the task beside it never ran");
        remove(root.as_path());
    }
}

/// What:
///  The awaited upload asks for a place of the process's bound before it looks `git-lfs` up,
/// so the filesystem lookup is a job of the blocking pool and never the task that shares the round.
/// Why:
///   A lookup on the task would hold every other directory of the round,
/// and no job would ever ask for the place this control holds
///  (`testing/parallel_test_support.rs`).
/// The bound is held for the life of the process,
/// so this runs in a test process of its own.
#[test]
fn the_awaited_upload_asks_for_a_place_of_the_processs_bound_before_it_reads_the_path() {
    let root: PathBuf = fixture("lfs-upload-awaiting-place");
    std::fs::write(root.join("real-git"), b"#!/bin/sh\n").expect("stand-in real git");
    let bin: PathBuf = root.join("bin");
    slow_stand_in(bin.as_path(), 0);
    let context: PushGit = push_git_through(bin.as_path(), root.as_path());
    let (given_back, finished) = crate::testing::parallel_test_support::hold_the_only_place();
    let started = crate::lazy_runtime::spawn_on_runtime(async move { return run_upload_bounded_awaiting(&context, &plan()).await; });
    assert!(started.is_some(), "the runtime has workers, so the awaiting upload can run on it");
    let asked: bool = finished.recv_timeout(crate::testing::parallel_test_support::PATIENCE).expect("the holder of the place answered");
    assert!(asked, "the upload read the caller's PATH without asking for a place of the process's bound");
    // The place it asked for is free again, so the upload finishes; the stand-in answers nothing and succeeds.
    let answered = std::thread::scope(|scope| {
        return scope
            .spawn(|| {
                return crate::lazy_runtime::run_on_runtime(async move {
                    return match started {
                        Some(handle) => handle.await.expect("the upload ran"),
                        None => panic!("the runtime has workers"),
                    };
                });
            })
            .join()
            .expect("the thread that waits for the upload ended");
    });
    assert!(answered.expect("the runtime answered").expect("the stand-in git-lfs answers").is_none(), "the stand-in succeeds");
    assert!(given_back.load(std::sync::atomic::Ordering::SeqCst), "the holder gave the place back after the ask");
    remove(root.as_path());
}

//region An upload that shows no sign of work, and one that needed a login

/// The stall window these controls give the upload,
/// in whole seconds:
/// short enough to assert, long enough that a process start under load fits inside it.
const STALL_SECONDS: &str = "2";

/// How much longer than the stall window and the stop grace a control's run may take under host load,
/// in whole seconds.
const LOAD_ALLOWANCE_SECONDS: u64 = 60;

/// What:
///  A stand-in program that never answers,
/// which is the shape of a server that accepts the connection and never replies.
/// It records its own number and then waits far past every bound.
/// Why:
///   The bound must be shown to end the program and to report the stop,
/// not to wait for it to decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fs.writeFileSync(bin/git-lfs, `#!/bin/sh\necho $$ > '${pid}'\nexec sleep ${HOLD}`)
/// ```
fn never_answering_program(bin: &std::path::Path, pid_file: &std::path::Path) {
    std::fs::create_dir_all(bin).expect("bin directory");
    let source: String = format!("#!/bin/sh\necho $$ > '{}'\nexec /usr/bin/sleep {HOLD_SECONDS}\n", pid_file.display());
    executable(bin.join("git-lfs").as_path(), source.as_bytes());
}

/// Whether a process number still names a living process.
fn alive(pid: i32) -> bool {
    // SAFETY: signal 0 takes no pointers and only asks whether the number names a process.
    return unsafe { libc::kill(pid, 0) } == 0;
}

/// An upload that shows no sign of work is stopped at the push's stall window,
/// its program is ended,
/// and the person is told the stop,
/// never a status the program chose.
#[test]
fn an_upload_that_shows_no_sign_of_work_is_stopped_at_the_stall_window() {
    let root: PathBuf = fixture("lfs-upload-stall");
    let pid_file: PathBuf = root.join("upload.pid");
    let bin: PathBuf = root.join("bin");
    never_answering_program(bin.as_path(), pid_file.as_path());
    let state: String = root.join("state").to_string_lossy().into_owned();
    let variables = [("XDG_STATE_HOME", state.as_str()), (crate::bounded_push::PUSH_DEADLINE_VARIABLE, STALL_SECONDS)];
    let started: std::time::Instant = std::time::Instant::now();
    let (code, diagnostics) = run_with(root.as_path(), &[bin.as_path()], &[], &plan(), &variables);
    let elapsed: std::time::Duration = started.elapsed();
    assert_eq!(code, Some(i32::try_from(STOPPED_EXIT_CODE).unwrap_or(1)), "{diagnostics}");
    assert!(diagnostics.contains(&format!("made no progress for {STALL_SECONDS} seconds and was stopped")), "{diagnostics}");
    assert!(diagnostics.contains("was not started and nothing was published"), "{diagnostics}");
    assert!(
        elapsed < std::time::Duration::from_secs(STALL_SECONDS.parse::<u64>().expect("a whole number") + STOP_GRACE.as_secs() + LOAD_ALLOWANCE_SECONDS),
        "the upload was stopped at its window and its grace, not when the program chose to end: {elapsed:?}"
    );
    let recorded: String = std::fs::read_to_string(pid_file.as_path()).unwrap_or_default();
    let pid: i32 = recorded.trim().parse().unwrap_or(0);
    assert!(pid > 0, "the stand-in recorded its number: {recorded:?}");
    assert!(!alive(pid), "the stopped upload's program is still running as {pid}");
    remove(root.as_path());
}

/// The line Git and git-lfs print when a login is missing and no prompt may open,
/// as Git 2.56.0 writes it.
const LOGIN_NEEDED_LINE: &str = "fatal: could not read Username for 'https://lfs.example.invalid': No such device or address";

/// A destination that carries a user and a token,
/// as the incident's `lfs.url` did.
const CREDENTIALED_DESTINATION: &str = "https://user:secret@lfs.example.invalid/objects";

/// A plan for that destination,
/// with no refs,
/// so the upload runs the scan form the incident's push used.
fn credentialed_plan() -> UploadPlan {
    return UploadPlan {
        remotes: words(&[CREDENTIALED_DESTINATION]),
        refs: Vec::new(),
        everything: true,
        submodules: false,
        full_scan: true,
        objects: None,
    };
}

/// What:
///  A failed upload whose program said it needed a login is explained with the server's name,
/// the reason nothing asked,
/// and every way to give `git-lfs` a login,
/// with no credential in any of it.
/// Why:
///   This refusal is the wrapper's own doing (it refused to prompt),
/// so it is the one whose way forward only the wrapper can name
///  (`doc/troubleshooting/lfs-credential-in-url.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(diagnostics).toContain('the server `lfs.example.invalid`'); expect(diagnostics).not.toContain('secret');
/// ```
#[test]
fn an_upload_that_needed_a_login_names_the_server_and_how_to_give_one() {
    let root: PathBuf = fixture("lfs-upload-login");
    let counter: PathBuf = root.join("count.txt");
    failing_stand_in(root.join("bin").as_path(), counter.as_path(), 99, LOGIN_NEEDED_LINE, 128);
    let state: String = root.join("state").to_string_lossy().into_owned();
    let variables = [("XDG_STATE_HOME", state.as_str())];
    let (code, diagnostics) = run_with(root.as_path(), &[root.join("bin").as_path()], &[], &credentialed_plan(), &variables);
    assert_eq!(code, Some(128), "{diagnostics}");
    assert!(diagnostics.contains("the server `lfs.example.invalid`"), "the server is named: {diagnostics}");
    assert!(diagnostics.contains("no controlling terminal"), "the reason nothing asked is stated: {diagnostics}");
    assert!(diagnostics.contains("credential.helper"), "the way to store a login is named: {diagnostics}");
    assert!(diagnostics.contains("doc/troubleshooting/lfs-credential-in-url.md"), "the note is cited: {diagnostics}");
    assert!(!diagnostics.contains("secret"), "no credential is repeated: {diagnostics}");
    remove(root.as_path());
}

/// A failure that says nothing about a login gets no login paragraph:
/// a missing object keeps the words it had.
#[test]
fn a_failure_that_names_no_login_keeps_the_words_it_had() {
    let root: PathBuf = fixture("lfs-upload-no-login");
    let counter: PathBuf = root.join("count.txt");
    failing_stand_in(root.join("bin").as_path(), counter.as_path(), 99, "Git LFS upload failed:\n  (missing) big.bin (abc)", 2);
    let state: String = root.join("state").to_string_lossy().into_owned();
    let variables = [("XDG_STATE_HOME", state.as_str())];
    let (code, diagnostics) = run_with(root.as_path(), &[root.join("bin").as_path()], &[], &credentialed_plan(), &variables);
    assert_eq!(code, Some(2), "{diagnostics}");
    assert!(!diagnostics.contains("needed a login"), "{diagnostics}");
    assert_eq!(diagnostics, format!("Git LFS upload failed:\n  (missing) big.bin (abc)\n{}", failed_message("exited with status 2")), "{diagnostics}");
    remove(root.as_path());
}

//endregion

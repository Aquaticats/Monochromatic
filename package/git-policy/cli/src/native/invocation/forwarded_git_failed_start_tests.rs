//! What:
//!  The wrapper's own answers after a real Git it cannot start:
//!  the status it ends by,
//!  the words it says,
//!  and both even when its standard error is a pipe nobody reads,
//!  at each of the two places that answer a failed start.
//! Why:
//!  Real Git does not run,
//!  and `SPEC.md`'s "Stream and exit contract" says `2` means usage,
//!  config,
//!  patch,
//!  transaction,
//!  or engine failure,
//!  so a failed start is that status and not a death by `SIGPIPE`.
//!  `Command::exec` sets `SIGPIPE` back to the kernel's default before it tries,
//!  and a start that cannot run its program returns and leaves that default in the process that goes on,
//!  so the other half of the record is the action `exec_retrying` puts back
//!  (`doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md`,
//!  issue 6607).
//!  `invocation/forwarding_tests.rs` proves the replacement and the words in a child that rewrites both;
//!  these controls read what a person reads,
//!  so a change to the status either place returns turns one of them red
//!  (the alpha.49 behaviour read,
//!  answer 8).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(runInChild('entry', 'captured').stderr).toContain('cannot start');
//! expect(runInChild('entry', 'gone').status).toBe(2);
//! expect(runInChild('coordinated', 'captured').stderr).toContain('cannot start');
//! expect(runInChild('coordinated', 'gone').status).toBe(2);
//! ```

use crate::action::ENGINE_FAILURE_EXIT_CODE;
use crate::child_test_process::{
    ChildTest,
    finished,
};
use crate::forwarded_command::{
    ResolvedCommand,
    Route,
};
use crate::index_writer_forward::CoordinatedForward;
use crate::invocation::entry_run_forwarded_actions::run_coordinated_action;
use crate::start_directory::StartDirectory;
use crate::test_support::{
    REAL_GIT,
    executable,
    fixture,
    isolated_overlay,
    remove,
    repository,
};
use crate::worktree_identity::resolve_worktree_identity;
use std::ffi::{
    OsStr,
    OsString,
};
use std::os::fd::OwnedFd;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Output,
    Stdio,
};

/// The variable that marks the child of these controls.
const FAILED_START_CHILD_VARIABLE: &str = "CLI_GIT_FAILED_START_OF_THE_ENTRY_CHILD";

/// The variable that names the command the entry control's child runs.
const FAILED_START_COMMAND_VARIABLE: &str = "CLI_GIT_FAILED_START_OF_THE_ENTRY_COMMAND";

/// What:
///  The program the wrapper finds as its real Git,
///  which the kernel refuses to start because the interpreter it names does not exist.
/// Why:
///  The file is there and it is executable,
///  so the wrapper selects it as Git and the failure is the start and nothing earlier,
///  which is the shape the alpha.49 behaviour read measured against both released binaries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNSTARTABLE_GIT = '#!/nonexistent/interpreter-for-this-control\nexit 0\n';
/// ```
const UNSTARTABLE_GIT: &[u8] = b"#!/nonexistent/interpreter-for-this-control\nexit 0\n";

/// The words the entry says when real Git cannot be started,
/// as `invocation/entry.rs` and `invocation/entry_run_forwarded_actions.rs` word them.
const CANNOT_START: &str = "cannot start";

/// What:
///  The two queries of the entry control,
///  each with the words the child hands the entry.
/// Why:
///  A query is what the entry forwards without asking Git for the repository's location first,
///  which is the read an unstartable Git fails at in every command that names a subcommand
///  (measured 2026-11-27 with the built binary on a fixture repository:
///  `status`,
///  `commit`,
///  `add`,
///  `reset` and `stash` each ended with
///  "could not start ... to read the repository location" before any forward).
///  The two are one arm of the entry's answer and one spelling of a query Git answers itself,
///  so the control shows the forward and not the announced answer of `--version`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_QUERIES = [['exec-path', ['--exec-path']], ['html-path', ['--html-path']]];
/// ```
const THE_QUERIES: &[(&str, &[&str])] = &[
    ("exec-path", &["--exec-path"]),
    ("html-path", &["--html-path"]),
];

/// What:
///  A command the entry forwards whose real Git cannot be started ends with status `2` and its own words,
///  and ends with that status even where its standard error is a pipe nobody reads.
/// Why:
///  Status `2` is what the record gives a command where real Git does not run,
///  and a death by `SIGPIPE` at the first word is what a caller read before issue 6607's fix.
///  The body runs in a child of its own because the entry replaces this process when it succeeds and
///  changes a signal's action when it fails.
///  Each arm reads only what a person reads:
///  the status the entry returned,
///  and the words said on standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [name] of THE_QUERIES) {
///   const captured = startChild('entry', name, false);
///   expect(captured.status.code).toBe(2);
///   expect(captured.stderr).toContain('cannot start');
///   expect(startChild('entry', name, true).status.code).toBe(2);
/// }
/// ```
#[test]
fn a_wrapper_whose_git_cannot_start_ends_with_status_two_and_its_words() {
    if !this_is_the_child() {
        for (name, _words) in THE_QUERIES {
            let captured: Output = run_the_entry_in_a_child(name, false);
            let stderr: String = String::from_utf8_lossy(captured.stderr.as_slice()).into_owned();
            assert_eq!(
                captured.status.code(),
                Some(ENGINE_FAILURE_EXIT_CODE),
                "a `{name}` whose real Git cannot be started ends with the status the record gives a command \
                 where Git does not run, and this one ended as {:?}: {stderr}",
                captured.status
            );
            assert!(
                stderr.contains(CANNOT_START) && stderr.contains("git"),
                "the entry says why the start failed, naming the program it found: {stderr}"
            );
            let gone: Output = run_the_entry_in_a_child(name, true);
            assert_eq!(
                gone.status.code(),
                Some(ENGINE_FAILURE_EXIT_CODE),
                "a `{name}` whose real Git cannot be started and whose standard error no reader holds ends with \
                 the same status, and this one ended as {:?}: a write of the wrapper's own words ended the \
                 process by the signal instead",
                gone.status
            );
        }
        return;
    }
    // What: the query this child runs, which its parent named, in a directory that is no repository.
    // Why:  a query inside a repository would ask Git for the repository's location first, and that read is
    //       another place's answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const words = THE_QUERIES.find(([name]) => name === process.env[FAILED_START_COMMAND_VARIABLE])[1];
    // ```
    let name: String =
        std::env::var(FAILED_START_COMMAND_VARIABLE).expect("the query this child runs");
    let words: &[&str] = THE_QUERIES
        .iter()
        .find(|(named, _words)| return *named == name)
        .expect("a query of the two")
        .1;
    let root: PathBuf = fixture(format!("entry-failed-start-{name}").as_str());
    let _git: PathBuf = the_git_that_cannot_start(root.as_path());
    let started_in: PathBuf = a_directory_outside_every_repository(root.as_path());
    let mut arguments: Vec<OsString> =
        vec![OsString::from("-C"), started_in.clone().into_os_string()];
    arguments.extend(words.iter().map(|word| return OsString::from(*word)));
    let environment: Vec<(OsString, OsString)> = the_environment(root.as_path());
    let start: StartDirectory = StartDirectory::at(started_in.as_path());
    end_the_child_by_the_entry(arguments.as_slice(), &environment, &start, root.as_path());
}

/// What:
///  The forward that stays alive around Git answers a failed start at its own place:
///  status `2` and the words,
///  and that status where its standard error is a pipe nobody reads.
/// Why:
///  `invocation/entry_run_forwarded_actions.rs` returns this status from the arm that replaces the process
///  after a failed start,
///  and no path through the entry reaches that arm with an unstartable Git:
///  every command that names a subcommand asks Git for the repository's location first,
///  so that read answers first with its own engine failure.
///  So this control calls the coordinated action itself,
///  with a forward whose Git cannot start,
///  and a working index of its own so the coordination declines before it would start Git.
///  A command whose add or commit the wrapper keeps alive around Git is the shape the arm exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const captured = startChild('coordinated', '', false);
/// expect(captured.status.code).toBe(2);
/// expect(captured.stderr).toContain('cannot start');
/// expect(startChild('coordinated', '', true).status.code).toBe(2);
/// ```
#[test]
fn a_coordinated_forward_whose_git_cannot_start_ends_with_status_two_and_its_words() {
    if !this_is_the_child() {
        let captured: Output = run_the_coordinated_action_in_a_child(false);
        let stderr: String = String::from_utf8_lossy(captured.stderr.as_slice()).into_owned();
        assert_eq!(
            captured.status.code(),
            Some(ENGINE_FAILURE_EXIT_CODE),
            "the coordinated forward ends with the status the record gives a command where Git does not run, \
             and this arm ended as {:?}: {stderr}",
            captured.status
        );
        assert!(
            stderr.contains(CANNOT_START) && stderr.contains("git"),
            "the coordinated forward says why the start failed, naming the program it found: {stderr}"
        );
        let gone: Output = run_the_coordinated_action_in_a_child(true);
        assert_eq!(
            gone.status.code(),
            Some(ENGINE_FAILURE_EXIT_CODE),
            "the coordinated forward whose standard error no reader holds ends with the same status, and this arm \
             ended as {:?}: a write of the wrapper's own words ended the process by the signal instead",
            gone.status
        );
        return;
    }
    // What: an index of the command's own, so the coordination declines and the process is replaced.
    // Why:  with the real index the writer runs Git as a child and reports its status, which is another arm.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const root = fixture('entry-failed-start-coordinated'); const repo = repository(root, 'repo');
    // const forward = { realGit: join(root, 'bin/git'), arguments: ['-C', repo, 'commit', '--message=control'], overlay: [...isolatedOverlay(), ['GIT_INDEX_FILE', join(root, 'another-index')]], identity, command: { subcommand: 'commit', subcommandIndex: 2 }, environment: [], copy: undefined };
    // ```
    let root: PathBuf = fixture("entry-failed-start-coordinated");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git: PathBuf = the_git_that_cannot_start(root.as_path());
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        repo.clone().into_os_string(),
        OsString::from("commit"),
        OsString::from("--message=control"),
    ];
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((
        OsString::from("GIT_INDEX_FILE"),
        root.join("another-index").into_os_string(),
    ));
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.into_os_string()];
    let identity = resolve_worktree_identity(
        Path::new(REAL_GIT),
        prefix.as_slice(),
        isolated_overlay().as_slice(),
    )
    .expect("the fixture repository's identity");
    let forward: CoordinatedForward = CoordinatedForward {
        real_git: git,
        arguments: arguments.clone(),
        overlay,
        identity,
        command: ResolvedCommand {
            arguments,
            subcommand_index: 2,
            subcommand: Some(OsString::from("commit")),
            route: Route::Direct,
            shell_command: None,
        },
        environment: Vec::new(),
        copy: None,
    };
    let code: i32 = run_coordinated_action(&forward);
    remove(root.as_path());
    std::process::exit(code);
}

/// Whether this process is the child these controls started.
fn this_is_the_child() -> bool {
    return std::env::var_os(FAILED_START_CHILD_VARIABLE).is_some();
}

/// Run the entry for one query in a child,
/// end that child by the status the entry answers,
/// and answer what the child wrote.
fn run_the_entry_in_a_child(
    name: &str,
    standard_error_is_gone: bool,
) -> Output {
    let environment: [(&str, &OsStr); 2] = [
        (FAILED_START_CHILD_VARIABLE, OsStr::new("1")),
        (FAILED_START_COMMAND_VARIABLE, OsStr::new(name)),
    ];
    return finished(child_request(
        "invocation::entry_run_forwarded_actions::forwarded_git_failed_start_tests::a_wrapper_whose_git_cannot_start_ends_with_status_two_and_its_words",
        environment.as_slice(),
        standard_error_is_gone,
    ))
    .expect("the child ends within the deadline");
}

/// Run the coordinated action in a child,
/// end that child by the status the action answers,
/// and answer what the child wrote.
fn run_the_coordinated_action_in_a_child(standard_error_is_gone: bool) -> Output {
    let environment: [(&str, &OsStr); 1] = [(FAILED_START_CHILD_VARIABLE, OsStr::new("1"))];
    return finished(child_request(
        "invocation::entry_run_forwarded_actions::forwarded_git_failed_start_tests::a_coordinated_forward_whose_git_cannot_start_ends_with_status_two_and_its_words",
        environment.as_slice(),
        standard_error_is_gone,
    ))
    .expect("the child ends within the deadline");
}

/// What:
///  The request of a child of this test binary that runs `exact_test` with `environment`,
///  whose standard error is either a pipe this control reads or one whose reader is gone before it starts.
/// Why:
///  `head` that stopped reading leaves a pipe with no reader,
///  which is the ending these controls are about,
///  and the reader must be gone before the child writes rather than after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { exactTest, environment, standardError: 'pipe' }; if (gone) { const [r, w] = pipe(); r.close(); request.standardError = w; } return request;
/// ```
fn child_request<'a>(
    exact_test: &'a str,
    environment: &'a [(&'a str, &'a OsStr)],
    standard_error_is_gone: bool,
) -> ChildTest<'a> {
    let mut request: ChildTest<'_> = ChildTest {
        exact_test,
        environment,
        ..ChildTest::default()
    };
    if standard_error_is_gone {
        let (reader, writer) = std::io::pipe().expect("a pipe for the child's standard error");
        drop(reader);
        request.standard_error = Stdio::from(OwnedFd::from(writer));
    }
    return request;
}

/// What:
///  Run the entry over one command,
///  end this process by the status it answers,
///  and remove the fixture however the entry ends.
/// Why:
///  The entry returns only when it did not replace itself,
///  and a child that returned from its test would end with the harness's zero.
///  The entry's status is the whole assertion,
///  so the child ends by it rather than by a test's own result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { process.exit(runProcess(arguments, environment, start)); } finally { remove(root); }
/// ```
fn end_the_child_by_the_entry(
    arguments: &[OsString],
    environment: &[(OsString, OsString)],
    start: &StartDirectory,
    root: &Path,
) -> ! {
    let code: i32 = crate::entry::run_process(arguments, environment, start);
    remove(root);
    std::process::exit(code);
}

/// What:
///  The file the wrapper finds as its real Git,
///  which the kernel refuses to start.
/// Why:
///  One helper names it,
///  so the entry control and the coordinated one hand the code the same failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theGitThatCannotStart(root: string): string { mkdir(join(root, 'bin')); executable(join(root, 'bin/git'), UNSTARTABLE_GIT); return join(root, 'bin/git'); }
/// ```
fn the_git_that_cannot_start(root: &Path) -> PathBuf {
    let bin: PathBuf = root.join("bin");
    std::fs::create_dir_all(bin.as_path())
        .expect("the directory that holds the Git the wrapper finds");
    let git: PathBuf = bin.join("git");
    executable(git.as_path(), UNSTARTABLE_GIT);
    return git;
}

/// The directory the entry control starts in,
/// which is outside every repository.
fn a_directory_outside_every_repository(root: &Path) -> PathBuf {
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.as_path()).expect("a directory outside any repository");
    return elsewhere;
}

/// What:
///  The environment a session hands the wrapper,
///  with every place of it named inside the fixture:
///  the one `PATH` directory holding the Git the wrapper selects,
///  a home and the three `XDG_*_HOME` directories,
///  and a runtime directory with no bus.
/// Why:
///  The wrapper reads its configuration from the home its invocation names and records what it learned about Git
///  in the cache place,
///  so a fixture that named none of them would write in the person's home.
///  `PATH` names one directory because the wrapper promotes the conventional `/usr/bin/git` and
///  `/usr/local/bin/git` to the front of its search wherever `PATH` exposes them,
///  so a `PATH` that also held `/usr/bin` would hand it the real Git
///  (the alpha.49 behaviour read's probe named this one directory for the same reason).
///  No bus address is handed in,
///  so no service of this account can be asked for anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theEnvironment(root) // PATH = `${root}/bin`, HOME and XDG_* under root
/// ```
fn the_environment(root: &Path) -> Vec<(OsString, OsString)> {
    let overwritten: [&str; 12] = [
        "PATH",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_STATE_HOME",
        "XDG_RUNTIME_DIR",
        "DBUS_SESSION_BUS_ADDRESS",
        "GIT_INDEX_FILE",
        "GIT_CONFIG_GLOBAL",
        "GIT_DIR",
        "GIT_WORK_TREE",
        "CLI_GIT_TEST_ONLY_RUNTIME_PROBE",
    ];
    let mut environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    environment.retain(|(name, _value)| {
        return !overwritten.iter().any(|listed| return name == listed);
    });
    let pairs: [(&str, OsString); 7] = [
        ("PATH", root.join("bin").into_os_string()),
        ("HOME", root.join("home").into_os_string()),
        ("XDG_CONFIG_HOME", root.join("config").into_os_string()),
        ("XDG_CACHE_HOME", root.join("cache").into_os_string()),
        ("XDG_STATE_HOME", root.join("state").into_os_string()),
        ("XDG_RUNTIME_DIR", root.join("run").into_os_string()),
        (
            "GIT_CONFIG_GLOBAL",
            root.join("nonexistent-global-config").into_os_string(),
        ),
    ];
    for (name, value) in pairs {
        environment.push((OsString::from(name), value));
    }
    return environment;
}

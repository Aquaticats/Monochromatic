//! What:
//!  Exact-forwarding controls through the built executable.
//! Why:
//!  For commands the wrapper adds nothing to,
//!  a caller must not be able to tell
//!      the wrapper from Git:
//!  same argument bytes,
//!  streams,
//!  exit status and signals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, args)).toEqual(run('/usr/bin/git', args));
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, REAL_GIT, bounded, direct, executable, fixture, git, observe, remove,
    repository, wrapped,
};
/// The handle that makes a signal reach one process and never its successor.
use super::supervisor_pin::{Delivered, Pin};
/// One process's start time,
/// read from the kernel's table.
use super::supervisor_proc::stat_of;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::io::{BufRead, Read};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Run the same arguments through the wrapper and through real Git,
///  in one directory.
fn both(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[OsString],
    input: &[u8],
) -> (Observed, Observed) {
    let through_wrapper: Observed = observe(
        wrapped(fixture).current_dir(directory).args(arguments),
        input,
    );
    let through_git: Observed = observe(
        direct(fixture).current_dir(directory).args(arguments),
        input,
    );
    return (through_wrapper, through_git);
}

/// Build owned arguments from text.
fn text_arguments(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// `git --version` through a `git`-named link prints real Git 2.56.0's line,
///  then exactly one banner line naming the wrapper,
/// its version and the Git it shadows.
#[test]
fn version_is_gits_line_followed_by_the_banner() {
    let fixture: Fixture = fixture("version");
    let (through_wrapper, through_git) = both(
        &fixture,
        fixture.root.as_path(),
        text_arguments(&["--version"]).as_slice(),
        b"",
    );
    assert_eq!(through_git.stdout, b"git version 2.56.0\n");
    assert_eq!(through_git.code, Some(0));
    let mut expected: Vec<u8> = through_git.stdout.clone();
    expected.extend_from_slice(
        format!(
            "monochromatic-git-policy-cli {}, shadowing git at {REAL_GIT}\n",
            env!("CARGO_PKG_VERSION")
        )
        .as_bytes(),
    );
    assert_eq!(through_wrapper.stdout, expected);
    assert_eq!(through_wrapper.stderr, through_git.stderr);
    assert_eq!(through_wrapper.code, through_git.code);
    remove(&fixture);
}

/// The queries that are not announced stay byte-identical to real Git:
///  stdout,
/// stderr and exit code,
/// with and without global options before them.
#[test]
fn path_queries_are_identical_to_native_git() {
    let fixture: Fixture = fixture("path-queries");
    for rest in [
        vec!["--exec-path"],
        vec!["--html-path"],
        vec!["--man-path"],
        vec!["--info-path"],
        vec!["-C", "/", "--html-path"],
    ] {
        let (through_wrapper, through_git) = both(
            &fixture,
            fixture.root.as_path(),
            text_arguments(rest.as_slice()).as_slice(),
            b"",
        );
        assert_eq!(through_git.code, Some(0), "{rest:?}");
        assert!(!through_git.stdout.is_empty(), "{rest:?}");
        assert_eq!(through_wrapper, through_git, "{rest:?}");
    }
    remove(&fixture);
}

/// Every argument reaches Git as its own unchanged element,
///  including non-UTF-8 and empty ones.
#[test]
fn argument_bytes_reach_git_unchanged() {
    let fixture: Fixture = fixture("argv");
    let adversarial: Vec<OsString> = vec![
        OsString::from_vec(b"non-utf8-\xff\xfe\x80".to_vec()),
        OsString::new(),
        OsString::from("two words"),
        OsString::from("--"),
        OsString::from("-C"),
        OsString::from("$(touch injected); `id` 'single' \"double\" \\ * ? ~ ! #"),
        OsString::from("line\nbreak\ttab"),
        OsString::from("--no-enforce-worktree"),
        OsString::from("../../etc/passwd"),
        OsString::from("é\u{1F600}"),
    ];
    // Real Git echoes its arguments shell-quoted; identical bytes mean identical argv.
    let mut quoted: Vec<OsString> = text_arguments(&["rev-parse", "--sq-quote"]);
    quoted.extend(adversarial.iter().cloned());
    let (through_wrapper, through_git) =
        both(&fixture, fixture.root.as_path(), quoted.as_slice(), b"");
    assert_eq!(through_git.code, Some(0));
    assert!(through_git.stdout.windows(3).any(is_raw_marker));
    assert_eq!(through_wrapper, through_git);
    // A probe standing in for real Git prints each argument NUL-terminated.
    std::fs::create_dir(fixture.root.join("probe")).expect("probe directory");
    executable(
        fixture.root.join("probe/git").as_path(),
        b"#!/bin/sh\nprintf '%s\\0' \"$#\" \"$@\"\n",
    );
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("probe"));
    // `version` needs no repository fact, so the probe is started once, with the caller's arguments.
    let mut probe_arguments: Vec<OsString> = text_arguments(&["version"]);
    probe_arguments.extend(adversarial.iter().cloned());
    let probed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
        )
        .args(probe_arguments.as_slice()),
        b"",
    );
    // The wrapper's own hooks-off options come first; the caller's bytes follow them unchanged.
    let mut off: Vec<String> = vec![String::from("-c"), format!("core.hooksPath={}", git_policy_cli::hooks_off::NO_HOOKS_DIRECTORY)];
    for event in git_policy_cli::hooks_off::switched_off_events() {
        off.push(String::from("-c"));
        off.push(format!("hook.{event}.enabled=false"));
    }
    let count: usize = off.len() + 1 + adversarial.len();
    let mut expected: Vec<u8> = format!("{count}\0").into_bytes();
    for argument in off.iter().map(String::as_str).chain(std::iter::once("version")) {
        expected.extend_from_slice(argument.as_bytes());
        expected.push(0);
    }
    for argument in &adversarial {
        expected.extend_from_slice(argument.as_bytes());
        expected.push(0);
    }
    assert_eq!(probed.stdout, expected);
    assert_eq!(probed.code, Some(0));
    assert!(!fixture.root.join("injected").exists());
    remove(&fixture);
}

/// Named predicate:
///  the three raw non-UTF-8 bytes of the first adversarial argument.
fn is_raw_marker(window: &[u8]) -> bool {
    return window == b"\xff\xfe\x80";
}

/// A non-UTF-8 directory in the global prefix and a non-UTF-8 path in output are preserved.
#[test]
fn non_utf8_paths_select_and_report_exactly() {
    let fixture: Fixture = fixture("bytes");
    let name: OsString = OsString::from_vec(b"repo-\xff\xfe".to_vec());
    let repo: PathBuf = repository(&fixture, name.as_os_str());
    std::fs::write(repo.join(OsStr::from_bytes(b"file-\xfd.txt")), b"content\n")
        .expect("non-UTF-8 file name");
    for rest in [
        vec!["rev-parse", "--show-toplevel"],
        vec!["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        vec!["ls-files", "--others", "-z"],
    ] {
        let mut arguments: Vec<OsString> =
            vec![OsString::from("-C"), repo.clone().into_os_string()];
        arguments.extend(text_arguments(rest.as_slice()));
        let (through_wrapper, through_git) =
            both(&fixture, fixture.root.as_path(), arguments.as_slice(), b"");
        assert_eq!(through_git.code, Some(0), "{rest:?}");
        assert!(
            through_git.stdout.windows(1).any(is_high_byte),
            "{rest:?} must print raw path bytes"
        );
        assert_eq!(through_wrapper, through_git, "{rest:?}");
    }
    remove(&fixture);
}

/// Named predicate:
///  a byte that cannot appear in ASCII output.
fn is_high_byte(window: &[u8]) -> bool {
    return window[0] >= 0xfd;
}

/// Standard input reaches Git and binary standard output returns unchanged.
#[test]
fn stdin_and_binary_stdout_pass_through() {
    let fixture: Fixture = fixture("streams");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("blob.bin"), b"\x00\xff\xfe binary \r\n\x80").expect("blob");
    git(&fixture, repo.as_path(), &["add", "blob.bin"]);
    let listed = git(&fixture, repo.as_path(), &["ls-files", "--stage"]);
    let object_id: Vec<u8> = listed.stdout[7..47].to_vec();
    let mut input: Vec<u8> = object_id.clone();
    input.extend_from_slice(b"\nHEAD\nmissing-object\n");
    for rest in [
        vec!["cat-file", "--batch"],
        vec!["cat-file", "--batch-check"],
    ] {
        let (through_wrapper, through_git) = both(
            &fixture,
            repo.as_path(),
            text_arguments(rest.as_slice()).as_slice(),
            input.as_slice(),
        );
        assert_eq!(through_git.code, Some(0), "{rest:?}");
        assert!(
            through_git.stdout.starts_with(object_id.as_slice()),
            "{rest:?}"
        );
        assert_eq!(through_wrapper, through_git, "{rest:?}");
    }
    remove(&fixture);
}

/// Git's own exit codes and diagnostics are reported unchanged for failures and usage errors.
#[test]
fn exit_codes_and_stderr_match_native_git() {
    let fixture: Fixture = fixture("exit");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "tracked.txt"]);
    git(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=tracked"],
    );
    std::fs::write(repo.join("tracked.txt"), b"two\n").expect("modify");
    for (directory, arguments, code) in [
        (
            repo.as_path(),
            vec!["rev-parse", "--verify", "--quiet", "refs/heads/absent"],
            1,
        ),
        (
            repo.as_path(),
            vec!["rev-parse", "--verify", "refs/heads/absent"],
            128,
        ),
        (repo.as_path(), vec!["diff", "--exit-code", "--quiet"], 1),
        (repo.as_path(), vec!["diff", "--exit-code"], 1),
        (repo.as_path(), vec!["show", "--no-such-option"], 128),
        (fixture.root.as_path(), vec!["log"], 128),
        (
            fixture.root.as_path(),
            vec!["--no-such-global-option", "status"],
            129,
        ),
        (fixture.root.as_path(), vec!["-C"], 129),
        (fixture.root.as_path(), vec![], 1),
        (fixture.root.as_path(), vec!["--no-pager"], 1),
        (repo.as_path(), vec!["status", "--porcelain"], 0),
        (repo.as_path(), vec!["branch", "--list"], 0),
    ] {
        let (through_wrapper, through_git) = both(
            &fixture,
            directory,
            text_arguments(arguments.as_slice()).as_slice(),
            b"",
        );
        assert_eq!(through_git.code, Some(code), "{arguments:?}");
        assert_eq!(through_wrapper, through_git, "{arguments:?}");
    }
    remove(&fixture);
}

/// What:
///  Run a command whose reader takes one line and then goes away,
/// and report how the command ended:
/// its exit
///  code and the signal that ended it,
/// one of which is absent.
/// Why:
///  This is the shape of `git log -p | head -1`:
/// the pipeline's reader exits after its one line,
/// the pipe
///  closes,
/// and the command writing to it is ended by `SIGPIPE`.
/// A caller tells the two apart by `WIFSIGNALED` and by the
///  status a shell reports (128 + the signal).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn(cmd, { stdio: ['ignore', 'pipe', 'pipe'] }); readOneLine(child.stdout); child.stdout.destroy(); return wait(child);
/// ```
fn ended_after_one_line(mut command: Command) -> (Option<i32>, Option<i32>) {
    let mut child: Child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the command starts");
    let stdout: std::process::ChildStdout = child.stdout.take().expect("piped standard output");
    let mut reader: std::io::BufReader<std::process::ChildStdout> = std::io::BufReader::new(stdout);
    let mut line: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut line).expect("the first line");
    assert!(!line.is_empty(), "the command wrote no line, so a reader that leaves proves nothing");
    // Dropping the reader closes the pipe, which is what a reader that exits does.
    drop(reader);
    // Standard error is drained so that a command writing there cannot block on a pipe nobody reads.
    let mut errors: std::process::ChildStderr = child.stderr.take().expect("piped standard error");
    let drained = std::thread::spawn(move || {
        let mut kept: Vec<u8> = Vec::new();
        let _read = errors.read_to_end(&mut kept);
        return kept;
    });
    let status: std::process::ExitStatus = child.wait().expect("the command ends");
    let _said: Vec<u8> = drained.join().expect("the reader of standard error");
    return (status.code(), status.signal());
}

/// What:
///  A read-only command whose reader goes away after one line,
/// run in a linked worktree,
/// where the wrapper
///  keeps a process around Git to settle the ignored-state copy.
/// Why:
///  Native Git dies of `SIGPIPE`,
/// and so does a wrapper that replaced itself with Git;
/// the copy route runs Git as a child
///  and reported its death by a signal as exit status 1,
/// so a script that tells "the reader went away" from "Git failed" took the wrong branch
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(endedAfterOneLine(wrappedIn(linked))).toEqual(endedAfterOneLine(directIn(linked)));
/// ```
#[test]
fn a_reader_that_goes_away_ends_the_run_by_the_signal_native_git_dies_of() {
    let fixture: Fixture = fixture("sigpipe-copy");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    // Past one pipe buffer, so Git is still writing when the reader has gone.
    let body: String = (0..20_000).map(|line| { return format!("line {line}\n"); }).collect();
    std::fs::write(repo.join("big.txt"), body).expect("a file whose patch is past one pipe buffer");
    git(&fixture, repo.as_path(), &["add", "big.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=big"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("UTF-8")]);
    let mut native: Command = direct(&fixture);
    native.current_dir(&linked).args(["log", "-p", "--all"]);
    let through_native: (Option<i32>, Option<i32>) = ended_after_one_line(native);
    assert_eq!(through_native, (None, Some(libc::SIGPIPE)), "the control: native Git ends of SIGPIPE");
    let mut through: Command = Command::new(fixture.root.join("bin/git"));
    super::support::hermetic(&mut through, &fixture, super::support::wrapper_first_path(&fixture).as_os_str());
    through.current_dir(&linked).args(["log", "-p", "--all"]);
    let through_wrapper: (Option<i32>, Option<i32>) = ended_after_one_line(through);
    assert_eq!(through_wrapper, through_native, "the wrapper's caller sees what Git's caller sees");
    remove(&fixture);
}

/// What:
///  Write a stand-in `git` that answers the wrapper's own reads with the real Git,
/// and where the wrapper runs the person's
///  command (`add`),
/// records its own process number in `pid_file` and then waits on standard input.
/// Why:
///   The coordinated route keeps a process around Git,
/// so the control needs a Git child it can name and
///  signal while the wrapper waits for it.
/// The wrapper also runs the command line itself to expand an alias,
/// with the output
///  captured and the input closed,
/// and that read is marked in the environment every start carries
/// (`started_as_git.rs`):
/// answering it with the blocking branch would leave that read waiting on the person's terminal.
/// A command whose standard input is an open
///  pipe of the caller's is what Git is given here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExecutable(git, `#!/bin/sh\ncase "$CLI_GIT_STARTED_AS_GIT" in *query*) exec ${REAL_GIT} "$@";; esac\n for token in "$@"; do if [ "$token" = add ]; then echo $$ > '${pidFile}'; read line; exit 0; fi; done; exec ${REAL_GIT} "$@"`);
/// ```
fn stand_in_that_waits_before_writing_the_index(path: &Path, pid_file: &Path) {
    let script: String = format!(
        "#!/bin/sh\n\
         case \"${{CLI_GIT_STARTED_AS_GIT}}\" in\n\
         *query*) exec {REAL_GIT} \"$@\" ;;\n\
         esac\n\
         for token in \"$@\"; do\n\
         if [ \"$token\" = \"add\" ]; then\n\
         echo $$ > '{pid_file}'\n\
         read -r line\n\
         exit 0\n\
         fi\n\
         done\n\
         exec {REAL_GIT} \"$@\"\n",
        pid_file = pid_file.display(),
    );
    executable(path, script.as_bytes());
}

/// What:
///  The process number of the stand-in the wrapper is waiting on,
/// once the file names a process that is still there.
/// Why:
///   The control must signal the wrapper's Git and not a process that ended,
/// and the stand-in writes the number only where
///  the wrapper runs the person's command,
/// so a number this returns is one the wrapper is waiting for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitForFile(pidFile) // parseInt(readFileSync(pidFile, 'utf8'))
/// ```
fn stand_in_number(pid_file: &Path) -> i32 {
    for _attempt in 0..2_000 {
        if let Ok(text) = std::fs::read_to_string(pid_file) {
            let number: String = text.trim().to_owned();
            if let Ok(pid) = number.parse::<i32>()
                && pid > 1
            {
                return pid;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the stand-in never recorded its process number at {}", pid_file.display());
}

/// What:
///  One run of `git add` through the wrapper with the stand-in as Git,
/// whose caller holds its standard input open,
/// so the wrapper
///  waits for a Git that waits.
/// Why:
///   A signal control needs the child alive at the moment the signal is sent,
/// and every other input of the invocation has to be
///  the one an ordinary command gets.
/// No directory of real Git is on this `PATH`:
/// a conventional location that `PATH` exposes is promoted to the front of the search,
/// so `/usr/bin` here
///  would put the system Git ahead of the stand-in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(wrapper, ['add', 'a.txt'], { cwd: repo, env: { PATH: `${bin}:${scripts}` }, stdio: ['pipe', 'pipe', 'pipe'] })
/// ```
fn wrapper_waiting_on_a_git_that_waits(fixture: &Fixture, repo: &Path, scripts: &Path) -> Child {
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(scripts.as_os_str());
    let mut command: Command = Command::new(fixture.root.join("bin/git"));
    super::support::hermetic(&mut command, fixture, path.as_os_str());
    return command
        .current_dir(repo)
        .args(["add", "a.txt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the wrapper starts");
}

/// What:
///  Sends `signal` to the Git child of the pinned `wrapper`,
///  through a handle taken while that child was proven to be the wrapper's child.
/// Why:
///  A process number written by a stand-in names that process only while it lives:
/// once it is reaped the number
///  names the next process the account starts,
/// and a signal sent to it ends an unrelated program
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 3 of the round-4
///  review).
/// The number the stand-in wrote is a candidate;
/// the pin proves it is the wrapper's child,
/// and the pidfd it
///  holds makes the signal reach that child or nothing,
/// never a successor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = Pin.openChild(pinOfOurWrapper, numberFromTheFile); child?.signal('SIGTERM');
/// ```
fn signal_the_wrappers_git(wrapper: &Child, number: i32, signal: i32) -> bool {
    let wrapper_pin: Pin = pin_our_own_child(wrapper);
    let Some(child) = Pin::open_child(&wrapper_pin, number) else { return false };
    return child.signal(signal) == Delivered::Sent;
}

/// What:
///  The pin of a child of this test that it has not reaped yet.
/// Why:
///   A number is safe while the process is our own unreaped child,
/// so the start time is read from the table without a
///  race,
/// and the pin is taken from that reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Pin.open(Number(child.pid), statOf(child.pid).started)
/// ```
fn pin_our_own_child(child: &Child) -> Pin {
    let number: i32 = i32::try_from(child.id()).expect("a process number");
    let started: u64 = stat_of(number).expect("our unreaped child is in the process table").started;
    return Pin::open(number, started).expect("our unreaped child can be pinned");
}

/// What:
///  How long a signalled wrapper may take to end before the control calls it a hang.
/// Why:
///   A signal ends the wrapper at once,
/// so anything near this is a defect rather than a slow host,
/// and it is far under
///  the harness's own backstop for a command that never ends
/// (`binary_supervisor::SHARED_BACKSTOP_SECONDS`,
/// 600 seconds).
/// The control waits with this bound instead of
///  `Child::wait`,
/// which a test may not call without one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIGNAL_WAIT = 30_000;
/// ```
const SIGNAL_WAIT: std::time::Duration = std::time::Duration::from_secs(30);

/// What:
///  Waits for a child with a bound,
/// and ends it when the bound passes.
/// Why:
///   A test may not wait without a bound:
/// the covered run holds its own standard input open,
/// so a signal that never arrives
///  leaves the wrapper waiting for ever
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4 of the round-2 review).
/// A child the bound ends is named with what it printed,
/// so the failure says what happened rather than hanging.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitWithin(child: Child, bound: number): ExitStatus { const began = Date.now(); while (Date.now() - began < bound) { const status = child.tryWait(); if (status) return status; sleep(10); } child.kill(); throw new Error(`did not end within ${bound} ms`); }
/// ```
fn wait_within(child: &mut Child, bound: std::time::Duration) -> std::process::ExitStatus {
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < bound {
        if let Some(status) = child.try_wait().expect("the child's state") {
            return status;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let _killed = child.kill();
    let status: std::process::ExitStatus = child.wait().expect("the ended child");
    panic!("the command did not end within {bound:?}: {status:?}");
}

/// What:
///  A `SIGTERM` sent to the Git the wrapper runs for an index writer ends the wrapper by that signal,
/// exactly as it
///  ends native Git.
/// Why:
///  The coordinated route keeps a process around Git,
/// so Git's death comes back as a value,
/// and that value
///  is now the signal itself
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4 of the round-1
///  review).
/// The control beside it sends the same signal to the wrapper's own process,
/// which proves this run can show a death
///  by a signal at all.
/// Each run has a stand-in and a process-number file of its own,
/// and the control's standard input
///  is closed before the covered run starts:
/// a stand-in the control left behind reads its standard input,
/// and with the pipe closed it
///  ends instead of waiting,
/// so the covered run can never take its process number for its own
/// (finding 4 of the
///  round-2 review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const through = spawn(wrapper, ['add', 'a.txt']); kill(pidOfItsStandIn, 'SIGTERM'); expect(waitWithin(through)).toMatchObject({ code: null, signal: 'SIGTERM' });
/// ```
#[test]
fn a_signal_to_the_git_of_an_index_writer_ends_the_wrapper_by_that_signal() {
    let fixture: Fixture = fixture("signal-coordinated");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"content\n").expect("the file the command stages");
    let control_scripts: PathBuf = fixture.root.join("scripts-control");
    let covered_scripts: PathBuf = fixture.root.join("scripts-covered");
    std::fs::create_dir_all(&control_scripts).expect("control script directory");
    std::fs::create_dir_all(&covered_scripts).expect("covered script directory");
    let control_pid_file: PathBuf = fixture.root.join("control.pid");
    let covered_pid_file: PathBuf = fixture.root.join("covered.pid");
    stand_in_that_waits_before_writing_the_index(control_scripts.join("git").as_path(), control_pid_file.as_path());
    stand_in_that_waits_before_writing_the_index(covered_scripts.join("git").as_path(), covered_pid_file.as_path());
    // The control: the same invocation, and the signal sent to the wrapper rather than to its Git.
    let mut control: Child = wrapper_waiting_on_a_git_that_waits(&fixture, repo.as_path(), control_scripts.as_path());
    let control_pin: Pin = pin_our_own_child(&control);
    assert_eq!(control_pin.signal(libc::SIGTERM), Delivered::Sent, "the signal reaches the wrapper itself");
    let control_status: std::process::ExitStatus = wait_within(&mut control, SIGNAL_WAIT);
    // Closing the control's input ends a stand-in it left behind, whose number no other run can then take.
    drop(control.stdin.take());
    assert_eq!(control_status.signal(), Some(libc::SIGTERM), "the control: this run shows a death by a signal");
    assert_eq!(control_status.code(), None);
    // The covered case: the signal reaches only the Git the wrapper started.
    let mut through: Child = wrapper_waiting_on_a_git_that_waits(&fixture, repo.as_path(), covered_scripts.as_path());
    let stand_in: i32 = stand_in_number(covered_pid_file.as_path());
    assert!(signal_the_wrappers_git(&through, stand_in, libc::SIGTERM), "the signal reaches the wrapper's Git");
    // Standard input stays open until the wrapper is gone, so nothing but the signal ends the stand-in.
    let status: std::process::ExitStatus = wait_within(&mut through, SIGNAL_WAIT);
    drop(through.stdin.take());
    assert_eq!(status.code(), None, "not a normal exit with the status a shell prints for the death");
    assert_eq!(status.signal(), Some(libc::SIGTERM), "the caller sees the death native Git's caller sees");
    remove(&fixture);
}


/// What:
///  A child that never ends is ended by the bound,
/// and the control fails instead of waiting for ever.
/// Why:
///  A test may not wait without a bound:
/// the covered run of the signal control holds its own standard input open,
/// so a
///  signal that never arrives leaves the wrapper waiting until the test runner ends the whole binary
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4 of the round-2 review).
/// This is the bound itself,
/// with a child that waits on a pipe nobody closes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stubborn = spawn('/bin/sh', ['-c', 'read line'], { stdio: ['pipe'] }); expect(() => waitWithin(stubborn, 200)).toThrow(); expect(stubborn.pid).toBeGone();
/// ```
#[test]
fn a_wait_without_an_end_is_ended_at_its_bound() {
    let mut stubborn: Child = Command::new("/bin/sh")
        .arg("-c")
        .arg("read line")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the child starts");
    let began: std::time::Instant = std::time::Instant::now();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return wait_within(&mut stubborn, std::time::Duration::from_millis(200)); }));
    assert!(caught.is_err(), "the bound ends the wait");
    assert!(began.elapsed() < std::time::Duration::from_secs(10), "at its bound, not later");
    assert!(stubborn.try_wait().expect("the child's state").is_some(), "the child was ended, not left behind");
}


/// What:
///  A number a stale file names,
///  whose process the wrapper did not start,
///  is never signalled by the control.
/// Why:
///  The stand-in writes its number into a file,
/// and once it is reaped that number names the next process the
///  account starts;
/// a control that signals the number ends an unrelated program and its own assertion cannot
///  tell,
/// because `kill` succeeds
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 3 of the round-4
///  review).
/// The rule of this repository is that a number is safe only while the process is our own unreaped
/// child,
/// and  anything else goes through a pidfd taken while the number was proven to name the wrapper's child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = spawn('sh', ['-c', 'read line']); writeFileSync(pidFile, stale.pid);
/// expect(signalTheWrappersGit(wrapper, Number(readFileSync(pidFile)))).toBe(false); expect(staleAlive()).toBe(true);
/// ```
#[test]
fn the_control_never_signals_a_number_that_is_not_the_wrappers_child() {
    let fixture: Fixture = fixture("signal-not-the-wrappers-child");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"content\n").expect("the file the command stages");
    let scripts: PathBuf = fixture.root.join("scripts");
    std::fs::create_dir_all(&scripts).expect("script directory");
    stand_in_that_waits_before_writing_the_index(scripts.join("git").as_path(), fixture.root.join("covered.pid").as_path());
    let mut through: Child = wrapper_waiting_on_a_git_that_waits(&fixture, repo.as_path(), scripts.as_path());
    // A live process of this test's own, which the wrapper did not start.
    let mut unrelated: Child = Command::new("/bin/sh")
        .arg("-c")
        .arg("read line")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the unrelated child starts");
    let unrelated_number: i32 = i32::try_from(unrelated.id()).expect("a process number");
    assert!(
        !signal_the_wrappers_git(&through, unrelated_number, libc::SIGTERM),
        "a number the wrapper did not start is not its child, so nothing is sent"
    );
    assert!(unrelated.try_wait().expect("the unrelated child's state").is_none(), "the unrelated process is untouched");
    // The write end must be closed, not merely taken: a handle that lives to the end of the control keeps the child
    //  waiting on its input.
    drop(unrelated.stdin.take());
    let _ended = through.kill();
    let _reaped = through.wait();
    let _reaped_unrelated = unrelated.wait();
    remove(&fixture);
}

/// Wait until the process with this ID is real Git,
///  proving the wrapper replaced itself.
fn wait_until_git(child: &Child) {
    let link: PathBuf = PathBuf::from(format!("/proc/{}/exe", child.id()));
    for _attempt in 0..2_000 {
        if let Ok(target) = std::fs::read_link(&link)
            && target == Path::new(REAL_GIT)
        {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("process {} never became {REAL_GIT}", child.id());
}

/// A signal sent to the wrapper's process ID ends Git itself,
///  and the caller sees that signal.
#[test]
fn signals_reach_git_in_the_same_process() {
    let fixture: Fixture = fixture("signal");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    for (name, number) in [("TERM", 15), ("INT", 2), ("HUP", 1)] {
        // No time-bound helper here: the signalled process ID must be the wrapper's own.
        let mut child: Child = Command::new(fixture.root.join("bin/git"))
            .env_clear()
            .env("PATH", super::support::wrapper_first_path(&fixture))
            .env("HOME", fixture.root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .current_dir(&repo)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("wrapper starts");
        // Git blocks reading the open pipe; the same process ID now runs Git.
        wait_until_git(&child);
        let sent = Command::new("/bin/sh")
            .arg("-c")
            .arg(format!("kill -{name} {}", child.id()))
            .status()
            .expect("signal sender");
        assert!(sent.success());
        let status = child.wait().expect("wait for the signalled process");
        assert_eq!(status.signal(), Some(number), "SIG{name}");
        assert_eq!(status.code(), None, "SIG{name}");
    }
    remove(&fixture);
}

/// Git receives the lock PID injection after the caller's own numbered configuration,
/// the forward-target marker,
///  and every other variable unchanged.
#[test]
fn environment_overlay_reaches_git() {
    let fixture: Fixture = fixture("environment");
    std::fs::create_dir(fixture.root.join("probe")).expect("probe directory");
    let probe: PathBuf = fixture.root.join("probe/git");
    executable(
        probe.as_path(),
        b"#!/bin/sh\nprintf '%s\\0' \"$GIT_CONFIG_COUNT\" \"$GIT_CONFIG_KEY_0\" \"$GIT_CONFIG_VALUE_0\" \"$GIT_CONFIG_KEY_1\" \"$GIT_CONFIG_VALUE_1\" \"$CLI_GIT_NATIVE_FORWARD_TARGET\" \"$RAW_VALUE\"\n",
    );
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(fixture.root.join("probe"));
    let observed: Observed = observe(
        bounded(
            &fixture,
            fixture.root.join("bin/git").as_path(),
            path.as_os_str(),
        )
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "user.name")
        .env("GIT_CONFIG_VALUE_0", "Caller Value")
        .env("RAW_VALUE", OsStr::from_bytes(b"raw-\xff"))
        .arg("version"),
        b"",
    );
    let mut expected: Vec<u8> = b"2\0user.name\0Caller Value\0core.lockfilePid\0true\0".to_vec();
    expected.extend_from_slice(probe.as_os_str().as_bytes());
    expected.extend_from_slice(b"\0raw-\xff\0");
    assert_eq!(observed.stdout, expected);
    assert_eq!(observed.code, Some(0));
    remove(&fixture);
}

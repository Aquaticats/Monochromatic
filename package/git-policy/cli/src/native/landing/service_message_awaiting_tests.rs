//! Controls for the awaited faces of the commit message and the identities:
//! a `-F` file that gives no answer holds no worker,
//! both faces make the same message and the same identities,
//! a message whose Git must be asked is built inside a task,
//! and a file read nobody answered is the answer an unreadable file gives.

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use crate::command_commit::CommitRegion;
use crate::command_options_query::WrapperFlags;
use crate::config_schema::CliGitConfig;
use crate::service_commit_args::parse_service_commit;
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
use crate::wrapper_controls::{Controls, no_controls};
use std::ffi::OsString;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The identity line a fixture Git resolves,
/// so the sign-off and the trailers have the committer they would have in a commit.
const COMMITTER: &str = "Fixture <fixture@example.invalid> 1762176000 +0000";

/// The shortest a wait can be and still leave a task beside it time to run on this host.
const SHORT_CEILING: Duration = Duration::from_millis(100);

/// How often a control looks at the flag a task beside the work sets.
const THE_TICK: Duration = Duration::from_millis(50);

/// The most a control waits for that task,
/// far beyond what a free worker needs,
/// so only a held worker reaches it.
const THE_PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  Stand a pipe with no writer at `path`,
/// so a read of it waits in the kernel until a writer opens it.
/// Why:
///  This is the one way to make a read of a file hold its thread with no end,
/// which is what a mount that stopped answering does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
fn stall_a_file(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  fixture takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where the file would be: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends the read a control left behind.
/// Why:
///  The read of the pool is abandoned,
/// never cancelled,
/// so the thread that holds it ends only when a writer appears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { close(openSync(path, 'w')); return } catch { await sleep(10) } }
/// ```
fn release_the_pipe(path: &Path) {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < THE_PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            Err(_not_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// What:
///  The arguments a command line puts after `commit`,
/// as the parser reads them.
/// Why:
///  The parser is what the service reads a line with,
/// so a control states a command line the way the person writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parsed = parseServiceCommit(['-m', 'subject'])
/// ```
fn parsed(arguments: &[&str]) -> (Vec<OsString>, ServiceCommitArgs) {
    let owned: Vec<OsString> = arguments.iter().map(|argument| { return OsString::from(argument) }).collect();
    let read: ServiceCommitArgs = parse_service_commit(owned.as_slice()).expect("a command line the service covers");
    return (owned, read);
}

/// What:
///  One command line's invocation,
/// over the arguments and the configuration every fixture shares.
/// Why:
///  Both faces take an invocation,
/// so a control hands the two of them the same one and nothing but the awaits can differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invocation = { context, cwd, arguments, subcommandIndex: 0, region, config, controls };
/// ```
fn invocation_of<'a>(
    context: &'a GitContext,
    repository: &'a Path,
    arguments: &'a [OsString],
    region: &'a CommitRegion,
    config: &'a CliGitConfig,
    controls: &'a Controls,
) -> CommitInvocation<'a> {
    return CommitInvocation { context, environment: &[], cwd: repository, arguments, subcommand_index: 0, region, config, controls };
}

/// Nothing of a commit's region matters to a message,
/// so every shape here starts from a commit that names nothing.
fn a_plain_region() -> CommitRegion {
    return CommitRegion {
        all: false,
        only: None,
        include: false,
        interactive: false,
        patch: false,
        dry_run: false,
        amend: false,
        allow_empty: false,
        fixup: None,
        pathspec_from_file: None,
        pathspec_file_nul: false,
        pathspecs: Vec::new(),
        wrapper: WrapperFlags { escape: Vec::new(), other: Vec::new() },
    };
}

/// What:
///  A `-F` file that gives no answer holds no worker:
/// a task beside the read runs while the read waits,
/// and the read ends at its ceiling as an unreadable file does.
/// Why:
///  The read is a file read of the working tree,
/// which waits in the kernel on a mount that stopped answering;
/// a task
///  that read it itself would hold a worker the runtime needs,
/// and no timer on that worker can end it.
/// The answer of a
///  read that gave nothing in time is the one an unreadable file gives,
/// so the caller hands the line to native `git commit`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(messagePath); assert(await runsBeside(() => messageFileWithin(100, messagePath)) === undefined)
/// ```
#[test]
fn a_message_file_that_gives_no_answer_does_not_hold_a_worker() {
    let root: PathBuf = fixture("message-awaiting-stalls");
    let stalled: PathBuf = root.join("MESSAGE");
    stall_a_file(stalled.as_path());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let reading_path: PathBuf = stalled.clone();
    let reading = runtime.spawn(async move { return message_file_within(SHORT_CEILING, reading_path.as_path()).await; });
    let began: std::time::Instant = std::time::Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    // The read ends at its ceiling, so its task is finished before the pipe is let go, and a read that held the worker cannot
    //  hold this control either.
    while !reading.is_finished() && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    release_the_pipe(stalled.as_path());
    let answered: Option<Vec<u8>> = runtime.block_on(async { return reading.await.expect("the reading task ends"); });
    assert!(ran_beside, "the task beside a message file that gives no answer still runs, so no worker is held by that read");
    assert_eq!(answered, None, "a message file that gave nothing in time is answered as an unreadable one, so native Git builds the line");
    assert!(began.elapsed() < THE_PATIENCE, "the read ended at its ceiling and never waited on the file without end: {} ms", began.elapsed().as_millis());
    remove(root.as_path());
}

/// What:
///  Both faces make the same message of every shape of command line the service covers.
/// Why:
///   The synchronous face is what a commit uses today and the awaited one is what it uses once the commit path is a task,
/// so the two must agree on the cleaning,
/// the trailer interpreter,
/// the reused commit and the file the person named,
/// or a
///  commit would change its words with the conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const line of shapes) assert.deepEqual(...both(line));
/// ```
#[test]
fn the_awaited_message_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("message-awaiting-agrees");
    let repository: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repository.join("MESSAGE").as_path(), b"from the file\n\nand its body\n").expect("the message file");
    std::fs::write(repository.join("COMMENTED").as_path(), b"subject\n# a comment\n").expect("a message file with a comment line");
    for line in ["From the commit", "An earlier subject"] {
        let named: String = format!("--message={line}");
        git(repository.as_path(), &["commit", "--quiet", "--allow-empty", named.as_str()]);
    }
    let earlier: String = git_text(repository.as_path(), &["rev-parse", "HEAD~1"]);
    let shapes: [(&[&str], Option<&str>); 10] = [
        (&["-m", "subject"], None),
        (&["-m", "subject", "-m", "body"], None),
        (&["-F", "MESSAGE"], None),
        (&["-C", "HEAD"], None),
        (&["--fixup", earlier.as_str()], None),
        (&["--amend", "--no-edit"], Some("the message it replaces\n")),
        (&["-m", "subject", "--signoff", "--trailer", "Reviewed-by: Reviewer <reviewer@example.invalid>"], None),
        (&["-F", "COMMENTED", "--cleanup=strip"], None),
        (&["-m", "", "--allow-empty-message"], None),
        (&["--cleanup=verbatim", "-m", "kept as given"], None),
    ];
    let context: GitContext = git_context();
    let region: CommitRegion = a_plain_region();
    let config: CliGitConfig = CliGitConfig::defaults();
    let controls: Controls = no_controls();
    for (arguments, previous) in shapes {
        let (owned, args): (Vec<OsString>, ServiceCommitArgs) = parsed(arguments);
        let invocation: CommitInvocation<'_> = invocation_of(&context, repository.as_path(), owned.as_slice(), &region, &config, &controls);
        let synchronous = build_message(&invocation, &args, COMMITTER, previous);
        let awaited = crate::lazy_runtime::run_on_runtime(build_message_awaiting(&invocation, &args, COMMITTER, previous)).expect("the runtime");
        assert_eq!(format!("{synchronous:?}"), format!("{awaited:?}"), "both faces make one message of {arguments:?}");
    }
    // The message the file carries is the person's text, cleaned as Git cleans it.
    let (owned, args): (Vec<OsString>, ServiceCommitArgs) = parsed(&["-F", "MESSAGE"]);
    let invocation: CommitInvocation<'_> = invocation_of(&context, repository.as_path(), owned.as_slice(), &region, &config, &controls);
    let asked: Result<Option<Built>, TransactionStop> = crate::lazy_runtime::run_on_runtime(build_message_awaiting(&invocation, &args, COMMITTER, None)).expect("the runtime");
    let built: Built = asked.expect("the awaited face makes the message of the file").expect("a message");
    assert_eq!(built.message, "from the file\n\nand its body\n", "the awaited face reads the message file the person named");
    remove(root.as_path());
}

/// What:
///  A message whose Git must be asked is built inside a task,
/// and the synchronous face cannot build it there at all.
/// Why:
///   The synchronous face runs its children by entering the runtime,
/// which a task may not do,
/// so a commit that
///  becomes a task needs the awaited face;
/// the message it makes is the reused commit's own,
/// which is what pins that the
///  children ran and were awaited rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reused = await runtime.spawn(() => buildMessageAwaiting(...)); const refused = await runtime.spawn(() => buildMessage(...)); assert(reused.ok && refused.err)
/// ```
#[test]
fn a_message_whose_git_must_be_asked_is_built_inside_a_task() {
    let root: PathBuf = fixture("message-awaiting-in-a-task");
    let repository: PathBuf = repository(root.as_path(), "repo");
    git(repository.as_path(), &["commit", "--quiet", "--allow-empty", "--message=the reused message"]);
    let raw: String = String::from_utf8(git_output(repository.as_path(), &["cat-file", "commit", "HEAD"]).stdout).expect("the raw commit as text");
    let author: String = String::from(raw.lines().find_map(|line| { return line.strip_prefix("author "); }).expect("the author header of the commit"));
    let context: GitContext = git_context();
    let region: CommitRegion = a_plain_region();
    let config: CliGitConfig = CliGitConfig::defaults();
    let controls: Controls = no_controls();
    let (owned, args): (Vec<OsString>, ServiceCommitArgs) = parsed(&["-C", "HEAD"]);
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let taken: Vec<OsString> = owned.clone();
    let inside: PathBuf = repository.clone();
    let asked: GitContext = context.clone();
    let building = invocation_of(&asked, inside.as_path(), taken.as_slice(), &region, &config, &controls);
    let built: Result<Option<Built>, TransactionStop> = runtime.block_on(async { return build_message_awaiting(&building, &args, COMMITTER, None).await; });
    let message: Built = built.expect("the awaited face builds the reused message inside a task").expect("the reused commit's message");
    assert_eq!(message.message, "the reused message\n", "the awaited face reads the message of the commit the person reused");
    assert_eq!(message.reused_author.as_deref(), Some(author.as_str()), "the awaited face keeps the author line of the commit it reused");
    let refused: Result<Option<Built>, TransactionStop> = runtime.block_on(async { return build_message(&building, &args, COMMITTER, None); });
    assert!(refused.is_err(), "the synchronous face cannot start its Git from inside the runtime, which is why a task asks the awaited face");
    remove(root.as_path());
}

/// What:
///  A message file no look answered is answered as an unreadable file is,
/// so the line goes to native `git commit`,
/// and the
///  work still ran.
/// Why:
///  The two answers a wait can give without a value
/// (no thread to start the job,
/// and a job that outlasted its
///  ceiling) cannot be arranged from a control,
/// and folding either into "the file is empty" would commit an empty
///  message,
/// while folding it into a failure would fail a commit Git could have made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const answer of ['notRun', 'noAnswer']) { using _stated = stateTheAnswer(answer); assert.equal(await buildMessageAwaiting(...), undefined) }
/// ```
#[test]
fn a_message_file_no_look_answered_is_answered_as_an_unreadable_file() {
    let root: PathBuf = fixture("message-awaiting-no-answer");
    let repository: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repository.join("MESSAGE").as_path(), b"from the file\n").expect("the message file");
    let context: GitContext = git_context();
    let region: CommitRegion = a_plain_region();
    let config: CliGitConfig = CliGitConfig::defaults();
    let controls: Controls = no_controls();
    let (owned, args): (Vec<OsString>, ServiceCommitArgs) = parsed(&["-F", "MESSAGE"]);
    let invocation: CommitInvocation<'_> = invocation_of(&context, repository.as_path(), owned.as_slice(), &region, &config, &controls);
    for answer in [AStatesAnswer::NotRun, AStatesAnswer::NoAnswer] {
        let stated = state_the_answer(answer);
        let built: Result<Option<Built>, TransactionStop> =
            crate::lazy_runtime::run_on_runtime(build_message_awaiting(&invocation, &args, COMMITTER, None)).expect("the runtime");
        drop(stated);
        assert!(matches!(built, Ok(None)), "a message file no look answered is the answer an unreadable file gives, so native Git builds the line");
    }
    let asked: Result<Option<Built>, TransactionStop> = crate::lazy_runtime::run_on_runtime(build_message_awaiting(&invocation, &args, COMMITTER, None)).expect("the runtime");
    let read: Built = asked.expect("the awaited face reads the file").expect("a message");
    assert_eq!(read.message, "from the file\n", "a read the control did not stand in for still reads the file");
    remove(root.as_path());
}

/// What:
///  Both faces read the same identities,
/// an identity Git does not resolve included,
/// and the awaited one reads them inside a task.
/// Why:
///   The identity is the key that joins a commit's timing record to the journal,
/// and a date Git does not read
///  hands the line to native `git commit`,
/// so a face that answered another identity or another nothing would change
///  what the person's commit is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(...both('GIT_COMMITTER_IDENT', [])); assert.deepEqual(...both('GIT_AUTHOR_IDENT', [['GIT_AUTHOR_DATE', 'not a date']]))
/// ```
#[test]
fn the_awaited_identity_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("identity-awaiting-agrees");
    let repository: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let overrides: Vec<(&str, String)> = vec![("GIT_AUTHOR_NAME", String::from("Fixture")), ("GIT_AUTHOR_EMAIL", String::from("fixture@example.invalid"))];
    let unreadable: Vec<(&str, String)> = vec![("GIT_AUTHOR_NAME", String::from("Fixture")), ("GIT_AUTHOR_DATE", String::from("not a date"))];
    for (variable, given) in [("GIT_COMMITTER_IDENT", Vec::new()), ("GIT_AUTHOR_IDENT", overrides), ("GIT_AUTHOR_IDENT", unreadable)] {
        let synchronous = identity(&context, repository.as_path(), variable, given.as_slice());
        let awaited = crate::lazy_runtime::run_on_runtime(identity_awaiting(&context, repository.as_path(), variable, given.as_slice())).expect("the runtime");
        assert_eq!(format!("{synchronous:?}"), format!("{awaited:?}"), "both faces read one identity for {variable}");
    }
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let waiting: GitContext = context.clone();
    let answering: PathBuf = repository.clone();
    let read: Result<Option<String>, TransactionStop> = runtime.block_on(async move { return identity_awaiting(&waiting, answering.as_path(), "GIT_COMMITTER_IDENT", &[]).await; });
    assert!(read.expect("the awaited face asks Git for the identity inside a task").is_some(), "the awaited face reads the committer Git resolves");
    remove(root.as_path());
}

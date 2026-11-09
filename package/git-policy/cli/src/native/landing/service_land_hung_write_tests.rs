//! Controls for a service whose branch write does not return:
//! the owner,
//! 2026-11-04,
//! "Never cut a branch write (Recommended)",
//! whose
//!  option read "a truly hung write keeps the service busy,
//! and later commits fail at their own 20-minute ceiling with an error
//!  naming it".
//! The write here waits for a release file instead of hanging for good,
//! so every control ends in seconds and leaves
//!  no process behind;
//! the ceilings of the clients are seconds too.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_client::{Activation, ClientConfig, Submitted, submit};
use crate::service_messages::ClientFailure;
use crate::service_server::{Bind, ServeSettings, serve};
use crate::test_support::{git_text, remove, while_test_lives};
use std::os::unix::net::UnixListener;
use std::time::{Duration, Instant};

/// The most a control waits for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long the service's write runs before the service calls it stuck:
/// short,
/// so the controls need not wait a minute.
const STUCK_AFTER: Duration = Duration::from_secs(1);

/// The ceiling the second client gives the service's answer:
/// short,
/// so the control ends in moments.
const SECOND_CEILING: Duration = Duration::from_secs(2);

/// The ceiling the client inside the write gives the service's answer:
/// past the moment the service tells it,
/// which is the stuck
/// threshold of the control plus up to one beat after the write began.
const INSIDE_CEILING: Duration = Duration::from_secs(5);

/// The ceiling the second client gives the service's answer to its withdrawal:
/// far past the control's own length,
/// so an answer
/// that arrives in seconds is the early answer,
/// not the ceiling.
const SECOND_WITHDRAWAL: Duration = Duration::from_secs(45);

/// A Git whose reference update of the landing (`update-ref --stdin`) touches `entered`,
/// then waits until `release` exists,
/// then does
/// what the real one does.
/// It gives up on its own when the test is gone.
fn git_that_holds_the_branch_write(world: &World) -> (PathBuf, PathBuf, PathBuf) {
    let entered: PathBuf = world.root.join("write-entered");
    let release: PathBuf = world.root.join("write-release");
    let program: PathBuf = world.root.join("holding-update-ref-git");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" update-ref --stdin \"*)\n    : > '{entered}'\n    while [ ! -e '{release}' ] && {alive}; do sleep 0.05; done\n    exec '{real}' \"$@\";;\n  *) exec '{real}' \"$@\";;\nesac\n",
        entered = entered.display(),
        release = release.display(),
        alive = while_test_lives(world.root.as_path()),
        real = crate::test_support::REAL_GIT
    );
    std::fs::write(&program, script).expect("script");
    std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return (program, entered, release);
}

/// A directory with a path short enough for a socket.
fn short_directory(name: &str) -> PathBuf {
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory for the short directory"));
    let directory: PathBuf = home.join("temp/agent").join(format!("hw-{}-{name}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("short directory");
    return directory;
}

/// A service on a socket of its own,
/// serving on a thread until the control ends.
struct Serving {
    /// The socket directory,
    /// removed at the end.
    directory: PathBuf,
    /// Where the clients connect.
    socket: PathBuf,
}

/// Start the service of `context` with the stuck threshold of the controls.
fn serve_on_a_thread(context: LandContext, name: &str) -> Serving {
    return serve_on_a_thread_stuck_after(context, name, STUCK_AFTER);
}

/// Start the service of `context` with the given threshold after which a write counts as stuck.
fn serve_on_a_thread_stuck_after(context: LandContext, name: &str, stuck_after: Duration) -> Serving {
    let mut settings: ServeSettings = ServeSettings::new(PATIENCE);
    settings.stuck_after = stuck_after;
    return serve_on_a_thread_with_settings(context, name, settings);
}

/// Start the service of `context` with the settings of the run,
/// which a control shortens so it ends in seconds.
fn serve_on_a_thread_with_settings(context: LandContext, name: &str, settings: ServeSettings) -> Serving {
    let directory: PathBuf = short_directory(name);
    let socket: PathBuf = directory.join("s");
    let listener: UnixListener = UnixListener::bind(&socket).expect("bind");
    std::thread::spawn(move || { serve(listener, &context, &settings); });
    return Serving { directory, socket };
}

/// A client of the service with the given ceilings.
fn client(serving: &Serving, answer_ceiling: Duration, withdraw_ceiling: Duration) -> ClientConfig {
    return client_with_extra_wait(serving, answer_ceiling, withdraw_ceiling, crate::wait_ceilings::SHORT_EXTRA_WAIT);
}

/// A client of the service with the given ceilings and the given extra wait for a request inside the branch write.
fn client_with_extra_wait(serving: &Serving, answer_ceiling: Duration, withdraw_ceiling: Duration, extra_wait: Duration) -> ClientConfig {
    return ClientConfig {
        address: Bind::Path(serving.socket.clone()),
        activation: Activation::Managed,
        start_wait: PATIENCE,
        heartbeat_silence: PATIENCE,
        start_watch: None,
        keep_fresh: None,
        answer_ceiling,
        withdraw_ceiling,
        waiting_notice_after: PATIENCE,
        extra_wait,
        start_ceiling: PATIENCE,
        units_record_dir: None,
    };
}

/// A client running on a thread of its own,
/// which is waited for no longer than `PATIENCE`,
/// so a regression that waits for ever
/// fails the control and does not hang it.
struct Submitting {
    /// Where the client sends what it ended with and printed.
    ended: std::sync::mpsc::Receiver<(Result<Submitted, ClientFailure>, String)>,
}

impl Submitting {
    /// What the client ended with and printed,
    /// or an error when it was still waiting after `PATIENCE`.
    fn join(self) -> Result<(Result<Submitted, ClientFailure>, String), String> {
        return self.ended.recv_timeout(PATIENCE).map_err(|_| { return String::from("the client was still waiting after the longest a control waits"); });
    }
}

/// A client that submits `request` on a thread of its own.
fn submit_on_a_thread(config: ClientConfig, request: CommitRequest) -> Submitting {
    let (sender, ended) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut notices: Vec<u8> = Vec::new();
        let answered = submit(&config, &request, &mut notices);
        let _sent = sender.send((answered, String::from_utf8(notices).expect("UTF-8")));
    });
    return Submitting { ended };
}

/// Waits until `path` exists.
fn wait_for(path: &Path) {
    let began: Instant = Instant::now();
    while !path.exists() {
        assert!(began.elapsed() < PATIENCE, "{} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Waits until the service has finished with the batches it landed:
/// none is open,
/// its markers are gone and the index lock is given up.
/// The service goes on
/// working in the fixture after the answer (the index build,
/// the records),
/// and a fixture removed under it is not empty when the
/// removal reaches it.
fn wait_until_settled(world: &World) {
    let began: Instant = Instant::now();
    loop {
        let open: usize = crate::service_journal::list_intents(world.context.journal.as_path(), crate::service_journal::OPEN_EXTENSION).len();
        // The landing thread removes the markers of its group after it has answered.
        // A withdrawal's tombstone stays by design; what must be gone is a request still marked as the landing thread's, and the step.
        let marked: bool = std::fs::read_dir(world.context.journal.join("busy")).is_ok_and(|entries| {
            return entries.flatten().any(|entry| { return entry.file_name() == "step" || std::fs::read_to_string(entry.path()).is_ok_and(|content| { return content == "landing"; }); });
        });
        if open == 0 && !marked && !world.repo.join(".git/index.lock").exists() {
            break;
        }
        assert!(began.elapsed() < PATIENCE, "the service never finished with its batches");
        std::thread::sleep(Duration::from_millis(20));
    }
    // The after-commit work (push, upkeep) of the last landing starts a moment after the answer.
    std::thread::sleep(Duration::from_millis(500));
}

/// The subjects on the branch after `base`,
/// oldest first.
fn subjects_since(world: &World, base: &str) -> String {
    return git_text(world.repo.as_path(), &["log", "--reverse", "--format=%s", format!("{base}..HEAD").as_str()]);
}

/// A commit queued behind a branch write that does not return is withdrawn at its own ceiling,
/// in seconds and not after the
/// withdrawal ceiling,
/// with the error naming the write;
/// the write is never ended;
/// when it returns,
/// the commit that was in it lands
/// and the withdrawn one never does.
#[test]
fn a_commit_queued_behind_a_branch_write_that_does_not_return_is_withdrawn_at_its_ceiling_naming_the_write() {
    let world: World = world("service-hung-write-queued");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let (hanging, entered, release) = git_that_holds_the_branch_write(&world);
    let mut context: LandContext = LandContext { git: crate::test_support::git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    context.git.real_git = hanging;
    let serving: Serving = serve_on_a_thread(context, "queued");
    let first = submit_on_a_thread(client(&serving, PATIENCE, PATIENCE), request(&world, "first", "first.txt", "first\n"));
    wait_for(entered.as_path());
    std::thread::sleep(STUCK_AFTER + Duration::from_secs(1));
    let began: Instant = Instant::now();
    let second = submit_on_a_thread(client(&serving, SECOND_CEILING, SECOND_WITHDRAWAL), request(&world, "second", "second.txt", "second\n"));
    let (second_answer, second_notices) = second.join().expect("second client");
    let waited: Duration = began.elapsed();
    let Err(ClientFailure::StillWithService { blocker, .. }) = &second_answer else { panic!("{second_answer:?} {second_notices}") };
    let words: &str = blocker.as_deref().expect("the error names what the service was stuck in");
    assert!(words.contains("git update-ref --stdin") && words.contains("never ends a write"), "{words}");
    assert!(waited < SECOND_WITHDRAWAL / 2, "withdrawn at its own ceiling, not at the withdrawal ceiling: {waited:?}");
    assert!(second_notices.contains("git update-ref --stdin"), "the client was told while it waited: {second_notices}");
    // The write is still held: nothing ended it, and nothing landed.
    assert_eq!(subjects_since(&world, base.as_str()), "", "the held write has not landed anything");
    std::fs::write(&release, b"").expect("let the write go");
    let (first_answer, first_notices) = first.join().expect("first client");
    assert!(matches!(first_answer, Ok(Submitted::Landed { .. })), "{first_answer:?} {first_notices}");
    // A third commit lands after the second would have, so the second's absence is the withdrawal and not timing.
    let third = submit_on_a_thread(client(&serving, PATIENCE, PATIENCE), request(&world, "third", "third.txt", "third\n"));
    let (third_answer, third_notices) = third.join().expect("third client");
    assert!(matches!(third_answer, Ok(Submitted::Landed { .. })), "{third_answer:?} {third_notices}");
    assert_eq!(subjects_since(&world, base.as_str()), "message of first\nmessage of third", "the withdrawn commit never landed");
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}

/// What:
///  A commit the service lands after a wait is told its place on the wire,
/// and its client prints it:
/// the whole path from the batch to the person.
/// Why:
///   The place was never sent to a lone request before,
/// so a control that only reads the deciding function does not prove the
///  settings,
/// the socket and the client's words line up.
/// The service's threshold is zero here,
/// so the place of a batch that begins at once is sent,
/// and the client's is zero too,
/// which makes what it prints the whole of what it is told.
#[test]
fn a_commit_the_service_takes_is_told_its_place_on_the_wire() {
    let world: World = world("service-place-of-a-commit");
    let mut settings: ServeSettings = ServeSettings::new(PATIENCE);
    settings.place_after = Duration::ZERO;
    let context: LandContext = LandContext { git: crate::test_support::git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    let serving: Serving = serve_on_a_thread_with_settings(context, "place", settings);
    let config: ClientConfig = ClientConfig { waiting_notice_after: Duration::ZERO, ..client(&serving, PATIENCE, PATIENCE) };
    let (answer, notices) = submit_on_a_thread(config, request(&world, "placed", "placed.txt", "placed\n")).join().expect("client");
    assert!(matches!(answer, Ok(Submitted::Landed { .. })), "{answer:?} {notices}");
    let place: String = crate::service_messages::position_notice(1, 1);
    let lines: Vec<&str> = notices.lines().collect();
    assert_eq!(lines.len(), 2, "two lines: {notices:?}");
    assert!(lines[0].contains("has not said where this commit stands"), "the service not having said where it stands first: {notices:?}");
    assert_eq!(lines[1], place, "then the place it sent: {notices:?}");
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}

/// The commit whose own branch write does not return:
/// its client reaches its ceiling,
/// asks to be withdrawn,
/// and the service cannot
/// say whether it landed,
/// so the error says the result is not known,
/// names the write and where the commit is,
/// and the commit lands
/// when the write returns.
#[test]
fn the_commit_inside_a_branch_write_that_does_not_return_is_reported_as_not_known_and_lands_when_the_write_returns() {
    let world: World = world("service-hung-write-inside");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let (hanging, entered, release) = git_that_holds_the_branch_write(&world);
    let mut context: LandContext = LandContext { git: crate::test_support::git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    context.git.real_git = hanging;
    let serving: Serving = serve_on_a_thread(context, "inside");
    // The withdrawal ceiling is far past the control: a client that was told its request is inside the write does not wait for it.
    let extra_wait: Duration = Duration::from_secs(2);
    let only = submit_on_a_thread(client_with_extra_wait(&serving, INSIDE_CEILING, SECOND_WITHDRAWAL, extra_wait), request(&world, "only", "only.txt", "only\n"));
    wait_for(entered.as_path());
    let began: Instant = Instant::now();
    let (answer, notices) = only.join().expect("client");
    let waited: Duration = began.elapsed();
    assert!(waited < SECOND_WITHDRAWAL / 3, "returned at its own bound plus the extra wait, not at the withdrawal ceiling: {waited:?}");
    let Err(ClientFailure::WithdrawalUnconfirmed { blocker }) = &answer else { panic!("{answer:?} {notices}") };
    let words: &str = blocker.as_deref().expect("the error names the write");
    assert!(words.contains("git update-ref --stdin") && words.contains("refs/worktree/cli-git-service/"), "{words}");
    let text: String = crate::service_messages::render(&answer.clone().expect_err("an error"));
    assert!(text.contains("git log -1") && text.contains("not known whether the commit was made") && text.contains(words.trim()), "{text}");
    std::fs::write(&release, b"").expect("let the write go");
    let released_at: Instant = Instant::now();
    while subjects_since(&world, base.as_str()) != "message of only" {
        assert!(released_at.elapsed() < PATIENCE, "the commit never landed: {}", subjects_since(&world, base.as_str()));
        std::thread::sleep(Duration::from_millis(50));
    }
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}

/// The error for a commit withdrawn behind a hung write names the log and the restart,
/// the way out when the write never returns.
#[test]
fn the_error_for_a_commit_behind_a_hung_write_names_the_way_out() {
    let failure: ClientFailure = ClientFailure::StillWithService { minutes: 20, blocker: Some(String::from("monochromatic-git-policy-cli: the commit service has been inside a write for 21 minutes")) };
    let text: String = crate::service_messages::render(&failure);
    assert!(text.contains("21 minutes") && text.contains(crate::service_messages::RESTART_COMMAND) && text.contains(crate::service_messages::LOG_COMMAND), "{text}");
}

/// A request whose commit is on the branch but whose batch is still open (the index update did not install,
/// so the batch waits for a
/// restart to finish it) is never answered "withdrawn" by a connection thread,
/// whatever the landing thread is stuck in:
/// the landing
/// thread knows the commit landed and answers "already" when the write returns.
/// The finished batches are not enough to tell it:
/// this one is open.
/// A request that is neither in flight nor named by any batch is withdrawn,
/// as the control that must move.
#[test]
fn a_request_whose_commit_is_on_the_branch_in_an_open_batch_is_not_withdrawn_early() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let world: World = world("service-hung-write-open-batch");
        let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
        land_with_failed_index(&world, 1, &[landed]);
        let board: std::sync::Arc<crate::service_busy::LandingBoard> = crate::service_busy::LandingBoard::new(world.context.journal.clone(), Duration::ZERO);
        crate::service_busy_step_record_on_this_thread::use_on_this_thread(std::sync::Arc::clone(&board));
        assert_eq!(board.begin_group(&[String::from("stuck")]), crate::service_busy::GroupAdmission::default());
        let _step = crate::service_busy_step_record_on_this_thread::enter_step(String::from("git update-ref --stdin in /repo"));
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(board.withdraw_early("landed"), crate::service_busy::EarlyAnswer::Queued, "its commit is on the branch");
        assert!(matches!(board.withdraw_early("never-seen"), crate::service_busy::EarlyAnswer::Withdrawn(_)), "a request no batch names is withdrawn");
        remove(world.root.as_path());
    });
}

/// The client whose request is inside the write is told so when the write starts,
/// not after the write has been stuck for the
/// service's threshold (sol-1:
/// the seven-second path was missed when the write began near the ceiling):
/// the control makes that
/// threshold an hour,
/// and the client still ends at its ceiling plus the extra wait.
#[test]
fn a_client_is_told_it_is_inside_the_write_when_the_write_starts() {
    let world: World = world("service-hung-write-told-at-start");
    let (hanging, entered, release) = git_that_holds_the_branch_write(&world);
    let mut context: LandContext = LandContext { git: crate::test_support::git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    context.git.real_git = hanging;
    let serving: Serving = serve_on_a_thread_stuck_after(context, "told-at-start", Duration::from_secs(3600));
    let extra_wait: Duration = Duration::from_secs(1);
    let only = submit_on_a_thread(client_with_extra_wait(&serving, Duration::from_secs(4), SECOND_WITHDRAWAL, extra_wait), request(&world, "only", "only.txt", "only\n"));
    wait_for(entered.as_path());
    let (answer, notices) = only.join().expect("client");
    assert!(matches!(answer, Err(ClientFailure::WithdrawalUnconfirmed { .. })), "{answer:?} {notices}");
    assert!(notices.contains("this commit is inside a write of the commit service"), "{notices}");
    std::fs::write(&release, b"").expect("let the write go");
    wait_until_settled(&world);
    remove(serving.directory.as_path());
    remove(world.root.as_path());
}

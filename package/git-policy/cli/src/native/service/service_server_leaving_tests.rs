//! What:
//!  Controls of each way out of the serving loop:
//! a retire asked for by a client,
//! the idle exit,
//! a worktree whose units are the service's having gone,
//! and a wait on the queue that the runtime refuses.
//!  Each is shown to wait within `THE_EXITS_SYNCS_ARE_MADE_WITHIN` for the syncs of the landings it answered,
//!  to say in its log that the bound ended that wait,
//!  and to end by its own reason.
//! Why:
//!   The bound is one number for four ways out, and a control of one of them says nothing about the other three:
//! the arm that reads the reason, the arm that asks for a worktree's units to be stopped, and the arm that ends without
//!  either are separate code, and each was written at `1bb658349d` without a control of its own.
//!  The sync is held inside this process at the service's own named step (`service_hold_a_step_for_a_control.rs`,
//! `INSIDE_A_SYNC`), so "a sync that does not end" is a state and not a slow disk.
//! The exit's log is captured on the thread that serves (`service_timing::capture`), because the exit line is the only
//!  witness a control has of which arm it took.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdInsideASync(); await serveUntil('retire'); expect(exitLine.reason).toBe('retire'); expect(took).toBeGreaterThanOrEqual(CEILING)
//! ```

use super::*;
use crate::service_client_listen_and_ask_to_retire::read_message;
use crate::service_hold_a_step_for_a_control::{INSIDE_A_SYNC, hold_these_steps_for_a_control, let_go_file, reached_file};
use crate::service_protocol::{ClientMessage, ServiceMessage, encode_client};
use crate::service_timing::capture::{Captured, entries_of};
use std::fs::File;
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

/// The line the exit writes when the bound ended its wait for the syncs of its last landings,
/// which `service_server.rs` writes and these controls read.
const A_SYNC_LEFT_TO_THE_WRITE_BACK: &str = "leaving with the syncs of its last landings still waiting";

/// The instance the controls' service is named by,
/// which is what the worktree-gone arm is asked to stop.
const INSTANCE: &str = "0000000000000000000000000000000e";

/// The file a control's stop writes,
/// beside the instance it was asked about.
const STOPPED_FILE: &str = "stopped-instance";

/// How often the controls' service looks at its own worktree:
/// short, so a control ends in moments.
const CHECK: Duration = Duration::from_millis(20);

/// The idle exit of the control that drives it:
/// short, so the run is a moment rather than three hours.
const SHORT_IDLE: Duration = Duration::from_millis(50);

/// The longest a control waits for the serving thread to end:
/// far beyond the ceiling,
/// so only a wait with no end reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// One disposable world:
/// a journal directory,
/// a socket short enough for the kernel's hundred bytes,
/// a worktree's Git directory,
/// and a record file whose sync the batch of a landing carries.
struct World {
    /// The directory this control owns, removed at the end.
    root: PathBuf,
    /// The Git directory of the worktree this service's units would be.
    git_dir: PathBuf,
    /// The journal the service reads and writes.
    journal: PathBuf,
    /// The socket the service listens on.
    socket: PathBuf,
    /// Where the held step says it was reached and is let go.
    holds: PathBuf,
    /// A file the landing's batch syncs.
    record: PathBuf,
}

fn world(label: &str) -> World {
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory"));
    // The socket's path must fit the kernel's 108 bytes, which the fixture tree's depth does not allow.
    let root: PathBuf = home.join("temp/agent").join(format!("leaving-{}-{label}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("short directory");
    let git_dir: PathBuf = root.join("checkout/.git");
    std::fs::create_dir_all(&git_dir).expect("the worktree's Git directory");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    let holds: PathBuf = root.join("holds");
    std::fs::create_dir_all(&holds).expect("the directory the hold goes in");
    let record: PathBuf = root.join("record");
    std::fs::write(&record, b"a landing's record").expect("the record");
    return World { socket: root.join("s"), git_dir, journal, root, holds, record };
}

impl World {
    /// The settings of a service of this world.
    fn settings(&self, idle: Duration) -> ServeSettings {
        let mut settings: ServeSettings = ServeSettings::new(idle);
        settings.instance = Some(String::from(INSTANCE));
        settings.own_units_stop = record_the_stop;
        return settings;
    }

    /// The context the service lands in.
    fn context(&self) -> LandContext {
        return LandContext {
            git: crate::test_support::git_context(),
            journal: self.journal.clone(),
            on_objects_written: None,
            lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
        };
    }

    /// Whether the held sync has been reached, which is the state a control waits for.
    fn reached(&self) -> bool {
        return reached_file(self.holds.as_path(), INSIDE_A_SYNC).exists();
    }

    /// Let the held sync go,
    /// so no thread of this process is left waiting when the control ends.
    fn let_go(&self) {
        std::fs::write(let_go_file(self.holds.as_path(), INSIDE_A_SYNC), b"").expect("the file that lets the held sync go");
    }

    /// What the worktree-gone arm asked to be stopped,
    /// when it asked.
    fn stopped(&self) -> Option<String> {
        return std::fs::read_to_string(self.journal.join(STOPPED_FILE)).ok();
    }
}

impl Drop for World {
    fn drop(&mut self) {
        self.let_go();
        let _removed = std::fs::remove_dir_all(self.root.as_path());
    }
}

/// A stop that names no manager and writes who it was asked about instead,
/// so a control reads it from the journal it shares with the service.
fn record_the_stop(instance: &str, journal: &Path) -> bool {
    std::fs::write(journal.join(STOPPED_FILE), instance).expect("the control's stop is recorded");
    return true;
}

/// What a serving thread answered when it ended:
/// how long it served,
/// and every line it logged on its own thread.
struct Served {
    /// What the control held for, until the thread ended.
    took: Duration,
    /// The entries the serving thread logged,
    /// which its exit line is among.
    said: Vec<Captured>,
}

impl Served {
    /// The words of every entry the serving thread logged,
    /// as one text.
    fn text(&self) -> String {
        return self.said.iter().map(|entry| { return entry.message.clone(); }).collect::<Vec<String>>().join("\n");
    }
}

/// What:
///  Queue a batch of a landing's records whose sync the control holds,
///  wait until the batch is inside the held step,
///  and then serve on a thread of its own, answering what it said and how long it served.
/// Why:
///   The hold is reached inside the batch, before its syncs,
/// so the barrier every exit waits on is answered only after the hold is let go:
/// the exit is then shown to wait for the syncs and to stop waiting at its ceiling.
/// The landing is made on the control's own thread and the serving starts afterwards, so the batch is queued and nothing
///  else is in flight when the way out is taken.
fn serve_with_its_records_held(world: &World, settings: ServeSettings) -> Served {
    queue_a_held_batch(world);
    let listener: UnixListener = UnixListener::bind(&world.socket).expect("bind");
    let context: LandContext = world.context();
    return serve_on_a_thread(listener, context, settings);
}

/// Queues one batch of a landing's records and waits until its sync is inside the held step.
fn queue_a_held_batch(world: &World) {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let record: File = File::open(world.record.as_path()).expect("the record");
        let deferral = crate::service_durable::begin(false);
        crate::service_durable::flush_file(&record, "intent-file").expect("queued");
        deferral.settle(true);
    });
    let waited: Instant = Instant::now();
    while !world.reached() {
        assert!(waited.elapsed() < PATIENCE, "the batch never reached the step the control holds, so no sync of it is in flight");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Runs `serve` on a thread of its own and answers what it said and how long it served.
fn serve_on_a_thread(listener: UnixListener, context: LandContext, settings: ServeSettings) -> Served {
    let (told, answered): (Sender<Served>, Receiver<Served>) = channel::<Served>();
    let _serving = std::thread::spawn(move || {
        let began: Instant = Instant::now();
        let said: Vec<Captured> = entries_of(|| { serve(listener, &context, &settings); });
        let _told = told.send(Served { took: began.elapsed(), said });
    });
    return answered.recv_timeout(PATIENCE).expect("the serving thread ended within the control's patience");
}

/// What:
///  Ask the service listening at `socket` to retire, on a connection of its own.
/// Why:
///   A retire reaches the loop as a message of a client and not as a function,
///  so a control writes the line a newer release writes.
fn retire_at(socket: &Path) {
    let stream: UnixStream = UnixStream::connect(socket).expect("the service listens");
    let _timeout = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream.try_clone().expect("a second handle on the connection"));
    let greeted: Option<ServiceMessage> = read_message(&mut reader);
    assert!(matches!(greeted, Some(ServiceMessage::Welcome { .. })), "the service greets the client that replaces it");
    let mut line: String = encode_client(&ClientMessage::Retire);
    line.push('\n');
    (&stream).write_all(line.as_bytes()).expect("the client asks it to retire");
}

/// What:
///  An exit whose syncs are held waited for them and stopped waiting at `THE_EXITS_SYNCS_ARE_MADE_WITHIN`,
///  and said so.
/// Why:
///   All four ways out end by the same two steps,
///  so the assertions of one arm are the assertions of the others:
/// the exit was not taken before the ceiling (a way out that returned at once would pass a bound with no wait in it), it was
///  taken after it, the log says the bound ended the wait rather than the queue emptying, and the line names the arm.
fn assert_the_bound_ended_the_wait(served_at: &Served, reason: &str) {
    let text: String = served_at.text();
    assert!(text.contains(format!("exiting reason={reason}").as_str()), "the service ended by the reason this control drove, whose word is {reason}: {text}");
    assert!(text.contains(A_SYNC_LEFT_TO_THE_WRITE_BACK), "the exit said what became of the syncs it stopped waiting for: {text}");
    assert!(
        served_at.took >= THE_EXITS_SYNCS_ARE_MADE_WITHIN,
        "the exit was taken before its ceiling, so no wait for the held sync happened: {:?}",
        served_at.took
    );
    assert!(
        served_at.took < THE_EXITS_SYNCS_ARE_MADE_WITHIN + PATIENCE,
        "the exit ended within its ceiling and not behind the sync it left: {:?}",
        served_at.took
    );
}

/// What:
///  A client that asks the service to retire ends it within the bound, and the exit says the bound ended its wait.
/// Why:
///   This is the way out an upgrade takes:
/// a newer release asks the service of the older one to retire and gives it `RETIREMENT_WAIT` (ten seconds) to leave,
/// and a service held by a sync of its own records fails that newer release's commit.
#[test]
fn a_client_that_asks_it_to_retire_ends_it_within_the_bound() {
    let world: World = world("retire");
    hold_these_steps_for_a_control(&[INSIDE_A_SYNC], world.holds.as_path());
    crate::service_durable::serve_on_again_for_a_control();
    queue_a_held_batch(&world);
    let listener: UnixListener = UnixListener::bind(&world.socket).expect("bind");
    let socket: PathBuf = world.socket.clone();
    let asking: std::thread::JoinHandle<()> = std::thread::spawn(move || { retire_at(socket.as_path()); });
    let served_at: Served = {
        let (told, answered): (Sender<Served>, Receiver<Served>) = channel::<Served>();
        let context: LandContext = world.context();
        let settings: ServeSettings = world.settings(IDLE_EXIT);
        let _serving = std::thread::spawn(move || {
            let began: Instant = Instant::now();
            let said: Vec<Captured> = entries_of(|| { serve(listener, &context, &settings); });
            let _told = told.send(Served { took: began.elapsed(), said });
        });
        answered.recv_timeout(PATIENCE).expect("the serving thread ended within the control's patience")
    };
    world.let_go();
    let _joined = asking.join();
    assert_the_bound_ended_the_wait(&served_at, "retire");
    assert_eq!(world.stopped(), None, "a retire asks for no unit to be stopped: the manager stops the service it replaced");
}

/// What:
///  A service with nothing to do leaves at its idle exit within the bound, and says the bound ended its wait.
/// Why:
///   The idle exit is reached hours after the last work,
/// and a sync of the last landing is often still in flight then:
/// it is the way out of a service nobody is talking to, and it must not hold the service for the device.
#[test]
fn the_idle_exit_ends_it_within_the_bound() {
    let world: World = world("idle");
    hold_these_steps_for_a_control(&[INSIDE_A_SYNC], world.holds.as_path());
    crate::service_durable::serve_on_again_for_a_control();
    let served_at: Served = serve_with_its_records_held(&world, world.settings(SHORT_IDLE));
    world.let_go();
    assert_the_bound_ended_the_wait(&served_at, "idle");
    assert_eq!(world.stopped(), None, "an idle exit asks for no unit to be stopped");
}

/// What:
///  A service whose worktree is gone asks for its units to be stopped and ends within the bound,
///  and says the bound ended its wait.
/// Why:
///   This arm does more than end:
/// it asks for the units of a worktree that is gone to be stopped,
/// and the request is made before the wait,
/// so a control reads both the request and the bound in one run.
#[test]
fn a_worktree_that_went_ends_it_within_the_bound() {
    let world: World = world("gone");
    hold_these_steps_for_a_control(&[INSIDE_A_SYNC], world.holds.as_path());
    crate::service_durable::serve_on_again_for_a_control();
    let mut settings: ServeSettings = world.settings(IDLE_EXIT);
    settings.own_git_dir = Some(world.git_dir.clone());
    settings.own_worktree_check = CHECK;
    // The worktree's Git directory is gone, and the directory that held it is there: the repository is not on a drive that
    //  is not mounted, which is the other state a missing Git directory can mean.
    std::fs::remove_dir_all(world.git_dir.as_path()).expect("the worktree's Git directory is gone");
    assert!(world.root.join("checkout").is_dir(), "the directory that held the Git directory stays, so the worktree reads as gone");
    let served_at: Served = serve_with_its_records_held(&world, settings);
    world.let_go();
    assert_the_bound_ended_the_wait(&served_at, "worktree-gone");
    assert_eq!(world.stopped().as_deref(), Some(INSTANCE), "the service asks for the units of the worktree that is gone to be stopped");
}

/// What:
///  A wait on the queue that the runtime refuses ends the loop by its error arm within the bound,
///  and that exit says the bound ended its wait.
/// Why:
///   `Received::Closed` is reached when every sender of the queue is gone or when the runtime refuses the wait.
/// The loop's own `sender` lives as long as the loop (`serve_in_a_landing` keeps it for the whole run),
/// so no closed channel reaches this arm;
/// a wait the runtime refuses does, which is what a service served from inside the runtime makes,
/// so the arm is reachable but not by a queue that closes.
#[test]
fn a_wait_the_runtime_refuses_ends_it_within_the_bound() {
    let world: World = world("refused");
    hold_these_steps_for_a_control(&[INSIDE_A_SYNC], world.holds.as_path());
    crate::service_durable::serve_on_again_for_a_control();
    queue_a_held_batch(&world);
    let listener: UnixListener = UnixListener::bind(&world.socket).expect("bind");
    let context: LandContext = world.context();
    let settings: ServeSettings = world.settings(IDLE_EXIT);
    let (told, answered): (Sender<Served>, Receiver<Served>) = channel::<Served>();
    let _serving = std::thread::spawn(move || {
        let began: Instant = Instant::now();
        let said: Vec<Captured> = entries_of(|| {
            // The runtime refuses a wait asked for from inside it, which is how the loop reads its queue as closed.
            let _served = crate::lazy_runtime::run_on_runtime(async move { serve(listener, &context, &settings) });
        });
        let _told = told.send(Served { took: began.elapsed(), said });
    });
    let served_at: Served = answered.recv_timeout(PATIENCE).expect("the serving thread ended within the control's patience");
    world.let_go();
    assert_the_bound_ended_the_wait(&served_at, "error");
    assert_eq!(world.stopped(), None, "a queue that could not be waited on asks for no unit to be stopped");
}

//! What:
//!  A person's withdrawal and the note of a beat get their turn while every place the service's own work may take is
//!  held by a job that outlasts its ceiling.
//! Why:
//!  A general place bounds a job only as long as the ceiling of the step that asked for it, and a step that outlasts its
//! ceiling keeps its place until its work ends, which a filesystem call that stopped answering can outlast without end.
//! The review of `95de6573c6` (finding 1, `reproductions/place-starvation/`) held every place with such a step against the
//! shipped functions and showed what follows: a queued client's withdrawal was never decided, so the person was told it
//! was not known whether the commit would land, and a new commit's look answered `Unavailable`, which ends a commit.
//! So the answers a client waits for take one of the places kept for them
//! (`PLACES_KEPT_TO_ANSWER_A_CLIENT`) and wait for it no longer than `CLIENT_ANSWER_PLACE_CEILING`, and these controls
//! hold every place of the service's own work to prove that the answers still run, and are answered here rather than
//!  handed on.
//! What they do not fix, and do not claim: a *commit's* look still asks for a general place, so a commit whose look finds
//! every one of them held still answers `Unavailable`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdEveryPlaceOfTheServicesOwnWork(); expect(await withdrawalDecidedHere()).toBeWithin(BOUND); expect(await noteReadStarted()).toBeWithin(BOUND)
//! ```

use super::*;
use crate::blocking_jobs_within_the_budget::places_for_the_services_own_work;
use crate::service_fault::hooks;
use crate::service_protocol::{ClientMessage, ServiceMessage, encode_client};
use std::path::Path;
use std::sync::Mutex;

/// The room these controls give the process,
/// from the relation they test:
/// the place kept for the answers a client waits for,
/// and two places of the service's own work
///  (`PLACES_KEPT_TO_ANSWER_A_CLIENT`, and the least a process keeps for its work beside it).
const THE_ROOM: usize = crate::blocking_jobs_within_the_budget::PLACES_KEPT_TO_ANSWER_A_CLIENT + 2;

/// How long a stuck job's caller waits before it gives up:
/// long enough that its own wait for a place is never the thing that ends it, on a loaded machine,
/// and still far shorter than the control's own bound,
/// so the job holds its place past its ceiling while the control goes on.
const THE_CALLERS_CEILING: Duration = crate::wait_ceilings::CLIENT_ANSWER_PLACE_CEILING;

/// The longest a control waits for an answer once every place is held:
/// twice the ceiling the answers a client waits for are granted, which is the place's wait and the read that follows it,
///  with room for a loaded machine.
const A_BOUND: Duration = crate::wait_ceilings::CLIENT_ANSWER_PLACE_CEILING.saturating_mul(2);

/// The request these controls name.
const A_REQUEST: &str = "a-request-of-a-client-that-waits";

/// The journal a control writes the stuck write of the landing thread into.
/// Six minutes before this control's board calls it slow, which is longer than its own `slow_after`, and a landing thread
/// that writes nothing for that long is the state `withdraw_early` exists for.
const THE_STUCK_WRITE_BEGAN_MS_AGO: u128 = 120_000;

/// What a stuck job is told to do:
/// take its place,
/// say so,
/// and hold the place until a control lets it go,
/// which is a step that outlasted its ceiling and whose work has not returned.
fn hold_a_place_past_its_ceiling((told, held): (std::sync::mpsc::Sender<()>, Arc<Mutex<std::sync::mpsc::Receiver<()>>>)) {
    let _said = told.send(());
    let _ = held.lock().expect("the control's release").recv();
}

/// What this control's connections share,
/// the journal they share it in,
/// and the end where the landing thread reads its queue.
fn shared_in_a_control(name: &str) -> (ConnectionShared, PathBuf, tokio::sync::mpsc::UnboundedReceiver<Job>) {
    let journal: PathBuf = crate::test_support::fixture(name);
    let (queue, jobs) = tokio::sync::mpsc::unbounded_channel::<Job>();
    let (work, _wakes) = tokio::sync::mpsc::unbounded_channel::<Wake>();
    let board: Arc<LandingBoard> = LandingBoard::new(journal.clone(), Duration::from_secs(60));
    let shared: ConnectionShared = ConnectionShared {
        queue,
        work,
        drawn: crate::service_shadow_draw::DrawnShadows::none_yet(),
        board,
        release: String::from("test"),
        instance: Some(String::from("an-instance")),
        own_git_dir: None,
    };
    return (shared, journal, jobs);
}

/// What:
///  Every place of the service's own work,
///  held by jobs whose callers have given up.
/// Why:
///  The hold is the state the review's finding needs:
/// a place kept past its ceiling by a step whose work has not returned,
/// which is what a filesystem call that stopped answering makes of a look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await holdEveryPlace(); // the jobs took every place and their callers gave up
/// ```
fn hold_every_place_of_the_work(name: &str) -> (Vec<tokio::task::JoinHandle<()>>, std::sync::mpsc::Sender<()>) {
    let held_places: usize = places_for_the_services_own_work();
    assert!(held_places >= 1, "{name} needs at least one place of the service's own work to hold");
    let (told, told_by_a_job) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let the_places_are_held: Arc<Mutex<std::sync::mpsc::Receiver<()>>> = Arc::new(Mutex::new(waiting));
    let mut stuck: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    for _ in 0..held_places {
        let told_one = told.clone();
        let still_held = Arc::clone(&the_places_are_held);
        let control: String = String::from(name);
        let holder = crate::lazy_runtime::spawn_on_runtime(async move {
            let answer = run_a_job_awaited_no_longer_than(THE_CALLERS_CEILING, (told_one, still_held), hold_a_place_past_its_ceiling).await;
            // The caller gave up while the job kept the place it took, which is the state every control here needs.
            assert!(
                matches!(answer, TheAnswerOfAJobAwaited::NoAnswer),
                "{control}: the caller of a stuck job did not give up while the job kept its place: {answer:?}"
            );
        });
        stuck.push(holder.expect("the runtime has a worker, so a stuck job can take a place"));
    }
    let mut took: usize = 0;
    for _ in 0..held_places {
        assert!(told_by_a_job.recv_timeout(crate::parallel_test_support::PATIENCE).is_ok(), "{name}: a stuck job never took its place");
        took += 1;
    }
    assert_eq!(took, held_places, "{name}: every place of the service's own work is held");
    return (stuck, release);
}

/// What:
///  The record of a write the landing thread has been inside for longer than the board calls slow,
///  which is the state `withdraw_early` decides a withdrawal in.
/// Why:
///  Without it the board answers `Queued` before it looks at anything,
/// and a control of the withdrawal's *decision* needs the state that decision exists for.
/// The record is the file the landing thread writes when it enters a write that is never ended
/// (`service_busy.rs`, `STEP_FILE`), written here as that thread would write it.
fn write_the_landing_threads_stuck_write(journal: &Path) {
    let clock: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_before_the_epoch) => 0,
    };
    let busy: PathBuf = journal.join("busy");
    std::fs::create_dir_all(busy.as_path()).expect("the journal's busy directory");
    let record: String = format!(
        "{}\ngit update-ref refs/heads/main\n",
        clock.saturating_sub(THE_STUCK_WRITE_BEGAN_MS_AGO)
    );
    std::fs::write(busy.join("step"), record.as_bytes()).expect("the record of the write the landing thread is inside");
}

/// What:
///  The answer the client reads within the bound:
/// a message that decides its request,
/// and nothing when none comes in time.
/// Why:
///  A withdrawal is answered with the words of the write the landing is inside before the decision itself
/// (`service_server_connection_handling.rs`, `withdraw`),
/// so a control that read one message would read the notice and never the answer.
async fn the_answer_within<R>(client: &mut tokio::io::BufReader<R>, bound: Duration) -> Option<ServiceMessage>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let deadline: tokio::time::Instant = tokio::time::Instant::now() + bound;
    loop {
        let left: Duration = deadline.saturating_duration_since(tokio::time::Instant::now());
        if left.is_zero() {
            return None;
        }
        let mut line: String = String::new();
        let read = tokio::time::timeout(left, tokio::io::AsyncBufReadExt::read_line(client, &mut line)).await;
        let Ok(Ok(_read)) = read else { return None };
        let Ok(message) = crate::service_protocol::decode_service(line.trim_end_matches('\n')) else { continue };
        if matches!(message, ServiceMessage::Withdrawn { .. } | ServiceMessage::Landed { .. }) {
            return Some(message);
        }
    }
}

/// What:
///  A person's withdrawal is decided where the connection reads it,
///  within the bound,
///  while every place of the service's own work is held.
/// Why:
///  This is the review's finding 1 as a control of the shipped connection handler:
/// the withdrawal is a real `ClientMessage::Withdraw` through `answer_request`,
/// the journal holds the record of a write the landing thread has been inside,
/// and the answer asserted is the decision itself,
///  `withdrawn`, which is what tells the person the commit will not land.
/// A cheaper answer would pass by handing the request to the landing thread,
/// which is not a decision and is what a starved withdrawal does.
#[test]
fn a_withdrawal_is_decided_while_every_place_of_the_work_is_held() {
    let _room: usize = crate::parallel_test_support::room_for("a_withdrawal_is_decided_while_every_place_of_the_work_is_held", THE_ROOM);
    let (shared, journal, mut jobs) = shared_in_a_control("connection-withdrawal-while-places-are-held");
    write_the_landing_threads_stuck_write(journal.as_path());
    let (stuck, release) = hold_every_place_of_the_work("a_withdrawal_is_decided_while_every_place_of_the_work_is_held");
    let line: String = encode_client(&ClientMessage::Withdraw { id: String::from(A_REQUEST) });
    // The connection is a task of its own, so the answer is read while it is being written, and so a connection that
    //  hands the request to a landing thread that never answers is abandoned here rather than waited for.
    let (heard, handed_on): (Option<ServiceMessage>, bool) = crate::lazy_runtime::run_on_runtime(async move {
        let (service, client_end) = UnixStream::pair().expect("a connected pair");
        let (_read_half, mut write_half) = service.into_split();
        let mut client = tokio::io::BufReader::new(client_end);
        let asking = crate::lazy_runtime::spawn_on_runtime(async move {
            answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        })
        .expect("the runtime has a worker for the connection");
        let heard: Option<ServiceMessage> = the_answer_within(&mut client, A_BOUND).await;
        let handed_on: bool = jobs.try_recv().is_ok();
        asking.abort();
        return (heard, handed_on);
    })
    .expect("the runtime");
    let _let_the_places_go = release.send(());
    for holder in stuck {
        holder.abort();
    }
    assert!(
        !handed_on,
        "the withdrawal was handed to the landing thread instead of being decided where the client asked"
    );
    match heard {
        Some(ServiceMessage::Withdrawn { id }) => assert_eq!(id, A_REQUEST, "the decision names the request the person withdrew"),
        Some(other) => panic!("the person's withdrawal was answered with {other:?} while every place of the service's own work was held"),
        None => panic!("the person's withdrawal was not decided within {A_BOUND:?} while every place of the service's own work was held"),
    }
}

/// What:
///  The note of a beat is read within the bound while every place of the service's own work is held,
///  so the words a waiting person reads are not left out by the starvation.
/// Why:
///  The note is what a queued client reads between two beats' heartbeats,
/// and its read is a journal read like the withdrawal's decision:
/// it takes the same kept place and the same ceiling.
/// A control holds the read where it starts, so what is asserted is that the read *ran*,
///  which a read given no place never does;
/// the words a board with nothing to say answers with are not this control's subject.
#[test]
fn the_note_of_a_beat_is_read_while_every_place_of_the_work_is_held() {
    let _room: usize = crate::parallel_test_support::room_for("the_note_of_a_beat_is_read_while_every_place_of_the_work_is_held", THE_ROOM);
    let (shared, _journal, _jobs) = shared_in_a_control("connection-note-while-places-are-held");
    let (stuck, release) = hold_every_place_of_the_work("the_note_of_a_beat_is_read_while_every_place_of_the_work_is_held");
    let (reached, was_reached) = tokio::sync::oneshot::channel::<()>();
    let (let_go, still_held) = std::sync::mpsc::channel::<()>();
    hooks::hold_on_any_thread(hooks::HoldOnAnyThread { point: String::from(IN_THE_NOTE_READ), reached, let_go: still_held });
    let started: bool = crate::lazy_runtime::run_on_runtime(async move {
        let mut notice: StuckNotice = stuck_notice(Arc::clone(&shared.board), Some(String::from(A_REQUEST)));
        // The read is a task of its own, so the control can see it start without waiting for words the board has none of.
        let reading = crate::lazy_runtime::spawn_on_runtime(async move { return notice.note().await }).expect("a worker for the read");
        let began: bool = tokio::time::timeout(A_BOUND, was_reached).await.is_ok_and(|heard| { return heard.is_ok(); });
        reading.abort();
        return began;
    })
    .expect("the runtime");
    let _let_the_read_go = let_go.send(());
    let _let_the_places_go = release.send(());
    for holder in stuck {
        holder.abort();
    }
    assert!(started, "the note of a beat was not read within {A_BOUND:?} while every place of the service's own work was held");
}

/// The ceiling this control grants one ask of the routing look:
/// a fifth of a second, so a starved look reaches its answer in a moment instead of the shipped minute.
const THE_LOOKS_CEILING: Duration = Duration::from_millis(200);

/// The longest this control watches a commit whose look is starved:
/// one pause of the look and three of its asks, which is longer than the answer a look with a place comes to.
const A_WATCH: Duration =
    crate::wait_ceilings::ROUTING_LOOK_RETRY_PAUSE.saturating_add(THE_LOOKS_CEILING.saturating_mul(3));

/// What:
///  A commit whose routing look finds no general place is beaten and taken up once a place comes,
///  and it is never answered `Unavailable`.
/// Why:
///  The project's rule is that a commit never fails: a look starved of a place by storage that stopped answering used to
///  answer `Unavailable`, which ends the command. The look is asked again while the client waits instead
///  (`ROUTING_LOOK_PATIENCE`), and the request is queued only once the look answers.
/// The general places are held by the package's own jobs, not by a slow disk.
#[test]
fn a_commit_is_never_failed_by_a_starved_routing_look() {
    let _room: usize = crate::parallel_test_support::room_for("a_commit_is_never_failed_by_a_starved_routing_look", THE_ROOM);
    crate::wait_ceilings::hold_the_routing_look_to(THE_LOOKS_CEILING);
    let (shared, _journal, mut jobs) = shared_in_a_control("connection-commit-starved-look");
    let (stuck, release) = hold_every_place_of_the_work("a_commit_is_never_failed_by_a_starved_routing_look");
    let heard: Vec<ServiceMessage> = crate::lazy_runtime::run_on_runtime(async move {
        // The pair and the connection are made inside the runtime, where the reactor they belong to runs.
        let (service, client_end) = UnixStream::pair().expect("a connected pair");
        let serving = crate::lazy_runtime::spawn_on_runtime(handle_connection(service, shared)).expect("a worker for the connection");
        let (read_half, mut write_half) = client_end.into_split();
        let line: String = encode_client(&ClientMessage::Commit(Box::new(a_commit_request_named("starved-look"))));
        tokio::io::AsyncWriteExt::write_all(&mut write_half, format!("{line}\n").as_bytes()).await.expect("the request line is sent");
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut heard: Vec<ServiceMessage> = Vec::new();
        let deadline: tokio::time::Instant = tokio::time::Instant::now() + A_WATCH;
        while tokio::time::Instant::now() < deadline {
            let mut heard_line: String = String::new();
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            let Ok(Ok(_read)) = tokio::time::timeout(left, tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut heard_line)).await else { break };
            if let Ok(message) = crate::service_protocol::decode_service(heard_line.trim_end_matches('\n')) {
                heard.push(message);
            }
        }
        serving.abort();
        return heard;
    })
    .expect("the runtime");
    let failed_by_the_look: bool = heard.iter().any(|message| {
        return matches!(message, ServiceMessage::Failed { code, .. } if *code == FailureCode::Unavailable);
    });
    // The places are let go, and the look that was starved now finds one, so the request is taken up.
    let _let_the_places_go = release.send(());
    for holder in stuck {
        holder.abort();
    }
    let queued: bool = crate::lazy_runtime::run_on_runtime(async {
        return tokio::time::timeout(A_WATCH, jobs.recv()).await.is_ok();
    })
    .expect("the runtime");
    assert!(!failed_by_the_look, "the commit was answered `unavailable` by a look that found no place: {heard:?}");
    assert!(queued, "the request was not taken up once a general place came: {heard:?}");
}

/// A commit request this control sends,
/// naming a Git directory of no repository,
/// so nothing of it is landed.
fn a_commit_request_named(id: &str) -> crate::service_protocol::CommitRequest {
    return crate::service_protocol::CommitRequest {
        id: String::from(id),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: PathBuf::from("/nowhere/repo/gitdir"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: "a".repeat(40),
        changes: Vec::new(),
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/nowhere/repo/gitdir/index"),
        scratch: PathBuf::from("/nowhere/repo/gitdir/monochromatic-git-policy-cli-service-a-0"),
        shadow: None,
        captured: 0,
        holder: String::new(),
    };
}

/// What:
///  A withdrawal is decided while a beat's note read is stuck on storage,
///  so the place kept for a withdrawal's answer is not held by a note.
/// Why:
///  Withdrawals and notes once shared one answering place, and a note read that stopped on stopped journal storage held it
///  until its work ended, so every later withdrawal lost its early decision (the review of `0b847be249`, F3). The note now
///  waits on a job of its own and takes none of those places.
#[test]
fn a_withdrawal_is_decided_while_a_note_read_is_stuck() {
    let (shared, journal, mut jobs) = shared_in_a_control("connection-withdrawal-beside-a-stuck-note");
    // The journal holds the record of a write the landing thread is inside, which is the state a withdrawal is decided in.
    write_the_landing_threads_stuck_write(journal.as_path());
    // A note read is held on whichever thread reaches it, as storage that stopped answering holds one.
    let (reached, was_reached) = tokio::sync::oneshot::channel::<()>();
    let (let_go, still_held) = std::sync::mpsc::channel::<()>();
    hooks::hold_on_any_thread(hooks::HoldOnAnyThread { point: String::from(IN_THE_NOTE_READ), reached, let_go: still_held });
    let board_for_the_note: Arc<LandingBoard> = Arc::clone(&shared.board);
    let note_task = crate::lazy_runtime::spawn_on_runtime(async move {
        let mut notice: StuckNotice = stuck_notice(board_for_the_note, Some(String::from(A_REQUEST)));
        let _words = notice.note().await;
    })
    .expect("a worker for the note");
    let note_is_stuck: bool = crate::lazy_runtime::run_on_runtime(async {
        return tokio::time::timeout(A_BOUND, was_reached).await.is_ok();
    })
    .expect("the runtime");
    assert!(note_is_stuck, "the note read never reached the storage it reads");
    // The withdrawal, with this body playing the landing thread that answers it if it is handed on.
    let line: String = encode_client(&ClientMessage::Withdraw { id: String::from(A_REQUEST) });
    let heard: Option<ServiceMessage> = crate::lazy_runtime::run_on_runtime(async move {
        let (service, client_end) = UnixStream::pair().expect("a connected pair");
        let serving = crate::lazy_runtime::spawn_on_runtime(handle_connection(service, shared)).expect("a worker for the connection");
        let (read_half, mut write_half) = client_end.into_split();
        tokio::io::AsyncWriteExt::write_all(&mut write_half, format!("{line}\n").as_bytes()).await.expect("the withdrawal line is sent");
        let mut reader = tokio::io::BufReader::new(read_half);
        let heard: Option<ServiceMessage> = the_answer_within(&mut reader, A_BOUND).await;
        serving.abort();
        return heard;
    })
    .expect("the runtime");
    let handed_on: bool = jobs.try_recv().is_ok();
    let _let_the_note_go = let_go.send(());
    note_task.abort();
    assert!(!handed_on, "the withdrawal was handed to a landing thread instead of being decided beside a stuck note read");
    match heard {
        Some(ServiceMessage::Withdrawn { id }) => assert_eq!(id, A_REQUEST),
        other => panic!("the withdrawal was not decided while a note read was stuck: {other:?}"),
    }
}

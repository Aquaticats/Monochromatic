//! Controls for a connection when the system refuses every thread:
//! the connection's reads of the journal run on the blocking pool,
//! so a pool that cannot start a thread is the refusal path of the connection task.
//! The refusal is an answer the client reads (`Unavailable`),
//! a withdrawal handed to the landing thread,
//! or a beat without its note,
//! and never a panic of tokio's `spawn_blocking` ("OS can't spawn worker thread"):
//! every blocking call of the service goes through `run_blocking_job`,
//! which catches that panic and gives `NoThreadToStart`.

use super::*;
use crate::service_fault::hooks::{self, HoldOnAnyThread};
use crate::service_protocol::{decode_service, encode_client};
use crate::service_protocol_messages_and_name_checks::HEARTBEAT_SILENCE;
use crate::ticker_beside_a_wait::with_a_ticker_beside;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt};
use std::path::Path;

/// The instance this control's service serves.
const INSTANCE: &str = "an-instance";

/// The places this control gives the process:
/// two,
/// because a pool of one thread is not the pool an ordinary service runs,
/// and a round that takes every place of it is what the controls of the waiting path must survive.
const CONTROL_PLACES: usize = 2;

/// What a connection of a control shares,
/// and the end where the landing thread would read its queue.
fn shared_in(name: &str) -> (ConnectionShared, UnboundedReceiver<Job>) {
    let journal: PathBuf = crate::test_support::fixture(name);
    let (queue, jobs) = unbounded_channel::<Job>();
    let (work, _wakes) = unbounded_channel::<Wake>();
    let board: Arc<LandingBoard> = LandingBoard::new(journal, Duration::from_secs(60));
    let shared: ConnectionShared = ConnectionShared {
        queue,
        work,
        drawn: crate::service_shadow_draw::DrawnShadows::none_yet(),
        board,
        release: String::from("test"),
        instance: Some(String::from(INSTANCE)),
        own_git_dir: None,
    };
    return (shared, jobs);
}

/// A connection's share for a service that was named for one worktree,
/// which is the shipped shape.
fn shared_serving(name: &str, own_git_dir: PathBuf) -> (ConnectionShared, UnboundedReceiver<Job>) {
    let (mut shared, jobs) = shared_in(name);
    shared.own_git_dir = Some(own_git_dir);
    return (shared, jobs);
}

/// The two ends of a connection:
/// the service's halves and the client's reader.
fn connection() -> (OwnedReadHalf, OwnedWriteHalf, BufReader<UnixStream>) {
    let (service, client) = UnixStream::pair().expect("a connected pair");
    let (read_half, write_half) = service.into_split();
    return (read_half, write_half, BufReader::new(client));
}

/// A message about a Git directory is answered `Unavailable` where the pool cannot look at its instance,
/// and nothing panics.
#[test]
fn where_no_thread_can_start_a_message_about_a_git_directory_is_answered_unavailable() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_a_message_about_a_git_directory_is_answered_unavailable") {
        return;
    }
    let (shared, _jobs) = shared_in("connection-no-thread-instance");
    let line: String = encode_client(&ClientMessage::Warm { git_dir: PathBuf::from("/srv/repo/.git"), repository: PathBuf::from("/srv/repo") });
    let ((), _ticks) = with_a_ticker_beside(async {
        let (_read_half, mut write_half, mut client) = connection();
        answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        drop(write_half);
        let mut answer: String = String::new();
        client.read_line(&mut answer).await.expect("an answer arrives");
        assert_eq!(decode_service(answer.trim_end_matches('\n')), Ok(ServiceMessage::Failed { id: String::new(), code: FailureCode::Unavailable }));
    });
}

/// The positive control of the refusal:
/// with threads available the same message about a Git directory of another instance is refused `Rejected`,
/// so `Unavailable` above comes from the pool's refusal and not from the message.
#[test]
fn with_threads_the_same_message_about_another_instance_is_rejected() {
    let (shared, _jobs) = shared_in("connection-threads-instance");
    let line: String = encode_client(&ClientMessage::Warm { git_dir: PathBuf::from("/srv/repo/.git"), repository: PathBuf::from("/srv/repo") });
    let ((), _ticks) = with_a_ticker_beside(async {
        let (_read_half, mut write_half, mut client) = connection();
        answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        drop(write_half);
        let mut answer: String = String::new();
        client.read_line(&mut answer).await.expect("an answer arrives");
        assert_eq!(decode_service(answer.trim_end_matches('\n')), Ok(ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected }));
    });
}

/// A withdrawal that the pool cannot decide is handed to the landing thread,
/// which answers between two landings,
/// and nothing panics.
#[test]
fn where_no_thread_can_start_a_withdrawal_goes_to_the_landing_thread() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_a_withdrawal_goes_to_the_landing_thread") {
        return;
    }
    let (shared, mut jobs) = shared_in("connection-no-thread-withdraw");
    let line: String = encode_client(&ClientMessage::Withdraw { id: String::from("a-request") });
    let ((), _ticks) = with_a_ticker_beside(async {
        let (_read_half, mut write_half, _client) = connection();
        // The connection waits for the landing's answer,
        // so it runs beside this body, which plays the landing thread.
        let waiting = answer_request(line.as_str(), &mut write_half, &shared, Instant::now());
        let landing = async {
            loop {
                if let Ok(job) = jobs.try_recv() {
                    return job;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        };
        let (job, ()) = tokio::join!(
            async {
                let job: Job = landing.await;
                // Dropping the reply is answering the request: the waiting connection ends.
                if let Job::Withdraw(id, reply) = job {
                    assert_eq!(id, "a-request");
                    drop(reply);
                    return true;
                }
                return false;
            },
            waiting
        );
        assert!(job, "the withdrawal reached the landing thread's queue");
    });
}

/// A beat's note that the pool cannot read is no note,
/// and the beat itself is still written by the writer.
#[test]
fn where_no_thread_can_start_the_note_of_a_beat_is_left_out() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_note_of_a_beat_is_left_out") {
        return;
    }
    let (shared, _jobs) = shared_in("connection-no-thread-note");
    let ((), _ticks) = with_a_ticker_beside(async {
        let mut notice: StuckNotice = stuck_notice(Arc::clone(&shared.board), Some(String::from("a-request")));
        assert_eq!(notice.note().await, None, "no note, and no panic");
    });
}

/// A request the fake services below never look at,
/// so it needs no repository.
fn a_request_of(id: &str) -> crate::service_protocol::CommitRequest {
    return a_request_at(Path::new("/nowhere/repo/gitdir"), id);
}

/// The same request,
/// naming the Git directory it is about.
fn a_request_at(git_dir: &Path, id: &str) -> crate::service_protocol::CommitRequest {
    return crate::service_protocol::CommitRequest {
        id: String::from(id),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: git_dir.to_path_buf(),
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
        index: git_dir.join("index"),
        scratch: git_dir.join("monochromatic-git-policy-cli-service-a-0"),
        shadow: None,
        captured: 0,
        holder: String::new(),
    };
}

/// The next message the client reads.
async fn next_line(client: &mut BufReader<UnixStream>) -> ServiceMessage {
    let mut line: String = String::new();
    client.read_line(&mut line).await.expect("a line arrives");
    return decode_service(line.trim_end_matches('\n')).expect("a message");
}

/// What:
///  A commit whose check of the Git directory's name waits is beaten while it waits,
/// and its answer is written after those beats.
/// Why:
///   A client ends a service it has heard nothing from for five seconds and sends the commit again;
/// the check resolves the directory's real path through the filesystem,
/// which under a busy disk took thirty-two
///  seconds in the record of 2026-11-10T17:50:19Z,
/// where six sends of one commit were answered by nothing at all and the commit was
///  reported failed although it landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await nextLine(client)).toBe(heartbeat); expect(await queuedJob).toBeDefined();
/// ```
#[test]
fn a_commit_whose_directory_check_waits_is_beaten_while_it_waits() {
    // Six beats: wide enough that no delay of this loaded machine passes for the wait itself, and a line that never
    //  came is the failure this control names.
    let the_longest_a_beat_may_wait: Duration = HEARTBEAT_EVERY.saturating_mul(6);
    let (shared, mut jobs) = shared_in("connection-slow-directory-check");
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, write_half, mut client) = connection();
        // The check answers only when this control lets it, so "while the check waits" is a state and not a duration.
        let (released, held) = tokio::sync::oneshot::channel::<()>();
        let deciding = async move {
            let _released = held.await;
            return Some(false);
        };
        let serving: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            let mut writing: OwnedWriteHalf = write_half;
            queue_a_commit(Box::new(a_request_of("slow-check")), &mut writing, &shared, Instant::now(), deciding).await;
        });
        // The first line is a beat: the check has not answered, so nothing of the landing has been said yet.
        let first: ServiceMessage = tokio::time::timeout(the_longest_a_beat_may_wait, next_line(&mut client))
            .await
            .expect("the beat comes while the check waits");
        assert_eq!(first, ServiceMessage::Heartbeat, "the beat comes while the check waits");
        released.send(()).expect("the check is let go");
        // The request reaches the landing once the check says it is this instance's, and only then is it answered.
        let Job::Commit(_request, reply, _arrival) = jobs.recv().await.expect("the request is queued") else {
            panic!("a commit request is queued");
        };
        let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("slow-check"), commit: String::from("c"), already: false, landing: None };
        reply.send(&landed);
        drop(reply);
        loop {
            let heard: ServiceMessage = next_line(&mut client).await;
            if heard == landed {
                break;
            }
            assert_eq!(heard, ServiceMessage::Heartbeat, "only beats come between the check and the answer");
        }
        serving.await.expect("the connection ends with the answer");
    });
}

/// What:
///  A commit that names this worktree's own Git directory exactly reaches the landing where the system refuses every
/// thread:
/// the instance it belongs to is decided without asking the filesystem.
/// Why:
///   The ask that resolves a directory's real path sits on the answering path of every commit and every wake,
/// and under a
///  busy disk it waited thirty-two seconds in the record of 2026-11-10T17:50:19Z while the client heard nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await queuedJob).toBeDefined(); // and no `Unavailable` answer
/// ```
#[test]
fn where_no_thread_can_start_a_commit_naming_this_worktree_still_reaches_the_landing() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_a_commit_naming_this_worktree_still_reaches_the_landing") {
        return;
    }
    let own: PathBuf = PathBuf::from("/srv/repo/gitdir");
    let (shared, mut jobs) = shared_serving("connection-own-directory-no-thread", own.clone());
    let line: String = encode_client(&ClientMessage::Commit(Box::new(a_request_at(own.as_path(), "own-directory"))));
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, mut write_half, _client) = connection();
        let answered: Option<Job> = tokio::select! {
            () = answer_request(line.as_str(), &mut write_half, &shared, Instant::now()) => None,
            queued = jobs.recv() => queued,
        };
        assert!(matches!(answered, Some(Job::Commit(..))), "the request named this worktree and reached the landing");
    });
}

/// What:
///  A commit that arrives on a connection is beaten while the check that decides whose request it is waits,
/// and the answer of that check is written after the beats.
/// Why:
///   A client ends a service it has heard nothing from for five seconds (`HEARTBEAT_SILENCE`,
/// `service_client.rs`) and sends the commit again,
///  and the check resolves a Git directory's real path through the filesystem,
/// which under a busy disk took thirty-two seconds in the record of
///  2026-11-10T17:50:19Z.
///  The control above proves that `queue_a_commit` beats beside a check it was handed;
/// this one drives the path a real connection takes,
///  where the answer of the first check of a commit used to be awaited before any writer existed,
/// so a check that waited left the client silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await nextLine(client)).toBe(heartbeat); // while the directory check is held
/// ```
#[test]
fn a_commit_sent_over_a_connection_is_beaten_while_its_directory_check_waits() {
    // Six beats: wide enough that no delay of this loaded machine passes for the wait itself.
    let the_longest_a_beat_may_wait: Duration = HEARTBEAT_EVERY.saturating_mul(6);
    let own: PathBuf = PathBuf::from("/srv/repo/gitdir");
    let (shared, mut jobs) = shared_serving("connection-held-directory-check", own);
    let line: String = encode_client(&ClientMessage::Commit(Box::new(a_request_at(Path::new("/elsewhere/repo/gitdir"), "held-check"))));
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, mut write_half, mut client) = connection();
        // The read of the directory's instance answers only when this control lets it go, so "while the check waits" is a
        //  state and not a duration.
        let (reached, reached_wait) = tokio::sync::oneshot::channel::<()>();
        let (let_go, held) = std::sync::mpsc::channel::<()>();
        hooks::hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_INSTANCE_READ), reached, let_go: held });
        let answering = async {
            answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        };
        let watching = async {
            tokio::time::timeout(the_longest_a_beat_may_wait, reached_wait).await.expect("the check of the Git directory reaches its read").expect("the check says it reached its read");
            let while_waiting = tokio::time::timeout(the_longest_a_beat_may_wait, next_line(&mut client)).await;
            // The read is let go whatever came, so a run that saw no beat still ends and still reads the answer.
            let_go.send(()).expect("the read is let go");
            let answer: ServiceMessage = next_line(&mut client).await;
            return (while_waiting, answer);
        };
        let ((), (while_waiting, answer)) = tokio::join!(answering, watching);
        assert!(jobs.try_recv().is_err(), "a commit another instance serves never reaches the landing");
        assert_eq!(
            answer,
            ServiceMessage::Failed { id: String::from("held-check"), code: FailureCode::Rejected },
            "the answer of the check is written after the beats"
        );
        assert_eq!(
            while_waiting.expect("a beat arrives while the check waits"),
            ServiceMessage::Heartbeat,
            "the beat comes while the check waits"
        );
    });
}

/// What:
///  A commit whose journal note waits is beaten while it waits,
///  more than once,
///  and its answer is written after the note is let go.
/// Why:
///  A client ends a service it has heard nothing from for five seconds (`HEARTBEAT_SILENCE`,
/// `service_client.rs`),
/// sends the commit again,
///  and may report it failed although it landed.
///  Every beat asks the note what the landing thread is inside,
/// and that note reads the journal,
/// which can wait on storage:
///  a beat that asked the note before writing anything would hold back the note,
/// the answer,
///  and the proof of life itself,
/// and a note that is still waiting when the next beat is due would hold back the beats after it.
///  Two beats are read while the note waits,
/// so a holder of the first beat and a writer that never came back to its interval both fail this control.
///  The note is let go here whatever happened,
///  so a run that saw no beat still ends and still reads the answer.
#[test]
fn a_commit_is_beaten_while_the_note_read_waits() {
    // Six beats: wide enough that no delay of this loaded machine passes for the wait itself.
    let the_longest_a_beat_may_wait: Duration = HEARTBEAT_EVERY.saturating_mul(6);
    let own: PathBuf = PathBuf::from("/srv/repo/gitdir");
    let (shared, mut jobs) = shared_serving("connection-held-instance-and-note", own);
    let line: String = encode_client(&ClientMessage::Commit(Box::new(a_request_at(Path::new("/elsewhere/repo/gitdir"), "held-check-and-note"))));
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, mut write_half, mut client) = connection();
        // The instance read and the note's journal read answer only when this control lets them go, so "while they wait" is a
        //  state and not a duration.
        let (instance_reached, instance_wait) = tokio::sync::oneshot::channel::<()>();
        let (instance_go, instance_held) = std::sync::mpsc::channel::<()>();
        hooks::hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_INSTANCE_READ), reached: instance_reached, let_go: instance_held });
        let (note_reached, note_wait) = tokio::sync::oneshot::channel::<()>();
        let (note_go, note_held) = std::sync::mpsc::channel::<()>();
        let answering = async {
            answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        };
        let watching = async {
            tokio::time::timeout(the_longest_a_beat_may_wait, instance_wait).await.expect("the instance read is reached").expect("the instance read says it is reached");
            // The hold is set after the instance read is reached, because one registry holds one point at a time.
            hooks::hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_NOTE_READ), reached: note_reached, let_go: note_held });
            tokio::time::timeout(the_longest_a_beat_may_wait, note_wait).await.expect("the note's journal read is reached").expect("the note says it reached its read");
            let mut beats: Vec<ServiceMessage> = Vec::new();
            while beats.len() < 2 {
                let heard: ServiceMessage = tokio::time::timeout(the_longest_a_beat_may_wait, next_line(&mut client))
                    .await
                    .expect("a beat arrives while the instance and journal reads wait");
                beats.push(heard);
            }
            // Both reads are let go before any assertion, so a run that saw no beat still ends and still reads the answer.
            note_go.send(()).expect("the note's read is let go");
            instance_go.send(()).expect("the instance read is let go");
            let answer: ServiceMessage = loop {
                let heard: ServiceMessage = tokio::time::timeout(the_longest_a_beat_may_wait, next_line(&mut client)).await.expect("the answer arrives after both reads are released");
                if heard == ServiceMessage::Heartbeat {
                    continue;
                }
                break heard;
            };
            return (beats, answer);
        };
        let ((), (beats, answer)) = tokio::join!(answering, watching);
        assert!(jobs.try_recv().is_err(), "a commit another instance serves never reaches the landing");
        assert_eq!(answer, ServiceMessage::Failed { id: String::from("held-check-and-note"), code: FailureCode::Rejected });
        assert_eq!(beats, vec![ServiceMessage::Heartbeat, ServiceMessage::Heartbeat], "the note must not gate the beats");
    });
}

/// What:
///  A note read that outlives its beat is waited for again by the beat behind it,
///  and never started over.
/// Why:
///  Every beat asks the journal what the landing thread is inside,
/// and that read can wait on storage:
///  a source that started a read a beat would queue a read a second behind a storage that is already late,
///  each of them holding a thread of the blocking pool that the commits of every other waiting client need,
///  and the words of the read nobody came back to would never be written.
///  The point is held a second time as soon as the first read is reached,
/// so a read started by a later beat is seen here and fails this control;
///  both points are let go before any assertion,
/// so a run that saw a restart still ends and still reads the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readsStarted).toBe(1); // over three beats
/// ```
#[test]
fn a_note_read_that_outlives_its_beat_is_not_started_again() {
    // Six beats: wide enough that no delay of this loaded machine passes for the wait itself.
    let the_longest_a_beat_may_wait: Duration = HEARTBEAT_EVERY.saturating_mul(6);
    let own: PathBuf = PathBuf::from("/srv/repo/gitdir");
    let (shared, mut jobs) = shared_serving("connection-held-note-read-again", own);
    let line: String = encode_client(&ClientMessage::Commit(Box::new(a_request_at(Path::new("/elsewhere/repo/gitdir"), "held-note-read"))));
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, mut write_half, mut client) = connection();
        // The instance read answers only when this control lets it go, so the connection waits where the beats are written.
        let (instance_reached, instance_wait) = tokio::sync::oneshot::channel::<()>();
        let (instance_go, instance_held) = std::sync::mpsc::channel::<()>();
        hooks::hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_INSTANCE_READ), reached: instance_reached, let_go: instance_held });
        let answering = async {
            answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        };
        let watching = async {
            tokio::time::timeout(the_longest_a_beat_may_wait, instance_wait).await.expect("the instance read is reached").expect("the instance read says it is reached");
            let (first_reached, first_wait) = tokio::sync::oneshot::channel::<()>();
            let (first_go, first_held) = std::sync::mpsc::channel::<()>();
            hooks::hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_NOTE_READ), reached: first_reached, let_go: first_held });
            tokio::time::timeout(the_longest_a_beat_may_wait, first_wait).await.expect("a beat's note reaches the journal read").expect("the note says it reached its read");
            // The hold behind the first one: a beat that started its own read reaches the point again and answers it.
            let (second_reached, mut second_reach) = tokio::sync::oneshot::channel::<()>();
            let (second_go, second_held) = std::sync::mpsc::channel::<()>();
            hooks::hold_on_any_thread(HoldOnAnyThread { point: String::from(IN_THE_NOTE_READ), reached: second_reached, let_go: second_held });
            let mut beats: usize = 0;
            while beats < 3 {
                let heard: ServiceMessage = tokio::time::timeout(the_longest_a_beat_may_wait, next_line(&mut client)).await.expect("a beat arrives while the note's read waits");
                assert_eq!(heard, ServiceMessage::Heartbeat, "only beats come while the note's read waits");
                beats += 1;
            }
            let restarted: bool = second_reach.try_recv().is_ok();
            // Both holds and the instance read are let go before any assertion, so a failed run still ends and still reads the
            //  answer; a hold nobody took answers its send with an error, which is no failure of this control.
            let _unheld: Result<(), _> = second_go.send(());
            first_go.send(()).expect("the first note read is let go");
            instance_go.send(()).expect("the instance read is let go");
            let answer: ServiceMessage = loop {
                let heard: ServiceMessage = tokio::time::timeout(the_longest_a_beat_may_wait, next_line(&mut client)).await.expect("the answer arrives after both reads are released");
                if heard == ServiceMessage::Heartbeat {
                    continue;
                }
                break heard;
            };
            return (restarted, answer);
        };
        let ((), (restarted, answer)) = tokio::join!(answering, watching);
        assert!(jobs.try_recv().is_err(), "a commit another instance serves never reaches the landing");
        assert_eq!(answer, ServiceMessage::Failed { id: String::from("held-note-read"), code: FailureCode::Rejected });
        assert!(!restarted, "the beat behind a slow note read waits for the read it found instead of starting another");
    });
}

/// POSITIVE CONTROL of the check itself:
/// with threads available,
/// a commit naming another worktree is refused `Rejected`,
/// so the queueing above comes from the
///  name and not from the check being skipped for every request.
#[test]
fn with_threads_a_commit_naming_another_worktree_is_rejected() {    let own: PathBuf = PathBuf::from("/srv/repo/gitdir");
    let (shared, mut jobs) = shared_serving("connection-other-directory", own);
    let line: String = encode_client(&ClientMessage::Commit(Box::new(a_request_at(Path::new("/elsewhere/repo/gitdir"), "other-directory"))));
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, mut write_half, mut client) = connection();
        answer_request(line.as_str(), &mut write_half, &shared, Instant::now()).await;
        drop(write_half);
        assert!(jobs.try_recv().is_err(), "a request for another worktree never reaches the landing");
        assert_eq!(
            next_line(&mut client).await,
            ServiceMessage::Failed { id: String::from("other-directory"), code: FailureCode::Rejected },
            "the refusal is written where the client reads it"
        );
    });
}


/// What:
///  A commit's question about a drawn shadow is answered while the landing thread holds a batch it has not begun,
///  the tree leaves the stock with that answer,
///  and the question takes no place in the landing's queue.
/// Why:
///   A commit's wait for its tree must not grow while another commit lands,
///  which is the first of the record's four cons for the option the owner chose.
///  The landing thread here holds one job and has taken nothing,
///  which is a landing under way from the answering side,
///  and the answer still comes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queue.send(aCommit); expect(await askForTheTree()).toEqual({ type: 'drawn', tree }); expect(jobs.tryRecv()).toBe(theCommit);
/// ```
#[test]
fn a_question_for_a_drawn_shadow_is_answered_while_the_landing_thread_holds_a_batch() {
    let root: PathBuf = crate::test_support::fixture("connection-shadow-answer");
    let git_dir: PathBuf = root.join("gitdir");
    let drawn: PathBuf = crate::shadow_drawn::drawn_root_for(root.as_path(), git_dir.as_path()).join("drawn-00000000000000000000000000000003");
    std::fs::create_dir_all(drawn.as_path()).expect("a drawn tree");
    // The instance is named for the Git directory the question names, so the answer is this service's to give without
    //  asking the filesystem whose the directory is.
    let (shared, mut jobs) = shared_serving("connection-shadow-answer", git_dir.clone());
    shared.drawn.note_ready(drawn.clone());
    let scratch: PathBuf = git_dir.join("monochromatic-git-policy-cli-scratch").join("monochromatic-git-policy-cli-service-ask-0");
    std::fs::create_dir_all(scratch.parent().expect("the scratch place").to_path_buf().as_path()).expect("the scratch place");
    let asked: String = encode_client(&ClientMessage::Shadow { git_dir: git_dir.clone(), scratch: scratch.clone() });
    // A commit the landing thread has been given and has not begun to land.
    let (reply, _messages) = crate::service_reply::Reply::new();
    let busy = Job::Commit(Box::new(a_request_at(git_dir.as_path(), "the-commit-in-the-landing")), reply, Arrival { accepted: Instant::now(), read: Duration::ZERO });
    assert!(shared.queue.send(busy).is_ok(), "one commit is in the landing thread's hands");
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (_read_half, mut write_half, mut client) = connection();
        answer_request(asked.as_str(), &mut write_half, &shared, Instant::now()).await;
        drop(write_half);
        assert_eq!(next_line(&mut client).await, ServiceMessage::Drawn { tree: drawn.clone() }, "the tree is the answer while the landing holds a batch");
        // A second commit that asks now is told there is none: the first one's tree left the stock with its answer.
        let again: String = encode_client(&ClientMessage::Shadow { git_dir, scratch });
        let (_second_read, mut second_write, mut second_client) = connection();
        answer_request(again.as_str(), &mut second_write, &shared, Instant::now()).await;
        drop(second_write);
        assert_eq!(next_line(&mut second_client).await, ServiceMessage::NoShadow, "one tree answers one commit");
    });
    assert!(matches!(jobs.try_recv(), Ok(Job::Commit(..))), "the landing's own job is still the queue's whole");
    assert!(jobs.try_recv().is_err(), "the question took no place in the landing's queue");
    crate::test_support::remove(root.as_path());
}

/// The next message a client reads on either half of a connection.
async fn a_line<Reader: AsyncBufRead + Unpin>(reader: &mut Reader) -> ServiceMessage {
    let mut line: String = String::new();
    reader.read_line(&mut line).await.expect("a line arrives");
    return decode_service(line.trim_end_matches('\n')).expect("a message");
}

/// What:
///  Holds every place of the process's blocking pool with a job that waits,
/// as a round of the after-commit work holds them while it empties a trash.
/// Why:
///  The state the control needs is "the pool is full",
/// not a duration,
/// so each job says it has its place and waits for the barrier the caller holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = holdEveryPlace(places); // await each job's word, then gate.wait() to let them go
/// ```
async fn hold_every_place(places: usize) -> Arc<std::sync::Barrier> {
    let gate: Arc<std::sync::Barrier> = Arc::new(std::sync::Barrier::new(places + 1));
    let mut entered: Vec<tokio::sync::oneshot::Receiver<()>> = Vec::with_capacity(places);
    for _place in 0..places {
        let (said, said_waits) = tokio::sync::oneshot::channel::<()>();
        entered.push(said_waits);
        let held_by_this_job: Arc<std::sync::Barrier> = Arc::clone(&gate);
        let _holding = tokio::spawn(async move {
            let _held = crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
                let _said = said.send(());
                held_by_this_job.wait();
            })
            .await;
        });
    }
    for said_waits in entered {
        tokio::time::timeout(HEARTBEAT_SILENCE, said_waits).await.expect("a place of the pool is held").expect("the job that holds a place says so");
    }
    return gate;
}

/// What:
///  A client is greeted and beaten within its own silence window while every place of the process's blocking pool is held,
/// as a round of the after-commit work holds them while it empties a trash.
/// Why:
///   The beat asks the landing's board for the note it writes,
/// and that read is a job of the blocking pool
/// (`service_busy.rs`);
/// a round's trash removals are jobs of the same pool,
/// one per entry
///  (`worktree_trash_entries.rs`),
/// so a beat that waits for its note waits for a place the round's own work holds,
/// and the client hears
///  nothing.
/// Measured in production on 2026-11-10:
/// 27 to 32 s of silence while a round was busy,
/// five silent windows of five seconds,
///  and the same commit sent six times
/// (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
/// The client ends a service it has heard nothing from for `HEARTBEAT_SILENCE`,
/// so the greeting and the beat must never wait for
///  a place the round's work holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = await holdEveryPlace(places); expect(await aLine(client)).toBe(welcome); expect(await within(SILENCE, aLine(client))).toBe(heartbeat); gate.wait();
/// ```
#[test]
fn a_client_is_greeted_and_beaten_while_every_place_of_the_pool_is_held() {
    let places: usize = CONTROL_PLACES;
    let _room: usize = crate::room_for_blocking_jobs_in_a_control::for_a_control(places);
    let (shared, mut jobs) = shared_serving("connection-held-pool", PathBuf::from("/srv/repo/gitdir"));
    let mut request_line: String = encode_client(&ClientMessage::Commit(Box::new(a_request_at(Path::new("/srv/repo/gitdir"), "held-pool"))));
    request_line.push('\n');
    let outcome = crate::lazy_runtime::run_on_runtime(async move {
        let gate: Arc<std::sync::Barrier> = hold_every_place(places).await;
        // The client connects and sends a commit that names this worktree's own Git directory, which is the shape every
        //  commit of a service that knows its worktree has: the check of whose the request is asks no thread.
        let (service, client_stream) = UnixStream::pair().expect("a connected pair");
        let (client_read, mut client_write) = client_stream.into_split();
        let mut client = BufReader::new(client_read);
        let serving = tokio::spawn(handle_connection(service, shared));
        let greeting = tokio::time::timeout(HEARTBEAT_SILENCE, a_line(&mut client)).await.expect("the service greets the client while every place is held");
        assert!(matches!(greeting, ServiceMessage::Welcome { .. }), "the greeting is the first line, whatever the pool is doing");
        client_write.write_all(request_line.as_bytes()).await.expect("the client sends its request");
        // The beat is the proof of life the client's wait rests on, so it is the first thing this control asks for.
        let beaten = tokio::time::timeout(HEARTBEAT_SILENCE, a_line(&mut client)).await;
        let beat: ServiceMessage = beaten.expect("a beat arrives while every place of the pool is held, though the note of a beat cannot be read");
        assert_eq!(beat, ServiceMessage::Heartbeat, "the beat never waits for a place the round's work holds");
        let queued = tokio::time::timeout(HEARTBEAT_SILENCE, jobs.recv()).await.expect("the request reaches the landing although every place is held");
        let Job::Commit(_request, reply, _arrival) = queued.expect("a commit request is queued") else {
            panic!("a commit request is queued");
        };
        let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("held-pool"), commit: String::from("c"), already: false, landing: None };
        reply.send(&landed);
        drop(reply);
        let answered = tokio::time::timeout(HEARTBEAT_SILENCE, async {
            loop {
                let heard: ServiceMessage = a_line(&mut client).await;
                if heard == landed {
                    return heard;
                }
            }
        })
        .await
        .expect("the answer arrives");
        gate.wait();
        let _joined = tokio::time::timeout(HEARTBEAT_SILENCE, serving).await;
        return answered;
    })
    .expect("the process runtime runs this control");
    assert_eq!(outcome, ServiceMessage::Landed { id: String::from("held-pool"), commit: String::from("c"), already: false, landing: None }, "the answer is written while every place is held");
}

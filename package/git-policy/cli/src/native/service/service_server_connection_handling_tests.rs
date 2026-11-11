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
use crate::ticker_beside_a_wait::with_a_ticker_beside;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::io::AsyncBufReadExt;
use std::path::Path;

/// The instance this control's service serves.
const INSTANCE: &str = "an-instance";

/// What a connection of a control shares,
/// and the end where the landing thread would read its queue.
fn shared_in(name: &str) -> (ConnectionShared, UnboundedReceiver<Job>) {
    let journal: PathBuf = crate::test_support::fixture(name);
    let (queue, jobs) = unbounded_channel::<Job>();
    let (work, _wakes) = unbounded_channel::<Wake>();
    let board: Arc<LandingBoard> = LandingBoard::new(journal, Duration::from_secs(60));
    let shared: ConnectionShared = ConnectionShared { queue, work, board, release: String::from("test"), instance: Some(String::from(INSTANCE)), own_git_dir: None };
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

/// POSITIVE CONTROL of the check itself:
/// with threads available,
/// a commit naming another worktree is refused `Rejected`,
/// so the queueing above comes from the
///  name and not from the check being skipped for every request.
#[test]
fn with_threads_a_commit_naming_another_worktree_is_rejected() {
    let own: PathBuf = PathBuf::from("/srv/repo/gitdir");
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

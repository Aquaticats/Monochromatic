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
use crate::service_protocol::{decode_service, encode_client};
use crate::ticker_beside_a_wait::with_a_ticker_beside;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::io::AsyncBufReadExt;

/// The instance this control's service serves.
const INSTANCE: &str = "an-instance";

/// What a connection of a control shares,
/// and the end where the landing thread would read its queue.
fn shared_in(name: &str) -> (ConnectionShared, UnboundedReceiver<Job>) {
    let journal: PathBuf = crate::test_support::fixture(name);
    let (queue, jobs) = unbounded_channel::<Job>();
    let (work, _wakes) = unbounded_channel::<Wake>();
    let board: Arc<LandingBoard> = LandingBoard::new(journal, Duration::from_secs(60));
    let shared: ConnectionShared = ConnectionShared { queue, work, board, release: String::from("test"), instance: Some(String::from(INSTANCE)) };
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

/// A request the fake services below never look at, so it needs no repository.
fn a_request_of(id: &str) -> crate::service_protocol::CommitRequest {
    return crate::service_protocol::CommitRequest {
        id: String::from(id),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: PathBuf::from("/nowhere/.git"),
        reference: String::from("refs/heads/main"),
        base: "a".repeat(40),
        changes: Vec::new(),
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/nowhere/.git/index"),
        scratch: PathBuf::from("/nowhere/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-a-0"),
        captured: 0,
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

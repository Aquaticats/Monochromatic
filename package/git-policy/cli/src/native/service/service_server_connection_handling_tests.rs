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

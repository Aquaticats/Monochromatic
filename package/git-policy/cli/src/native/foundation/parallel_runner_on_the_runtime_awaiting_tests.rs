//! The awaited ordered run on the runtime:
//! every item runs,
//! the report keeps input order however the items finish,
//! the items really overlap,
//! a lane's items run one after another,
//! and an item that awaits a Git child is what this form exists for.

use super::*;
use crate::line_sink::LineSink;
use crate::parallel_runner::Event;
use crate::parallel_test_support::{PATIENCE, Rendezvous};
use crate::test_support::{fixture, git_context, remove, repository};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

/// Milliseconds one item of the timing controls keeps its work busy.
const BUSY_MILLIS: u64 = 25;

/// How many items the timing controls run.
const ITEMS: usize = 4;

/// One lane per item,
/// so items may finish in any order.
fn single_lanes(count: usize) -> Arc<[Vec<usize>]> {
    let lanes: Vec<Vec<usize>> = (0..count).map(lane_of_one_item).collect();
    return Arc::from(lanes);
}

/// The lane of an item that must not wait for another:
/// itself alone.
fn lane_of_one_item(index: usize) -> Vec<usize> {
    return vec![index];
}

/// A run over one lane per item.
fn run_beside_each_other<R: Send + 'static>(items: Vec<u64>, work: Arc<dyn OrderedAwaitingWork<u64, R>>, on_event: &mut dyn OrderedEvents<R>) -> Result<(), OrderedRunError> {
    let count: usize = items.len();
    return crate::lazy_runtime::run_on_runtime(run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::from(items),
        lanes: single_lanes(count),
        workers: count,
        work,
        on_event,
    }))
    .expect("the runtime");
}

/// Items that wait for the time they are given say one line and return ten times their index.
struct WaitThenSay;
impl OrderedAwaitingWork<u64, u64> for WaitThenSay {
    fn run<'a>(&'a self, index: usize, item: &'a u64, mut say: LinesToTheConsumer<u64>) -> Pin<Box<dyn Future<Output = u64> + Send + 'a>> {
        return Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(*item)).await;
            say.say(format!("item {index}").as_str());
            return index as u64 * 10;
        });
    }
}

/// Every event,
/// as text,
/// in the order the consumer saw them.
#[derive(Default)]
struct SeenText {
    /// The events as text.
    seen: Vec<String>,
}

impl OrderedEvents<u64> for SeenText {
    fn on_event(&mut self, event: Event<'_, u64>) {
        self.seen.push(match event {
            Event::Begin(index) => format!("begin {index}"),
            Event::Line(index, text) => format!("line {index} {text}"),
            Event::End(index, result) => format!("end {index} {result}"),
        });
    }
}

/// Every item runs and the report keeps input order although the first item ends last.
#[test]
fn every_item_runs_and_the_report_keeps_input_order() {
    let mut seen: SeenText = SeenText::default();
    run_beside_each_other(vec![4 * BUSY_MILLIS, 1, 2 * BUSY_MILLIS, 2], Arc::new(WaitThenSay), &mut seen).expect("the run finished");
    let expected: Vec<String> = vec![
        String::from("begin 0"),
        String::from("line 0 item 0"),
        String::from("end 0 0"),
        String::from("begin 1"),
        String::from("line 1 item 1"),
        String::from("end 1 10"),
        String::from("begin 2"),
        String::from("line 2 item 2"),
        String::from("end 2 20"),
        String::from("begin 3"),
        String::from("line 3 item 3"),
        String::from("end 3 30"),
    ];
    assert_eq!(seen.seen, expected, "the consumer saw the events out of input order");
}

/// An item that waits for every other item at a point a serial run can never pass:
/// the barrier opens only when all of them are there,
/// and the wait gives up after the patience.
struct MeetTheOthers {
    /// The point all items must reach.
    point: Arc<tokio::sync::Barrier>,
}

impl OrderedAwaitingWork<u64, bool> for MeetTheOthers {
    fn run<'a>(&'a self, _index: usize, _item: &'a u64, _say: LinesToTheConsumer<bool>) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        return Box::pin(async move {
            return tokio::time::timeout(PATIENCE, self.point.wait()).await.is_ok();
        });
    }
}

/// How many items met the others.
#[derive(Default)]
struct CountArrived {
    /// The items that met the others.
    arrived: usize,
}

impl OrderedEvents<bool> for CountArrived {
    fn on_event(&mut self, event: Event<'_, bool>) {
        if let Event::End(_index, came) = event
            && came
        {
            self.arrived += 1;
        }
    }
}

/// The items really run at the same time:
/// every one of them reaches the meeting point of all of them.
#[test]
fn items_run_at_the_same_time() {
    let mut counted: CountArrived = CountArrived::default();
    let work: Arc<MeetTheOthers> = Arc::new(MeetTheOthers { point: Arc::new(tokio::sync::Barrier::new(ITEMS)) });
    run_beside_each_other(vec![0; ITEMS], work, &mut counted).expect("the run finished");
    assert_eq!(counted.arrived, ITEMS, "a run whose items overlap has every item at the meeting point");
}

/// A lane's items run one after another:
/// the second item of a lane enters only after the first has left,
/// which is what keeps two items of one worktree apart.
struct EnterAndLeave {
    /// The order the items entered and left in.
    order: Arc<Mutex<Vec<String>>>,
}

impl OrderedAwaitingWork<u64, u64> for EnterAndLeave {
    fn run<'a>(&'a self, index: usize, _item: &'a u64, _say: LinesToTheConsumer<u64>) -> Pin<Box<dyn Future<Output = u64> + Send + 'a>> {
        return Box::pin(async move {
            self.order.lock().expect("lock").push(format!("enter {index}"));
            tokio::time::sleep(Duration::from_millis(BUSY_MILLIS)).await;
            self.order.lock().expect("lock").push(format!("leave {index}"));
            return index as u64;
        });
    }
}

/// Nothing counts here;
/// the order in the work is the answer.
#[derive(Default)]
struct CountNothing;

impl OrderedEvents<u64> for CountNothing {
    fn on_event(&mut self, _event: Event<'_, u64>) {}
}

/// The two items of one lane never overlap,
/// though the lane beside them runs at the same time.
#[test]
fn a_lane_runs_its_items_one_after_another() {
    let order: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let work: Arc<EnterAndLeave> = Arc::new(EnterAndLeave { order: Arc::clone(&order) });
    let mut counted: CountNothing = CountNothing::default();
    crate::lazy_runtime::run_on_runtime(run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::from(vec![0; ITEMS]),
        lanes: Arc::from(vec![vec![0, 1], vec![2, 3]]),
        workers: 2,
        work,
        on_event: &mut counted,
    }))
    .expect("the runtime")
    .expect("the run finished");
    let seen: Vec<String> = order.lock().expect("lock").clone();
    let first_of_the_first_lane: usize = seen.iter().position(|said| { return said == "leave 0"; }).expect("the first item left");
    let second_of_the_first_lane: usize = seen.iter().position(|said| { return said == "enter 1"; }).expect("the second item entered");
    assert!(first_of_the_first_lane < second_of_the_first_lane, "the second item of the lane entered before the first left: {seen:?}");
}

/// A Git question asked by an item of this run is answered,
/// which no job of the blocking pool can do:
/// its work is awaited on the runtime,
/// and the child's end is awaited through its pidfd.
struct AskGit {
    /// The repository the question is put to.
    context: crate::transaction_git::GitContext,
    /// The directory of the repository.
    directory: PathBuf,
}

impl OrderedAwaitingWork<u64, Option<String>> for AskGit {
    fn run<'a>(&'a self, _index: usize, _item: &'a u64, _say: LinesToTheConsumer<Option<String>>) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
        return Box::pin(async move {
            let request: crate::transaction_git::GitRequest = crate::transaction_git::GitRequest::new(self.directory.as_path(), &["rev-parse", "--show-toplevel"]);
            let output = crate::transaction_git_awaiting::run_git_awaiting(&self.context, &request).await.ok()?;
            return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
        });
    }
}

/// The answers of the items,
/// as text.
#[derive(Default)]
struct SeenAnswers {
    /// One answer per item.
    answers: Vec<String>,
}

impl OrderedEvents<Option<String>> for SeenAnswers {
    fn on_event(&mut self, event: Event<'_, Option<String>>) {
        if let Event::End(_index, answer) = event {
            self.answers.push(answer.unwrap_or_else(|| return String::from("no answer")));
        }
    }
}

/// An item may await a Git child,
/// which is why this form exists beside the blocking one.
#[test]
fn an_item_may_await_a_git_child() {
    let root: PathBuf = fixture("parallel-runner-awaiting-git");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let work: Arc<AskGit> = Arc::new(AskGit { context: git_context(), directory: repo.clone() });
    let mut seen: SeenAnswers = SeenAnswers::default();
    let finished: Result<(), OrderedRunError> = crate::lazy_runtime::run_on_runtime(run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::from(vec![0; ITEMS]),
        lanes: single_lanes(ITEMS),
        workers: ITEMS,
        work,
        on_event: &mut seen,
    }))
    .expect("the runtime");
    finished.expect("the run finished");
    let expected: String = repo.display().to_string();
    assert_eq!(seen.answers, vec![expected; ITEMS], "every item is answered by the Git child it awaited");
    remove(root.as_path());
}

/// An item whose work is a job of the blocking pool says its lines as it works,
/// and the consumer sees each one before the job ends.
/// Why:
///  A job must own what it reads (`run_blocking_job_within_the_budget`),
/// so the sink an item says its lines through has to be a value
/// the item can move into that job.
struct SayFromThePool {
    /// The point the consumer reaches when it has seen the item's line.
    seen_by_the_consumer: Arc<Rendezvous>,
}

impl OrderedAwaitingWork<u64, u64> for SayFromThePool {
    fn run<'a>(&'a self, index: usize, _item: &'a u64, say: LinesToTheConsumer<u64>) -> Pin<Box<dyn Future<Output = u64> + Send + 'a>> {
        return Box::pin(async move {
            let seen: Arc<Rendezvous> = Arc::clone(&self.seen_by_the_consumer);
            // The job takes the sink: it is `'static`, and a sink the item owns may travel with it.
            let said: bool = crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
                let mut sink: LinesToTheConsumer<u64> = say;
                sink.say("halfway");
                return seen.wait_for_target();
            })
            .await
            .expect("the job ran");
            assert!(said, "the consumer saw the line only after the job ended");
            return index as u64;
        });
    }
}

/// Every line the consumer is handed,
/// and the point it reaches on the first one.
struct ArriveOnTheFirstLine {
    /// The point the item's job waits for.
    point: Arc<Rendezvous>,
    /// The events as text.
    seen: Vec<String>,
}

impl OrderedEvents<u64> for ArriveOnTheFirstLine {
    fn on_event(&mut self, event: Event<'_, u64>) {
        self.seen.push(match event {
            Event::Begin(index) => format!("begin {index}"),
            Event::Line(index, text) => {
                self.point.arrive_without_waiting();
                format!("line {index} {text}")
            }
            Event::End(index, result) => format!("end {index} {result}"),
        });
    }
}

/// A line said inside a job of the blocking pool reaches the consumer while that job still runs.
#[test]
fn a_line_said_inside_a_blocking_job_reaches_the_consumer_as_it_happens() {
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(1, PATIENCE));
    let mut seen: ArriveOnTheFirstLine = ArriveOnTheFirstLine { point: Arc::clone(&point), seen: Vec::new() };
    let work: Arc<SayFromThePool> = Arc::new(SayFromThePool { seen_by_the_consumer: Arc::clone(&point) });
    run_beside_each_other(vec![0], work, &mut seen).expect("the run finished");
    let expected: Vec<String> = vec![String::from("begin 0"), String::from("line 0 halfway"), String::from("end 0 0")];
    assert_eq!(seen.seen, expected, "the line of a job of the blocking pool did not reach the consumer as it happened");
}

//! Controls for a thread of the wrapper started before the child it waits for.

use super::*;

/// What:
///  A refused start runs nothing and says why,
/// and the refusal is used up by that one start.
/// Why:
///   The callers that report a refused thread rely on the refusal reaching them once:
/// the start after it has to work,
///  or a control of the reported failure would also fail at the next step.
#[test]
fn a_refused_start_runs_nothing_and_only_once() {
    let ran: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let for_the_body: std::sync::Arc<std::sync::atomic::AtomicBool> = ran.clone();
    refuse_one_start();
    let refused: std::io::Error = start_named_thread(
        "cli-git-control-refused",
        Box::new(move || {
            for_the_body.store(true, Ordering::Release);
        }),
    )
    .expect_err("the start a control refuses fails");
    assert!(refused.to_string().contains("refused a thread"), "{refused}");
    assert!(!ran.load(Ordering::Acquire), "the body of a refused start never runs");
    let started: std::thread::JoinHandle<()> = start_named_thread(
        "cli-git-control-started",
        Box::new(move || {
            ran.store(true, Ordering::Release);
        }),
    )
    .expect("the start after a refusal works");
    started.join().expect("the thread ends");
}

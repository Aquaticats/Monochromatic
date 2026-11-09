//! Controls for the look at the processes under its ceiling:
//! the scan runs on the blocking pool and the wait for it is awaited,
//! so a task beside the look keeps running,
//! and a scan that outlasts the ceiling counts as "may be running".

use super::*;
use crate::scan_awaited_under_a_ceiling::scan_under_ceiling;
use crate::ticker_beside_a_wait::with_a_ticker_beside;

/// How long the scan of a control takes.
const SLOW_SCAN: Duration = Duration::from_millis(150);

/// A look with the given ceiling.
fn look_with(ceiling: Duration) -> ServiceLook {
    return ServiceLook { instance: String::from("abc"), journals: Vec::new(), own_pid: 1, own_user: effective_user(), ceiling };
}

/// A scan that takes `SLOW_SCAN` and finds no service.
fn slow_scan_that_finds_nothing(_look: &ServiceLook, _proc_root: &Path) -> Option<Holding> {
    std::thread::sleep(SLOW_SCAN);
    return None;
}

/// The look of a control,
/// awaited under `ceiling` as the look functions await it.
async fn look_awaited(ceiling: Duration) -> Option<Holding> {
    return scan_under_ceiling(look_with(ceiling), ceiling, PathBuf::from("/proc"), slow_scan_that_finds_nothing, cannot_look).await;
}

/// A look whose scan takes a while is awaited:
/// the task beside it runs meanwhile,
/// and the scan's answer is the look's.
#[test]
fn a_look_that_takes_a_while_is_awaited_and_leaves_the_other_tasks_their_turns() {
    let (held, ticks) = with_a_ticker_beside(look_awaited(SLOW_SCAN * 100));
    assert!(held.is_none(), "the scan found no service, so nothing holds");
    assert!(ticks > 0, "the wait for the scan was awaited on the runtime");
}

/// A scan that outlasts the ceiling is "cannot look",
/// which counts as a service that may be running,
/// and the wait ends at the ceiling while the scan goes on in the pool.
/// Why the ticker is not what says the wait was awaited:
///   A current-thread runtime has one worker,
/// and an operating system that does not schedule it for longer than the scan
///  takes (measured:
/// this control read 336 ms of a 150 ms scan in a folder run of 745 controls,
/// with no turn
///  for the ticker)
/// denies the ticker its ticks without the wait having blocked anything,
/// so the ticks are evidence only
///  of how the host scheduled the worker.
/// What the control reads instead is the two bounds of the ceiling:
/// the wait reached it,
///  and it did not last for the scan.
#[test]
fn a_look_that_outlasts_its_ceiling_counts_as_may_be_running_at_the_ceiling() {
    let began: std::time::Instant = std::time::Instant::now();
    let (held, _ticks) = with_a_ticker_beside(look_awaited(SLOW_SCAN / 10));
    assert!(held.is_some(), "a look that could not finish counts as a hold");
    assert!(began.elapsed() >= SLOW_SCAN / 10, "the wait ran to its ceiling: {:?}", began.elapsed());
    assert!(began.elapsed() < SLOW_SCAN * 10, "the wait ended at the ceiling and not when the scan did");
}

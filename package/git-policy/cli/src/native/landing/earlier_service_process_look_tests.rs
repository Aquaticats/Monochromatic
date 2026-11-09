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

/// How long a held scan waits for its control to let it go,
/// after which it ends by itself:
/// the safety valve that makes a wait which blocks the worker fail the control instead of hanging it.
/// Why longer than `CEILING_OF_A_HELD_LOOK`:
///   The look must reach its ceiling while the scan is still held,
/// and a machine that takes seconds to schedule either
///  thread still leaves the two apart.
const RELEASE_BOUND: Duration = Duration::from_secs(10);

/// The ceiling of the held look;
///  far below `RELEASE_BOUND`,
/// and long enough that the scan has started before it,
/// whatever the machine's speed.
const CEILING_OF_A_HELD_LOOK: Duration = Duration::from_secs(2);

/// What:
///  The rendezvous a held scan and the control that reads a look share:
/// the scan says it started,
/// waits to be let go,
/// and ends once it is.
/// Why:
///   Whether the wait for a scan is awaited or blocks the worker is a fact about two threads,
/// not about how
///  long anything took:
/// an awaited wait reaches its ceiling while the scan is still inside the blocking pool,
/// and a wait
///  that blocked the worker reaches it only after the scan ended.
///  The scan's own bound keeps a blocking wait from hanging
///  the control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const halt = new Halt(); // started, then released by the control, then held until let go
/// ```
struct Halt {
    /// Signs that the scan entered the pool.
    started: std::sync::mpsc::Sender<()>,
    /// Whether the control let the scan go.
    released: std::sync::Mutex<bool>,
    /// Wakes the scan when it is let go.
    let_go: std::sync::Condvar,
    /// Whether the scan has ended,
    /// by the control's release or by its own bound.
    finished: std::sync::atomic::AtomicBool,
}

impl Halt {
    /// A rendezvous no scan has touched.
    fn new() -> Halt {
        let (started, _receiver) = std::sync::mpsc::channel();
        return Halt {
            started,
            released: std::sync::Mutex::new(false),
            let_go: std::sync::Condvar::new(),
            finished: std::sync::atomic::AtomicBool::new(false),
        };
    }

    /// Wait until the scan is let go,
    /// or until `RELEASE_BOUND` has passed.
    fn wait_to_be_let_go(&self) {
        let Ok(released) = self.released.lock() else { return };
        let _waited = self
            .let_go
            .wait_timeout_while(released, RELEASE_BOUND, |let_go| { return !*let_go; });
    }

    /// Let the held scan go.
    fn let_go(&self) {
        if let Ok(mut released) = self.released.lock() {
            *released = true;
        }
        self.let_go.notify_all();
    }
}

/// The rendezvous the one control that holds a scan installs;
///  `None` in every other control of this file.
static HALT: std::sync::OnceLock<std::sync::Arc<Halt>> = std::sync::OnceLock::new();

/// A scan that says it started,
/// waits to be let go,
/// and finds no service.
fn held_scan_that_finds_nothing(_look: &ServiceLook, _proc_root: &Path) -> Option<Holding> {
    let halt: &std::sync::Arc<Halt> = HALT.get()?;
    let _sent = halt.started.send(());
    halt.wait_to_be_let_go();
    halt.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    return None;
}

/// The look of a control under the held scan.
async fn held_look_awaited(ceiling: Duration) -> Option<Holding> {
    return scan_under_ceiling(look_with(ceiling), ceiling, PathBuf::from("/proc"), held_scan_that_finds_nothing, cannot_look).await;
}

/// A scan that outlasts the ceiling is "cannot look",
/// which counts as a service that may be running,
/// and the wait ends at the ceiling while the scan goes on in the pool.
/// Why a held scan says what the wait did, and a clock does not:
///   The proof that the wait was awaited is that the ceiling's own timer fired:
/// the runtime can drive that timer only
///  while the task that waits is not occupying its worker,
/// so a wait that blocked the worker cannot return at the ceiling,
///  whatever the machine's speed.
/// The held scan makes the scan's own end a fact this control chooses,
/// not a duration,
/// and it is
///  still inside the blocking pool when the look returns.
/// A ticker or an elapsed bound read how the operating system scheduled this
///  runtime's single worker instead (this control read 336 ms of a 150 ms scan in a folder run of 745
///  controls),
/// which is the same reading a blocking wait gives.
/// The scan is let go only after the look returned,
/// so a wait that
///  blocked the worker cannot reach the release:
/// the scan ends at `RELEASE_BOUND` by itself,
/// and the control
///  fails with what that means instead of hanging.
/// The look runs on a runtime of its own here,
/// because the ticker helper's runtime does not drive a ceiling's timer until
///  the blocking job it covers ends (measured),
/// which hides the fact this control reads.
#[test]
fn a_look_that_outlasts_its_ceiling_counts_as_may_be_running_at_the_ceiling() {
    let halt: std::sync::Arc<Halt> = std::sync::Arc::new(Halt::new());
    let _installed: Result<(), std::sync::Arc<Halt>> = HALT.set(std::sync::Arc::clone(&halt));
    let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    let began: std::time::Instant = std::time::Instant::now();
    let held: Option<Holding> = runtime.block_on(held_look_awaited(CEILING_OF_A_HELD_LOOK));
    assert!(
        !halt.finished.load(std::sync::atomic::Ordering::SeqCst),
        "the scan was still held when the look returned, so the wait was awaited and not blocked on it"
    );
    assert!(held.is_some(), "a look that could not finish counts as a hold");
    halt.let_go();
    assert!(began.elapsed() >= CEILING_OF_A_HELD_LOOK, "the wait ran to its ceiling: {:?}", began.elapsed());
    assert!(began.elapsed() < RELEASE_BOUND, "the wait ended at the ceiling and not when the scan did: {:?}", began.elapsed());
}

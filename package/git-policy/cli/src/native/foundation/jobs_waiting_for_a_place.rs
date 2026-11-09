/// What:
///  How many jobs of this process are waiting for a place of its bound on blocking jobs right now,
///  and a way to wait until one is.
/// Why:
///  A control proves that the work it measures takes a place by holding the only place and giving it back
///  only when another job asks for one;
///  a clock alone would let work that bypasses the bound pass,
///  because the holder would give the place back before that work ever tried.
/// The count is of the whole process,
/// which is why such a control runs in a test process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ask = noteThatAJobWaits(); const asked = waitUntilAJobWaits(5_000); // the holder: hold until asked
/// ```
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// The jobs waiting for a place.
static WAITING: Mutex<usize> = Mutex::new(0);

/// Woken whenever a job starts or stops waiting.
static CHANGED: Condvar = Condvar::new();

/// One job that is waiting for a place,
/// counted until the value is dropped.
pub(crate) struct AJobWaits(());

/// Take one count for a job that is about to wait for a place.
pub(crate) fn note_that_a_job_waits() -> AJobWaits {
    let mut waiting = WAITING.lock().expect("lock");
    *waiting += 1;
    CHANGED.notify_all();
    return AJobWaits(());
}

/// Give the count back,
/// with the job that held it no longer waiting.
impl Drop for AJobWaits {
    fn drop(&mut self) {
        let mut waiting = WAITING.lock().expect("lock");
        *waiting = waiting.saturating_sub(1);
        CHANGED.notify_all();
    }
}

/// What:
///  Wait until at least one job is waiting for a place,
///  or until `patience` runs out.
/// Why:
///  A control that holds the only place gives it back on an ask rather than on a clock.
/// The answer is what the holder reports:
///  false means no job of the work under test ever asked,
/// which is what work that bypasses the bound looks like.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked: boolean = waitUntilAJobWaits(5_000);
/// ```
pub(crate) fn wait_until_a_job_waits(patience: Duration) -> bool {
    let counted = WAITING.lock().expect("lock");
    let (still_counted, outcome) = CHANGED.wait_timeout_while(counted, patience, |seen: &mut usize| { return *seen == 0; }).expect("wait");
    let asked: bool = !outcome.timed_out();
    drop(still_counted);
    return asked;
}

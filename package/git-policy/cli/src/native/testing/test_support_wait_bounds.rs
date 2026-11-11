//! What:
//!  The bounds a control's wait gives up under,
//!  and the two faces that wait within them.
//! Why:
//!   A control's event (a push that lands,
//! a claim that is settled) takes as long as the disk takes,
//! so a wait on a wall clock fails work that is still coming.
//! One face waits for as long as the run keeps moving and gives the event up only when it stopped;
//! the other runs work on its own thread under a deadline the control states.
//! Both judge on the evidence the watchdog reads (`run_progress`),
//! so the run's own report reaches the reader before the watchdog ends it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! waitUntil(() => landed()); // for as long as the run keeps working
//! await finishedWithinAt(150_000, () => refresh()); // undefined when the deadline passed
//! ```

/// What:
///  Run `work` on its own thread and answer its result,
///  or nothing when it did not finish within five seconds.
/// Why:
///   A read of a named pipe waits for a writer that never comes,
/// and a control that called such a read
///  directly would hold the suite for ever instead of failing;
/// the deadline turns the wait into a failure the control
///  can name,
/// and the thread the read holds is abandoned with the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await finishedWithin(() => readPlainText(path)); // undefined when five seconds passed
/// ```
pub(crate) fn finished_within<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    return finished_within_at(std::time::Duration::from_secs(5), work);
}

/// What:
///  `work`'s value,
/// or nothing when it did not finish within `deadline`;
/// the thread that runs it is abandoned with the process.
/// Why:
///   A control whose work waits on another job's work needs a bound that covers that work,
/// which costs what the
///  fixture and the host make it cost;
/// a fixed short clock over work fails a busy host while the work itself is fine,
/// so
///  such a control states its own deadline and the wait it guards takes no clock.
/// A panic inside the work is the work's own failure and is raised here as itself,
/// so this
///  deadline cannot be blamed for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await finishedWithinAt(150_000, () => refresh()); // undefined when the deadline passed
/// ```
pub(crate) fn finished_within_at<T: Send + 'static>(deadline: std::time::Duration, work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (sender, receiver) = std::sync::mpsc::channel::<std::thread::Result<T>>();
    std::thread::spawn(move || {
        let _ = sender.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)));
    });
    return match receiver.recv_timeout(deadline) {
        Ok(Ok(value)) => Some(value),
        Ok(Err(payload)) => std::panic::resume_unwind(payload),
        Err(_passed_or_abandoned) => None,
    };
}

/// How long a control's wait lets the run show nothing before it counts the event it waits for as lost.
/// Why:
///   A wall clock cannot tell a service that lost its work from one whose push is slow on a contended disk.
/// Measured on 2026-11-09 in the push-retry controls of this crate while the host ran at a load average of 100 and more:
/// a local one-commit push took 9.6 s and 12.0 s (`push_ms=9605`,
/// `push_ms=12009`) against 1.2 to 3 s alone,
/// and controls whose wait allowed 11.8 s failed while the push went on to land just after it.
/// The judgment is the watchdog's own (`run_progress`),
/// whose window is 120 s on the same evidence
/// (`QUIET_WINDOW` in `testing/test_support.rs`:
/// the longest stretch a working run showed nothing was 8.0 s);
/// this window is half of it,
/// so a control's own report reaches the reader before the watchdog ends the run.
pub(crate) const STILL_WORKING_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);

/// How often a control's wait looks for its event and at the run's own work.
pub(crate) const WAIT_LOOK_EVERY: std::time::Duration = std::time::Duration::from_millis(50);

/// What:
///  One wait and how long it lets the run show nothing.
/// Why:
///   The controls wait for their events with `STILL_WORKING_WINDOW`,
/// and their own controls wait with a window they can reach in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WaitingFor = { window: Duration; done: () => boolean };
/// ```
pub(crate) struct WaitingFor<'a> {
    /// How long nothing may move before the event counts as lost.
    pub(crate) window: std::time::Duration,
    /// Whether the event the wait is for has happened.
    pub(crate) done: &'a dyn Fn() -> bool,
}

/// What:
///  Wait until `done` holds,
/// for as long as the run keeps working,
/// and give whether it held.
/// Why:
///   A control's event (a push that lands,
/// a claim that is settled) takes as long as the disk takes,
/// and a wait that gives up on a clock fails while the event is still coming.
/// Only a run that shows no sign of work is over:
/// the wait ends when the run has moved nothing for its window,
/// which is the evidence the watchdog judges,
/// so a slow push is waited for and a plan that was lost fails.
/// A run that keeps working for ever is ended by the test runner's own per-test limit
/// (`.config/nextest.toml`,
/// `slow-timeout` 60 s times `terminate-after` 5),
/// so nothing waits without a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitUntil = (done) => waitUntilFor({ window: STILL_WORKING_WINDOW, done });
/// ```
pub(crate) fn wait_until(done: &dyn Fn() -> bool) -> bool {
    return wait_until_for(WaitingFor { window: STILL_WORKING_WINDOW, done });
}

/// `wait_until` with the window given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitUntilFor({ window, done }: WaitingFor): boolean {
///     using reading = Reading.begin(); // this thread's own looks are not the run's work
///     ...
/// }
/// ```
pub(crate) fn wait_until_for(waiting: WaitingFor<'_>) -> bool {
    let WaitingFor { window, done } = waiting;
    // This thread reads the run at every look: registered for the whole wait, so no look taken meanwhile counts its readings,
    //  while the work its predicate does on this thread is still counted (`run_progress::look`).
    let _reading: Option<crate::run_progress::Reading> = crate::run_progress::Reading::begin();
    let mut judge: crate::run_progress::Judge = crate::run_progress::Judge::new(window);
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        if done() {
            return true;
        }
        std::thread::sleep(WAIT_LOOK_EVERY);
        // The look is taken after the wait for the event, so a run that stopped working while the event was checked is still seen to have moved.
        if let Some(ending) = judge.observe(began.elapsed(), crate::run_progress::look_now()) {
            eprintln!("a control's wait gave up after {:.1} s because the run {ending}", began.elapsed().as_secs_f64());
            return done();
        }
    }
}

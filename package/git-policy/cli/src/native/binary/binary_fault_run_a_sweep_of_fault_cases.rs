//! What:
//!  A sweep of fault cases:
//!  how many run at once,
//!  the run of the whole list in batches,
//!  and the message a failure carries with the fixture's service log.
//! Why:
//!  A sweep runs several cases at once,
//!  so the line that settles one failure is mixed with the other cases',
//!  and only the log the fixture keeps says which service said what;
//!  batching and that message are one job,
//!  and they live in a file of their own so the run file stays inside the per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const failures = inBatches(cases, check); // six at a time, each failure carrying the service log
//! ```

use super::support::Fixture;

/// How many cases of a sweep run at once:
/// enough to finish a sweep in seconds,
/// few enough to leave the host its other work.
pub(super) const SWEEP_WIDTH: usize = 6;

/// Runs `check` over every item,
/// `SWEEP_WIDTH` at a time,
/// and returns the messages of the items that failed.
pub(super) fn in_batches<T: Sync>(
    items: &[T],
    check: &(dyn Fn(&T) -> Result<(), String> + Sync),
) -> Vec<String> {
    let failures: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    for batch in items.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for item in batch {
                let collected: &std::sync::Mutex<Vec<String>> = &failures;
                scope.spawn(move || {
                    if let Err(message) = check(item) {
                        collected.lock().expect("failure list").push(message);
                    }
                });
            }
        });
    }
    return failures.into_inner().expect("failure list");
}

/// What a failure of a fault control says:
/// the message it was given and what the fixture's services logged.
/// Why:
///   A service that refuses a step or ends says so only on its own standard error,
/// and a sweep runs several cases at once,
/// so the line that settles one failure is mixed with the other cases';
/// the log the fixture keeps says which service said what
/// (the recorded failure of 2026-11-07 could be settled only by such a line).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = `${message}\n${serviceLog(fixture)}`;
/// ```
pub(super) fn with_service_log(
    fixture: &Fixture,
    message: String,
) -> String {
    return format!(
        "{message}\n{}",
        super::support::service_log(fixture.root.as_path())
    );
}

/// The verdict of a fault case,
/// with the fixture's service log carried on a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = raw.ok ? raw : { error: withServiceLog(fixture, raw.error) };
/// ```
pub(super) fn keeping_the_service_log(
    fixture: &Fixture,
    verdict: Result<(), String>,
) -> Result<(), String> {
    return match verdict {
        Ok(()) => Ok(()),
        Err(message) => Err(with_service_log(fixture, message)),
    };
}

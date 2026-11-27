//! What:
//! End-to-end tests of the rule "a shadow never starts a shadow,
//! of any version,
//! as its real `mpm`".
//! Each test builds a disposable `PATH` of shadow copies,
//! launcher records and real programs,
//! starts the built `mpm-shadow` under the name `mpm`,
//! and gives it a deadline.
//! Why:
//! Two shadows at different paths used to choose each other as the real tool (measured 2026-11-07 for `search-shadow`:
//! about 105 starts per second under `strace`).
//! A test of such a loop must itself never wait without end,
//! so every run here is polled against a deadline and killed when it passes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runWithDeadline(spawn(join(first, 'mpm'), [...]), LOOP_BOUND);
//! assert(outcome.kind === 'finished');
//! ```

/// What:
///     `#[path = "..."]` names the file a module is read from;
///     what every test here shares lives in
///     `tests/never_start_another_shadow/fixtures_and_bounded_runs.rs`.
/// Why:
///     One test binary links the built shadow once,
///     so the fixtures that build a `PATH` of it are compiled once and shared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fixtures from './never_start_another_shadow/fixtures_and_bounded_runs';
/// ```
#[path = "never_start_another_shadow/fixtures_and_bounded_runs.rs"]
mod fixtures_and_bounded_runs;

/// The runs that show a shadow never starts another shadow,
/// whichever way it meets one.
#[path = "never_start_another_shadow/shadows_that_must_not_start_each_other.rs"]
mod shadows_that_must_not_start_each_other;

/// The runs that show a launcher's record never leads back to a launcher.
#[path = "never_start_another_shadow/launchers_that_must_not_start_themselves.rs"]
mod launchers_that_must_not_start_themselves;

/**
 Time bounds and parallelism of the daily update's package checks.

 Every value cites the measurement or the owner's word that sets it (`doc/planning/toolchain-always-latest.md`,
 "Checking every package after the update", has the whole measured run).

 @module
 */

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one minute.
 */
const MILLISECONDS_PER_MINUTE: number = MILLISECONDS_PER_SECOND * SECONDS_PER_MINUTE;

/**
 Minutes a Rust check may run.
 Measured 2026-11-07 on all 31 crates in a fresh worktree, load average 10 to 40: the longest passing check took
 332 seconds and the longest failing one (a container build) 452 seconds, the median about 60 seconds.
 Four times the longest.
 */
const RUST_CHECK_MINUTES = 30;

/**
 Minutes a TypeScript check may run.
 Measured 2026-11-07 on 218 packages: median 3.5 seconds, 90th percentile 6.5 seconds, longest 97 seconds.
 Six times the longest.
 */
const TYPESCRIPT_CHECK_MINUTES = 10;

/**
 Minutes one pass over every check may take before the checks that have not started are skipped.
 Measured 2026-11-07 in a unit capped at four cores, which the service halves: the first 185 checks (179 TypeScript, 6
 Rust) took 5.1 minutes, and the remaining 25 Rust and 39 TypeScript checks took 19.5 minutes.
 A pass after a nightly change rebuilds every crate, which was not measured; 90 minutes is about twice the service-cap
 estimate and keeps two passes inside the service time bound.
 */
const CHECK_PASS_MINUTES = 90;

/**
 Seconds one `gh` call may take.
 Measured 2026-11-07: listing open issues took 5.2 seconds for a thousand with bodies and 0.5 seconds for the search the
 update uses. Creating an issue was not measured, because no real issue may be opened to measure it. Ten times the
 slowest listing.
 */
const GH_CALL_SECONDS = 60;

/**
 Seconds `mise tasks ls --all` may take.
 Measured 2026-11-07: 5.2 seconds cold and 0.5 seconds warm for 3,058 tasks. More than twenty times the cold run.
 */
const TASK_LISTING_SECONDS = 120;

/**
 Longest a Rust check may run, in milliseconds.
 */
export const RUST_CHECK_TIMEOUT_MS: number = RUST_CHECK_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 Longest a TypeScript check may run, in milliseconds.
 */
export const TYPESCRIPT_CHECK_TIMEOUT_MS: number = TYPESCRIPT_CHECK_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 Longest one pass over every check may take, in milliseconds.
 */
export const CHECK_PASS_DEADLINE_MS: number = CHECK_PASS_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 Longest one `gh` call may take, in milliseconds.
 */
export const GH_CALL_TIMEOUT_MS: number = GH_CALL_SECONDS * MILLISECONDS_PER_SECOND;

/**
 Longest the task listing may take, in milliseconds.
 */
export const TASK_LISTING_TIMEOUT_MS: number = TASK_LISTING_SECONDS * MILLISECONDS_PER_SECOND;

/**
 Rust checks that run side by side.
 Measured 2026-11-07 with two Rust and four TypeScript checks at once under four cores: the whole run used 42 minutes
 of CPU in 19.5 minutes (2.2 cores on average), peaked at 5.5 GiB of memory including page cache, and the service's
 two cores and 8 GiB hold that.
 */
export const RUST_CHECK_PARALLEL = 2;

/**
 TypeScript checks that run side by side.
 Measured with the Rust pool above: a TypeScript check takes seconds, so four at once cost under a core.
 */
export const TYPESCRIPT_CHECK_PARALLEL = 4;

/**
 Compile jobs Cargo may start for one Rust check.
 Two checks at once with two jobs each is four compiler processes, which is what the measured run used.
 */
export const CARGO_BUILD_JOBS_PER_CHECK = 2;

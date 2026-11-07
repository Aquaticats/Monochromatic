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
 PLACEHOLDER
 */
const RUST_CHECK_MINUTES = 30;

/**
 Minutes a TypeScript check may run.
 PLACEHOLDER
 */
const TYPESCRIPT_CHECK_MINUTES = 10;

/**
 Minutes one pass over every check may take before the checks that have not started are skipped.
 PLACEHOLDER
 */
const CHECK_PASS_MINUTES = 90;

/**
 Seconds one `gh` call may take.
 PLACEHOLDER
 */
const GH_CALL_SECONDS = 60;

/**
 Seconds `mise tasks ls --all` may take.
 PLACEHOLDER
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
 PLACEHOLDER
 */
export const RUST_CHECK_PARALLEL = 2;

/**
 TypeScript checks that run side by side.
 PLACEHOLDER
 */
export const TYPESCRIPT_CHECK_PARALLEL = 4;

/**
 Compile jobs Cargo may start for one Rust check.
 PLACEHOLDER
 */
export const CARGO_BUILD_JOBS_PER_CHECK = 2;

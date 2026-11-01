//region Monotonic clock
// The clock every duration, deadline, hold, pace and budget in the package is
// read on (ledger B78). The system clock is the time of day, which a person
// or a time daemon can set: stepped back an hour mid-call, a duration read on
// it went negative, a hold or a pace waited an hour longer than asked, and a
// budget lasted an hour more. The wall clock is read only as a stamp,
// `new Date().toISOString()`, for a moment that outlives the process.

/**
 Whole milliseconds on a clock the system time cannot move.

 Node's `performance.now()` reads `uv_hrtime` (`src/node_perf.cc`), which on
 Linux reads `CLOCK_MONOTONIC` (libuv `src/unix/linux.c`): unaffected by a
 jump in the system time, never going backwards, and not counting time the
 machine is suspended, as libuv's timers do not. Floored, so a difference of
 two readings is a whole number of milliseconds, as every log line and
 artifact writes durations, and never negative.

 Its origin is near the process's start, so a reading means nothing outside
 the process that took it and is never stored as a moment: only differences
 of readings in one process are durations. A reading can be 0 in a process's
 first millisecond, so 0 marks nothing as absent.

 @returns Milliseconds since an origin near the process's start, floored

 @example
 ```ts
 const startedAt = monotonicMs();
 const tookMs = monotonicMs() - startedAt;
 ```
 */
export function monotonicMs(): number {
  return Math.floor(performance.now(),);
}

//endregion Monotonic clock

# The analysis cache commits one transaction per drained batch of writes

## Status

Implemented on branch `fix/music-player-cache-batched-commits`, measured on 2026-10-17 and 2026-10-18 by an agent, on
the caller's instruction to fix finding 1 of
[`music-player-release-build-measurements-2026-10-15.md`](../audit/music-player-release-build-measurements-2026-10-15.md).
Not merged; open to the owner's veto.
The Android choice in the section "Android keeps one commit per row" is a decision the owner still has to make.
The section "What was not measured" lists what these numbers do not cover.

## Context

On the desktop release build, the first analysis of 10,000 tracks took 27 to 99 s of wall time while the measuring
sweep itself finished in 0.2 to 2.8 s.
`strace` counted 10,073 `fsync` calls for 10,000 stored rows, 9.1 ms each when timed by wall clock.
The desktop actor stored each queued decision with `DecisionCache::put`
(`package/music-player/truepeak-core/src/cache.rs`), which runs one autocommit upsert per row.

## Cause

The cause is the default durability level of the Turso crate the lockfiles pin (`turso` and `turso_core` 0.8.2 in
`package/music-player/truepeak-core/Cargo.lock`, `desktop-app/Cargo.lock` and `android-app/rust/Cargo.lock`), read in
the crate source under the cargo registry:

- Every connection starts at `SyncMode::Full` (`turso_core-0.8.2/database.rs`, the `Connection` constructor sets
  `sync_mode: AtomicSyncMode::new(SyncMode::Full)`).
  The mode belongs to the connection and is not stored in the database file.
- A commit under `Full` submits a WAL `fsync` and waits for it (`turso_core-0.8.2/storage/pager.rs`, the
  `CommitState::WaitSync` branch: `sync_mode == SyncMode::Full && need_fsync` calls `wal.sync`).
  Under `Normal` or `Off` that branch submits nothing.
- The setting is the SQL pragma `PRAGMA synchronous`, parsed in `turso_core-0.8.2/translate/pragma.rs`
  (`0` is `Off`, `1` is `Normal`, anything else is `Full`).
  There is no `Builder` flag for it in `turso-0.8.2/src/lib.rs`.
- The journal mode is the write-ahead log: the current docs say Turso supports only that mode, and the 0.8.2 checkpoint
  threshold is 1000 frames (`storage/wal.rs`, `checkpoint_threshold: 1000`).
- `DecisionCache::put` passes one `INSERT ... ON CONFLICT DO UPDATE` to `Connection::execute` with no open
  transaction, so each row is its own commit and so its own `fsync`.

The current Turso docs (`https://docs.turso.tech/sql-reference/pragmas.md`, fetched 2026-10-17) say "Only `OFF` and
`FULL` are supported in Turso", describe `FULL` as "fsync after every transaction", and state no default.
The 0.8.2 source accepts `NORMAL` and implements it, so the docs and the source disagree on that value.
The audit's 73 `fsync` calls beyond the 10,000 rows are checkpoint traffic: the benchmark below counts 70 `fsync`
calls with one commit per row and no commit `fsync` (the `normal` configuration).

## Callers

Every caller of `DecisionCache::put` in the repository:

- `package/music-player/desktop-app/src/peakcache_service.rs`, the actor, one call per queued `Write` (before this
  change).
  The desktop write channel is fire-and-forget, so the sweep fills it far faster than the disk drains it.
- `package/music-player/android-app/rust/src/service.rs`, the function `put`, one call per `Mutation::Write`.
- `package/music-player/truepeak-core/src/service.rs`, `cached_or_resolve`, which no application calls; only tests do.

## Measurements

The ignored benchmark `cache::tests::bench_cold_write` in `package/music-player/truepeak-core/src/cache_tests.rs`
stores 10,000 invented exact decisions into a fresh database file, with fingerprints spread across the key space the
way content hashes are.
The directory was `/var/mnt/encrypted/temp/agent/cachebench` on `btrfs` (`compress=zstd:3`, `ssd`) over the encrypted
device `/dev/mapper/crypt_sda`, checked with `findmnt`, so it is a real disk and not `tmpfs`.
Each configuration ran five times, interleaved by repeat, each run once without tracing (wall time) and once under
`strace` (`fsync` count, so the tracing cost never enters the wall time).
Every run was a separate systemd user unit with `MemoryMax=8G`.
Positive and negative controls for the counts: the `row` runs counted one call per row, and the `batch-normal:256`
runs, which commit 40 times without any commit `fsync`, counted 7, which is the fixed overhead of the traced command.

```bash
# shell: one run (set CACHE_BENCH_MODE to row, normal, batch or batch-normal; CACHE_BENCH_BATCH sets rows per commit)
CACHE_BENCH_DIR=/var/mnt/encrypted/temp/agent/cachebench CACHE_BENCH_MODE=batch CACHE_BENCH_BATCH=256 \
  strace --follow-forks --summary-only --trace=fsync,fdatasync --output summary.txt \
  mise run //package/music-player/truepeak-core:bench:cache
```

Results, wall time of the 10,000 stores (minimum, median, maximum of five) and `fsync` calls (identical across the
repeats unless a range is given):

- One commit per row (`row`, the behavior before this change): 75.9 s, 83.5 s, 87.1 s; 10,070 to 10,078 calls.
  That is 8.4 ms per row at the median, which reproduces the audit's 9.1 ms and its 27 to 99 s range on the desktop
  build (the audit's fastest runs were on quieter hosts).
- `PRAGMA synchronous = NORMAL`, one commit per row (`normal`): 0.95 s, 1.41 s, 1.81 s; 70 calls.
- One transaction per 256 rows (`batch:256`): 0.49 s, 0.52 s, 0.64 s; 47 calls (40 commits plus checkpoint traffic).
- One transaction per 256 rows and `NORMAL` (`batch-normal:256`): 0.23 s, 0.24 s, 0.42 s; 7 calls.
- One transaction per 16 rows (`batch:16`): 5.1 s, 9.0 s, 13.2 s; 656 calls.
- One transaction per 64 rows (`batch:64`): 1.35 s, 1.57 s, 2.53 s; 173 calls.
- One transaction per 1,000 rows (`batch:1000`): 0.26 s, 0.28 s, 0.55 s; 14 calls.

Run-to-run spread on unchanged input is large for the flush-bound configurations (`row`: 75.9 to 87.1 s; `batch:16`:
5.1 to 13.2 s), so only differences larger than that spread are claimed.
The change from `row` to `batch:256` is 160 times at the median and the ranges do not touch.

## Decision

The desktop actor takes the write it just received, then every write already waiting in its queue up to
`WRITE_BATCH_LIMIT` (256, in `truepeak-core/src/cache.rs`), and stores them with `DecisionCache::put_batch`, one
transaction and so one `fsync`.
Nothing waits for more writes to arrive: a lone write is a batch of one and is stored as soon as before, and a
backlog (the cold sweep) drains in batches of 256.
The durability of every commit is unchanged, because each commit is still a full-durability commit.
When a batch fails, `put_batch` rolls it back and retries each row on its own, so one refused row cannot discard its
neighbours; the call then reports how many rows failed.

The limit of 256 is read from the benchmark.
A commit of 256 rows took about 13 ms (0.52 s over 40 commits), and a commit of 1,000 rows about 27 ms (0.28 s over
10 commits); each figure is the median wall time divided by the commit count.
The actor serves reads between batches, so a read waits behind at most one batch; 13 ms stays within one display frame
of a 60 Hz screen and 27 ms does not.
Going from 256 to 1,000 saves 0.24 s of a drain that is no longer the bottleneck.
A limit of 64 costs 1.1 s more than 256 and 16 costs 8.5 s more, and neither buys any shorter read wait that matters.

## Crash proof

The tests in `package/music-player/truepeak-core/src/cache_tests.rs` start a child process that stores batches of 64
rows forever, kill it with SIGKILL at varying instants (a pause of 0 to 3.999 ms after a report), reopen the same
database file, and repeat on the same file:

- `batched_writes_survive_kill_nine_with_whole_batches_only` runs four kills each under `synchronous = FULL` and
  `NORMAL`.
  Every reopen passes `PRAGMA integrity_check`, holds every row the child reported as committed, holds a prefix of
  the invented keys with no gap, and holds a whole number of batches.
- `row_writes_after_kill_nine_leave_partial_batches` is the control: the same kills with one commit per row leave a
  partial batch in at least one of twelve rounds, which shows the kills land inside a batch's worth of work.
- Guard check: with the transaction removed from `put_batch` (committed first, then removed, then restored), the batch
  test failed with "a batch was torn (reported 320, found 447)".
- Thirty repeats of the crash tests together with the `put_batch` tests passed (`cargo nextest ... --stress-count 30`).

A kill -9 does not lose OS-cached writes, so it proves that a batch is atomic and that the database reopens, and it
does not prove the behavior after power loss.
The power-loss behavior of `put_batch` under `FULL` is the same as a single `put` under `FULL` today, because the
commit path is the same.

## Rejected alternatives

- `PRAGMA synchronous = NORMAL` alone, set in `DecisionCache::open`: 0.95 to 1.81 s, one line, and it would cover the
  Android actor too.
  Rejected because the value is outside what the current Turso docs list as supported, its power-loss behavior was not
  tested here (no way to cut power in this session), and it lowers durability for a gain the batch already delivers.
  It remains the candidate for Android if the owner wants one (see the next section).
- Batching together with `NORMAL`: 0.24 s against 0.52 s for batching alone.
  Rejected because 0.28 s of a drain that now takes about half a second does not repay lowering durability.
- `PRAGMA synchronous = OFF`: the docs state a risk of corruption on a crash.
- Buffering writes for a time slice before committing: it delays a lone write by the slice and loses the whole slice in
  a crash, where draining costs nothing when the queue is empty.
- One transaction for the whole sweep: a crash loses every row, and a read in the middle sees none of them.
- A buffer inside `DecisionCache` that returns from `put` before the commit: a `get` straight after a `put` would stop
  seeing it unless the buffer were also read, which is a larger design than a drained queue.
- MVCC journal mode: the docs call it experimental.

## Android keeps one commit per row

The Android actor (`package/music-player/android-app/rust/src/service.rs`) calls the same `DecisionCache::put` under
the same `SyncMode::Full` from the same lockfile version, so each stored row there is also one commit and one `fsync`.
That is a code argument; the cost per `fsync` on a phone's flash was not measured.
The impact differs from desktop, and batching the drained queue would not help:

- The warming path calls `put_and_wait` (service.rs), which blocks the calling thread until its row is committed.
- The foreground sweep runs `max(2, cores / 2)` such threads (`FOREGROUND_SWEEP_WORKERS` in `PeakMeasurer.kt`) and the
  background upkeep worker runs one (`workers: 1` in `PeakSweepWorker.kt`), and each thread decodes a whole track
  before it writes.
  The queue therefore never holds more than the number of waiting workers, so a drain would almost always find one
  row.
- The playback path calls `put`, fire-and-forget, for single tracks, which is a handful of rows and not a backlog.

The Android native crate does not call `put_batch`.
If a measurement on a device shows the commit cost matters there, the options are relaxed `synchronous` on that
connection or a commit coalescing window; both are the owner's decision and need a device or emulator measurement
first.

## What was not measured

- The desktop app's cold start was not rerun end to end; the 27 to 99 s figure is inferred to fall to the benchmark's
  0.5 s of cache time, not shown on the release build.
- Whether the desktop queue reaches the 256-row limit in the real app is inferred from the audit (the sweep ends in
  0.2 to 2.8 s while the drain took 27 to 99 s, so thousands of writes were queued), not logged.
  `drain_writes` logs the batch size at trace level for a check.
- The 180 ms of CPU per added file at 10,000 tracks and the 23 to 25 s busy tail after the second sweep were not
  investigated; the benchmark measures only the cache write path.
- Power-loss behavior, and every Windows, macOS and device run.

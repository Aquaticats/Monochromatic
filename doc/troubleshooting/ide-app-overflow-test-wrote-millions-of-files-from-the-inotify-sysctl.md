# `package/ide/app` test `queue_overflow_requests_a_full_reread` wrote about five million files because it sized its burst from `fs.inotify.max_queued_events`

Recorded 2026-11-07T13:50:00-05:00.

## Symptom

`mise run //package/ide/app:test` sat on one test for more than twenty minutes
(`SLOW [>1080.000s] (───────) ide::change_watch recovery::queue_overflow_requests_a_full_reread`).
More than four million files were on disk under the worktree's `package/ide/app/target/tmp/.tmp<random>/burst` when the run was stopped.
The 2026-10-29 record in `doc/planning/rust-nightly-always-latest.md` already listed this test as the single failing test of that package.

## Root cause

The test read the host-wide limit and wrote a burst from it (`tests/change_watch/recovery.rs`,
before the fix):

```rust
let limit: usize = fs::read_to_string("/proc/sys/fs/inotify/max_queued_events") /* ... */;
for index in 0..(limit / 2 + 64) {
    fs::File::create(folder.join(format!("{index}.txt"))).expect("burst file");
}
```

This host sets `max_queued_events` to 10000000,
so the count is 5000064 (printed from the same formula before any file was written).
The kernel default is 16384,
the value the test was written for.

The limit cannot be lowered for a test's own inotify instance:
`fs/notify/inotify/inotify_user.c` keeps one `static int inotify_max_queued_events` (the `max_queued_events` sysctl entry,
default 16384)
and every new group takes it (`inotify_new_group(inotify_max_queued_events)`;
Linux v6.12 source,
line numbers 50,
81-82,
707,
867).
Changing it needs a root write to `/proc/sys`,
so no real-kernel test can overflow a queue it controls.

## Verification

The count:
`node` reading `/proc/sys/fs/inotify/max_queued_events` printed `max_queued_events=10000000`
and the formula gave `5000064`;
the 4000000-file version was not rerun.
The files were removed (`package/ide/app/target/tmp` holds no entry afterwards).
No other test in the repository sizes writes from a `/proc/sys` value.
`rg` for `max_queued_events`,
`max_user_watches` and `/proc/sys` over `*.rs`,
`*.ts` and `*.toml` in `package/` finds:
message texts in `package/ide/app/src/change_watch`;
the `inspect-watch-limit.ts` script (which writes `/proc/sys/user/max_inotify_watches` inside a user namespace it enters);
`/proc/sys/kernel/random/boot_id` reads in process-identity code;
`/proc/sys/kernel/pid_max` read once as the number of an unused process id
(`git-policy/cli` `single_owner_claim.rs`,
`mutation-test/cli-cargo-mutants` `claim-test-support.ts`);
and a `ns_last_pid` write inside a throwaway PID namespace (`git-policy/cli` `stale_ref_locks_tests.rs`).
None of these makes a count of files or processes from the value.

## Verified workaround

The recovery path is driven through `ChangeWatcher::deliver`,
the call notify's handler makes
(`src/change_watch.rs`,
`record::record`).
notify 8.2.1 builds the overflow event as `Event::new(EventKind::Other).set_flag(Flag::Rescan)` (`src/inotify.rs`);
the tests deliver that event with a path,
so only the overflow branch of `record.rs` can ask for the full reread:

- `a_queue_overflow_event_requests_a_full_reread` passes,
  and fails with the `Rescan` branch of `record.rs` disabled
  (`an overflowed queue's full reread was not reported: Changes { ... everything: false ... }`);
- `an_event_without_the_overflow_flag_requests_no_full_reread` is the control.

Tradeoff:
the kernel's own overflow and notify's reading of it are no longer exercised together;
the seam trusts notify's mapping of `IN_Q_OVERFLOW`.

## What does not work

A test that lowers the queue limit:
the limit is a host-wide sysctl.
A host check that skips the test above a threshold:
it would hide the path on the machines it matters on.

## Upstream filing decision

Not applicable:
the defect is in this repository's test.

# Two wrapper controls wait on a guessed wall clock and on flushes the service defers

Two families of `package/git-policy/cli` controls passed alone and failed inside a larger run on 2026-11-09
(load average about 48 on eight processors,
twelve gigabytes of swap in use,
about twenty agents committing):

- `landing::service_land::adoption_tests::a_socket_that_cannot_start_the_earlier_helper_does_not_hold_the_adoption`,
  `an_open_batch_of_the_earlier_helper_is_finished_by_the_new_service` and
  `the_adoption_says_what_it_did_with_each_batch` reached the runner's per-test bound
  (`TIMEOUT` at 306 to 337 s in a run of 341 unit tests that took 932 s).
  Alone the whole 22-test module passed in 26.7 s.
- `package_repair::a_running_repair_does_not_hold_another_add` failed once at 58.5 s in a run of 36 tests,
  passed alone in 2 s,
  and passed in a rerun of its set.

Both are arrangement defects,
not product waits:
neither control claimed what it measured.
The product's own bound on each of them is below,
and both fixes keep it.

## The adoption controls measured the disk

The three controls that timed out are the three that finish a batch,
and each lands a setup batch first.
The setup landing ran outside a landing task,
so every flush of a record it wrote was made on the control's own thread,
while the service queues those flushes for the background task
(`package/git-policy/cli/src/native/landing/service_durable.rs`,
"Force at leaving,
else background",
and the control `a_landing_that_publishes_waits_for_no_wrapper_sync`).

Measured on 2026-11-09,
one control under `strace -f -tt -T`,
`landing::service_land::adoption_tests::the_adoption_says_what_it_did_with_each_batch`:

- 79.63 s in all;
- 10 `fsync` calls,
  all on the control's own thread,
  summing 66.358 s:
  5.921 s and 3.252 s for the intent file and its directory,
  5.894 s for the index-lock marker,
  12.913 s and 2.945 s for a fabricated intent of a landing in progress,
  3.545 s and 27.521 s for two more lock markers the adoption took,
  and 3.781 s and 0.560 s for the index it installed.

The same module passed in 0.784 s on the same host later,
which is the same shape as the original report:
the control's time is the disk's,
not the code's.

## The repair control measured its own guesses

`package_repair::a_running_repair_does_not_hold_another_add`
held a fixture `pnpm` for a guessed 15 s and waited a guessed 30 s for that repair to start.
Under the load above the first add reached its repair after more than 30 s,
so the control failed with

```text
# cargo nextest run --test native_binary package_repair
thread 'package_repair::a_running_repair_does_not_hold_another_add' (244830) panicked at src/native/binary/binary_package_repair_tests.rs:250:5:
the first add's repair never started
```

after 45.571 s,
and the five other controls of the same selection were `SLOW` past 60 s and passed at 63.8 to 65.4 s.
The same guessed hold failed the other way as well:
a healthy second add that took longer than 15 s expired the hold,
and the control reported a repository-wide lock that is not there.

## The fixes

The adoption controls' setup batch now lands inside a landing task,
and the intents they fabricate are written the way a landing writes them
(`earlier_service_journal_adopted_tests.rs`,
`write_as_a_landing_in_progress`),
so their flushes are queued for the background task exactly as the service queues them.
A control fails if one is ever waited for again:

```text
# cargo nextest run --lib the_batch_these_controls_leave_open_is_landed_without_waiting_for_a_flush
the setup landing waited for a flush: ["sync-waited:intent-file", "sync-waited:intent-dir", "sync-waited:lock-marker"]
```

The repair control's hold now ends at the release the control writes,
at the end of its fixture,
or with the control process (`while_test_lives`),
and it can end no later than the repair's own quiet window
(`REPAIR_TOOL_CEILING` in `package/git-policy/cli/src/native/service/wait_ceilings.rs`).
The wait for the repair to start ends at the event:
the repair starting,
or the first add ending,
which is reported with the first add's own output:

```text
# the fixture pnpm exiting before it marks the repair started
the first add ended (exit status: 0) before its repair started, so its worktree was never repaired: monochromatic-git-policy-cli: copied 3 ignored filesystem entries ...
# the fixture pnpm marking it started and ending at once
the first add ended before the second returned, so the second waited for it: monochromatic-git-policy-cli: copied 3 ignored filesystem entries ...
```

One backstop is left,
200 s,
and it names what it waits for
("the first add's repair had not started after 200 s and the first add was still running,
so the add is stuck before pnpm").
It is below the runner's per-test bound (`.config/nextest.toml`,
60 s times 5),
so a stuck repair is a named failure and not a timeout,
and it is not what decides a healthy run:
the wait ends at the event,
however slow the host is.

## Verification

- `mise run //package/git-policy/cli:native:test:host -- adoption_tests`:
  23 of 23 pass,
  0.784 s (`Summary [0.784s] 23 tests run: 23 passed`).
- `mise run //package/git-policy/cli:native:test:binary:host -- package_repair`:
  11 of 11 pass,
  8.830 s (`Summary [8.830s] 11 tests run: 11 passed`).
- The same traced control after the change:
  0.61 s,
  7 `fsync` calls of 1.081 s in all,
  of which 2 are on the control's own thread (0.260 s and 0.006 s) and 5 on the background task.
- Both new guards were shown to fail before they were kept:
  the flush control above with the landing task removed,
  and the repair control with its fixture `pnpm` exiting before the repair marks itself started,
  then with it marking itself started and ending at once
  (the two quoted messages).

## What is not claimed

- No product wait was changed.
  The service already queued the landing's record flushes,
  and the recovery still makes its own on the thread that asks
  (`service_durable_tests.rs`,
  "a flush outside a landing is made now,
  as recovery needs it").
  That is the 2 flushes of 0.266 s left on the control's thread,
  and a disk that stalls those for minutes would still stretch a control that finishes a batch.
  Deferring them is a durability decision for the owner,
  not a test repair.
- The runner's per-test bound is unchanged.
  Raising it would have hidden both defects:
  the adoption controls would still have measured the disk,
  and the repair control would still have reported the guess.
- The module times above were taken at a load average of 8 to 48 on this host,
  not under the load of the failing run.
  What carries over is the attribution (the flushes of the control's own thread,
  and the guessed bounds),
  not the seconds.

## Records this changes

`doc/handover/cli-git-rust-implementation.md` listed both controls as open under `AGENTS.md` `SUF`;
that entry is gone with this record,
which holds the measured cause and the two arrangement fixes.

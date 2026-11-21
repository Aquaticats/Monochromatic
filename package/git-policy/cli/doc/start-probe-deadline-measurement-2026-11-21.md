# The start probe's deadline, measured 2026-11-21

This measurement is unapproved and non-final.
It was taken by an agent on 2026-11-21 in one throwaway `mvm` libvirt guest.
Only the owner declares the value settled.

## What was measured

M4's whole-probe timing (`doc/planning/cli-git-zfs-start-probe-design.md`,
"M4:
what the probe costs"),
as the ratio of the root start probe to a control arm of the same binary with
the probe disabled,
interleaved in the same rounds.
`CLI_GIT_TEST_ONLY_START_PROBE=disabled` is the control:
it writes a skipped record and mounts nothing.

The times are `date +%s%N` readings around the process,
in microseconds.
The probe reading and the control reading are the whole process,
while the deadline starts later,
so the report also gives their difference as an estimate of the worker phase.

## What the deadline bounds

`probe_mode.rs` starts the clock just before it forks the worker,
after the compatibility gate,
the fingerprint read and the started record.
The deadline therefore bounds the worker and holder phases only:
the namespace,
the walk,
the attach,
the read-back,
the two scan passes,
the move,
the unmount and the closing scan,
plus the mode's wait for them.
It does not bound the gate,
the fingerprint or the started record,
and it does not bound the boot unit that step 19 adds.

The control runs the same gate,
fingerprint and started record and then nothing,
so the probe round minus the control round estimates the worker phase.
A negative difference is control noise,
not a negative worker phase,
and a high control round in the boot-first arm shows that noise.

## Where

The throwaway `mvm` libvirt guest of the ZFS binding's recipe,
Ubuntu 24.04,
kernel `6.8.0-147-generic`,
module and userspace OpenZFS `2.4.4-1`,
pool `zpcachyos` on a 1 GiB file-backed image,
probe dataset `zpcachyos/nosnap/git-worktree-probe` with its empty `ready`
snapshot.
The host's own storage and namespace count were not measured,
which the design says is the owner's act on the owner's authorisation.

## The rounds

Every round is kept in the run's logs:
`guest-green.log` for the warm green arm,
`guest-timings.log` for the warm,
cache-dropped and many-namespace arms,
`guest-timings-boot-first.log` for the first run after a guest boot,
and `guest-m3-kill.log` for the kill matrix.

### Warm

Five interleaved rounds on the already-used binary and pool:

- Round 1:
  probe 102,505,
  control 77,264,
  difference 25,241.
- Round 2:
  probe 94,973,
  control 77,487,
  difference 17,486.
- Round 3:
  probe 96,237,
  control 86,751,
  difference 9,486.
- Round 4:
  probe 102,293,
  control 93,432,
  difference 8,861.
- Round 5:
  probe 103,230,
  control 75,889,
  difference 27,341.

The warm green arm's own five rounds were probe 126,089 to 153,850 and
control 93,313 to 126,929,
differences 17,721 to 32,776.

### After dropping caches

The pool was exported,
the page cache dropped,
the pool imported again with `cachefile=none`,
the cache dropped again,
and then one probe round and its control ran.
Each round is followed by a warm round.

- Round 1:
  probe 104,147,
  control 77,403,
  difference 26,744;
  after-cold probe 99,305.
- Round 2:
  probe 108,328,
  control 77,914,
  difference 30,414;
  after-cold probe 97,472.
- Round 3:
  probe 104,912,
  control 78,709,
  difference 26,203;
  after-cold probe 90,527.

Dropping the caches moved the whole probe by about the control's own spread,
not by the 75 milliseconds the earlier draft attributed to a cold binary.

### The first run after a guest boot

The guest was rebooted,
the built `2.4.4` module was loaded again from
`/lib/modules/$(uname --kernel-release)/extra`,
the pool was imported,
and the first probe run of that boot was kept:
145,525 microseconds,
verdict `passed`.
The five warm rounds after it were probe 99,415 to 119,242 and
control 78,581 to 164,744;
one high control round makes its differences range from -46,415 to 21,125,
which is the control noise this document refuses to read as worker time.

### Many mount namespaces

The guest started with 9 mount namespaces.
Each added namespace held 20 tmpfs mounts.

With 109 namespaces,
five interleaved rounds:

- Round 1:
  probe 153,667,
  control 85,599,
  difference 68,068.
- Round 2:
  probe 120,499,
  control 81,624,
  difference 38,875.
- Round 3:
  probe 123,368,
  control 80,322,
  difference 43,046.
- Round 4:
  probe 134,996,
  control 95,088,
  difference 39,908.
- Round 5:
  probe 136,458,
  control 94,022,
  difference 42,436.

With 309 namespaces,
five interleaved rounds:

- Round 1:
  probe 198,795,
  control 92,446,
  difference 106,349.
- Round 2:
  probe 200,301,
  control 88,485,
  difference 111,816.
- Round 3:
  probe 197,714,
  control 86,572,
  difference 111,142.
- Round 4:
  probe 205,790,
  control 93,111,
  difference 112,679.
- Round 5:
  probe 211,176,
  control 92,673,
  difference 118,503.

Every run in both sets passed.
The scan reads one table per namespace three times,
so the worker phase grows with the number of namespaces and with each table's
size.

## What the value is

`PROBE_DEADLINE` stays 350 milliseconds.

The slowest whole-probe round kept here is 211,176 microseconds,
with 309 namespaces,
and the slowest worker-phase estimate is 118,503 microseconds.
The 350-millisecond bound is the slowest phase estimate plus the
231-millisecond `PROBE_DEADLINE_MARGIN`,
which is 349.5 milliseconds,
and that margin is larger than the 109,642-microsecond spread between the
slowest and fastest phase estimates.
A run inside the range these rounds saw therefore does not fail closed on
noise.
The mode kills the worker's process group and the holder at the deadline,
reaps the worker with `WNOHANG`,
and writes a timed-out record naming the step in flight and the last step
that finished,
which the green arm checks with the `hold-move` arm,
the kill matrix checks at every step,
and the raised-deadline control checks with the same hold and a bound above
it.

The earlier draft's 230,892-microsecond slowest round,
its 116,570-microsecond fastest round and its "first run after a build" row
are not in any kept log.
This document replaces them with the rounds above.

## Limits

- The guest's pool is a file-backed image,
  so these numbers are the guest's semantics rather than this host's storage;
  the design says a reading on this host's storage is the owner's act.
- The same guest could not combine a cold cache with several hundred
  namespaces,
  and it did not run on this host's storage.
  The two effects were measured apart,
  so their sum is not measured here.
- This host's root namespace count is not measured:
  as the account, this run could read only a small part of this machine's
  tasks.
- Per-arm timings were not taken:
  the record names each step's outcome but carries no arrival time,
  so the arm that dominates the run is not measured.
- The boot-and-login half of M4 is step 19's,
  where the unit that carries the probe measures the delay it adds to the
  first login.
- The difference between the probe and control rounds is an estimate of the
  worker phase,
  not a direct reading,
  and a noisy control round makes it negative or too high.

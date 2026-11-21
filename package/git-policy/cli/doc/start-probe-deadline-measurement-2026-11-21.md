# The start probe's deadline, measured 2026-11-21

## What was measured

M4's whole-probe timing (`doc/planning/cli-git-zfs-start-probe-design.md`,
"M4:
what the probe costs"),
as the ratio of the root start probe to a control arm of the same binary with
the probe disabled,
interleaved in the same rounds.
`CLI_GIT_TEST_ONLY_START_PROBE=disabled` is the control:
it writes a skipped record and mounts nothing,
so the difference is the probe's own work.
Each round ran the probe and then the control,
five rounds per green run,
and the driver ran three green runs.
The times are `date +%s%N` readings around the process,
in microseconds.

## Where

The throwaway `mvm` libvirt guest of the ZFS binding's recipe,
Ubuntu 24.04,
kernel `6.8.0-147-generic`,
module and userspace OpenZFS `2.4.4-1`,
pool `zpcachyos` on a 1 GiB file-backed image,
probe dataset `zpcachyos/nosnap/git-worktree-probe` with its empty `ready`
snapshot.
The host's own storage was not measured,
which the design says is the owner's act on the owner's authorisation.

## The rounds

Probe,
in microseconds,
in the order the rounds ran:

- Run A:
  137228,
  131934,
  129315,
  130223, 127672.
- Run B:
  121946,
  133401,
  117635,
  120539, 117453.
- Run C:
  149450,
  128510,
  144710,
  127813, 116570.

Control,
the same order:

- Run A:
  81828,
  79623,
  81077,
  85164, 84037.
- Run B:
  74121,
  77943,
  81550,
  80997, 73130.
- Run C:
  76080,
  96622,
  75607,
  74551, 77268.

The probe's slowest round was 149450 and its fastest 116570,
a spread of 32880.
The control's slowest was 96622 and its fastest 73130.
The interleaved ratio was 1.3 to 2.0,
with a median near 1.6;
the control's own cost is the process start,
the compatibility gate,
the fingerprint and the record,
and the probe adds about 30 to 55 milliseconds over it.

## What the value is

`PROBE_DEADLINE` is 200 milliseconds,
which is the slowest measured round (149450) plus a margin of 51 milliseconds,
and that margin is larger than the 32880-microsecond spread the rounds
covered.
`PROBE_DEADLINE_MARGIN` records the 51 milliseconds on its own.
A run inside the range the harness saw therefore does not fail closed on
noise.
The mode kills the worker's process group and the holder at the deadline,
reaps the worker with `WNOHANG`,
and writes a timed-out record naming the step in flight and the last step
that finished,
which the green run checks with the `hold-move` arm.

## Limits

- The guest's pool is a file-backed image,
  so these numbers are the guest's semantics rather than this host's storage;
  the design says a reading on this host's storage is the owner's act.
- Per-arm timings were not taken:
  the record names each step's outcome but carries no arrival time,
  so the arm that dominates the run is not measured.
  The deadline bounds the whole run,
  which is what it is for,
  and the step in flight is still named when it fires.
- The ratio is against the control in the same round only.
  The control's own spread (73 to 97 milliseconds) is partly the guest's
  process start,
  so the ratio is a bound on the probe's share rather than its exact cost.
- The boot-and-login half of M4 is step 19's,
  where the unit that carries the probe measures the delay it adds to the
  first login.

# Second-pass reads of the alpha releases: what each confirmed

What the second pass over each alpha release of this package confirmed with the
released binaries,
one section per release.

Status:
**unapproved and non-final**.
Written by the coordinating session from the second passes' reports,
with the alpha.18 section written by the round that answered that read,
which says so where the section begins.
Each pass was another agent of the same vendor,
so a second pass and not an independent review,
and each is named by the release it read.
Every figure below is one that pass measured;
the behaviour it confirmed is stated where this package's `README.md` states
it,
and a section holds what the released binaries themselves showed,
and what that answer changed where its own round answered the read.

## The alpha.13 pass: cost of a trivial call against alpha.11

The pass compared `0.1.5-alpha.13` with `0.1.5-alpha.11`,
and with `0.1.5-alpha.12` beside them.
In six interleaved rounds a trivial shadow call took 1.085 times alpha.11's time
at the median (0.667 to 2.205),
against 1.097 (0.776 to 2.364) for a same-binary control and 1.982 for alpha.12.
It asked the manager two questions a call where alpha.12 asked ten,
and no stop in 24 scoped runs.

## The alpha.14 read: cost of a call against alpha.11, and what the harness could not tell apart

The read compared `0.1.5-alpha.14` with `0.1.5-alpha.11`,
and found no decision or output that differs from `0.1.5-alpha.13` beyond what
the merges name.
The manager is asked two questions a call in alpha.11,
alpha.13 and alpha.14 alike,
over 260 timed calls.
The cost of a call against alpha.11 had a median ratio of 1.021 for alpha.14
(0.409 to 2.072 over 20 rounds),
with a byte-identical copy of alpha.11 as the control at 0.907
(0.457 to 2.019),
so the harness tells alpha.14 from neither alpha.13 nor alpha.11.

## The alpha.15 read: what the released binaries showed, and the resolution of a timing here

The read compared `0.1.5-alpha.15` with `0.1.5-alpha.14`.
Between the two released binaries nothing differs but the version and the
sentences the merges name:
226 of 232 compared outputs were identical,
the rest version lines,
and alpha.12 as the case that must differ did,
in 21.
A trivial call asks the manager two questions in alpha.11,
alpha.14 and alpha.15 against six to ten in alpha.12.
Timing cannot see a small change on this host:
a byte-identical copy's own ratio ranged from 0.414 to 1.500.

## The alpha.16 read: what the released binaries showed across a reexec

The read compared `0.1.5-alpha.16` with `0.1.5-alpha.15`.
Across real reexecs of a container's own manager,
13 of 13 calls per arm that began inside a reexec took the weaker path under
`0.1.5-alpha.15` and the delegated route under `0.1.5-alpha.16`.
The kinds of later run after a kept journal,
and the fast path,
behave the same on both releases.
Where the manager answers at once the two differ in 12 of 240 compared files:
the version lines,
and the help of `install` and `update`,
which now prints the long layout for every option,
a change no record names.

## The alpha.18 read: a stop that outlasted the discovery's bound, and a path the manager reports

The read was a same-vendor second pass over `0.1.5-alpha.18`
(sha256 `2cccd94c`),
and this section is written by the round that answered its two Medium points,
so it holds what the read found and what each answer changed,
not only what the released binary showed.
The round that wrote it is named by its branch in
`doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`,
which also holds the measurements and the controls.

### Medium 1: a stop that outlasts the bound was taken as a route, not refused

The package's records said a slice whose stop outlasts the discovery's bound
"is refused by that bound rather than waited for".
The released binary did the opposite:
its start was killed at the bound,
the job that start had queued stayed in the manager,
the ask after it reported the path and that job,
and the call took the path.
The scope launcher then waited for that job with no bound of this tool's.
The read measured 29.8 s to 29.9 s in six of six calls behind a stop of 30 s,
and a call still waiting at 45 s behind a stop that never ends.

What changed:
a path beside a job after a start that spent `DISCOVERY_BOUND` is no route.
The refusal says it stops the call,
and the call stops before anything is started,
so no lockfile is changed and the person's command does not run with `--locked` added.

What the round that answered it measured
(2026-11-28,
a rootless container whose own user manager was systemd 262.2,
a service in `app.slice` that ignores SIGTERM under `TimeoutStopSec=30`,
the stop issued about 0.7 s before each call,
three rounds,
one call of each arm in every round and the arms alternated within it,
so three calls an arm,
every call meeting the state):

- `0.1.5-alpha.18`:
  exit 0,
  29.584 s,
  29.579 s and 29.375 s,
  on the delegated route.
- the answering branch:
  exit 1,
  10.027 s,
  10.036 s and 10.026 s,
  with the refusal naming the program,
  the bound and the job.
- the ratio of the released call to the branch's,
  per round:
  2.95,
  2.95 and 2.93.
- without `--dry-run` from the same state:
  the released binary exited 0 after 29.477 s and left the lockfile updated,
  and the branch exited 1 after 10.029 s and left the lockfile exactly as it found it.

### Medium 2: a path the manager reports was used without checking it was the slice's

The discovery accepted any path whose directory exists and holds `cgroup.kill`.
A stand-in answering `/system.slice` or `/init.scope` for `app.slice`
gave a call that exited 0,
reported the delegated route and left its straggler alive,
where the same call on the real route killed it.
The read found no real mechanism that reports such a path,
and staged it with a stand-in only.

What changed:
an answer whose last component is not the slice that was asked about is refused as a route,
because a unit's own control group always carries the unit's name.
The run then takes the weaker path and says so,
which is what it does on a host with no manager,
and it claims no quiescence it has not shown.

What the round that answered it measured:
both stand-in answers were taken as `app.slice`'s route before the change
(`Route { ... slice_dir: "/sys/fs/cgroup/system.slice" }`) and are refused after it;
and in the same container,
through a stand-in answering `ControlGroup=/system.slice`,
the released binary exited 0 in 146 ms on the delegated route
where the branch took the weaker path in 32 ms and named the path and the reason.

### What this round left

- The read's six Low findings.
  They are answered by the round named in
  `cargo-shadow-alpha-18-read-controls-that-can-fail-and-sentences-that-say-what-runs`,
  not here.
- Which account's slice a reported path belongs to.
  The check compares the unit's name and not the `user-<number>.slice` above it,
  because the manager's slice is spelled differently on different hosts.
  It is in the package document's Open list.
- Whether a refused route should instead be the weaker path plus the notice.
  The record's own sentence says the bound refuses it,
  the round read that as the call stopping,
  and the way not taken is in the same Open list with its cost.

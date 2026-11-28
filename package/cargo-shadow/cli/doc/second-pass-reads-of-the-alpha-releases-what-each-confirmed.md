# Second-pass reads of the alpha releases: what each confirmed

What the second pass over each alpha release of this package confirmed with the
released binaries,
one section per release.

Status:
**unapproved and non-final**.
Written by the coordinating session from the second passes' reports.
Each pass was another agent of the same vendor,
so a second pass and not an independent review,
and each is named by the release it read.
Every figure below is one that pass measured;
the behaviour it confirmed is stated where this package's `README.md` states
it,
and this file holds only what the released binaries themselves showed.

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

# The one synchronous read in the subagent driver

## Rule and configuration

`package/config/oxlint/src/rule/restriction.ts` enables
`no-restricted-syntax/no-sync` as an error,
and this package's lint exits 1 on any warning or error.
The rule has no options and no allow-list,
so turning it off for this package would loosen a lint rule,
which needs prior approval.
One statement therefore carries a disable comment:
the read of `/proc/<pid>/stat` in `lifeOfOneProcessNow`
(`src/reads-of-the-process-folder.ts`),
which is `statSync` and `readFileSync`.

## What the read is for

A `processes` line of a run's feed keeps what the run's own `process` tool said about the processes it started.
The number alone names no process:
the kernel gives a freed number to the next process,
which may work in another folder and another control group.
On 2026-11-14 a review of this package proved that a build which had ended was reported as still running,
from the number a later process held.

So the feed line also carries the start the kernel gave the process,
in clock ticks since the boot,
read at the moment the line is written.
A scan later compares those ticks with the ticks the kernel gives the process at that number now,
and two processes at one number never share a start.
Ticks counted from the boot do not move when the wall clock does:
`/proc/stat`'s `btime` is the current wall clock minus uptime,
so a wall-clock step of 2026-11-15's review would have moved every instant worked out from it.

The record keeps the ticks only when they agree with the start time the tool itself gave,
within the two-sided tolerance beside the constant:
a sample that names another process,
or a tool whose clock differs from this machine's by more than that,
leaves the record with no ticks and its work as work that may still run.

## Why the asynchronous read cannot do this

The line is written while Pi's events arrive,
in the order they arrived:
`record.observe` calls `note` from the connection's event listener,
which is synchronous,
and the driver's ordering of the feed is what makes the feed readable at all.
An awaited read would let the next event's line be written first,
so a record's number could be written beside ticks read after that number had been reused,
which is the defect this field exists to close.
Making the whole event path asynchronous would reorder deliveries from Pi,
which `doc/lint/sequential-awaits.md` records as the design the watch depends on.

## Why it cannot block

An earlier version of this read opened the path and could wait forever on a named pipe written where a process's line belongs,
which a review of 2026-11-15 reproduced with a three-second bound.
The read now asks the path's kind first with `statSync`,
which opens nothing,
and reads only a regular file.
A named pipe, a folder and a path that is not there all answer no birth,
and the record is treated as one that could not take it.

## What it costs

One `stat` and one read of one file of a few hundred bytes,
taken once for each process a `process` tool answer names,
which is a handful per run and never on the watch's own tick.
The file is on `/proc`,
which is memory,
not a disk.

## What was measured

With a process this machine started and ended,
`/proc/<pid>/stat` answered `ENOENT` once the process had gone,
and its start time was the same value on every read while it lived.
A process whose parent had not reaped it still answered,
with the state `Z` and its start time unchanged.

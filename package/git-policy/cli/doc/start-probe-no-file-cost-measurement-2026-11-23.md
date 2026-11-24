# The start probe's cost with no configuration file, measured 2026-11-23

This measurement is unapproved and non-final.
It was taken by an agent from 2026-11-23T20:24-05:00 to 2026-11-24T00:10-05:00
in one throwaway `mvm` libvirt guest (Ubuntu 24.04,
OpenZFS 2.4.4),
destroyed at the end of the round.
Only the owner declares the cost settled.

## What was measured

The probe with no configuration file against the probe with a file naming the
same dataset and the same checkout,
interleaved in the same round.
The harness is a round file,
not part of the branch:
`~/temp/agent/zfs-start-probe-no-file-cost-2026-11-23/keep/no-file-cost-round.ts`
(the two log readers beside it are `analyze-arms.ts` and `analyze-timeline.ts`).

Five arms,
each a fresh `start-probe` of the built handler:

- the file naming both entries (`f1`),
  which is the control arm,
- a byte-identical repeat of that file (`f2`),
  the control of the method,
- no file (`n`),
- a file naming the dataset alone (`c`),
- a file naming the checkout alone (`d`),
- an empty object (`o`).

Every round rotates or shuffles the order,
each arm is one `date +%s%N` reading around the whole process,
and every arm's record said `passed`.

## The spread of an unchanged arm, before any comparison

Thirty rounds of `f1` against `f2` alone,
rotated,
60 runs:

- `f1` median 279.5 ms (range 223.7 to 528.1),
- `f2` median 271.6 ms (range 220.0 to 803.9),
- median per-round ratio 1.028,
  per-round range 0.775 to 1.747.

A single round therefore resolves nothing,
and only the medians of many paired
rounds carry a claim.

## The arms, 40 rounds, before the change

A guest loaded by this round's own builds at the time:

- `f1` 304.9 ms,
  `f2` 316.7 (0.978 of `f1`),
- `n` 377.7 (1.147),
  `c` 357.6 (1.106),
  `d` 359.8 (1.121),
  `o` 378.9 (1.160),
- paired deltas to `f1`:
  `n` +40.5 ms,
  `c` +38.6,
  `d` +38.9,
  `o` +46.8,
  `f2` -6.5.

A quiet guest,
40 rounds,
shuffled,
with both binaries built by the same toolchain from the same tree,
one with the change and one without:

- `old:f1` 114.3 ms,
  `old:f2` 113.4 (1.013 of `f1`),
  `new:f1` 115.6 (1.024),
  `new:f2` 113.8 (0.993),
- `old:n` 129.8 (1.115 of `old:f1`;
  paired delta +13.0 ms,
  38 of 40 rounds above zero),
- `new:n` 129.2 (1.123 of `new:f1`;
  +15.0 ms,
  39 of 40),
- `new:n` against `old:n`:
  1.000,
  paired delta median 0.0 ms (spread -169.5 to +53.7),
  20 of 40 above zero.

An idle guest,
later in the same round,
40 rounds,
shuffled,
on the tree merged from `main`,
with the pre-change arm rebuilt from `main`'s own version of the four touched
files:

- `base:f1` 120.0 ms,
  `base:f2` 120.5 (0.995 of `f1`),
  `base:n` 113.4 (0.974),
- `merged:f1` 117.8,
  `merged:f2` 115.2 (0.995),
  `merged:n` 116.8 (0.976),
- `merged:n` against `base:n`:
  0.995.

So the ratio of the no-file arm to the file-named arm went 1.147,
then 1.115 and 1.123,
then 0.974 and 0.976 across the three phases of one guest,
and the change itself moved it by nothing measurable in either pair.
So the second read of the mount table that the change removed moved the whole
command by no measurable time.

## Where the cost is, by the probe's own steps

One patch,
made only inside the guest's tree copies and never in the branch,
prints `SystemTime::now()` on every probe pipe write (all three writers go
through `probe_pipe_lines::write_all`),
so each step line carries a clock any
process can be compared on.
Twelve rounds,
four arms,
one guest,
the same rounds for every arm,
on the two builds made before `main` was merged in,
whose look-up code is the code under test:

- `old-inst:n`:
  worker phase 33.2 ms,
  of which look-up 12.98,
  holder start
  10.83,
  verify 0.11,
  all other windows 0.9 to 2.0;
- `old-inst:f1`:
  22.39 ms,
  of which look-up 0.08,
  holder start 0.98,
  verify
  12.20;
- `new-inst:n`:
  32.72 ms,
  of which look-up 12.89,
  holder start 10.85,
  verify
  0.12;
- `new-inst:f1`:
  23.02 ms,
  of which verify 13.37.

The no-file arm's cost is two stalls of the same size,
one in its look-up and
one in its holder's start,
while the file-named arm pays one of them at its
mount verify.
The second read the change removed was the difference between the
source windows:
0.11 ms before,
0.02 after.

## What the stall is, and what this round could not settle

Timed inside the guest,
with the same patch method:

- one whole-table read and parse of `/proc/self/mountinfo`:
  median 77.5 us
  (control:
  ten such reads 786.8 us,
  null control 20 ns),
- one path read and parse (`read_mount_facts`):
  87.1 us,
  of which the canonicalize is 15 to 62 us and the read 90 to 174 us,
- the whole checkout look-up:
  75.4 us,
  the dataset look-up:
  86.1 us,
- the first open of the table in the worker:
  93,285 us in the loaded guest and
  12,000 to 17,000 in the quiet one,
  with 3,824 page faults inside that open,
  while the read after it was 146 us and a second open and read in the same
  process 44 and 81 us with no faults.

A plain `python3` process in the same guest reads the same file in 33 to 47 us,
also inside an unshared mount namespace with a tmpfs and a bind mount,
so the
stall is not a property of the file or of the namespace alone.
It is a per-process first touch of the mount table,
it is what the tens of
milliseconds in the arms are,
and this round places it without explaining it.

## The change measured against its control

`no_file_asks_the_mount_table_once` counts the two questions a look-up can ask
the table,
one row listing and one path question,
and requires `(1, 0)` of the
no-file look-up:
one listing and no path question.
Red without the change,
with the second read planted back,
the same control
failed as `left: (1, 1)`,
`right: (1, 0)`
(`~/temp/agent/zfs-start-probe-no-file-cost-2026-11-23/keep/control-red-second-read.log`);
green restored,
17 of 17 tests of the module passed
(`keep/control-green-restored.log`).

A file that names the checkout alone still asks the table once,
for the dataset
only the table holds:
`(0, 1)`,
the second control.

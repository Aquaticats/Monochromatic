# Translation repair history 2026-08-21 to 2026-08-23: how often neither lane wins

The measurement across every ballot-carrying artifact of the path where neither candidate wins and the archive stands,
why the gate is not failing and nothing owns the question,
how the measurement demoted the leading option,
and a confounded observation about terminals changing between runs.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-23: settled-neither measured across every ballot-carrying artifact

`#181` was opened on one slice of one entry and framed around the size note.
The measurement says the note is incidental to it,
and that the path is roughly six times more common than the note-flagged case that revealed it.

Instrument:
`~/temp/agent/181-join.mjs`,
joining `laneSelection` ballots against the archive-side ratio tail per slice,
over every artifacts directory on disk.
Enum names,
ids,
counts and ratios only.

### The base is wider than one entry

43 artifacts across 16 run directories.
Ten settlements carry ballots,
covering eight distinct entries:
`Acheron`,
`dogesir_`,
`gaoyanger`,
`keyword233`,
`lintong`,
`wangzihao980`,
`Weideriche_`,
`Zha_Ke`.
37 contested slices in total.

`Zha_Ke` does carry ballots,
which had been the open question about it.
It contributes four contested slices and no `settled-neither` at all,
so it widens the denominator rather than adding a second coincidence.

### The numbers

    contested slices carrying ballots     37
    verdicts                              lane-won 31, settled-neither 6
    incumbent ratio tail tripped           2
    BOTH                                   1

    of tripped slices, settled-neither    1 of 2
    of settled-neither slices, tripped    1 of 6

`settled-neither` is 6 of 37 contested slices,
about 16%,
and only one of those six involves a flagged incumbent.
The other five carry ordinary archive ratios,
1.31,
1.98,
2.62,
3.31 and 3.40,
all well inside the 0.8 to 10 band.

The outcome is deterministic,
6 of 6:
every `settled-neither` slice records `terminal=no-standing-text`,
`gate=not-asked`,
`shipped=unchanged`.

It is concentrated by entry.
`dogesir_` takes this path on 4 of its 9 contested slices,
44%;
`wangzihao980` on 1 of 5;
`Acheron` on 1 of 4;
the remaining five settlements never do.
A per-entry rate is therefore not the corpus rate,
and one entry can dominate it.

### The gate is not failing; nothing owns the question

Read from source rather than inferred.
`src/consolidate-driver.ts:88` lists `no-standing-text` in `SETTLED_WITHOUT_A_GATE`,
reasoning that these terminals never reached a judge
and that none of them changes on a second asking of the same slate.
`src/corpus-run/would-ship-text.ts:178` says `no-standing-text`
asks what a slate must beat and correctly answers "nothing" on a decline.
`src/corpus-run/artifact-v2-consolidate.ts:58` records that this terminal carries the empty string as its text,
because the contest chose neither lane so nothing stands.

The gate compares a consolidation against standing text,
and on this path there is no consolidation to compare.
So the gap is not in the gate.
It is that no stage owns the question "is the archive itself acceptable?"
when both candidates are rejected.
The lane ballot cannot ask it,
because `LaneChoice` is `repair`,
`translate` or `neither`
and the archive is not a candidate.
The gate does not ask it,
by the design above.
Nothing else looks.

This does not establish that the archive is wrong at any of the six slices,
only that nothing evaluated it.

### What the measurement did to the options

The option that led before the measurement,
asking the gate anyway when the incumbent trips a ratio tail,
addresses 1 of the 6 observed cases and leaves five untouched.
The measurement demoted it to last.
The full re-ranking and its reasons are on `#181`,
which stays a decision item.

### One further observation, confounded, recorded so it is not lost

`Zha_Ke` settled twice,
in `vub-run1-20260821` and `vub171-20260822`,
at identical slice sizes both times.
Slices 0 and 1 swapped terminals between the runs:
c0 `consolidated` then `gate-kept-standing`,
c1 `gate-kept-standing` then `consolidated`.
`gaoyanger` likewise settled differently in `publish-vub-20260822` and `vub-run1-20260821` at the same sizes.

This is NOT evidence of non-determinism.
The pipeline changed between those runs,
`#171`'s naturalness cache among other things,
so run identity and code identity are confounded.
What it does establish is that a gate outcome is not a stable property of an entry across pipeline versions,
so nothing should treat a recorded terminal as one.
Settling it would need the same entry re-settled twice on ONE pipeline version,
which costs a pass and has not been authorized.

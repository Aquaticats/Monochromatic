# Translation repair open decisions: whether the editor calibration overlaps slices by default (question 11)

Question 11 with the 2026-08-26 overlap measurement,
the three options,
the owner's decision of overlap 4,
and the later move of the pass fallback.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 11: whether the editor calibration overlaps slices by default

`TRANSLATION_REPAIR_SLICE_OVERLAP` (`slice-overlap.ts`,
landed `ce5ca2368`) sets how many slices
`editor-calibrate` keeps in flight;
the default is 1.
Measured on 2026-08-26 over the same four slices,
one run per arm:
arm B (overlap 4) took 24.18 min against arm A's 43.18 (overlap 1),
heard 302 of 312
voices against 304,
and the calls themselves cost the same (stream sums 6249 s and 6312 s).
Arm A2
repeated arm A unchanged and took 58.95 min,
so the run-to-run band is 37% of wall clock and comes from
provider speed.
Normalized as wall clock over stream sum:
A 0.41,
A2 0.38,
B 0.23.
The overlap effect is
six bands wide.
The owner's 2026-08-26 answer was "measure the run-to-run band first";
it is measured.

### Options

A.
 **Default `editor-calibrate` to overlap 4,
keep the dial.**
    -   For:
halves the wall clock of the one command a session waits on most,
on evidence that clears the
        band six times over;
voices heard moved by 2 of 312,
inside what provider speed alone moves (A2 lost
        8 with no dial change).
    -   Against:
a four-slice standing is noise either way (GLM-5.2 swung 52.2% to 26.7% to 10.0% between
        identical runs),
so faster standings are faster noise until the sample grows;
`producer-calibrate`
        has no dial yet,
so the two calibrations would run under different defaults.
B.
 **Keep the default at 1;
document `TRANSLATION_REPAIR_SLICE_OVERLAP=4` as the recommended launch.**
    -   For:
nothing changes for a reader of an old log;
the pass drivers (question 3,
`#261`) still run at
        1,
so calibration and pass stay alike until the pass is measured.
    -   Against:
the recommendation lives in a doc a reader may not open;
every default run pays 19 minutes
        for nothing measurable.
C.
 **Move both calibrations and the pass to overlap 4 together,
after `#261` is measured.**
    -   For:
one story for every driver.
    -   Against:
couples a decided measurement to an undone one for no gain in evidence.

### Ranking

A > B > C.
A over B because the effect is measured and six bands wide,
and the standing-noise objection
applies equally at either overlap;
B over C because C delays a decided question behind an undecided one.

### DECIDED 2026-08-26: A, by the owner

`CALIBRATION_OVERLAP = 4` in `slice-overlap.ts`;
the dial's fallback is now the caller's,
so the pass keeps
1 until `#261`.
Record:
`doc/decision/translation-repair-calibration-overlap.md`.

`#261` was measured on four matched pass pairs on 2026-08-27 and 2026-08-28,
and on 2026-09-06 the pass fallback moved to 4 on that reading,
flagged for the owner's veto.
Record:
`doc/decision/translation-repair-pass-overlap.md`.

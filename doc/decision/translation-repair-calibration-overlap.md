# The editor calibration runs four slices at once, under a 300000 ms straggler window

Decided by the owner on 2026-08-26,
on two `AskUserQuestion` calls put after the measurements they had asked
for (`doc/planning/translation-repair-open-decisions.md`,
questions 11 and 12).
Both answers took the
recommended option.

## What was decided

-   `editor-calibrate` keeps four slices in flight when `TRANSLATION_REPAIR_SLICE_OVERLAP` is unset
    (`CALIBRATION_OVERLAP` in `package/translation-repair/slice-cache/src/corpus-run/slice-overlap.ts`).
    The variable still overrides it in either direction;
    `1` reproduces the sequential driver.
-   `editor-calibrate` runs its stage rounds under a 300000 ms straggler window when
    `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` is unset (`CALIBRATION_STRAGGLER_GRACE_MS` in
    `src/grace-override.ts`,
    applied by `adoptCalibrationGrace` through the same variable a launch can set).
    The variable still overrides it.
-   The corpus pass is unchanged:
    one slice at a time and the built-in 180000 ms window
    (`STRAGGLER_GRACE_MS` in `src/stage-round.ts`),
    until later work builds the overlap dial into the pass
    drivers and measures it there.
    The two settings move together or not at all,
    because the window's price
    is the wait and overlap is what fills the wait.

## The evidence, all on the same four bench slices and the same build

-   Arm A,
    overlap 1 at 180000 ms:
    43.18 min,
    304 of 312 voices heard,
    6 cut.
-   Arm B,
    overlap 4 at 180000 ms:
    24.18 min,
    302 of 312 heard,
    7 cut,
    stream sum within 1% of A's.
-   Arm C,
    overlap 1 at 300000 ms:
    53.87 min,
    306 of 311 heard,
    4 cut.
-   Arm A2,
    arm A repeated unchanged:
    58.95 min,
    312 of 320 heard,
    8 cut,
    stream sum 9294 s against A's
    6312 s.
    This is the run-to-run band:
    37% of wall clock,
    from provider speed alone.
-   Arm D,
    overlap 4 at 300000 ms:
    29.31 min,
    318 of 320 heard,
    2 cut,
    stream sum 7591 s.

Normalized as wall clock over stream sum:
A 0.41,
A2 0.38,
C 0.43,
B 0.23,
D 0.23.
The overlap effect is six
bands wide;
the window's cost at overlap 1 is inside the band;
under overlap the window costs nothing
measurable and D is the arm with the fewest cut voices of the five.

## What this does not decide

-   Whether the corpus pass overlaps slices.
    That is the overlap-dial work,
    measured on matched pass runs read the same
    normalized way,
    since single-run wall clock moves 37% on provider speed alone.
-   Whether `producer-calibrate` gets the same dial.
    It does,
    as follow-up work,
    so the two calibrations run
    under one default;
    until then it runs one slice at a time.
-   The four-slice standing itself,
    which stays noise at either overlap (a model's share swung from 52.2% to
    26.7% to 10.0% between identical runs);
    faster runs make more slices affordable,
    which is the remedy.

## Where it is recorded

`doc/decision/translation-repair-straggler-grace.md` (addendum of 2026-08-26),
the open-decisions register
(questions 11 and 12,
marked decided),
the package README ("Deciding who fills a seat"),
and the handover.

## Addendum 2026-09-06: the pass followed

Pass overlap was measured on four matched pass pairs on 2026-08-27 and 2026-08-28,
and the pass fallback moved to four on that reading:
`doc/decision/translation-repair-pass-overlap.md`.
The window stayed where the owner put it on 2026-09-03.

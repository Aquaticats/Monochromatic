# Translation repair open decisions: whether the straggler window moves from 180000 ms (question 12)

Question 12 with the arms that measured a longer window,
arm D read the same evening,
and the owner's decision of 300000 ms for the calibration.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 12: whether the straggler window moves from 180000 ms

`TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` (`grace-override.ts`,
landed `4c070f729`) overrides the window for
one run.
Arm C (300000 ms,
overlap 1) took 53.87 min against arm A's 43.18 and bought back 2 of 6 cut
voices,
all `qwen3.8-max`.
Arm A2 then showed the band:
58.95 min for an unchanged arm A,
8 cut voices
instead of 6.
Normalized,
A 0.41,
A2 0.38,
C 0.43:
the window's cost sits inside the band,
and so does its
recovery.
Arm D (overlap 4 at 300000 ms) is running;
under overlap the wait a longer window adds is what
overlap fills,
so D is the arm that could show a cheap window.

### Options

A.
 **Keep 180000 ms,
keep the dial,
read arm D before reopening.**
    -   For:
nothing about the longer window is shown at this scale;
the decision doc's own derivation
        still stands;
the dial lets any later trial run without a build.
    -   Against:
6 to 8 cut voices per four-slice calibration persist,
all on one model.
B.
 **Move to 300000 ms on arm C's 2-of-6.**
    -   For:
two voices came back on one run.
    -   Against:
the band swallows both the cost and the recovery;
A2 lost 8 voices with no window change,
        so 2 of 6 is not distinguishable from noise on single runs.
C.
 **Run interleaved repeats (A,
C,
A,
C in one sitting) and decide on the normalized pair.**
    -   For:
the only design that resolves a difference smaller than the band.
    -   Against:
about four hours of provider time for a dial whose largest plausible effect is two voices
        per calibration,
all from one model that `#121` already names as the window's single payer.

### Arm D, read the same evening

Overlap 4 at 300000 ms over the same four slices:
29.31 min against arm B's 24.18,
with slower calls (stream
sum 7591 s against 6249 s),
so the normalized figure is 0.23 for both.
Voices heard 318 of 320 against B's
302 of 312;
2 cut against B's 7 and A2's 8.
Time in grace 4591 s against 2819 s,
filled by the other slices
in flight.
Under overlap,
the longer window is free and it is the arm with the fewest cut voices of the five.

D.
 **Move the window to 300000 ms together with overlap 4 as the calibration default (question 11,
A).**
    -   For:
the one arm that pairs them shows no measurable wall-clock cost and the lowest voice loss of
        any arm;
the window's price was always the wait,
and overlap is what fills the wait.
    -   Against:
one run;
the voice-loss difference (2 against 7) is larger than the band's spread of cuts
        (4 to 8 across single-overlap arms) but not by much;
at overlap 1 the price returns,
so the two
        settings must move together or not at all.

### Ranking, revised

D > A > C > B.
D over A because it is measured where A was waiting for the measurement;
A over C and C over
B as before.
D is conditional on question 11 resolving to overlap 4;
at overlap 1 the window stays at
180000 ms.

### DECIDED 2026-08-26: D, by the owner

`CALIBRATION_STRAGGLER_GRACE_MS = 300_000` in `grace-override.ts`,
adopted by the calibration through the
variable;
the built-in `STRAGGLER_GRACE_MS` stays 180000 for the pass until `#261`.
Record:
`doc/decision/translation-repair-calibration-overlap.md`.

# Translation repair history 2026-08-07 to 2026-08-12: the state at the 2026-08-09 compaction: the sheet awaiting its grade, what landed, a withdrawn rate and the open tasks

The state at the 2026-08-09 compaction:
the round-three gate sheet drawn and waiting for the user's grade,
with the files in the order to open them,
the pass that was running,
what landed in that session,
the corroboration-rate claim withdrawn as unsupported,
and the open tasks in the order they unblock.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## State at the 2026-08-09 compaction

Branch `translation-repair-rebased`,
447 commits ahead of `main`,
nothing
unpushed.
Working tree carries only foreign drift (three plugin bundle `.mjs`,
the
IntelliJ jar,
untracked `.idea/.name`);
leave it alone.

### The one thing waiting on the user

Round three's gate sheet is DRAWN and awaiting its human grade.
The draw is spent and must not be repeated:
 `resolveSheetPath` refuses a final path that exists,
 and that refusal is the only thing protecting hours of grading that nothing
 else reproduces.
Verified rather than trusted:
 a second `--final` run refused with `GradedSheetExistsError` and left all three
 files byte-identical.

```text
grading-sheet-milestone-three-precision-round-three.md    ← grade this first
repair-sheet-milestone-three-precision-round-three.md     ← only after
sample-manifest-milestone-three-precision-round-three.json
pre-grades-milestone-three-precision-round-three.json     ← do NOT read first
```

Follow `doc/runbook/translation-repair-round-three-grading.md`,
which is
current.
NOTHING IN THIS SESSION HAS READ THE DETECTION SHEET,
so the blind pre-grade
 comparison is still clean.
Keep it that way.

### What is running

Corpus pass run 012,
logging to `pass8-run-012.log`.
It is the FIRST pass whose artifacts carry the naturalness audit,
which is why
 restarting it mattered.
Read its `DONE` line rather than its exit code:
 a pass exits 0 on its soft budget with most of the corpus unprocessed,
and
 elapsed always overruns the budget because the budget stops it STARTING
 entries while those in flight finish.

### Landed this session

-   The gate sheet drawn at 18 entries,
    5/7/5 contributing,
    pool 740,
    after the
     user chose to draw rather than accumulate further.
    Per-entry slot counts confirmed the round-robin empirically:
     `Dethelly` (198 candidates) and `Jennife80677612` (12) both got 3 slots.
-   Blind pre-grades for all 50,
    49 scored and 1 handed over.
-   Three defects that produced confident wrong numbers rather than failures:
     the probe join keyed ownership off region issue lists (many-to-one,
    so a
     graded position could receive another record's verdict);
     the majority rule compared CLAIM counts against a roster headcount;
     and an unguarded top-level `await main()` meant importing the LIBRARY ran a
     corpus scan.
-   The naturalness lane is audited (task 58),
    validated live against injected
     damage before being trusted.
-   Every item of task 62 (draw and probe-scoring durability).

### The correction worth carrying forward

`doc/decision/introduced-defect-probe-gating.md` claimed the corroboration rate
 was "roughly 1 in 120 and has stayed there rather than climbing."
That was written on TWO events and was not supportable;
 the interval around 2 in 246 comfortably contains the 7 in 412 measured later.
The sentence is withdrawn rather than updated.
At 38 entries it reads 8 in 508,
about 1.6%,
spread one apiece across distinct
 entries.
The deferral still stands,
but on "low and unvalidated" rather than on a column
 that barely moves.

### Open, in the order they unblock

-   #60 needs the human grades.
    When scoring,
    subtract `refinedJoined` FIRST:
    10 of the 50 positions have a
     probe verdict about wording the naturalness lane replaced,
    so those rows
     compare two different texts.
-   #48 closes with #60,
    since the agreement rate needs the same grades.
-   #63 is the deferred design work,
    now unblocked by the draw being spent:
     bind sheets to an exact draw by digest,
    and stop the telemetry reader
     returning claims with empty quote fields.
-   #51 and #31 are untouched and need quota,
    so they contend with the pass.

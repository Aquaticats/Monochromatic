# Translation repair history 2026-08-07 to 2026-08-12: the round-three gate sheet drawn and pre-graded blind

The tension between checking that a sheet renders and keeping later pre-grades blind,
the drawing of the round-three gate sheet at eighteen settled entries,
and the blind pre-grades recorded for it.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Verifying a sheet renders is in tension with pre-grading it blind

READING A GRADING SHEET CONTAMINATES ANY LATER BLIND PRE-GRADE OF THE ITEMS
READ.
This session printed the first 48 lines of the round-three preliminary
detection sheet to confirm it was well-formed,
which put items 1 through 5,
their claims,
and their source and target quotes into an agent context.
Those
five can no longer be pre-graded blind by that session.

WHY IT MATTERS BEYOND ONE SESSION:
`scoreGradeAgreement` weights every row the
same,
so a handful of sighted rows inflate the agreement figure that the
calibration task exists to produce,
and nothing in the artifact records which
rows were sighted.
The damage is invisible in the output.

HOW TO KEEP BOTH:
verify structure without reading claims.
Count `###` item
headings,
check the header block and the `[ ]` slots,
confirm the banner and the
corpus pin,
and stop there.
If item text must be inspected,
do it in a session
that will not produce the pre-grades,
or record the sighted indices alongside
the pre-grades so they can be excluded.

THIS PARTICULAR CONTAMINATION IS MOSTLY MOOT,
because the preliminary draw is
not the gate sheet and the final draw shifts as the pool grows,
so pre-grades
keyed to preliminary indices do not transfer anyway.
Do not let that specific
reprieve hide the general rule.

## The round-three gate sheet is drawn

Run 008 ended on its soft budget:

```text
SOFT budget reached after 48328009ms; not starting new entries
DONE processed=9 of pending=83; artifacts=18/92 elapsed=48328009ms
```

Note the elapsed time exceeds the 43200000ms soft budget.
The budget stops the driver STARTING entries;
 the ones already in flight finish,
so a pass always overruns by roughly its slowest surviving entry.

The gate sheet was drawn at 18 settled entries,
 with the pass stopped so nothing could be added mid-read,
 and the user chose that timing over accumulating further:

```text
SAMPLE final=true seed=milestone-three-precision-round-three pool=740 drawn=50 unrecordedRepairs=0 unrecordedInPool=0
```

Contributing entries were 5 small,
7 medium,
5 large.
The slot distribution is the thing to look at,
 because it settles the concentration question empirically rather than by
 argument:

```text
Arita:4 Futajuhuacha:3 Chinatsu_Suzuki:3 Dethelly:3 Jennife80677612:3
```

`Dethelly` brought 198 candidates and `Jennife80677612` brought 12,
 and both received 3 slots.
The two entries holding 81% of the large pool took 6 of its 16 slots.
Round-robin does what it claims.

The one-shot guard was then verified rather than trusted:
 a second `--final` run refused with `GradedSheetExistsError`,
 and all three files were byte-identical afterwards.

## What is true right now

-   The gate sheet,
    repair sheet,
    and manifest exist under the round-three seed
     and must not be redrawn.
    Nothing in this session has READ the detection sheet,
    so #48's blind
     pre-grade path is still clean.
    Keep it that way:
     do not `cat`,
    `head`,
    or `sed` it.
-   Corpus pass run 009 is running,
    logging to `pass8-run-009.log`.
    It cannot affect the drawn sheet,
    which is already written,
     and its entries serve round four,
    recall (#51),
    and the naturalness probe
     (#58).
-   `score-probe` reads 18 entries,
    740 shipped records,
    246 regions,
     `majorityIntroduced=2`.
    The join runs and reports `joined=50` against the preliminary pair;
     it has not been run against the FINAL pair because that needs human grades.

## Round-three blind pre-grades are recorded

Written to `pre-grades-milestone-three-precision-round-three.json` in the runs
directory,
 which is where `score-agreement` looks for them by seed.
50 items,
indices 1 to 50 complete,
49 scored and 1 left `unscored` and handed
 over as genuinely contested.
`parsePreGrades` accepts the file.

THE VERDICTS ARE NOT REPRODUCED HERE,
and were not reported to the user when
 they were written.
The decision recorded under "PRE-GRADES STAY IN THEIR OWN FILE" is that showing
 the agent's grade anchors the human toward agreeing,
and the same sheet produces the milestone gate number,
so the calibration would be bought by corrupting the measurement it calibrates
 against.
Naming which items the agent called false positives does that just as
 effectively as printing them on the sheet.
The per-item reasoning lives in the `note` field of each pre-grade,
 which stays outside git because it quotes corpus text.

Method worth repeating next round:
 the sheet deliberately shows no source anchor for addition-class claims,
 because an addition points at nothing in the original,
so an addition claim cannot be graded from the sheet alone.
For those the corpus was read directly at the pinned commit
 (`/var/home/user/one-among-us/data`,
read-only,
never committed),
 and several claims resolved cleanly in one direction or the other on evidence
 the sheet could not carry.
Grades reached that way are marked `VERIFIED AGAINST SOURCE` in their note.

This creates a real asymmetry to disclose at scoring time:
 the agent graded some items with more information than the sheet shows.
Disagreement on those items may reflect that asymmetry rather than judgment,
 and the agreement rate should be read with the marked items identified.

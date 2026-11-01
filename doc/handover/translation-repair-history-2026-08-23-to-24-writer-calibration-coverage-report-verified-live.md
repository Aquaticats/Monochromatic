# Translation repair history 2026-08-23 to 2026-08-24: the writer calibration's coverage report verified live

The provenance note of 2026-08-25 that precedes the sections moved unedited from the working handover,
then a three-slice writer calibration that exercised the new silence report for the first time,
naming the seats that wrote nothing and the retry path it also exercised.
The note is dated 2026-08-25 because that is the day the sections left the working handover;
the work in this file is dated 2026-08-24,
which is why the note sits in a group that stops at the 24th.
The silence report that its first paragraph says both calibrations gained the same day is recorded in
`translation-repair-history-2026-08-24-to-25-editor-calibration-built-naturalness-lane-never-run-and-writer-silent-model-list.md`,
in the 2026-08-24 to 2026-08-25 group.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Aged out of the working handover on 2026-08-25

Moved verbatim from `doc/handover/translation-repair.md` when it reached its two-thousand-line cap.
Nothing here was edited on the way across.

## The writer calibration's coverage report, verified live (2026-08-24)

Both calibrations gained a silence report earlier the same day
and neither new reporting path had ever run.
The finished 40-round writer run and the 29-round editor run
were both settled by the binary that preceded it.

A 3-slice writer calibration into `~/temp/agent/producer-coverage-vub-2026-08-24` closed that gap.
It printed:

```text
  WROTE NOTHING AT ALL: qwen3.8-max, minimax-m3, gemma-4-26b-a4b-it, deepseek-v4-pro-0813,
  deepseek-v4-flash-0731. No candidate of theirs reached any slate, so the table covers 5 of 10 seats.
```

All five Hyper-only seats named,
the denominator right,
and no spurious "wrote and was never voted on" group,
since all five Synthetic writers drew ballots.
Charm Hyper was dry throughout,
so the run also exercised the retry path:
`select: retry round 1 for 6 lost voices`,
three times,
before settling the slice.

The standing itself settles nothing at 3 rounds and says so in its own last line.
That was not what the run was for.

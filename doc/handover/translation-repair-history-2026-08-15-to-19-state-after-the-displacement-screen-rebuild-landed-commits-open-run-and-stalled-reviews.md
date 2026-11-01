# Translation repair history 2026-08-15 to 2026-08-19: the state after the displacement screen was rebuilt: what landed, the run still needed and the reviews presumed stalled

What landed after the rebuild,
the production-selection run that the open question still needs and how to read it per class,
the items left open and untouched,
the two reviews presumed stalled,
and the judgement carried forward about reading a slice in isolation.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The state after the displacement screen was rebuilt

### Where this leaves the night, and what is next

WHAT LANDED,
all committed and pushed on `translation-repair-rebased`:

- `add309333` the wider-window result,
  and two published numbers corrected.
- `2f7b0c1f9` tests for the fidelity window and the sheet it renders,
  with the
  narrow-arm guard shown to fail before it was trusted.
- `84fa7a0f7` the displacement screen rebuilt into a classification.
- `687bf02d6` the transcription screen over relocation candidates.
- `4813e5c88` and `ee599061d` the corrected numbers in the handover and in
  Question 5.
- `636dadf1b` `damageDetail`,
  so the next paired comparison reads a field.

THE ONE THING QUESTION 5 STILL NEEDS is `#108`,
created tonight and blocked on
nothing now:
run production selection over the flagged slices,
narrow and wide,
and read the REPLACEMENT rate per class.
The wide fidelity arm answered a
different question.
It said the TRIAL's declines came from the window,
on a
fixture with known ground truth.
It did not say what the lane does with fresh
candidates on those slices,
and Question 5's option E turns on that second
number.
Do not let one stand in for the other;
the handover section "The wider
window" says so and so does the ticket.

READ THAT RUN PER CLASS.
On relocation candidates the rate should FALL if
displacement is driving replacement.
On untranslated slices it should stay high
and it SHOULD,
since there is nothing there to preserve.
On target-only slices
nobody knows,
and that is the interesting one,
because it is Question 2's subject
arriving through a different door.
Other imbalances are the control.

STILL OPEN AND UNTOUCHED TONIGHT:
`#84`'s remaining items (self-preference rate,
production position bias,
a wider sample,
the hard fluent-paraphrase case),
and
the queue behind `#106`.
Two `sol` reviews launched yesterday evening are still
running,
`bt8g6brhj` on the locator rewrite and `b64uex7px` on fidelity
measurement.

PRESUMED STALLED,
AND THE EARLIER READING OF THEM WAS WRONG.
This document said
an empty output file means thinking rather than hung,
which was calibrated on
runs that eventually flushed.
Measured since:
`bt8g6brhj` has been alive 4 hours
32 minutes and `b64uex7px` 2 hours 58 minutes,
each with 00:00:00 of CPU time,
against sibling calls the same night that finished in minutes.
Treat both as
stalled.
Neither is killed,
since nothing authorised that,
but nothing should
wait on them.
If either eventually lands,
its findings describe PRE-REBUILD
source and must be re-checked against the current files before any of it is
acted on;
`b64uex7px`'s prompt predates `damageDetail`,
the wide arm and the
window plumbing,
so its findings are stale regardless of what it says.

A third review,
on the displacement instrument,
came back and its findings are
recorded above under what was adopted and what was refused.

ONE JUDGEMENT TO CARRY FORWARD,
since it cost the most time to learn tonight.
Both
of the design errors the acceptance gate caught were the same error:
deciding a
slice's class from the slice alone,
when the thing that separates the classes
sits in the NEIGHBOUR.
`target-only` and relocation share a shape;
a short donor
and a noise slice share a shape.
Every time the fix was to stop reading the slice
in isolation,
which is also,
one level up,
exactly what `#107` says about the
judges.

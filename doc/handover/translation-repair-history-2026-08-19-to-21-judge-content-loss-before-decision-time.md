# Translation repair history 2026-08-19 to 2026-08-21: whether the judge can notice content loss

Two findings that together raise the question of whether anything catches content loss at decision time,
and the pre-registered reading of the fidelity probe built to answer it.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Two findings that are worse together than apart

Noted 2026-08-20 while looking for a lane discriminator.

FIRST,
from `#107`'s work:
the repair lane deletes the archive's specifics.
Ninety-seven of two hundred and sixteen distinctive words on `saurikissa`,
and two hundred and fifty-five of one thousand two hundred and forty-eight
pooled over eleven pre-window entries.

SECOND,
from `doc/audit/incumbent-almost-never-wins-a-contest.md`:
on CONTESTED slices the translate judge replaces the archive thirty-seven times in
thirty-nine,
94.9%.

Each was recorded on its own and neither is new.
Put together they name one question:
the component that almost always prefers a fresh rendering
is the same component that would have to notice a rendering saying LESS.

If it cannot,
then nothing in the pipeline catches content loss AT DECISION TIME.
The content-survival check catches it at assembly,
after the choice is made,
and reports rather than acts.

THIS IS BIGGER THAN THE LANE QUESTION IT CAME FROM.
`#130` needs it to decide whether a per-slice judge between the lanes is viable,
but the answer bears on every slice the translate lane has ever replaced.

`#84` BUILT THE HARNESS FOR THIS AND ITS RESULT IS NOT RECORDED ANYWHERE.
`judge-fidelity-probe` constructs a damaged twin of a real archive slice,
word for word identical except one deletion or one spliced insertion,
and runs both directions and both ballot positions,
so a status-quo reflex scores half and only a judge that reads scores all four.
Two defect kinds,
because a deletion makes the faithful candidate the LONGER one
and an insertion makes it the SHORTER one,
so passing both is the only way to show it is not just preferring length.

Running it is the next thing,
and its reading is pre-registered:

```text
near 100%                     the judge reads for coverage; per-slice option viable
near 50%                      it keeps what it is given; the option is dead
deletion high, insertion low   it prefers more text rather than reading
```

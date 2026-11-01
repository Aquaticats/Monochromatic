# Translation repair: the archive block review reads one transcript per corroborated picture

Decision of 2026-09-16,
delegated by the owner the same day:
"I'll go with whatever you recommend."
Landed in `36b79bfa0`.

## What was decided

The archive block review's picture support sends one transcript per corroborated picture,
the one the other readers carry most,
instead of every corroborating reader's transcript.
`mostCarriedReading` (`src/most-carried-reading.ts`) picks the reading whose mean trigram overlap with every other
reading is highest,
using the same `trigramOverlap` the corroboration itself uses;
on a tie it takes the longer text,
because the overlap divides by the smaller side and a fuller transcript the others vouch for scores the same as
the shorter one it extends;
on a tie of length it keeps the earliest.
The support line reads
`CORROBORATED PICTURE SOURCE SUPPORT <asset> (<n> readers agree; the transcript the others carry most)`
followed by that one text.
No other stage reads the picture support this way;
the readers' corroboration and the readings the cache keeps are unchanged.

## Why

`Mio13` (2026-09-16,
frozen `cbedea357`) reviewed the first chat translation against six transcripts of its screenshots.
Seven of twelve seats spent their whole completion cap reasoning and sent no content:
Qwen3.8-27B 20,894 completion tokens (76,861 reasoning characters),
GLM-5.3-Flash 18,316,
minimax-m3 10,822,
deepseek-v4-flash-0731 16,543,
deepseek-v4-pro-0813 9,128,
glm-5.3 22,067,
deepseek-v4.1-flash 13,082.
The block before it,
on a prompt of the same size (about 9,000 tokens against about 7,000),
drew 640 to 1,268 completion tokens from the same seats.
Under class thirty-one (`db090b955`) the entry no longer stops on it,
but the block ships as the archive wrote it while the four heard reviewers named errors in it
(a "musculine" typo,
an inverted parenthetical,
dialogue the source does not carry,
mis-attributed speakers).

Three levers were put to the owner,
ranked:

- One transcript per picture (taken).
  Costs nothing extra;
  changes what evidence the reviewers see,
  from six readings that disagree in detail to the one the others agree with most.
- Re-ask a cap-cut reply once at twice its measured cap in the recovery round (rejected).
  Buys the review with the quota the caps exist to keep:
  up to a cap more per seat per block on Synthetic's weekly allowance and Hyper's daily limit,
  which the owner's instruction of 2026-09-09 ("do everything in our power to NOT bleed") protects.
- Leave it (rejected).
  Leaves the chat translations' known errors on the page.

## What it does not claim

That the one transcript ends the overrun is a hypothesis until measured:
`Mio14` (frozen `db090b955`,
six transcripts) is the baseline,
and the next Mio on `36b79bfa0` or later shows whether the same seven seats answer within cap on the same block.
If they do not,
the lever is spent and the doubled re-ask cap is the next question for the owner.
The corroboration verdict itself is still taken between the first two readings;
this decision touches only what the archive review is shown.

## Measured, 2026-09-16

Mio14 (six transcripts,
killed after its review) against Mio15 (one transcript),
read off both logs at the archive block review
(the pass log's "Mio15 read" heading dated 2026-09-16 06:46 UTC):
the four ordinary blocks on Mio15 cost 5 review cap cuts against 15 over Mio14's six blocks,
and every one reached selection with 6 or 7 heard reviews;
the first chat translation still fired the class thirty-one warning,
7 of 12 seats unreadable against Mio14's 6.
The decision stands for what it bought;
the doubled re-ask cap is the open question for the block it did not reach.

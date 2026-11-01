# OpenRouter and the corpus passes: the causes of the rotation 21 findings (2026-10-21)

The tracing of ten findings from the reads of `zheermao21` and `CuspariaKLSY21` to their causes,
their classes,
and what fixed them.
The tracing read code and logs at `3637592bd4` and ran nothing;
the fixes were then made and tested in a worktree on `main`,
and they are not in the build the two passes ran.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.
The reads are in
[the overview of the rotation restart](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md),
where each page file says "cause not yet traced" for what this file settles.

## Verdicts

- Finding 1,
  "was given to her" for 起的 (`CuspariaKLSY21` slice 1):
  no class.
  The translate lane's own wording,
  picked whole by the lane contest on tense.
  The archive said "coined",
  so the repair panel filed no issue about a giver and no accepted claim existed to carry.
  The polish gate's tie on this slice is the occasion of the ruling "Polished text wins ties" (ledger B730).
- Finding 2,
  吃不上饭 (`CuspariaKLSY21` slice 0):
  class A1,
  narrowed to the polish gate.
  Fixed by ledger B731.
  A possible class D (the "generic term" sentence of the house policy) stays held.
- Finding 3,
  已经:
  noise.
  No stage held the word;
  both lanes and the archive lack it.
- Finding 4,
  一直,
  观望,
  二次元:
  no class.
  The translate lane weakened the words and the lane contest picked that lane whole,
  as designed.
  Whether a consolidation writer carried 观望 is unverified.
- Finding 5,
  the opening quote (`zheermao21` slice 0):
  no class.
  The minimum selection weight of two worked as written.
- Finding 6,
  the email reply:
  class A2,
  the whole-revision pick of the archive block review.
  Addressed by ledger B712 (see the section "Finding 6 and class A2").
- Finding 7,
  two `U+2014` characters:
  not a defect.
- Finding 8,
  the probe closed at 1 of 3:
  the cache refusal is as designed,
  and its log line was wrong (class B).
  Fixed by ledger B733.
- Finding 9,
  0.372 Wh:
  no class.
  An in-flight hold.
- Finding 10,
  `withheld=` against `unserved=`:
  no class.
  Two different rules.

## Finding 2 and class A1: what the stored evidence shows

The slice is slice 0 of `CuspariaKLSY21` (the brief that asked for this called it slice 2,
which is the number of the finding).
In the stored artifact chunk 0 holds one adjudicated issue,
status `accepted`,
settled severity `minor`,
with two `accuracy/mistranslation` claims,
each supported 4 and unsupported 0,
and the issue is in the chunk's resolved issues.
The polish gate of that slice heard polished 3 and base 1 between the translate wording ("could not have a good meal")
and the rewording ("couldn't get a proper meal").
No dispute note existed,
because a dispute names additions at any severity and other accuracy claims only at major or worse,
so the sheet showed no claim.
So the answer to whether the slice had an accepted repair claim the gate sheet did not show is yes.
The fix gives the polish gate every accepted claim at the severity the panel settled,
as claims to check against the original (ledger B731).
Whether the judges then decide differently is not measured;
it needs a rerun measured against run-to-run spread.

## Findings 1 and 5: the tie rules

- The polish gate tied 2 to 2 on 起的 and shipped the base,
  because the gate needed a strict lead and shipped the base for anything else.
  The owner ruled on 2026-10-21,
  "Polished text wins ties":
  base,
  base,
  polished,
  polished and base,
  polished ship the polished text,
  below the minimum ballot count nothing changes,
  and the gate logs the tie and the rule (ledger B730).
- The select of `zheermao21` slice 0 declined at weight 1.5 against a minimum of 2.00,
  and its challenge round tied 1.5 to 1.5 so the standing stayed.
  The rule is documented and worked as written;
  all four reachable voices also wrote candidates,
  so none was wholly disinterested,
  which is the stakeholder arithmetic of the rule.
  The tie rules of the slate (run off once when the standing drew no ballot,
  then the standing) are unchanged.
  Whether the minimum should treat stakeholder votes differently on a four-voice bench is a question for the owner,
  not a defect.

## Finding 6 and class A2: the email

The archive block review of the email chose one whole revision by five of six ballots,
and the revision it did not choose carried most of the lines the other lacked
(D12 to D19 of the page record).
The build of the pass did not offer a composite.
The code that followed this build has it:
`compositeOfDisjointRevisions` in `archive/src/archive-composite-revision.ts`
(ledger B712,
with B716 for the assessments it carries to the judges),
called from `archive/src/archive-block-review-stage.ts`,
offers the lines of revisions that changed disjoint lines of a block of the same line count as one more candidate,
after the revisions and before the block as the archive wrote it.
`archive-composite-revision.unit.test.ts` drives the real stage with two revisions of disjoint lines and checks that
the composite is on the slate and wins when the judges name it.
That is the shape of the finding,
so the composite addresses class A2.
It is a trade and not a pure gain:
the revision that won also kept out the other's wrong gloss "(Antares)" and invented words,
and a composite can carry both a good line and a bad one of its contributors.
The anchor test the read looked at is not the cause:
it filters `source-supported` replies,
which never become candidates.
The line-break folding of a retention quote is still a behaviour change awaiting a decision.

## Finding 7: the two dashes, and the withdrawal of D21

The source has a full stop after 聊了许久 and a comma before 试图多挽留她一会儿,
and the shipped page carries the dashes at exactly those two breaks.
By the owner's dash ruling,
as the tracing gives it (a page's dash is a defect only where its source has no matching break or pause),
this is not a defect.
The departure D21 of
[the page of zheermao21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-page.md)
is withdrawn,
and its text stands there with a note.
A preference for commas would be an owner choice;
no code is at fault.

## Finding 8: the log line

On `zheermao21` slice 3 the log said the slice "ships unchanged and is NOT cached" after the introduced-defect probe closed
at 1 of 3.
The slice did change:
the repair result lists it among its changed slices,
the chunk records an accuracy patch selected,
and its repaired text reached the page.
The refusal to cache is the designed fallback of a probe that runs in shadow mode.
The line now says that the settlement is not cached,
that a later run buys the slice again,
and separately whether the slice's text changed (ledger B733).

## Finding 9: the 0.372 Wh gap

The meter takes the remaining energy of the day as the day's allowance less the larger of the ledger's spend and the
account's,
less the energy reserved for calls in flight (`neuralwatt-day-energy.ts`),
and the reserve is each call's estimate (`neuralwatt-ledger.ts`).
So the printed figure subtracts the holds of calls in flight as well.
Over the 25 `METERS` lines of `zheermao21` the difference between the meter and the account was 0.000 on most lines and
0.076,
0.152,
1.118,
1.117 and 0.372 on others.
The estimates are 0.0758 Wh for `deepseek-v4.1-flash` and 0.3725 Wh for `glm-5.3-flash`
(`neuralwatt-estimate.ts`),
and the gaps are exact multiples of them:
one,
two and three holds.
The gap of 0.372 Wh is one `glm-5.3-flash` hold,
and a `glm-5.3-flash` stream for consolidation slice 13 began at about the last `METERS` line (06:01:35 UTC)
and completed at 06:01:43 UTC.
That a call was in flight there fits the numbers;
that it is the cause is a guess supported by the exact multiples.
It is not drift,
and no tolerance is documented because none is needed.

## Finding 10: `withheld=` against `unserved=`

Both fields come from `judgeSeatsFor` in `run/src/corpus-run/run-seats.ts`.
`withheld=` names a model that a bench dropped on purpose:
`hf:moonshotai/Kimi-K3` is in `OPENROUTER_WITHHELD`
(served by OpenRouter and not bought,
by the owner's cost decision of 2026-09-03),
so with Synthetic dry its only wet route is a provider the run does not buy from.
`unserved=` names a model that stays on a bench but that no wet provider serves,
and is lost at call time as unreachable (the owner's rule of 2026-09-09).
`hf:Qwen/Qwen3.8-27B` is not in `OPENROUTER_WITHHELD`,
so it stays seated and is unserved.
The two words are different facts,
removed on purpose against kept and unreachable,
and the field documents of `JudgeSeats` already say so.
That Qwen has no OpenRouter route is a guess from the code comments.

## The ruling on the order of reason and vote

The side lead of the tracing noted that the contest and polish ballots declare the choice first and the select ballot
declares `best` first,
which the ruling of 2026-09-27 ("Reason before vote") did not cover.
The owner ruled on 2026-10-21,
"Reason first everywhere" (ledger B732).

## What fixed what

- Ledger B730,
  the polish gate tie:
  `f4682cd017` (red),
  `a3c2897c5b` (extraction),
  `45657407a6`.
- Ledger B731,
  the polish gate sheet:
  `b09097e067` (red),
  `5419332cef`.
- Ledger B732,
  the order of reason and vote:
  `8202d858a6` (red),
  `e5bc27f9f0`.
- Ledger B733,
  the log line:
  `f68c4964de` (red),
  `ed053bb36c` (a name for the line),
  `1ab73ab56d`.
- Ledger B734,
  the composite cases' bench start:
  with `e5bc27f9f0`.
- The cache versions,
  moved after the fixes:
  `4b9fc9e645` (`CONSOLIDATE_CACHE_VERSION` 29,
  `LANE_CONTEST_CACHE_VERSION` 14,
  `TRANSLATE_SLICE_CACHE_VERSION` 23,
  `SLICE_CACHE_VERSION` 41,
  `REFINE_CACHE_VERSION` 12).
- The source scans:
  `2e5c2f82c1`.
- None needed:
  findings 1 (beyond the tie rule),
  3,
  4,
  5,
  7,
  9 and 10,
  for the reasons the sections of each finding give.

## What is still not traced

The tracing covered ten findings.
These items of the page and run files are outside it,
and their text there stands as written:
- why the repair lane's re-seat line printed 23 times in `zheermao21` and never in `CuspariaKLSY21`;
- the 21.244 Wh gap between the first reading of `zheermao21` and the last of `zheermao20`;
- why slice 0 of `zheermao21` heard two seats that `zheermao20`'s did not;
- the twelve wording choices of `CuspariaKLSY21` for which no code cause was found,
  other than 已经 (finding 3);
- the departures of `zheermao21` labelled `WD`,
  `WB` and `W` other than D21,
  the opening quote (finding 5) and the email (finding 6);
- the cause of the overrun of the `XingZ6016` picture read beyond the guess about the `max` effort.

## What was not measured

- No rerun was made:
  whether the judges decide differently with the accepted claims in view,
  with a tie settled to the polished text,
  or with the reason before the vote is unmeasured,
  and a change of this kind needs a measurement against run-to-run spread.
- The rotation sweep (`source-scan:test:rotation-sweep`) was not run,
  because it runs the family's unit tests ten times.
  It is due before a change to the text of a sheet lands,
  and this change moves several sheets.

# Translation repair: the archive block review and the zheermao20 letter

On 2026-10-21 the read of run `zheermao20`
([the pass log of 2026-10-21](translation-repair-openrouter-2026-09-03-2026-10-21.md))
found the email letter of entry `zheermao101` worse than in `zheermao19`.
A read-only investigation of the two runs' logs and the source then traced how the archive block review produced it.
This file records what it found;
the fixes it led to are queued in
[the NeuralWatt handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next).

## What the first account got wrong

A first draft of the read called it the most likely reading
that Qwen's cut reply would have been the winning whole-letter rewrite of zheermao19.
The investigation contradicts that for this block,
and it corrects two other parts of the account:

- In zheermao19 the letter's Synthetic Qwen call also ended empty:
  it ran 350,706 ms to its cap,
  with 0 content characters and 79,741 reasoning characters.
  The winning text came from Qwen's re-ask on Hyper,
  which took 113 s;
  Hyper was dry for all of zheermao20.
  Across the 18 stored logs,
  21 Qwen streams that ended on their own after 250 s were counted,
  16 with content and 5 empty;
  that count pools mixed dates and window settings,
  so whether the zheermao20 stream would have finished is not known.
- All three zheermao19 revisions of the letter were Hyper-served and thorough,
  naming four to six defects each.
  The two zheermao20 revisions came from `mercury` (OpenRouter) and `Kimi-K3` (Synthetic)
  and named one to three each.
- Synthetic was wet during the letter's review:
  `METERS` read `synthetic=wet` at 02:28:38 and 02:30:43 UTC,
  and first read dry at 02:37:26 UTC.
  Only Hyper was out of budget for this block.

## The letter's review in zheermao20

The zheermao20 revisions changed disjoint lines of the 36-line block:
one changed lines 3 and 6 to 9
("a cat",
"evil",
"your QQ number"),
the other lines 12 and 13 and 26
("fearing death",
"scrape by",
"trans woman",
"us" and "our").
Neither touched the QQ sentence or the comfort lines.
The judges picked the second revision with weight 3 of 4 ballots;
it was better than the archive on every line it changed and changed nothing else,
and no ballot reason favoured it for being narrow.
In zheermao19 quorum was reached only when Qwen's Hyper re-ask returned at 464 s,
so nothing was cut;
in zheermao20 quorum came at 125 s,
and Qwen was then a surplus voice with 120 s left.
On the email block of the same run the race went the other way:
Qwen was needed for quorum,
finished at 239 s,
and its rewrite won with weight 5.5 of 6 ballots.
Both `source-supported` replies on the letter were dropped as unanchored
(`2 eligible assessments from 4 heard reviews`),
for a cause not yet traced.

## Gaps the source shows

The investigation found the code as its documents say:
one review round by design,
the archive kept when the review declines,
and the 120 s grace the owner's decision of 2026-09-03.
It found three gaps the source shows:

- The review round is set as a reader round,
  but its revise replies are writer voices,
  and the owner's writer rule is that a cut writer voice is a whole candidate lost
  (`doc/decision/translation-repair-straggler-grace.md`);
  the decision never classified the stage.
- Selection picks one whole revision,
  so fixes that two reviewers made on disjoint lines cannot both ship;
  the consolidation's own wire records the opposite reasoning for slices,
  and the editor lane already builds composite candidates.
- `repairArchiveBlocks` never reads the stage's findings,
  so the artifact cannot show that a block lost a writer.

## Remedies weighed

The investigation ranked the remedies,
with the cost of this block's review and select rounds as context
(463 s or 23 percent of the entry;
8.27 Wh;
USD 0.0073):

- A composite of disjoint revisions on the slate:
  no extra call,
  would have shipped every fix the two revisions made,
  and the editor lane has the precedent.
  It cannot fix what no reviewer named,
  and the judges may not prefer it.
- The writer window (180 s) for the review round:
  the owner's writer rule applied to a stage that fits it,
  at most 60 s more per cut round,
  bounded by the 360 s call deadline;
  whether it turns a cut into a rewrite or an empty cap-cut is not known.
- Whole-bench fan-out for the review round:
  up to 40 percent more asks against the 2026-09-09 spend instruction,
  and the added seats are not deep reasoners.
- A per-seat window for Qwen:
  names one vendor model in code.
- A second review round when a revise-capable seat was lost after quorum:
  about 8 minutes on a block like this,
  against the no-loop design,
  and it cannot find a defect nobody named.
- A reasoning-aware grace:
  needs plumbing through three packages,
  and a rule on answer characters would not have waited for the stream that mattered.

The bench depth was the largest factor in the pair:
every thorough zheermao19 revision was served by Hyper,
and restoring a deep route is an operating decision,
not a code change.

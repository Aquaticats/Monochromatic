# Translation repair history 2026-08-21 to 2026-08-23: the consolidation wrap closed on a premise that measurement refuted, 2026-08-22

The wrap task closed on a savings premise that measurement refuted,
the stronger justification found in the same data,
that both deciders approved bytes the run then changed,
and the change that wraps proposals before the slate is built.
The heading of its section also names the building of the publisher;
that half is filed in `translation-repair-history-2026-08-21-to-23-document-publisher-built-and-verified-2026-08-22.md`,
and this file holds the wrap half.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-22, later: `#162` closed on a refuted premise, and `#175` built the publisher

### `#162`: the savings premise was refuted before any code was written

The task was opened on a claim that whitespace-only consolidations were costing rounds:
3 of 20 shipped consolidations differed from their incumbent only in where lines broke.
That number came from an offline re-derivation over runs 8 and 9,
both of which predate `#151` wiring the wrap into the pipeline.

Re-measured over the two most recent runs of the same six entries,
`demoted` is 0 of 34,
and the one slice the claim named never reaches the gate at all.
The zero is real rather than a plumbing failure:
`rewrapped` rides the same forward and varies in the same files.

A stronger justification was found in the same data and replaced it.
Over those runs 15 of the 16 shipped consolidations came back from `wrapConsolidation` altered,
which means both deciders were approving bytes the run then changed.
That is a correctness defect of the same class as `#142`,
and it does not depend on any savings claim.

What landed is `wrapConsolidationProposals`,
called in `consolidate-settle.ts` before the slate is built rather than after the gate has spoken.
The change turned out much smaller than predicted:
`judgeTranslateSlate` already had a `soleIncumbent` early return,
and `SLATE_TERMINALS` already mapped `sole-candidate` to `slate-unjudged-standing`,
so a proposal that is only a re-wrapping now folds into the incumbent,
settles unjudged,
and buys neither a slate round nor a gate round,
with no new terminal and no artifact,
parser,
cache or census ripple.

The costly band-pair fork run was DROPPED with a stated reason rather than deferred.
Its primary signal compares zero against zero on the current pipeline,
and the band pair's own measured run-to-run spread,
about 2 slices in 13,
swamps any rate reading at this size.
A deterministic zero-round unit test replaces it.

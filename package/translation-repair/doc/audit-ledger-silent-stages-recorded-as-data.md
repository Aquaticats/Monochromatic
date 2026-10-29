# Audit ledger: a stage that heard nobody, recorded as data

A stage that fell short of its quorum,
how a settlement says so,
and the gates that decide from that fact whether a record is kept and resumed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B820: a panel that heard nobody on five packets was cached as examined, because its finding no longer began with the text the gate matched

Status:
fixed on 2026-10-22 (UTC),
cache version `SLICE_CACHE_VERSION` 43 to 44.
The brief's D3:
red `ca5419874c`,
fix `c265e8df95`.
The review's S1 and N3:
red `7f53745063`,
fix `e6544017ff`,
with `068d523090`,
`b423e15be8` and `9d6095ae7e`.
Found by the read of the XingZ6018 run (build `8864316eb2`);
the first reading of its cause was corrected by the second reader.

What the run showed:
slice 85 was cached with 12 issues,
every one `needs-human` (6 minor,
5 major,
1 critical),
and the log line `chunk 85: nothing to edit, unchanged`.
Its critic stage heard 4 of 6 voices.
Its panel ran 20 rounds,
19 of them `0/4 heard` and one `0/5 heard`,
and every one closed with `no quorum (0 of 3 needed)`.
No `NOT cached` warning printed for that slice,
though it printed for slices 82,
83,
84,
86 and 87.
The stored record (`slice-cache/XingZ60/3231001d79....json`) has `sliceIndex` 85,
`changed` false,
`heardCritics` 4 and 37 findings.
The critical issue is one critic's claim and the panel never ruled on it,
so it is an unadjudicated claim,
not a confirmed mistranslation.

The cause:
the first reading said the panel stage emits no quorum-unmet finding.
It does.
`adjudication/src/panel-stage.ts` maps every packet finding to `panel-packet (<clusterId>): <finding>`,
so the record holds five findings that contain `stage-quorum-unmet (panel 0/6)` (twice) and `(panel 0/5)` (three times)
and none that begins with it.
The gate (`repair/src/repair-cache-gate.ts`) refused a settlement only for `heardCritics` of 0
or for a finding for which `silentStagesOf` held,
and that function was `finding.startsWith('stage-quorum-unmet (')`.
It saw an empty list,
and `storedOutcome` (`repair-slice-settle.ts`) stored the slice.
The select,
checker and refiner stages locate nothing,
which is why their silent slices had been refused.
A later run on that cache directory would resume slice 85 as examined;
that is read from the code path and was not run,
because the two re-entries of the run stopped in preparation.

The fix:
the fact travels as data.
`ChunkRepairOutcome` carries `silentStages`,
required on every outcome,
and the panel result names every packet it heard fewer voices than quorum on.
`silentStageLabel` spells a label such as `panel 0/6 (cluster/abc)` once
and `silentStagesOfRecord` (`stage/src/stage-silence.ts`) reads the field first,
then the findings no layer relocated.
The writers of the outcome were checked one by one:
`unchangedAfterPanel` in `repair-chunk.ts` at the three exits past the panel and at the final return,
`repair-unchanged-outcome.ts` with an empty list for the exits before the panel
(a critic shortfall is still a plain quorum-unmet finding there),
`repair-not-applicable.ts`,
and `front-matter-repair.ts` by spread.
The cache gate reads the field,
and the slice-cache reader (`slice-cache-store.ts`) refuses a record that lacks it or names a silent stage,
so the records already written are redone.
`stage-quorum.ts` spells its shortfall with `silentStageLabel` (note N3 of the review),
so the label and the data it names cannot drift.

Cache versions:
`SLICE_CACHE_VERSION` moves 43 to 44 (records stand at 43,
written by XingZ6018):
the key hashes the version,
the run shape and the texts,
not the rule that decides what may be stored.
`REFINE_CACHE_VERSION` stays at 14:
its key and its question are unchanged,
and the reader refuses a refinement whose embedded outcome lacks the field.
The account in the TSDoc of `repair/src/repair-slice-key.ts` also names the versions that stay,
as the review's M28 check asked:
lane contest 16,
consolidate 32,
consistency 7,
page title 6,
translate slice 25,
pairing 3,
and the artifact schema.
The commit of B818 moves none,
because it changes run control and not what a settlement may be cached as.

Review S1:
the refine phase's write gate (`repair/src/refine-phase-slice.ts`) asked `everyStageHeard` of the refinement's own findings,
and the reader (`isResumableRefinement`) also refused a record whose embedded outcome named a silent panel.
For a slice whose panel heard nobody the repair slice is not cached,
the refine phase still ran on that outcome and wrote its file,
and the attempt queue counts every slice-named file as progress.
By the review's reading of the code (not run),
the entry would be queued for a REATTEMPT that buys the repair slice and the refinement again,
rewrites the same file name and ends STALLED:
one whole-entry attempt wasted for each affected entry,
with the "NOT cached" warning never firing.
The fix gives both gates one predicate in the stage package,
`refinementIsResumable({ settlement })` over `silentStagesOfSettlement`,
and removes `everyStageHeard`,
which nothing reaches any more.
The warning lists both sources.
The red,
`7f53745063`,
asserts that a refinement whose outcome names a silent panel writes no refine file.

Guard:
the cases of the two commits:
the cache gate refuses a settlement whose `silentStages` is not empty,
a deaf panel leaves one,
the slice-cache reader refuses a settlement shaped like slice 85,
and `silentStageLabel` and `silentStagesOfRecord` read the field first.
The red of `ca5419874c` failed because none of the new exports or the field existed.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Facts a cache gate reads":
a fact a gate depends on travels as data,
and a write gate and a read gate ask one predicate.
Earlier members of the family:
B141 and B160 in `audit-ledger-which-seats-a-quorum-counts.md`,
which gave the recheck and the probe the same reader.

## B1060: 32 repair chunks closed with the archive after a one-voice checker round, and a later run buys them again

Status:
recorded on 2026-10-29 (UTC),
measured from the log of `XingZ6022` (build `c6185d4086`),
by design (B820),
no code changed.
The ranking of three options below is a recommendation for the owner.

What the run showed:
33 repair chunks closed `unchanged, the archive won`.
Chunk 44 is B1024 (3 of 3 checkers heard, all `fixed`, changed by B1040).
The other 32 are chunks 52 to 87 except 56, 61, 64 and 83,
and each of them follows one shape.
Its checker round heard 1 of 3 (`no quorum (1 of 2 needed)`),
its introduced-defect probe heard 1 of 3,
and the chunk closed with 0 of the accepted issues it served resolved
(95 accepted,
94 served,
1 with no envelope).
The chunk then printed a warning that names `stage-quorum-unmet (checker 1/3)` and ends
`so this settlement is NOT cached and a later run buys the slice again`,
and its refinement printed the same refusal.
Chunk 84 shows the sequence:
checker round line 18101,
probe round 18198,
settlement 18201,
chunk warning 18202,
refinement warning 20361.
The 44 one-voice checker rounds of the log ran between 10:38:09 and 11:04:47 UTC,
after the Synthetic weekly credit read dry at 10:36:27.677 UTC (line 13492),
when Kimi-K3 and Qwen3.8-27B left every bench and Mimo v2.6 Flash was the one checker left of three.

Cause:
the verdict is the owner's rule working.
Two ballots decide and one voice never does (2026-10-25),
so no issue could be marked resolved,
and a patch that resolves nothing cannot beat the unchanged candidate.
The settlement is not kept because `silentStagesOfRecord` (`stage/src/stage-silence.ts`) reads the
`stage-quorum-unmet` finding and the cache gate refuses a settlement that carries one (B820).
A settlement bought on a bench under the floor says so on its own record and is never kept as a verdict
(the rule for next time of B835 in `audit-ledger-short-benches-and-the-last-live-seat.md`).
The older answer "zero voices is not cached, one voice is" (`doc/decision/translation-repair-question-answers.md`,
"Thin-roster slices") predates B820 and is not the rule now.

Cost of buying them again,
by SPEND lines and NeuralWatt energy lines that carry `[repair slice N]` for the 32 chunks:
637 calls in the chunk window (SLICE-START to SLICE-COST) and 84 in the refinement and recheck after it,
721 calls in all,
which is 19.5 percent of the run's 3,690.
Booked cost 2.2399 USD of 6.9656 USD (32.2 percent),
of which Sonnet 5.5 is 1.1224 USD in 32 calls and Haiku 5.5 0.7730 USD in 236.
NeuralWatt energy 49.13 Wh (42.32 in the chunk window and 6.81 after) of 238.272 Wh (20.6 percent).
The chunk windows add up to 114.2 minutes of slice time.
The rounds of the 32 slices by stage:
select 42.0 minutes,
panel 30.6,
critic 22.2,
checker 12.8,
probe 12.0,
refiner 5.2,
editor 3.1,
128.0 in all,
so the question the run could not seat (checker and probe) is 24.8 minutes of it
and 103.2 minutes re-buys work that was heard at quorum.
The method is in `/var/home/user/temp/agent/tr-fan/w379-accounts/rebuy.ts` and `stagesplit.ts`;
the figures are ceilings of a rerun,
for the reason in the next paragraph.

What a rerun actually pays:
`promptUniqueClient` (`stage/src/prompt-uniqueness-client.ts`) reads the payload store under the runs directory
before it buys,
so a rerun in the same runs directory replays every completed call whose prompt is unchanged
(`PROMPT-REUSE source=disk`) and buys the voices that were not asked.
The owner removed the pass's memory of skipped translate slices on that ground (B1011),
and B834 measured 0 NeuralWatt calls and 0 Wh in the second attempt of 18 slices.
`XingZ6022` and `XingZ6021` each ran in a new runs directory:
the 62 `PROMPT-REUSE` lines of `XingZ6022` all say `source=memory`,
and `XingZ6021` has none.
So the figures above are what a rerun pays in a new runs directory or after a prompt change.
What a same-directory rerun pays for these chunks was not measured,
because that takes a run.

Did `XingZ6021` buy the same chunks?
No.
Its four uncached chunks are 15 to 18 (23 accepted,
21 served,
0 resolved;
115 calls,
0.2914 USD and 5.88 Wh in the chunk window).
None is among the 32,
and `XingZ6022` repaired all four with 3 of 3 checkers on the chunk
(6 of 6,
5 of 5,
7 of 7 and 2 of 2 served issues resolved).
Its 32 chunks closed `nothing to edit, unchanged` (30) or `no validated claims, unchanged` (2) in `XingZ6021`,
under the earlier weight floors.
So the rebuy is prospective:
the next run pays for the chunks that a dry provider leaves unseated,
which are different ones each time.

Later commits:
`6d39223583` (B1030) took Qwen3.8-27B off the checker order,
so the static bench of three is Mimo v2.6 Flash,
Kimi-K3 and Haiku 5.5.
With the Claude key and Xiaomi wet and Synthetic dry mid-run,
the checker round would read Mimo v2.6 Flash and Haiku 5.5,
2 of 3 against a quorum of 2.
This is read from `RUN_CHECKER_ORDER` and B1030 and was not measured,
and it holds for this view only:
a run without the Claude key leaves the shape as it was.
`add3f1b27e` (B1024) added `patchLostOn` to the settlement line and does not change the 32.

The options for the question "should an unseated settlement be remembered":
- Keep the rule (a).
  Pros:
  no verdict that two ballots did not decide is carried into a run in which the providers are back
  (Synthetic named its return as 12:15:47 UTC,
  31 minutes after the run ended),
  no new cache surface,
  and the payload store already makes a same-directory rerun cheap.
  Cons:
  a new runs directory or a changed prompt pays the 721 calls again,
  and the warning says "buys the slice again" without saying that completed calls replay.
- Resume the heard stages and ask only the unseated ones when a bench returns (b).
  Pros:
  it spares the 103.2 of 128.0 round minutes that were heard at quorum even where payloads are lost.
  Cons:
  a second record keyed by the patch text,
  the archive text and the bench,
  which duplicates what the payload store keeps,
  the ground on which the owner removed the memory of skipped slices (B1011),
  and a patch kept against an archive that changed is a stale patch.
- Remember the whole settlement for the outage, until the named return (c).
  Pros:
  one rule that a reader can state.
  Cons:
  it keeps a verdict that one voice made,
  against the rule of 2026-10-25,
  a rerun before the return buys the same unseated round and saves nothing,
  and a cache key then carries a clock.

Ranking:
(a) over (b),
because (b) adds a record that the payload store already keeps and the owner has removed that kind of memory,
and its saving shows only where the payloads are gone.
(b) over (c),
because (b) keeps only what was heard at quorum and never a one-voice verdict,
where (c) keeps the verdict and saves nothing before the return.

What still stands:
the cost of the 32 chunks in a new runs directory,
the unmeasured cost in the same directory,
and the owner's choice among the three if the cost is thought too high.
The log line could say that completed calls replay from the payload store,
which is a wording change and was not made.

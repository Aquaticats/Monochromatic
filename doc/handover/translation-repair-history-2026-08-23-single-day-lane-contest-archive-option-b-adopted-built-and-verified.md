# Translation repair history 2026-08-23: option B for the archived text on the lane contest, adopted, built and verified live

The adoption of option B on a standing instruction,
the build plan drawn after reading the deciding source,
what landed and the guard-proof probe that nearly reported a false null,
the live check that the new field costs no voices,
and the guard for a contested slice whose archive is absent.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## #181 adopted: option B, on a standing instruction rather than a fresh preference

### Why this stopped being a question

The ranking B > C > A was already argued from the measurement.
What was left looked like a preference call between B and C,
and it is not one.

C accepts that nothing ever evaluates the shipped text
on about 16 percent of contested slices,
on the ground that the archive is the conservative default.
The standing instruction for this work is that whatever we produce must be good
even when the originals are not.
That instruction decides between them directly:
an unevaluated archive is exactly the case it was given for.
So B is adopted,
and the adoption is a derivation from an instruction already given
rather than a new decision taken on the owner's behalf.
It is open to veto on that basis.

A stays last for the reason the measurement gave:
it buys one case in six and adds a second trigger path to reason about,
so it is worse than doing nothing deliberately.

### What B has to solve, which the ranking did not settle

The collision named in the task is real and is the first thing to work out.
Both contests share `CONTEST_POLICY`,
and B needs a way for the contest to reach the incumbent
when both candidates are declined,
without the policy naming either side's candidates.

Two things the measurement already fixes about the shape.
It must cover all six observed cases rather than the one with a flagged incumbent,
because the size note is incidental to this path.
And it must not read a recorded terminal as a stable property of an entry:
Zha_Ke and gaoyanger both settled differently across pipeline versions at identical slice sizes,
so run identity and code identity are confounded in the record.

### What is still not shown

Nothing establishes that the archive at any of those six slices is WRONG,
only that nothing evaluated it.
Deciding any of them is wrong means reading the passage,
which is a separate act under the corpus rules.

## #181: the build plan, after reading the deciding source

### `CONTEST_POLICY` does not change at all

Step 1 concluded the archive question could be stated once in the shared policy.
It can be,
and it should not be.
The gap is lane-only.
`consolidate-gate-wire.ts` offers `neither` between `consolidated` and `standing`,
and `standing` is the archive,
so the gate's decline already lands on the archive by construction.
The lane contest's `neither` is the one that lands nowhere.

The two sheets already carry their own `Return JSON` instructions outside the shared policy,
so a lane-only field has an established place to live:
`lane-contest-wire.ts`,
beside its own instructions.
`CONTEST_POLICY` stays byte-identical,
which also leaves the measured `neither` wording untouched
(10 of 13 against an 8-of-13 general-preference control).
If the gate ever needs the same question,
the block lifts to shared then.

### The archive verdict must be recomputable, because the reader recomputes

`LANE_CONTEST_QUORUM = 2` is frozen,
and its TSDoc says why:
`artifact-v2-read-contest.ts:221` recomputes every recorded verdict
by calling `settleLaneContestBallots` over the stored ballots,
and refuses the artifact when the stored verdict and the recomputed one disagree.

So the archive verdict cannot be a number the stage decides and writes down.
It has to be a pure function of the stored ballots,
settled by mirroring `settleVotes` (`lane-contest-stage.ts:137`):
quorum of two,
and a clear winner or none.
Endorsed at two or more and strictly ahead of declined endorses;
declined at two or more and strictly ahead declines;
anything else is unjudged.
The threshold is mirrored rather than invented,
so no second frozen constant enters the artifact contract.

### The terminal question resolves differently than expected

The archive verdict does NOT belong in `ConsolidationTerminal`
(`consolidate-settle.ts:72`).
That union describes how a CONSOLIDATION ended.
The lane contest records into a different union,
`ArtifactContestVerdictV2` (`artifact-v2-contest.ts:119`),
whose three kinds are `quorum-not-met`,
`settled-neither` and `lane-won`.
`settled-neither` is exactly the state #181 is about,
so the archive verdict rides there,
and the eight-member consolidation terminal union is not touched.

This also answers the sub-question about `no-standing-text` carrying two meanings.
It does not have to be split:
the ambiguity it was accused of belongs to a different layer than the one being fixed.

### Ask always, count only on a decline

Asking only after a decline needs a second call,
a second cache entry and a second failure mode,
which is the same objection that sank option A.
The field is asked on every contest ballot and read only when the choice is `neither`.
Answers on lane-won slices cost nothing extra and are free telemetry on archive quality.

### Shipped bytes change nowhere

Worth stating plainly,
because it bounds the risk.
`no-standing-text` already ships the archive,
an endorsement ships the archive,
and a decline also ships the archive,
because nothing else exists.
The deliverable of option B is EVALUATION AND RECORD,
not different output.
The regression guard follows from that:
a re-run of a settled entry must publish byte-identical pages,
the #172 pattern.
What can move is judge behaviour and artifact shape,
and those are what the tests cover.

### Landmines to cover, not to assume away

  - The cache keep-predicate is `outcome.usable >= LANE_CONTEST_QUORUM`
    (`lane-contest-driver.ts:59`),
    which is on the voice count,
    not the verdict,
    so an unjudged archive still caches.
    Old cache entries carry no archive field;
    `lane-contest-cache-store.ts` must read their absence as unjudged rather than refuse them.
  - A one-sided slice has no archive to judge.
    The field's meaning has to be guarded there.
  - The archive must be judged on the SAME two questions as the candidates,
    or it gets a laxer standard and the endorsement is biased.
    Mostly-declined is a legitimate outcome,
    not a failure of the build.
  - A new required schema field can raise schema-mismatch voice loss on weak models.
    Measure contest-stage voice loss before and after on live slices;
    do not predict it.

### Scope check, run rather than assumed

`SETTLED_WITHOUT_A_GATE` (`consolidate-driver.ts:88`) names four terminals
that never reached a judge.
Whether `incumbent-only` and `slate-unjudged-standing` also ship unevaluated text
is the same class of gap as #181.
If they do,
that is a NEW task,
not #181 scope.

## #181: what landed, and the GFP that nearly reported a false null

### The shape, as built

The archive question is an orthogonal field on the lane contest ballot,
`publishable` or `flawed`,
asked on every ballot and read only where the choice is `neither`.
`CONTEST_POLICY` is byte-identical.
The gate does not ask,
because its `standing` choice IS the archive
and a judge preferring it has already said the archive is worth keeping.

`settleArchiveBallots` mirrors `settleVotes`:
quorum of two,
strict lead,
otherwise `unjudged`.
It is exported and shared with the artifact reader,
exactly as `settleLaneContestBallots` is,
so a stored verdict is re-derived from the ballots beside it
and `assertVerdictMatches` refuses the artifact when the two disagree.

Two decisions carry the backward compatibility,
and both are load-bearing rather than tidy:

  - The field is REQUIRED IN THE SCHEMA but OPTIONAL IN THE SHAPE GUARD.
    `isLaneContestWire` refuses a reply missing any field,
    so requiring it there would trade whole lane ballots for archive answers
    on models that ignore the schema.
  - An unjudged archive OMITS THE KEY rather than recording `unjudged`.
    `requireExactKeys` rejects only extra keys,
    so an omitted field means every artifact settled before today
    derives the same token it recorded,
    with no exception in the comparator.

### Measured, not asserted

Suite 539 PASS / 0 FAIL / exit 0,
up from 537 by the two new suites.
Lint 0 warnings 0 errors.
`lint:types` exit 0.
The full suite was also run BEFORE the new tests were added
and reported 537 PASS / 0 FAIL,
which is what shows the artifact reader still parses everything already on disk.

### The GFP, including one probe that lied

Three probes,
each rebuilt and re-run:

  - Quorum removed from `settleArchiveBallots`:
    the lone-voice case fails,
    `expected 'endorsed' to equal 'unjudged'`.
  - Omission rule removed,
    so the field is always recorded:
    the absent-key case fails.
  - Strict lead relaxed to `>=`:
    the tie case fails,
    node exits 1,
    passing suites fall from two to one.

The third probe reported NO FAILURE on its first run,
and that was an artifact.
`git checkout -- <src dir>` between probes discarded the barrel exports,
which were still UNCOMMITTED,
so the rebuilt bundle no longer exported `settleArchiveBallots`,
the test file failed at import,
and a grep looking only for assertion text found nothing and read as green.
This is the GFP warning about restoring over uncommitted work,
met in practice.
The fix was to commit the guard and the test first,
then re-probe while checking two things the first run never checked:
the import error count,
and the number of suites that still passed.
A null result from a probe that cannot run the assertion is not evidence.

### Still owed on #181

Shipped bytes change nowhere,
so what remains is live behaviour rather than output:

  - contest-stage voice loss measured before and after on live slices,
    because a new required schema field can raise schema-mismatch loss
    on weak models,
    and GLM-4.7-Flash already leads that table.
  - the wangzihao980 slice 3 against slice 4 control,
    where the field is consumed on one and ignored on the other.
  - a one-sided slice,
    where there is no archive to judge,
    checked rather than assumed.

Commits:
83cfaa4bd (plan),
377e3e62f (wire),
2d8fc8914 (settle and record),
37feccb35 (tests).

## #181 verified live: the field costs no voices, and the archive loses on every slice

Two arms over `wangzihao980`'s five contested slices,
same slices,
same roster of six,
one run each,
differing only in whether the ballot asks the archive question.
Run through `contestLaneSlice` against the live roster,
with source rebuilt from the corpus at the artifact's own commit.

### Voice loss did not rise, which was the risk worth measuring

  - asks-archive:
    30 of 30 voices usable,
    zero lost.
  - no-archive:
    27 of 30 usable,
    three lost.

The concern was that a new required schema field would raise schema-mismatch loss
on the weaker models,
GLM-4.7-Flash above all.
It did not:
the arm carrying the field sat at the ceiling and lost nothing,
and every one of its 30 ballots answered the new field.

Read the DIRECTION,
not the size.
Single runs cannot resolve three voices in thirty against the run-to-run band,
which was never measured here,
so the honest claim is that loss did not increase,
not that the field improves it.

### The control pair behaves as designed

  - chunk 2 and chunk 3 settled `neither`,
    so the archive verdict is CONSUMED there.
    These are exactly the slices `#181` was opened about:
    the archive ships and the record used to say nothing about it.
  - chunk 0,
    chunk 4 and chunk 5 had a winning lane,
    so the field is IGNORED
    and no archive key is recorded.

Choices agreed across the arms on four of five slices.
Chunk 5 differed,
`translate` against `repair`.
One slice on single runs is consistent with ordinary judge variation
and is not evidence that the field perturbs the choice.

### The archive lost everywhere, and that is a finding rather than a fault

29 of 30 archive answers said `flawed`;
one said `publishable`.
All five slices settle `declined`.
The roster judges the published rendering of this entry unfit as it stands
at every slice it was asked about,
including the three where a candidate beat it.

This is the outcome the design anticipated:
the deliverable is EVALUATION AND RECORD,
not endorsement,
and a mostly-declined archive is a legitimate answer.
Shipped bytes are unchanged either way,
because the archive already ships wherever the contest declines both candidates.

### Still owed, and now the only thing owed on #181

A contested slice whose archive is EMPTY has no archive to judge,
and the sheet would ask about a blank block.
Not reachable in this entry,
whose five contested slices all carry one,
so it is unmeasured rather than refuted.
The fix belongs in `buildLaneContestMessages`:
omit the archive question when `incumbentText` is empty,
and let the lenient guard settle it `unjudged` as it already does for a silent judge.

Probe:
`${HOME}/temp/agent/181-live.mjs`,
results in `181-live-asks-archive.json`
and `181-live-no-archive.json`.
Ids,
counts and verdicts only;
no passages.

## #181 closed: the absent archive is guarded, and the case was measured before it was built

The last owed case is built rather than deferred.
A contested slice whose archive is absent shows judges an empty block
while the schema still asks whether the archive is publishable,
so `contestLaneSlice` now drops archive answers when `incumbentText` is empty.

Stripped in the STAGE rather than where the record is built.
The recorded verdict is derived from exactly the ballots that get stored,
so removing the answer in one place keeps the stored ballots
and the verdict derived from them in agreement,
and the artifact reader re-derives `unjudged` from what it reads back.

MEASURED BEFORE BUILDING,
and the measurement did not excuse the work.
Across 13 settled artifacts,
all 108 comparison rows carry `incumbentKind: present`,
so none of the 92 contested slices has an empty archive.
`IncumbentKind` nevertheless admits `absent`,
and `wording-coherence.ts` handles it,
so the state is representable and merely unobserved in the artifacts to hand.
Unobserved is a reason to guard it,
not a reason to assume it away.

GFP:
removing the strip fails the DROPS case
with `expected 'declined' to equal 'unjudged'`,
node exits 1,
while the KEEPS positive control beside it still passes,
which is what shows the assertion is specific to the strip
rather than to the archive machinery in general.

FINAL GATE:
suite 540 PASS / 0 FAIL / exit 0,
lint 0 warnings 0 errors,
`lint:types` exit 0.
Commits:
83cfaa4bd,
377e3e62f,
2d8fc8914,
37feccb35,
2c08becfc,
f69f15342,
731dcc3e2.

# Critic attribution: recording who raised each accepted claim

Proposal,
not a ratified decision.
`#65` is the task it unblocks.

## The measurement that makes the case

Taken over `pass13`'s 12 settled entries,
433 accepted issues,
reading artifacts
 directly.
Counts and hashes only;
no corpus text was quoted anywhere in the
 measurement or in this document.

83.1% of accepted issues rest on exactly ONE deduplicated claim.
The remainder
 fall away quickly:
4.2% on two claims,
3.9% on three,
and a thin tail to eight.

That number is not what it looks like at first reading,
and the difference is
 the whole argument.
`aggregateClaims` collapses structurally identical claims
 by deterministic id BEFORE anything downstream counts them,
so six critics
 emitting the same claim produce ONE `AggregatedClaim`.
A cluster of size one
 therefore does not mean one critic found the defect.
It means one distinct
 claim survived deduplication,
and the artifact cannot say whether that claim
 came from one critic or from the whole roster agreeing exactly.

So for 83.1% of everything the pipeline accepts,
the record is silent on the
 question `#65` has to answer.

Exact-place duplicates,
meaning two accepted issues sharing a chunk,
a category
 and a span set,
account for 13.4% of accepted issues,
58 across 40 groups.

Both shares survive the concentration guard,
so both are population signals
 rather than one entry's behavior:

-   Single-claim:
    83.1% overall,
    80.0% after dropping the largest contributor,
    per-entry spread 58% to 91%.
-   Duplicates:
    13.4% overall,
    13.4% after dropping the largest contributor,
    per-entry spread 5% to 26%.

A first version of this measurement reported 81.3% duplicates.
It was wrong.
 `claims[i]` is an `AggregatedClaim` wrapper of shape `{ claimId, claim, }`,
so
 reading `claims[0].spans` yielded `undefined` for every issue,
the span key
 collapsed to a constant,
and every issue in a chunk with a shared category
 looked identical.
A positive control asking how many issues produced an empty
 span key returned all of them,
which is what exposed it.
The corrected reader
 reports zero unusable span keys.

## What the fix must not break

Adjudication is provenance-blind by deliberate design.
 `adjudicate-prompt.unit.test.ts` asserts the sheet "keeps proposer identity out
 of the sheet",
and `aggregate-claims.ts:17` records the reason:
a real defect
 can arrive with exactly one proposer,
and the reference run had
 `gpt-oss-120b` as the sole finder of a planted seed.
Proposer counts must never
 influence acceptance.
Attribution is for calibration only.

This is why attribution cannot simply be a field on `IssueClaim`.
It rides
 alongside,
keyed by the claim id `computeIssueClaimId` already assigns.

## Shape

Collected as `Map<claimId, Map<modelId, count>>` while resolving,
converted to
 sorted readonly arrays at the boundary,
because a native `Map` in the outcome
 would not survive `JSON.stringify`:

```ts
// package/module/translation-repair/src/critic-attribution.ts
export type ClaimAttribution = {
  readonly claimId: string;
  readonly proposers: readonly {
    readonly modelId: SyntheticModelId;
    readonly emissionCount: number;
  }[];
};
```

Moved on 2026-10-18 from `package/module/translation-repair/src/critic-attribution.ts`
to `package/translation-repair/adjudication/src/critic-attribution.ts`.

The count is not decoration.
`proposers.length` is independent support,
how many
 distinct critics found the defect;
`emissionCount` is self-repetition,
one
 critic saying the same thing twice in one report.
A plain `modelId[]` populated
 by appending conflates them,
and `#65` is precisely the question of which of
 the two produces its duplicates.

## Where it is built and carried

Built in `runCriticStage` inside the resolution loop at
 `repair-stages.ts:181-197`,
immediately after `resolution.resolved` succeeds,
 keyed by `computeIssueClaimId` over the RESOLVED claim.
Not over the wire
 claim:
the deterministic id is defined over the resolved form including
 anchored spans.

The input is already there.
`gather.voices` carries `HeardVoice`,
documented as
 "one heard voice with its speaker",
and `repair-stages.ts:153` maps it to
 `voice.value`,
discarding `modelId` one line after it arrives.

Carried through `CriticStageResult`,
then `ChunkCriticPhase`,
filtered to the
 claims surviving `screenNonTranslationVotes`,
and kept local in `repairChunk`.
 `aggregateClaims` receives only claims and `runPanelStage` receives only
 clusters,
both signatures unchanged.
Attribution is joined onto accepted issues
 only AFTER the panel returns.

## Traps

-   **Clustering is transitive.**
    Claim A may overlap B and B overlap C while A
    and C do not.
    A cluster can hold several real defects,
    some accepted and
    some rejected,
    and the panel may decline the merge.
    Credit only the source
    claims the resulting accepted issue actually represents;
    do not union every
    cluster member's proposers onto every issue emitted from that cluster.
-   **`clusterId` is a hash of member ids**,
    so member ids cannot be recovered
    from it later.
    If the adjudicated issue does not already retain its accepted
    source claim ids,
    it needs them.
    That field carries no proposer identity and
    is safe for a provenance-blind panel.
-   **Never recompute an id after adjudication.**
    `computeIssueClaimId` includes
    severity and summary,
    and adjudication is explicitly allowed to re-grade
    severity,
    so an id recomputed from a re-graded issue can be absent from the
    index.
    Preserve the original source claim ids through adjudication.
-   **Exact identity is narrower than semantic duplication.**
    Different
    summaries,
    severities or span order give different ids by design,
    and
    clustering handles the semantic proximity.
    For duplicate accepted issues,
    attribution per source claim is what distinguishes self-repetition from
    cross-critic discovery.
-   **Accepted-only attribution is not full calibration.**
    It gives accepted-hit
    counts,
    not precision or opportunity rates.
    Those additionally need heard
    critic model ids rather than the bare count at `repair-stages.ts:59-62`,
    attribution for rejected resolved claims,
    and resolution failures tied to
    the critic that emitted them.
    Separate telemetry,
    and still never a panel
    input.

## Eligibility should be recorded, so the population is readable

A change reaches only what is recorded after it lands,
exactly as the voice-loss
 telemetry does.
At entry granularity that means `pass13`'s artifacts split into
 entries written before attribution existed and entries written after.
Any
 calibration averaged over that directory mixes a population where a large share
 of entries CANNOT contribute,
and "critic X raised nothing in entry Y" reads
 identically to "entry Y predates the map".

That is the same failure the quiet-prober finding in `#68` is about:
a silent
 stage reads identically to a clean one.

So the artifact should record that attribution was AVAILABLE for the entry,
not
 merely carry the map.
A reader can then restrict to the eligible population
 instead of inferring absence from an empty map.

NOT UNRECOVERABLE,
though an earlier version of this section and of `#76` both
 said so.
Every artifact carries `tip`,
and `--plan` prints ONE tip for the
 whole process,
so every entry settled by a single pass shares it and a resumed
 run gets a different one.
Verified on `pass13`:
all 13 settled artifacts carry
 the same tip.
Entries predating the attribution commit are therefore
 identifiable after the fact by git ancestry on `tip`.

Build the marker anyway.
Self-describing beats git archaeology,
it costs
 nothing,
and a reader should not need the repository to interpret a run
 directory.
But the reason is ergonomics,
not data loss.

## Cost

A change to the cached outcome shape requires bumping `SLICE_CACHE_VERSION`
 (`repair-translation.ts:85`,
currently 9),
and that bump was missed on the very
 commit that added the gate,
so it is worth pricing rather than assuming.

Measured:
`corpus-pass.ts:566` discards an entry's slice cache the moment the
 entry settles,
so the cache only ever holds in-flight work.
On `pass13`,
12
 settled artifacts against a slice cache holding one entry,
`GLaDOSister`,
8
 files,
216K.
A bump costs one entry's partial slices.

The bump is also a design lever rather than only a cost.
If attribution lives in
 the cached outcome,
every post-bump slice carries it.
If it does not,
cache-hit
 slices silently produce no attribution,
reproducing the mixed-population
 problem at slice granularity,
where it is far harder to notice.
It belongs in
 the cached outcome,
and the version gets bumped.

`runRefinePhase` must preserve the new field wherever it reconstructs outcomes.

## Landed in full

The path runs from the critic stage to the artifact.
Each entry artifact now
 carries `chunkCritics`:
one record per chunk holding `chunkIndex`,
 `heardCriticIds` and `claimAttributions`.
`SLICE_CACHE_VERSION` moved to 10.

IT ADDS NO CORPUS TEXT TO THE ARTIFACT,
which is worth stating because the
 boundary matters and someone will ask.
`claimAttributions` holds claim-id
 hashes and model ids;
`heardCriticIds` holds model ids.
Artifacts already carry
 corpus text elsewhere and are already gitignored,
so the new key changes
 nothing about how they may be handled.

READERS VERIFIED BEFORE THE FIRST ARTIFACT WAS WRITTEN,
not after.
A passing
 test suite proved the WRITERS worked and said nothing about whether consumers
 accept an unexpected top-level key,
because every reader fixture still
 describes the old shape.
Checked on a throwaway copy of four settled artifacts
 with a realistic `chunkCritics` injected:
the draw path built its pools and
 reconciled `acceptedCount`,
and the probe telemetry reader produced its usual
 figures.
Both parsed real numbers out of those artifacts,
so neither result is
 a vacuous pass.

TWO OF THE TRAPS BELOW TURNED OUT MOOT,
by construction rather than by luck,
 and that is the argument for this shape:

-   Transitive clustering cannot miscredit anyone,
    because nothing is unioned
    onto an issue.
    Attribution stays keyed by claim id and a consumer joins on
    the ids an issue actually holds.
    There is no union to get wrong.
-   Re-grading severity cannot orphan an entry,
    because no id is ever
    recomputed.
    Adjudication re-grades the ISSUE;
    the member `AggregatedClaim`
    keeps the id computed at resolution.

Verified by mutation five times over,
each having left the suite green
 beforehand at least once:
deleting the emission push,
forcing the per-model
 count to one,
returning unfiltered attributions past the screen,
emptying the
 chunk records at the driver's return,
and emptying the roster at both chunk
 sites.
The last two are the ones that matter most,
because a change collecting
 attribution perfectly and writing none of it is indistinguishable from never
 having built it.

The eligibility marker was subsumed rather than built.
A nonempty
 `heardCriticIds` says attribution was available for that chunk AND supplies the
 denominator,
so a separate per-entry boolean would be strictly weaker.

## What has landed, and two defects the building found

Landed:
`177f3e7b3` the collector and the stop to the discard,
`5b93b5213` the
 wiring guard,
`4009a6da4` the carry through the screen,
`b9a279ffa` the
 ordering fix.
All cache-neutral,
so `SLICE_CACHE_VERSION` is still 9 and a
 resumed run cannot be perturbed.
The artifact carry and its bump remain open.

TWO DEFECTS SURFACED,
neither predicted by this document when it was written,
 and both about determinism of a value headed for a cached outcome:

-   `localeCompare` for proposer order.
    Locale-dependent,
    so two hosts could
    order the same proposers differently.
    Replaced with code-unit comparison.
    Caught by a test that failed on the expectation.
-   Insertion order for the attributions themselves.
    Insertion order follows
    voice ARRIVAL,
    and `gatherStageVoices` orders voices by retry round then
    roster position,
    so a run that heard one critic first and another on a
    retry serialized identical evidence differently.
    Replaced with a sort by
    claim id.

The second is the instructive one.
The determinism test that existed could not
 have caught it,
because it used a single claim and its outer array therefore
 always had length one.
Sorting the inner arrays looked like it settled the
 question and did not.

THE TESTING LESSON,
which cost a mutation to learn:
the fold had tests and the
 WIRING had none.
Deleting the single `emissions.push` in `runCriticStage` left
 the entire suite green,
so the exact discard this work exists to fix was
 reintroducible without anything noticing.
Whatever carries attribution to the
 artifact next,
delete that line and confirm something fails before believing
 it is guarded.

## Tests the change needs

-   One critic emitting an identical claim twice records that critic once with
    `emissionCount` two,
    not two proposers.
-   Two critics emitting an identical claim produce one deduplicated claim
    carrying both proposers.
-   A failed claim resolution produces no attribution entry.
-   Contradiction screening removes orphaned attribution.
-   Re-grading severity at adjudication retains the original source claim id.
-   A transitive cluster credits only the members the accepted issue represents.
-   A sentinel proposer model id never appears in any message passed to the
    panel client.
-   A cached outcome and a fresh outcome produce identical artifact attribution.

## The reader, and why it shipped the same night as the writer

`mise run //package/module/translation-repair:score-attribution` (moved on 2026-10-18 to
`//package/translation-repair/probe:score-attribution`) reads a run's
 settled artifacts and prints per-critic calibration.
It takes the runs
 directory from `TRANSLATION_REPAIR_RUNS_DIR` exactly as every other reader
 does.

It prints no corpus text.
Claim ids are hashes and model ids are model ids,
so
 its output is safe to paste where the artifacts themselves are not,
including
 into a third-party model.
That is a deliberate property of the output,
not an
 accident of the current fixtures.

Shape of what it prints:

-   `POPULATION` splits entries into eligible and ineligible and reports the
    chunk count,
    so a reader can see at once how much of the directory could
    contribute at all.
-   One row per critic:
    chunks heard,
    distinct claims raised,
    emissions
    including self-repeats,
    accepted issues backed,
    and the two per-chunk rates
    those imply.
-   `SUPPORT` counts accepted issues by what they rested on:
    `sole`,
    `multi`,
    `selfRepeated`,
    `unattributed`.

The rates are RATES,
not percentages.
One critic can raise several claims in one
 chunk,
so claims per chunk heard legitimately exceeds one.
Rendering them as
 percentages produced readings like `3400%`,
and nothing but running the task at
 the user boundary would have caught it:
every unit test passed on the numbers
 underneath.

BUILT ALONGSIDE THE WRITER ON PURPOSE.
The recurring failure this pipeline keeps
 producing is telemetry recorded and never read,
which `#71` documents at its
 sharpest:
the alignment findings were recorded for the whole corpus and nothing
 consulted them,
so a document-wide mispairing survived every stage.
Shipping a
 data path with no reader would have repeated it.
Unread data is
 indistinguishable from data never collected.

## Three defects the reader shipped with, and how they were found

Landed the same night,
before the first real artifact carrying attribution
 existed,
so no measurement was ever taken through the broken join.

-   **`indexProposers` overwrote on a shared claim id.**
    Walking an entry's
    chunks,
    it did a bare `index.set(claimId, proposers)`,
    so two chunks
    carrying the same id left only the last chunk's critics and silently
    deleted the earlier chunk's.
    That is the exact OPPOSITE of the error the
    writer takes care to avoid:
    `buildChunkCriticRecords` keys per chunk
    precisely so two chunks cannot merge into one inflated entry,
    and the
    reader then merged them the wrong way.
    Deflating is no more correct than
    inflating.
    Proposers are now concatenated,
    and the accepted-issue loop
    dedupes by model id as it already did.
-   **`chunkIndex` defaulted to zero** when it did not parse,
    alone among that
    parser's fields,
    every one of which drops the record instead.
    A malformed
    record became a real chunk 0 and inflated the chunk count,
    which is the
    denominator every rate divides by.
-   **The eligibility decision had no test at all.**
    Every case handed
    `chunkCritics` in by hand,
    so all of them exercised the fold and none the
    wiring,
    and the decision is not made in the fold:
    it is made in `toEntry`,
    which omits the key entirely for an artifact that predates attribution.

A collision needs two chunks producing identical category,
severity,
summary and
 absolute spans,
so the first is latent rather than live.
It was fixed anyway,
 because it is the same shape as the `claims[0].spans` mistake that produced a
 confident 81.3%:
a key that collapses yields a wrong number with no error
 anywhere,
and no measurement taken through it looks suspicious.

VERIFIED BY MUTATION,
each mutation having left the suite green beforehand:
 returning an empty array instead of omitting the key made every entry eligible;
 defaulting a malformed index to zero restored the inflated count;
overwriting
 instead of merging dropped one of two critics' accepted hits to zero.

The wiring-versus-fold hole is now the third instance in one session,
after the
 deleted `emissions.push` and the emptied `chunkCritics` at the driver return.
 The pattern is consistent enough to state as a rule:
when a value's MEANING
 depends on a decision made outside the function that computes it,
test the
 decision where it is made,
not the function.

## A lint filter that hid its own failures

Worth recording because it is a measurement failure,
not a coding one,
and the
 same shape has now cost time twice.

The lint run was being classified with `rg -- '-- '`,
which matches only lines
 containing a literal `-- `.
It reported no findings outside the ignored rule
 while five `no-function-root-let` errors,
one missing TSDoc and eleven
 `chain-per-line` warnings were present in the output the whole time.
The error
 COUNT was right there in the summary line and disagreed with the classification,
 which is what eventually exposed it.

The replacement counts rule names rather than trusting a substring:
 `rg --only-matching '^\s{0,3}\S{1,2} [a-z@/-]+\([a-z-]+\)' <log> | sort | uniq --count`.
 It reports every diagnostic marker `oxlint` emits,
error and warning alike,
so a
 census that misses a rule is visible as a census that does not sum.

## The join, verified against a real artifact before the run produced one

The reader's tests all build entries by hand,
so they prove the fold and say
 nothing about whether the join matches what the pipeline actually writes.
If
 the real issue structure nested claim ids differently,
every rate would read
 zero and nothing would error.

Checked on a throwaway copy of `Acheron.json`,
never the artifact itself.
The
 real structure is `issues[].issue.claims[].claimId`,
and the reader walks
 exactly that.
Its 16 accepted claim ids were injected back as `chunkCritics`
 across two chunks,
including one id deliberately placed in BOTH chunks with a
 different critic in each,
which is the collision case the merge exists for.

Result:
`eligible=1 ineligible=0 chunks=2`,
`sole=7 multi=4 selfRepeated=0
 unattributed=0`,
with both critics carrying nonzero raised and hit counts.

THE RESULT IS NOT VACUOUS,
which matters more than the numbers.
Had the reader
 failed to find claim ids at all,
`unattributed` would have been 11 rather than
 0,
since every accepted issue would have joined to nothing.
And `sole + multi`
 equals 11,
which is exactly the artifact's own accepted-issue count,
so the
 population reconciles against a figure the reader did not compute.

The artifact also carries a third status,
`needs-human`,
alongside `accepted`
 and `rejected`.
Only `accepted` counts toward hits,
which is what the reader
 does,
but the third value is worth naming:
a reader written against a
 two-status assumption would have been wrong here without failing.

## Second review, on the whole source rather than the diff

The first review read the change.
A second read the finished files,
and found
 that tolerance had been applied in the one place it must not be.
Everything
 below was fixed before any artifact carrying attribution existed,
so no
 measurement was ever taken through any of it.

MALFORMED WAS INDISTINGUISHABLE FROM LEGACY,
which is the serious one.
The
 reader omitted `chunkCritics` whenever the value was not an array,
so
 `{"chunkCritics": null}` and `{"chunkCritics": "corrupt"}` both read as
 artifacts settled before attribution existed.
Corruption moved an artifact into
 the pre-feature population,
and that population is what every number divides
 by.
Only an ABSENT key means legacy now.
A present one is decoded strictly by
 `attribution-decode.ts` and throws through the `ArtifactParseError` the repo
 already had,
naming the artifact and the path.

The same reading applies inside the subtree,
and each of these now throws:

-   A chunk index that is not a non-negative safe integer,
    and an emission count
    that is not a positive one.
    `typeof x === 'number'` admits negatives,
    fractions,
    and the `Infinity` that `JSON.parse` yields from `1e400`.
-   A repeated heard critic,
    chunk index,
    proposer or claim id.
    Each of those
    fields is a SET written as an array,
    and each repeat inflates a count in
    silence:
    a critic listed twice as heard on one chunk doubles its own
    denominator.

PARTIAL JOINS WERE COUNTED AS SUPPORT.
An accepted issue naming one attributed
 claim and one the index does not hold was reported as sole-proposer with zero
 unattributed.
Its support is unknown rather than sole,
because the missing
 member may have come from a critic nobody credited.
Those issues are now held
 out of every count and reported as `partialJoin`,
so a broken join reads as the
 defect it is instead of as a confident calibration.

A zero denominator no longer renders `0.00`.
That was FALSE rather than merely
 uninformative,
and indistinguishable from a critic heard often that raised
 nothing,
which is the exact conflation the heard roster exists to prevent.
A
 nonzero numerator over a zero denominator renders `INCONSISTENT`.

### Three tests that could not have failed

Worth recording separately,
because the suite was green throughout and none of
 these would have gone red no matter how the reader broke.

-   Every reader fixture had empty `claimAttributions` and empty `issues`,
    so
    implementations of the proposer,
    claim-id and issue readers that always
    returned nothing would have passed all of them.
-   The malformed-chunk case asserted only `toHaveLength(1)`,
    which holds
    equally if the implementation keeps the malformed record and discards the
    valid one.
-   Nothing anywhere asserted an `emissions` value,
    so folding `emissionCount`
    into `claimsRaised` was invisible.
    The difference between those two numbers
    IS the self-repetition that `#65` turns on.

### One finding handled at the rendering layer on purpose

The reader does not verify that every proposer appears in its chunk's
 `heardCriticIds`.
The invariant holds by construction,
since both come from the
 same `gather.voices`,
and enforcing it with a throw would risk losing the whole
 report on real data tonight for a condition that should be impossible.

It is surfaced rather than ignored:
a critic with claims but no heard chunks
 renders `INCONSISTENT` instead of a tidy zero.
That makes the violation visible
 without letting a telemetry invariant take down a report.

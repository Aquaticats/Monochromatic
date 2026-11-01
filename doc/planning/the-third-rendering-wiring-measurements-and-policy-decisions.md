# The third rendering: the stage wired in, and the policies measured on the way

The gate's refusal policy and its evidence fields,
each measured before anything acted on them;
the stage called inside the pass and the user-boundary verification built to prove resume;
the wrap against the line-structure rule,
the expansion band that did not survive a wider population,
and what re-measuring the pairing costs.

Part of [The third rendering](the-third-rendering.md),
which lists every file of this record.

## The gate's refusal policy, measured rather than asked

I had this down as a values question and was about to put it to the owner.
That was wrong,
and the correction is worth recording:
under a quality-first guideline,
a decision that measurement cannot settle means the measurement is wrong or too small,
not that the decision is a preference.

### The question

A gate judge may answer `neither`,
meaning the two renderings differ only in wording and neither is more faithful to the Chinese.
Today that behaves as an abstention that does not count,
so a consolidation can ship on a minority of decisive ballots.
The churn-averse alternative is to let a plurality of `neither` block a change.

### The population

Across runs 6 to 9 the gate ran at 37 slice-instances.
At 14 of them a consolidation shipped with refusals at or above its own vote count,
which is where the two policies differ.
Three of those 14 are already resolved by the semantic wrap,
which demotes a consolidation that only re-wraps what stands,
leaving 11 live cases across 5 distinct slices:
`keyword233#1` in all four runs,
`Acheron#1` in three,
`Weideriche_#0` in two,
and `keyword233#0` and `Weideriche_#1` once each.

### The instrument, which was already paid for

The gate ballot carries `unsupported` and `dropped` beside `choice`,
each a list naming whichever renderings say something the original does not
or omit something it does.
That evidence is recorded per ballot and nothing has ever read it.
It answers a different question from `choice`:
not which rendering a judge prefers,
but which one that judge could name a fault in.
A judge answering `neither` still fills these in.

### The result

Across the 11 live cases,
the ballots name the standing text faulty 24 times and the consolidation 15 times.
At no live case is the consolidation named faulty more often than the standing text;
the worst it does is tie,
at 4 of the 11.

So blocking on a refusal plurality would,
at every case it would have fired on,
keep a text the panel's own evidence calls equally or more faulty than the one it replaced.
The churn-averse policy costs quality at 7 of 11 cases and gains nothing at the other 4.

`neither` keeps behaving as an abstention that does not count.

### The positive control

A null from an instrument that cannot show the opposite means nothing,
so the same tally was run over all 37 gated instances.
It names the consolidation the worse text at 5 of them,
so it discriminates.
At 3 of those 5 the gate had already settled on the standing text,
which is the instrument and the panel agreeing.

### What the control turned up on its own

The other 2 are a defect,
and a different one from the question above.
`lintong#2` and `Weideriche_#0`,
both in run 8,
shipped a consolidation that the ballots name faulty MORE often than the standing text:
4 against 3,
and 3 against 2.
Neither is a refusal-plurality case,
so no refusal policy would have caught them.

The gate's `choice` field and its own evidence fields disagree there,
and nothing reads the evidence,
so the disagreement has never been actionable.

## The stage is called, and what that cost to reach

The stage now runs inside the pass.
`pass-entry.ts` asks for a third rendering after the lane contest,
and the settled artifact carries what it decided.
Two things worth keeping from getting there.

### What ships is a named absence, not a string

Six terminal states end a settlement and exactly one of them,
`consolidated`,
produces wording an assembly should write.
Every other one leaves the slice with whatever the lane contest put there,
and one of them,
`no-standing-text`,
carries the EMPTY STRING as its text:
the contest named neither lane,
so nothing stands.

A record with a plain `text` field per slice would therefore hand a consumer an empty string to write
over every passage whose contest declined,
which is deletion by default.
The record instead carries `shipped` as `{ kind: 'consolidated', text }` or `{ kind: 'unchanged' }`,
and the reader refuses a record that disagrees with its own terminal in either direction.

### The budget bound, decided rather than defaulted

The calibration bed bounded judging and gating separately,
at six and four times the per-call timeout.
The composed stage takes one signal for a whole settlement,
so the driver had to choose.

It is bounded by the ENTRY's signal and nothing narrower.
Each round the stage buys is already bounded by `RUN_PER_CALL_TIMEOUT_MS` and by the client's stream guards,
and every settled slice is persisted before the next begins.
A per-settlement ceiling would therefore cut a round that the per-call bound is already cutting,
and would throw away that slice's bought work to do it.

### A cache namespace nobody registered, found on the way past

Adding the consolidation's own cache namespace exposed a live defect in the one that registers them.
`belongsToNamespace` defines the repair lane by subtraction from a hand-written list of claimed prefixes,
and `contest.` and `pairing.` were both missing from that list.

Measured on a throwaway directory:
writing one file per namespace and then opening the repair cache at a new generation deleted
`abc.json`,
`contest.abc.json` and `pairing.abc.json`,
reporting `discarding 3 cached slices` as though all three were its own.
`translate.` and `picture.` survived,
because they were registered.

So every repair-lane generation change since the pairing cache landed has silently thrown away
that entry's whole block pairing and every contest ballot it had bought,
both of which are bought from the roster.
Some of the cost attributed to re-running entries after a pipeline change was this.

The list is now derived from a single `CLAIM_BY_ROLE` record that the named exports also read from,
so declaring a namespace and registering it are one act.
The test written to prevent this kept its own copy of the list and had drifted the same way;
the first GFP failed to fail for exactly that reason,
which is how the self-referential hole was found.

## Verifying the stage at the user boundary, and why two plain runs cannot do it

`#138` asks for the verification every other stage got:
run the pipeline the way an operator runs it,
then read what it produced.
Two runs,
because the first must show a consolidation reaching an artifact
and the second must show a resumed settlement being read back and accepted
rather than silently re-bought.

The second half turned out to be unreachable by simply running the pass twice,
for a reason that is correct behaviour rather than a defect.

### A settled entry's cache is deleted by design

`runOneEntry` calls `discardSliceCache` as soon as an entry reaches its artifact,
with the comment saying why:
the cache exists to make a large document resumable,
and an entry that settled has nothing left to resume.
The discard runs after the artifact write and never before,
so a failed write costs no bought work.

That leaves a second pass two possibilities,
neither of which exercises the resume path:

-   The artifact is still there,
    so `settledEntryIds` skips the entry outright.
    Nothing is bought,
    and nothing is read back either.
-   The artifact is removed,
    so the entry re-runs.
    Its cache is already gone,
    so every stage re-buys from the roster.

The path where a stage opens its cache,
finds a settlement written by an earlier run,
validates it and resumes,
is reached only when an entry did NOT settle.

### What the verification does instead

The run's slice cache is copied aside while the pass runs,
by a poller outside the package that never writes into the run directory.
The second run restores that capture into a fresh runs directory
at the same pipeline digest,
with no artifacts,
so every entry re-runs and every stage opens a cache that already holds its answers.

The bytes are the ones the real pass wrote.
What the second run proves is exactly what a resume proves,
and more of it than an interrupted run would:
every stage resumes,
rather than whichever ones happened to be finished when the interruption landed.

The pipeline digest was checked to be reproducible before either run:
two builds of the same source produced
`sha256-tree-v1:2384524b15c2482c37db147b9654b0036eeebfba7e24b6297854d7bcddef4cc0` both times.
A digest that moved between the runs would discard every namespace and re-buy everything,
which would look exactly like a validator that refuses its own output.

### What the capture also answers

The artifact records the settlement's TERMINAL and not the decision behind it.
`slate-kept-standing` is one terminal reached by several decisions,
and `#165` asks how often it is reached by a panel that DECLINED,
which is the case the translate lane buys a second round for.

That count cannot be read from the artifacts.
It can be read from the captured cache,
which holds the whole `ConsolidationSettlement` including `decided`,
so the measurement `#165` names comes free from a run bought for something else.

### What each run has to show, stated before either was read

Run 1,
over `Acheron`,
`Weideriche_`,
`Zha_Ke`,
`gaoyanger`,
`keyword233` and `lintong`:

-   Every artifact parses through `parseSettledArtifactV2`.
    This is the seam nothing has covered:
    the parser's own tests are built by hand,
    and the six-artifact compatibility check exercised only the `unrecorded` path.
    Builder output carrying a settled consolidation had never met the whole-artifact parser.
-   Every artifact records `consolidation` as `settled`.
-   At least one slice records `shipped` as `consolidated` with a non-empty text.
    Runs 8 and 9 consolidated at eight and six slices of thirteen,
    so zero across the whole set would be a finding to record against those,
    not evidence that the wiring failed.

Run 2,
restored from the capture into a fresh directory:

-   ZERO `reportStreamProgress` lines in its log.
    That is the primary instrument,
    because it covers every stage rather than the consolidation alone.
    Run 1 is its positive control:
    the same instrument counts in the hundreds there.
-   Every artifact's `consolidation` field byte-identical to run 1's.
    A resumed settlement is returned exactly as it was persisted,
    so a settlement the validator refused would be re-bought and would differ.
-   Wall time collapsed against run 1.

The capture is checked for completeness before run 2 is allowed to mean anything.
A file the discard raced away and a settlement the validator REFUSED both appear as a call to the roster,
and they mean opposite things.
The check reads each artifact's own consolidation record,
counts the slices `consolidationWorthResuming` would have persisted,
and compares that against the captured files.
`slate-kept-standing` is reported as ambiguous rather than counted,
because whether it persists turns on the decision behind it,
which is the field the artifact does not carry.

## The gate's evidence fields are not a second opinion, measured

`#164` was opened on a disagreement:
across the 37 gated slice-instances of runs 6 to 9,
the ballots name the consolidation faulty more often than the standing text at five of them,
and at two the gate shipped the consolidation anyway.
The task asked whether the evidence is right before anything acts on it.

It is not,
and the reason is structural rather than a question about any particular slice.

### What the evidence field actually is

`GateBallot` records `unsupported` and `dropped` as lists of RENDERING NAMES,
not as findings.
A judge either names a rendering as adding something the original does not support,
or as omitting something it says,
or does not name it.
The raw wording is kept separately in `unsupportedRaw` and `droppedRaw`.

So a ballot naming BOTH renderings faulty adds one to each side of a count of renderings named faulty.
It cannot move a comparison of the two counts.
Every differential comes from the asymmetric ballots.

### Two measurements over all four runs

Of 217 gate ballots across runs 6 to 9:

-   106 name neither rendering faulty.
-   35 name both,
    and therefore discriminate nothing.
-   63 name only the standing text faulty.
-   13 name only the consolidation faulty.

Read over the population,
the evidence favours the consolidation by nearly five to one.
The five disagreeing slices are local inversions of that,
not a trend it hides.

The second measurement is the one that settles it.
Of the 76 asymmetric ballots,
74 name faulty exactly the rendering they voted AGAINST.
Two do not.

The evidence field agrees with the ballot's own choice 97 percent of the time.
It is a restatement of the vote,
not a second opinion about the text.
Counting it beside the vote counts the same judge twice.

### What the five disagreements turn out to be

Every disagreement is one or two ballots wide:
`lintong#2` and `Weideriche_#0` in run 8 by one,
`Zha_Ke#1` in run 8 by two,
`Zha_Ke#1` and `Acheron#0` in run 9 by one.

At `lintong#2` the panel was two for the consolidation,
one for the standing text and one for neither,
and the single asymmetric evidence ballot is the one standing vote,
naming the rendering it voted against.
At `Weideriche_#0` the panel was five to one,
and the disagreement is again the single dissenting vote.

Calling that "the panel's evidence points the other way"
describes what every majority vote looks like from the minority.

### Decided: nothing weighs on this signal

No refusal,
no ballot weighting,
no surfacing.
A mechanism built on this field would act on the dissenting vote a second time,
under the appearance of independent evidence,
and would cost the slices where the same field favours the consolidation by 63 ballots to 13.

The genuine case the task hypothesised does exist and is rare:
two ballots in 217 name a fault in the rendering they nonetheless preferred.
That is the legitimate reading of a minor fault against a larger gain,
and at that rate it needs no machinery.

WHAT REMAINS OPEN,
and is a different question:
`unsupportedRaw` and `droppedRaw` carry the judge's own wording rather than a rendering name,
so they may carry content the name lists cannot.
Nothing reads them either.
Whether they are worth reading is not answered here.

## The wrap and the line-structure rule, measured against each other

`#162` proposed wrapping consolidation proposals before the slate,
so a proposal differing from the standing text only in its line breaks
would collapse into the incumbent and buy no ballots.
It was deferred with one question owed:
does `wrapReplacementText` split a line on the slices where line structure carries meaning?
If it never does,
the change is free.

### The band pair cannot answer it

Zero of the band pair's thirteen slices are `lineStructured`.
Measuring there returns a clean null from a probe that could not have shown anything else.
The corpus can answer it:
`isLineStructured` trips on 211 slices across 34 of the 92 pairs.

### It splits, on nine slices in ten

Over those 211 slices,
wrapping the archive's own English changes 189.
It never joins lines.
It splits them.

Of the 1091 non-empty lines those passages already carry,
621 survive the wrap intact and 470 are broken,
adding 750 lines net.

The archive's English is the right thing to wrap for this question.
`TRANSLATE_LINE_STRUCTURE_RULE` asks the producer to keep the passage's line structure,
so a proposal that obeyed it carries the structure that text carries.
Wrapping proposals before the judges would therefore show them
text breaking a rule they were told to enforce,
on 189 slices rather than none.

`#162`'s free version is off the table.
Its remaining item is a re-run comparison,
and that population needs a line-structured entry in it,
because the band pair cannot measure the degradation the item is about.

### The larger finding, which is not about proposals

None of the three shipped wrap sites is gated on `lineStructured`:
`repair-wrap.ts`,
`translate-wrap.ts` and `consolidate-wrap.ts` all wrap unconditionally.

So the pipeline already does to its own output what `#162` was worried about doing to proposals.
A producer that follows the line-structure rule has 43 percent of its lines broken at assembly,
after every decider approved the text it wrote.

That is not automatically wrong.
`doc/audit/line-structure-loss-when-a-replacement-ships.md` records why the wrap exists:
replacements were arriving as one long line,
326 `semantic-line-breaks` findings,
seventeen of them flattening a multi-line archive passage.
The wrap is a repair for a producer that lost the structure.
On a slice where the producer kept it,
the same repair is damage,
and nothing currently tells the two apart.

`#167` carries the decision.

### Reading the two runs, in order of authority

The capture completeness check is a LOWER BOUND,
not a verdict.
It can compare the captured settlement count against what the artifact says was worth persisting,
except at `slate-kept-standing`,
where persistence turns on a decision the artifact does not record.
A settled-slate file the discard raced away therefore passes the check silently.

So the order is:

-   Run 2's `reportStreamProgress` count is the authoritative instrument.
    Zero means every stage resumed.
-   A small nonzero count is a diagnosis,
    not a verdict.
    The completeness check's per-entry namespace census says which entry and which namespace to look at,
    and only after that split can a re-buy be called a raced capture or a refused settlement.
-   `vub-compare` covers the consolidation field alone,
    so it confirms rather than decides.

If an entry caps or fails in run 1,
the verification proceeds on the settled subset.
A failed entry keeps its cache in run 1's own directory,
by the same rule that discards a settled one,
so re-running that entry in place is the one resume case reached organically rather than restored.
It is worth taking if it happens and not worth buying.

## The expansion band does not survive a wider population

`#163` proposed guarding the ratio of English characters to Chinese characters,
on a band of 2.09 to 4.24 drawn from the 24 shipped consolidations of the band pair,
and owed a re-measurement on a wider population before it decided anything.

Measured over every paired slice of the pinned corpus as the ARCHIVE carries it,
1259 slices across 92 pairs,
at no quota cost.

### What the archive's own ratios look like

Prose slices,
1048 of them:
minimum 0.01,
fifth percentile 1.06,
first quartile 2.40,
median 2.96,
third quartile 3.52,
ninety-fifth percentile 5.11,
maximum 521.86.

Line-structured slices,
211 of them,
are not a separate distribution worth its own band:
median 3.06 against prose's 2.96,
and a ninety-fifth percentile of 4.33 against 5.11.
Item 3 of the task asked whether verse needs checking separately.
On this measurement it does not.

### The proposed band would fire on a quarter of the corpus

281 of 1048 prose slices fall outside 2.09 to 4.24,
which is 27 percent.
190 sit below the low end and 91 above the high end.
Across everything it is 328 of 1259.

A guard firing on one slice in four is not naming a failure mode.
The band was drawn from 24 observations of a stage that had already selected for a decided slice,
and it does not describe the population it would run over.

### Where the tail actually starts

Sweeping the cutoffs over the same 1259 slices:

-   Below 2.09 flags 223,
    below 1.5 flags 107,
    below 1.0 flags 26,
    below 0.8 flags 15,
    below 0.7 flags 14,
    below 0.5 flags 6.
-   Above 4.24 flags 105,
    above 6 flags 44,
    above 8 flags 30,
    above 10 flags 23,
    above 15 flags 15,
    above 20 flags 11.

So a band around 0.8 to 10 flags roughly 38 slices,
three percent,
which is a set small enough to read one by one.
The furthest outliers are the ones the task wanted found:
`shihai4h#53` at 521.86,
`Zha_Ke#1` at 94.78,
three `windward0032` slices between 55 and 85,
and at the other end three `shi_Yumiaoya` slices at 0.01 to 0.02.

WHAT THIS DOES NOT SETTLE.
Whether those outliers are mis-sliced or are correctly paired
passages the archive simply left mostly untranslated is a reading,
not a ratio,
and the flagged set is now small enough that the reading is affordable.
The action on a breach is still undecided,
and `#71` records what recording a finding and never acting on it costs.

### The shape of the outliers, and a second instrument

Reading the 38 slices a 0.8 to 10 band would flag,
by SHAPE rather than by wording:

At the short end the source carries many blocks and the target carries one.
`shi_Yumiaoya#2`,
`#3` and `#4` pair 11 to 15 Chinese blocks against a single English HEADING
of twelve to fourteen characters.
At the long end the source carries one block and the target carries several:
`shihai4h#53` pairs a 21-character HTML block against 28 blocks and 10959 characters,
and `windward0032#0`,
`#8`,
`#10` and `#14` each pair a bare Chinese heading
against an English heading plus its body.

So under nearly every outlier is a block-count disparity,
which is unit-free
and does not fire on a passage that is faithful but terse.

Measured over the same 1259 slices,
1104 of them,
88 percent,
carry EQUAL block counts.
A gap of more than one flags 43 slices,
more than two flags 15,
more than three flags 9.

The two instruments are complementary rather than nested.
Of the 38 ratio outliers,
17 also carry a block gap above one,
and 21 do not.
Of the 43 block-gap flags,
26 sit inside the ratio band.
Their union is 64 slices,
five percent,
which is the candidate set worth reading.

CAVEAT,
and it matters before anything is built on these numbers:
this is the RAW archive slicing.
`prepareDocumentPair` was called with no `blockPairings`,
so the LLM-assisted pairing stage `#131` shipped has not run over these slices.
That stage exists to reconcile exactly this kind of disparity,
so some of the 64 are already handled in a real pass and the guard would never see them.
The measurement bounds the population from above.

## What the pairing re-measurement actually costs

`#163` records its remaining item as "re-measure the 64 WITH the pairing
 applied",
and calls it free.
It is not free,
and the number is worth having before anyone schedules it.

`prepareDocumentPair` applies only a pairing it is HANDED.
A pairing exists for an entry only where a real pass has already bought one,
because the stage `#131` shipped is LLM-assisted.
So the re-measurement is free exactly to the extent that the flagged entries
 already have a cached pairing on disk.

Measured,
with the instrument first validated against the recorded numbers:

-   The union is 64 slices,
    spread across 29 entries.
-   Cached pairings exist anywhere under the agent scratch root for 5 entries.
-   Of those 5,
    exactly 1 is among the 29.
-   So 1 of 64 slices is re-measurable at zero quota,
    and 28 entries would need a fresh pairing call.

VALIDATING THE INSTRUMENT FIRST MATTERED.
A fresh reimplementation of the two flags returned a union of 88 rather than 64,
because it used a Han-character denominator where the recorded measurement used
 plain string length,
and because it skipped a slice with no Han characters before the block-gap flag
 could see it,
which silently dropped 5 gap flags.
Re-running the original script reproduced every recorded number exactly:
1259 slices,
1104 with equal block counts,
43 flagged by a block gap above one,
38 ratio outliers,
17 in both.
A number that does not reproduce is an instrument fault until proven otherwise.

### How to run it, and when

No source change is needed.
`prepareDocumentPairWithRoster` is exported from the built
 `dist/final/node/index.mjs`,
so a scratch script can pair the 28 entries and re-measure without touching the
 package.

It must NOT run while a corpus pass is alive.
A pairing sweep buys provider concurrency of its own,
and `doc/troubleshooting/synthetic-aggregate-concurrency-stall.md` measured
 every backend degrading above roughly seven aggregate streams.
The sweep is therefore queued behind the `#138` verification,
not blocked by it.

# Deciding who fills a seat

Part of [the family README](../README.md).

`judge-fidelity-probe` uses the source-reviewed manifest in `fidelity-reference-manifest.ts`.
It no longer calls the first long archive slice “clean” merely because it is unchanged.
The loader verifies pinned source/archive/donor ranges,
shared normalization,
reviewed local corrections,
reference length and every generated damage hash before any model call.
No corpus passage is committed in the manifest,
and local calibration corrections never edit the corpus or filter production translations.

`--cap 0` is explicitly metadata-only preflight,
not a quality result.
Unknown `--only` entries,
unreviewed `--context`,
a different configured corpus revision
and any requested damage family absent from the selected population are refused before corpus/provider access.
For example,
`--only gqt` cannot provide the default alteration arm;
`--only gqt --damage deletion` requests a supported narrower comparison.

Positive runs materialize the position/direction matrix before applying the row cap.
A capped prefix is explicitly marked incomplete,
not complete admission evidence.
Each invocation creates a fresh payload namespace and persists its logical plan,
completed rows and terminal result separately.
Interrupted execution can therefore retain rows without claiming completion.
The result includes exact reference provenance and its manifest digest.

A singleton panel's merged verdict can be underweight even when its individual vote is correct;
it is not the per-model admission score.
Admission additionally requires the registered complete question set,
peer participation and one score per actual question,
not duplicate scores for equivalent direction bookkeeping.

The historical `hakureico/7` and `noname/4` reference set contained unsupported facts and changed certainty.
Those scores are not source-grounded admission proof.
The [reviewed fixture record](../../../doc/planning/translation-repair-reviewed-calibration-fixtures-2026-09-11.md)
records the replacement references and each verified delta.
No existing role is retroactively removed solely on the old fixture diagnosis.

## Block pairing question and protocol

`blockPairingQuestion` (`package/translation-repair/slice-reading/src/block-pairing-question.ts`) builds the question
block pairing asks of one aligned parent:
the original and archive blocks under the local numbering,
the definition-order exemptions,
and the existing versioned cache key.
It buys no calls.
`blockPairingProtocol` (`package/translation-repair/slice-reading/src/block-pairing-protocol.ts`) builds the stage's
actual messages
and an owned copy of its response schema without creating a provider;
it carries no provider bodies,
model caps or attempt identity,
so it is not a complete record of a question asked.
`prepareBlockPairing` (`package/translation-repair/archive/src/prepare-block-pairing.ts`) prepares one aligned parent
through the production path:
singletons and empty sides take their zero-call paths,
historical cache records stay historical,
and only a queried result carries the seats' final outcomes.

## History: the provider-free preparation layer

From 2026-09-10 to 2026-09-15 this package grew a provider-free preparation layer
for preparing writer-calibration inputs without calling a provider.
It held frozen parent selection and its evidence readers
(`readFrozenPreparationSelection`,
`readPreparationSelectionEvidence`),
root-input reconstruction (`buildPreparationRootInputs`),
preparation attempts,
receipts,
occurrences and definition relations
(`createPreparationAttempt`,
`verifyPreparationAttempt`,
`readPreparationReceipt`,
`readPreparationOccurrence`,
`readRegisteredPreparationParent`,
`readPreparationDefinitionRelations`),
pairing qualification and request capture (`qualifyPreparedBlockPairing`,
`captureBlockPairingRequests`),
and a sealed producer-input runner (`runProducerInputComparison`,
`ProducerInputComparisonError`)
with its build outputs `producer-prepare.mjs`,
`producer-bootstrap.json`,
`producer-input-comparison.mjs` and `sealed-runtime.json`.
The `runtime:seal` and `bootstrap:seal` tasks built the sealed runtime and the bootstrap,
and the package's `test:unit` task depended on `bootstrap:seal`.

No pass or probe entry reached any of it.
With the owner's authorization of 2026-09-16,
`cbedea357` removed the layer,
its tests and fixtures,
both seal tasks,
the `test:unit` dependency
and `producer-input-verification.md`.
The block pairing functions in `Block pairing question and protocol` are what survived.
What was built and verified is recorded in
the [writer-input work record](../../../doc/planning/translation-repair-writer-unit-scope-2026-09-11.md)
and in the
[comparison work record](../../../doc/planning/translation-repair-preparation-plan-and-journal-2026-09-14.md);
`git show cbedea357` lists every removed file.

## Historical writer and editor runners

Two runners rank models on the job the seat actually does.
Both spend quota,
both write nothing to a corpus,
and both take a slice count after `--`.

```sh
mise run //package/translation-repair/bench:producer-calibrate -- 10
mise run //package/translation-repair/bench:editor-calibrate -- 14
```

`producer-calibrate` ranks **writers**.
It drives the translate stage:
a model writing English from Chinese with nothing in front of it but the source.

`editor-calibrate` ranks **editors**,
and reports the refiner standing off the same spend.
It keeps four slices in flight and waits 300000 ms on stragglers after quorum,
both the owner's decisions of 2026-08-26 on the five calibration arms
(`doc/decision/translation-repair-calibration-overlap.md`);
`TRANSLATION_REPAIR_SLICE_OVERLAP` and `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` override either for one launch,
and `120000` reproduces the pass's own window:
the pass's window moved from 180000 to 120000 on
2026-09-03 by the owner's decision on a measured pair (`doc/decision/translation-repair-straggler-grace.md`).
The pass keeps four slices in flight as well since 2026-09-06,
read off four matched pass pairs
(`doc/decision/translation-repair-pass-overlap.md`);
`TRANSLATION_REPAIR_SLICE_OVERLAP=1` reproduces the sequential driver for one launch.
`TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR` (`request-pace.ts`) sets how many Hyper requests may start
in any rolling hour (a whole number of one or more,
or the client refuses it),
retries and credit reads included;
the rest queue in arrival order instead of being refused with HTTP 429.
The default,
1,000,
is the account's limit as the owner stated it and as
XIEPT2's refusals on 2026-09-03 bear out (612 successes in 43 minutes once the hour's thousand was spent,
a trickle of about 12 a minute under refusal);
a value that is not a positive number leaves the default,
and the retry ladder separately waits as long as a refusal's "try again in Ns" asks.
The window starts empty at launch,
so a launch within an hour of a heavy run is refused until that run's requests leave the window.
The **writer** rounds,
editor,
refiner,
translate,
consolidate and the archive block review (classified so on 2026-10-21,
open to the owner's veto),
wait 180000 ms on stragglers after quorum (`WRITER_GRACE_MS` in `writer-grace-override.ts`,
the owner's decision of 2026-09-06 in `doc/decision/translation-repair-straggler-grace.md`),
since a cut writer voice is a whole candidate lost while a cut reader voice is one ballot of eight;
they follow the round window instead when a launch made that the longer one,
as the calibration's 300000 ms is,
so the built-in never shortens a writer round.
`TRANSLATION_REPAIR_WRITER_GRACE_MS` moves the writers for one launch in either direction.
Every launch prints `WRITER GRACE built in`,
or `WRITER GRACE OVERRIDDEN` when the dial is set,
beside the round note in both the pass and this calibration.
It drives the whole repair lane,
so the claims an editor works from are claims models really raised about that passage rather than fixtures.
That costs more per slice than the writer calibration,
because a slice buys critics,
a panel,
editors,
judges and checkers instead of one stage.

Every model writes on every slice in both.
A narrow slate would compare only the models that happened to be seated,
so a standing would mean something different for each of them.
Every model also judges,
matching production,
and each model's ballots on its own work are then discounted,
because counting self-votes ranks the most self-confident model first rather than the best-written one.

## The standing that costs nothing

Every settled artifact's repair chunks already record the slate judges were shown,
each candidate's producer,
and every ballot.
That is what a standing counts,
so one can be read off work already paid for:

```sh
mise run //package/translation-repair/bench:editor-standing-read -- <run dir> [<run dir> ...]
```

It spends nothing and touches no model.
Four things bound what it can say.

It is **observational**.
Only models that held a seat ever wrote a candidate,
so it ranks whoever was seated and is silent about everyone else.
An absent model is unmeasured,
not last.
That is the survivorship the controlled calibrations exist to defeat,
which is why this corroborates them and never replaces them.

It **never pools across pipeline digests**,
because two builds are two configurations and a figure summed over both describes neither.
Each digest is reported alone with its entry count,
which is the denominator that governs:
rounds inside one entry are correlated.

It **refuses an artifact from an earlier roster**,
by name.
Model ids are a closed set,
and reading an id the roster no longer seats as though it were current would let a standing mix two rosters silently.
Those artifacts are counted apart from malformed ones and named with the exact path that held the departed id.

It **separates an earlier schema from a defect**.
A repair result whose `chunks` field is absent entirely was settled before the lane recorded rounds at all.
That record is complete and correct for the build that wrote it;
it simply cannot answer this question.
It is counted as `earlierSchema`,
not as a parse failure,
because calling it broken would report a healthy archive as a damaged one.
Chunks present and not an array stays a parse failure.

The report accounts for every artifact it opened,
across `read`,
`earlierRoster`,
`earlierSchema` and refusals,
so a reader can see what fraction of an archive the standing actually rests on.
On the archives as of 2026-08-24 that is 41 artifacts:
2 read,
17 from an earlier roster,
22 from an earlier schema,
none malformed.

## Reading a standing honestly

Three things on the report decide whether a standing means anything.

The **counts** beside each share.
A share with no denominator cannot be told from a share one ballot wide,
and a lead smaller than its denominator supports is not a lead.

The **slices** that paid in,
printed as `from N of M slices`.
Adjudicated is not accepted:
a slice can buy ten critics and a ten-model panel,
have its issues rejected at the accept gate,
and contribute nothing to an editor standing.
A standing drawn entirely from one slice reads identically to one drawn evenly from six without this line.

The models the table **does not describe**,
named at the end.
A standing carries a row only for a model somebody voted on,
so every other seated model vanishes,
and its absence would otherwise read exactly like a model that wrote and lost.
During a provider outage that is half the roster.

Three different things put a seated model outside the table,
and the calibrations name them apart rather than reporting one absence:

-   `WROTE AND WAS NEVER VOTED ON` names a seat whose text reached a slate
    with no disinterested ballot cast over it,
    which is what a slice where every producer proposed the same wording does:
    it ships unjudged.
    That evidence is already paid for,
    and more slices are what would separate it.

-   `ANSWERED AND WAS NEVER SLATED` names a seat with at least one usable answer heard
    and none a candidate a judge saw:
    a rewriter that leaves a paragraph as it stands,
    or whose rewrite is dropped before judging.
    Re-running it buys the same again;
    slices with something to rewrite are what would seat it.
    Arm A of 2026-08-26 reported such a seat as silent beside a `SEAT` line saying it had answered 31 of 31,
    which is the misreport this state exists to end.

-   `ANSWERED NOTHING USABLE` names a seat with no usable answer heard.
    A provider out of budget,
    a refused sheet and a call that timed out all look identical from the report,
    and the `SEAT` lines and the run log name which.
    That evidence has not been bought yet,
    and re-running those seats buys it.

Only a seat that records who answered can tell the last two apart.
The refiner seat does (`settleRefinedSlice` returns `refinersHeard`);
the editor and translate seats carry only a heard count out of their stages,
so their silent line reads `NO CANDIDATE OF THEIRS REACHED ANY SLATE` and says the seat does not record who answered,
instead of calling the unknown silent.

The silent line carries both denominators,
as `covers N of M seats`,
so a table narrowed by an outage cannot read as a full roster comparison.

A standing,
a slate or an answer list naming a model the run never seated is refused,
because coverage of one roster cannot be read off another.

## Editor credit and refiner credit are separate columns

The lane unions them,
so the split takes work.
`collectRefinedAuthors` merges the editors with any refiner whose rewrite won,
so the refined outcome's authorship names both seats in one list that cannot be split back apart.

The editor column is therefore read off the accuracy lane's own outcome,
before refinement,
and the refiner column off `settleRefinedSlice`'s `refinedBy`,
which names the models whose rewrite is actually in the text that shipped.
`refinedBy` is empty on every path where no rewrite ships,
including one the recheck rolled back,
and it is deliberately kept off the cached settlement for the reason `asked` is:
a slice resumed from disk bought no rewrite.
`refinersHeard` rides beside it,
also uncached:
the refiners heard with a usable answer,
proposal or not,
which is what separates a seat that answered from one that never did.

## The editor calibration diverges from production in one place

Checkers self-certify there,
and only there.
Production forbids a checker from proving its own repair,
and seating all nine models of `RUN_ROSTER` as editors leaves nobody independent to check.
Rotating editors out instead would reintroduce the survivorship the shape exists to avoid.

It is safe for that measurement because checking runs after selection:
the ballots a standing reads are cast before any checker is asked,
so self-certification can move how many rounds happen,
not who won the ones that did.

# Unread signals: outcomes recorded without causes, and the walls that came down

The recurring wall,
where the pipeline records what happened and drops why,
with the cost of the remedy measured,
and the three instances fixed so far:
the quote needle,
the voice-loss diagnosis,
and the critic attribution that was discarded in the source after all.

Part of [Signals the pipeline emits and nobody reads](translation-repair-unread-signals.md),
which lists every file of this record.

## The recurring wall: outcomes are recorded, causes are not

This document is a census of signals the pipeline EMITS.
Four separate
 investigations in one session stalled on the opposite problem,
and they stall
 the same way,
so it is worth naming as one thing rather than four.

-   ~~`quote-not-found` records that a quote failed to anchor,
    and discards the
    QUOTE.~~
    FIXED 2026-08-13 in `b8c678e0a`:
    the finding now carries a bounded
    preview of the needle.
    The 225 misses already recorded stay undiagnosable,
    but future ones name the quote that missed.
    See "The wall came down on one
    of the four".
-   ~~`schema-mismatch` records that a model reply failed to parse,
    and
    `attemptStageCall` discards the `rawText` the outcome carries.~~
    FIXED
    2026-08-13 in `256520df7`.
    See "The second wall came down,
    and the client
    was never the problem".
-   Accepted issues record their claims but NOT which critic raised each one.
    So the duplicate-issue question cannot ask whether a duplicated issue is independent
    corroboration,
    because independence means "different critics" and the
    artifact cannot say.
-   The refiner's silences recorded `refiner 0/1` and no reason,
    so the cause
    was only recoverable once a live run was watched.

The shape is identical every time:
the pipeline records WHAT happened and drops
 WHY,
so any question about cause needs a fresh run rather than the artifacts
 already paid for.
Each of those runs costs days.

The four cases share one remedy:
retain the reason alongside the outcome at the
 point the outcome is recorded,
truncated where the payload is large.
That was
 first written here as "cheap",
which is a claim with a number behind it,
so
 here is the number:

```text
  56 artifacts, 15362 KiB total, median 274 KiB each

  failed quote text        255 x 120B =  30 KiB
  truncated model replies   34 x 500B =  17 KiB
  critic id per claim     5453 x  28B = 149 KiB

  TOTAL                                 196 KiB  = +1.3%
```

So the whole remedy costs about a hundredth of what the artifacts already
 occupy,
and the largest single part is the critic attribution that the duplicate-issue question
 needs.

One correction while measuring this:
the same paragraph originally justified
 caution by saying artifacts "already carry full repaired text",
implying that
 is the bulk.
It is not.
`repairedText` totals 240 KiB across all 56 artifacts,
 under 2% of the 15362 KiB;
the bulk is the 5453 recorded claims and their
 quoted spans.
The caution was pointing at the wrong thing.

Raised here rather than acted on,
on scope rather than on cost.
It would change
 what the duplicate-issue question,
the quote-anchoring finding and the voice-loss diagnosis can be answered from,
so it belongs in a priority
 conversation rather than in a commit landed overnight.

## The wall came down on one of the four

The recurring wall says outcomes are recorded and causes are not.
One of its
 four instances is now fixed,
and what forced it is worth keeping,
because the
 wall had been recorded twice without anything changing.

WHAT FORCED IT was a concrete blocked question rather than the principle.
`Futajuhuacha` runs at 35% wrap-explained where every other settled entry is
 near zero.
Trying to explain that:

-   The obvious hypothesis,
    that the entry is wrapped differently,
    is REFUTED.
    At 8.3 intra-paragraph line breaks per thousand characters it is
    unremarkable:
    `AkiraComplex` is 18.0,
    `Aniloviraw` 16.7 and `Anilovr` 14.4,
    all with zero or one wrap-explained miss,
    and `Dethelly` is higher again at
    8.9 with none.
-   The next hypothesis,
    that its critics quoted longer spans,
    CANNOT BE TESTED
    from the artifacts.
    Reconstructing the anchoring over every claim span in
    all 12 settled entries found 1027 target spans and every single one located
    exactly,
    with none needing a collapse and none missing.

That second result is the wall itself,
stated precisely.
Claims that FAIL
 anchoring never become issues,
so they never reach the artifact,
so the only
 quotes an artifact carries are the ones that succeeded.
The failures are
 counted and their text is gone.

THE FIX is one call site.
`lineBreakSuffix` already had the needle in scope,
so
 the finding now appends a bounded preview:
line breaks collapsed so a finding
 stays one line,
truncated at 60 characters so a paragraph-length quote cannot
 swamp a scorecard line.

WHY IT IS SAFE for consumers,
checked rather than assumed.
The only non-test
 reader of `unresolvedReasons` is `unresolvedCountOf`,
which takes `.length`;
 the field is documented as feeding prompt iteration,
which is exactly the use
 the needle serves;
and findings already carry variable payloads by convention,
 as `stage-quorum-unmet (${shortfall})` and `duplicate-check (${check.issue})`
 do.
Nothing groups them by string equality.

VERIFIED BY MUTATION,
not by the suite passing.
Removing the preview and
 rebuilding fails 24 assertions across the three test files,
so the two new
 cases covering truncation and flattening are load-bearing rather than
 decorative.

THE OTHER THREE WALLS still stand:
`schema-mismatch` discards `rawText`,
 accepted issues do not record which critic raised each claim,
and refiner
 silences record no reason.
Each is the same shape and each would take the same
 kind of change at the site where the discarded value is still in scope.

`Futajuhuacha` stays unexplained until a pass runs the new telemetry.
That is
 the honest state:
the instrument is built and has not yet been read.

## The second wall came down, and the client was never the problem

Two of the four recurring-wall instances are now fixed.
This one carried a
 wrong diagnosis in the earlier entry,
and the correction is the useful part.

THE EARLIER ENTRY SAID the client discards `rawText` and logs the sub-kind at
 debug.
Reading the source,
`synthetic-client.ts` does neither.
It already
 returns `rawText` on every failure outcome AND a `detail` naming which of the
 three faults fired:

```text
  truncated thinking   output was truncated inside its thinking block
  unparseable          content is not valid JSON: <parser detail>
  guard rejected       content parsed as JSON but failed the caller schema guard
```

The loss is ONE CALL DOWNSTREAM.
`attemptStageCall` in `stage-call.ts` had:

```text
  l.warn(`${stage} ${modelId}: ${outcome.kind}, voice lost`,);
```

`outcome.kind` is the flattened union tag.
Both the sub-kind and the model text
 sat in scope,
unread,
on the same object.
So the emitter was fine and the
 consumer threw the diagnosis away,
which is a different fix from the one the
 earlier entry implied and a much smaller one.

WHAT IT NOW RECORDS:
the sub-kind by name,
plus a bounded,
line-flattened
 opening of the model text.
The opening is the diagnostic part.
The Kimi-K3
 outage was a two-character channel marker prefixing otherwise valid JSON,
and
 it explained 507 mismatches in a single pass;
nothing shorter than the first
 few characters was needed to see it.

The refusal branch is covered too.
A refusal and a parse failure call for
 different responses and both read as a bare lost voice before this.

VERIFIED BY MUTATION:
restoring the bare-kind warning fails 9 assertions.
The
 test harness needed a capturing logger to make that possible,
because the
 existing cases asserted only that a voice was LOST and never read what the
 loss recorded,
which is how a warning this uninformative survived having tests.

WHEN IT WILL BE READ,
and this is NARROWER than the first version of this
 paragraph said.
It claimed the voice-loss diagnosis work's "74-and-counting voice losses become
 diagnosable",
which reads as covering the losses already on disk.
It does not.
 A warning is written once,
at the moment the voice is lost,
so the 74 losses
 `pass13` has already recorded stay bare permanently.
Only losses recorded
 AFTER a fresh process starts carry the sub-kind and the opening.
`pass13`
 cannot see the change at all,
being a frozen module graph;
the detached
 watcher resumes that run in a new process,
so the resumed half onward is
 diagnosable and the already-recorded 74 are not recoverable.

The needle write-up in this document got the equivalent point right,
saying the
 misses already recorded stay undiagnosable.
The voice-loss write-up did not,
 and the two changes have identical reach.

### Both fixes make the run log corpus-bearing, which it was not before

The raw opening is model output on a translation task,
so it is largely corpus
 prose,
and the needle preview carries corpus text by construction.
Before
 today the log recorded `schema-mismatch, voice lost` and `quote-not-found`,
 which name a fault and quote nothing.

As built this is safe.
Both land only in run logs,
which live outside git under
 `node_modules/.monochromatic` and are gitignored,
exactly like the artifacts
 and grading sheets that have always held corpus text.

The constraint is on what happens NEXT,
and it falls on this document.
Reading
 those diagnoses and writing up what they say must not quote a needle or a raw
 opening here,
because this file is committed,
and must not paste log excerpts
 into `pi`/sol or any third-party model.
`one-among-us/data` is unlicensed.
The
 same discipline the grading sheets already carry applies:
name the fault class,
 cite counts,
quote nothing.
`artifact-probe-read.unit.test.ts` states the rule
 for the quote fields it omits,
and it is the same rule.

THE REMAINING WALL is accepted issues not recording which critic raised each
 claim,
which blocks the duplicate-issue question.

It does NOT block the prober-disagreement question,
which earlier versions of this document
 claimed.
That question is about the three PROBERS in the
 introduced-defect probe,
not the critics,
and it already has per-model
 attribution:
it carries measured raise rates over the same 857 regions,
81
 claims against 45 against 5.
What it waits on is the probe calibration's human-graded ground
 truth,
which says which prober is right when they disagree.
Getting this
 straight changes the urgency rather than the work:
The duplicate-issue question is a DECISION task,
so
 critic attribution is groundwork,
not something gating a measurement about to
 be taken.

## The third wall is the same shape after all, and I said it was not

The remaining-wall paragraph of "Both fixes make the run log corpus-bearing,
which it was not before",
as `eab15a03c` wrote it,
said the attribution
 is "genuinely absent rather than discarded downstream",
and that commit's message says the same,
so the fix would need
 the claim to carry its speaker from the critic stage onward.
That is wrong,
and
 wrong in the direction that matters:
it makes a discarded value look like a
 data-flow redesign,
which is the difference between a contained change and one
 nobody starts.
Reading the source says otherwise.

`gatherStageVoices` returns `HeardVoice<ValueT>`,
whose TSDoc reads "One heard
 voice with its speaker" and whose fields are `modelId` and `value`
 (`stage-quorum.ts:53`).
`runCriticStage` then does this,
at
 `repair-stages.ts:153`:

```ts
// package/translation-repair/repair/src/repair-stages.ts
const reports = gather.voices
  .map(function toReport(voice,) {
  return voice.value;
},);
```

The speaker is discarded one line after arriving,
and every claim built after
 that line comes from `reports`,
which no longer knows who said anything.
This
 is the third instance of the same shape,
not a different one:
`locate-quote`
 had the needle in scope,
`stage-call` had the sub-kind and the raw text in
 scope,
and `runCriticStage` has the `modelId` in scope.

WHAT IS GENUINELY ABSENT is the destination,
not the source.
`issue-model.ts:10`
 and `aggregate-claims.ts:15` both describe a mechanism in the present tense,
 that "the shell maps claim ids to proposers for calibration,
never for judging".
 No such map exists.
Every occurrence of `proposer` under `src/` is a comment or
 a test name;
none is an implementation.
So the design named the right place,
 recorded why it must sit outside the claim,
and the map was never built while
 its one input was being thrown away upstream.

THE INVARIANT THE FIX MUST NOT BREAK,
and the reason attribution cannot simply
 be added to the claim:
adjudication is provenance-blind by deliberate design,
 `adjudicate-prompt.unit.test.ts` asserts the sheet "keeps proposer identity out
 of the sheet",
and `aggregate-claims.ts:17` records why.
A real defect can
 arrive with exactly one proposer;
the reference run had `gpt-oss-120b` as the
 sole finder of a planted seed.
Proposer counts must never reach a judging
 prompt.
A side map keyed by claim id satisfies this,
because
 `computeIssueClaimId` already gives every claim a stable handle and the panel
 never sees the map.

THERE IS A WORKING PRECEDENT in a sibling stage.
`editor-ensemble.ts:251` tracks
 "every distinct APPLIED proposal for this envelope,
first proposer" for edit
 candidates,
and `candidate-select.ts:29` uses provenance to stop a proposer
 grading its own work.
The pattern is established in this codebase;
the critic
 stage just never got it.

COST OF LANDING IT,
measured rather than assumed.
A change to the chunk outcome
 shape requires bumping `SLICE_CACHE_VERSION` (`repair-translation.ts:85`,
 currently 9),
and that bump was missed on the very commit that added the gate,
 so it is worth pricing before deciding when.
`corpus-pass.ts:566` calls
 `discardSliceCache` the moment an entry settles,
so the cache only ever holds
 in-flight work.
Confirmed on `pass13`:
12 settled artifacts,
and the slice
 cache holds one entry (`GLaDOSister`,
8 files,
216K).
A bump therefore costs
 one entry's partial slices,
not twelve entries.

# OpenRouter and the corpus passes: the defects of XingZ6018 on build 8864316eb2 (2026-10-22)

What was read on 2026-10-22 of the quality issues the critic panel raised in `XingZ6018` (entry `XingZ60`) before the stop,
the seven defects the log and the source at the run's tip show,
what the second reader's check corrected,
and what neither reader could settle.
Three of the defects have ledger entries,
and this record cites them and does not repeat them.

The log,
the stages and the plan energy are in
[the run of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-run.md),
and the providers,
the stop and the Mercury recheck are in
[the providers of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-providers.md).
Line numbers are of the log `temp/agent/corpus-pass-XingZ6018.log`;
source paths are of the run's tip `8864316eb2`.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## Quality issues the panel raised before the stop

The critic stage filed 484 claims (482 with a severity of minor,
major or critical,
two `neutral`) in 82 slices
(`quality.ts`).
By filer:
`deepseek-v4.1-flash` 144,
`inception/mercury-2.5` 118,
`minimax-m3` 90,
`gemma-4-26b-a4b-it` 79,
`hf:Qwen/Qwen3.8-27B` 27
and `hf:moonshotai/Kimi-K3` 24.
These counts were read by the first reader only.

- The largest classes:
  `accuracy/mistranslation` major 123,
  minor 57 and critical 11;
  `style/awkward-phrasing` minor 71;
  `policy/pronoun` 48 (43 major);
  `accuracy/omission` 42;
  `terminology/wrong-term` 23;
  `fluency/inconsistency` 24;
  `accuracy/addition` 19;
  and `accuracy/non-translation` critical 3.
- Tense.
  24 claims mention tense,
  in 10 slices (5,
  23,
  24,
  39,
  43,
  53,
  57,
  60,
  78 and 80;
  `tense.ts`),
  by all 6 filers.
  They are `fluency/inconsistency` (8 minor,
  8 major),
  `fluency/grammar` 2,
  `accuracy/mistranslation` 3,
  `style/awkward-phrasing` 2 and one alignment claim.
  The first is at line 1576,
  and the heaviest is slice 53 (six claims,
  lines 8921 to 8929,
  three of them major from Mercury).
  The claims say the life narrative of a deceased person must be held in the past tense
  and the translation drifts into the present
  (examples:
  line 4969 `think ... is`,
  line 9717 `my memory is really vague`,
  and line 13155).
  13 accepted cluster lines list a tense claim (19 issues).
  All of the tense slices settled `repaired` and resolved their served issues except slices 43,
  57 and 80,
  which are three of the four chunks left with an open issue (6 of 7,
  1 of 2 and 3 of 4).
  The log does not say whether the open issue in each is the tense one (no artifact exists),
  so this is an overlap and not an attribution.
  It is the same class as the unresolved tense issues of `CuspariaKLSY20` to `CuspariaKLSY23`,
  but here the repair lane resolved most of them.
- Critical and major accuracy claims left open by the stop sit in slice 85:
  a Lu Xun quotation rendered with the sentiments reversed (critical,
  line 14424),
  a work title left in Han characters (major,
  line 14428),
  and a contributor name left as a placeholder (major,
  line 14419).
  The critical claim is one critic's (Mercury,
  lines 14089 and 14424);
  deepseek filed the same quotation claim as major (lines 14086 and 14423),
  and the panel never ruled,
  so it is an unadjudicated claim and not a confirmed mistranslation.
  See defect D3.
  The 30 tail slices have no translation yet.
- Other lines:
  two `introduced repetition` warnings (lines 15764 and 15765:
  "admission score line that" in slices 29 and 30,
  and "gender-affirming hormone therapy (GAHT)" in slices 52 and 62);
  two `straightenProseQuotes` refusals (lines 15762 and 15763);
  and `content lost at slice N` lines (31 slices,
  lines 15766 to 15796;
  slice 68,
  for one,
  loses 12 distinctive archive words).
- Not looked at:
  the repaired text of any slice,
  since no page was written.

## Defects

None of these was fixed when the log was read.
D1 to D3 are the ones with a direct evidence chain,
and each was fixed afterwards (ledger B818 to B820);
D4 to D7 are candidates,
not traced,
and no ledger entry exists for them.
The labels D1 to D7 are the read's own;
they are not ids of the ledger's D family,
and the label D4 of the lead's brief for the model-level dryness work (ledger B822) is a different item.

### D1: the all-providers-dry error does not end the run

The stage layer absorbs it as a lost voice.
Fixed in ledger B818 (2026-10-22 UTC,
two rounds),
with the follow-ups B821 to B825.

- The design text is `provider/src/budget-routing.ts:49` to `:50` (`ENDS THE RUN, at the owner's instruction`)
  and the message's `this run ends` (`:87`).
  The second reader added three more places that make the promise:
  `:321` to `:322` (the `@throws` "ends the run"),
  `provider/src/budget-hold-wait.ts:27` (only a second all-dry reading "ends the run"),
  and the tests titled the same way (`provider-router.unit.test.ts:713`,
  `budget-routing.unit.test.ts:387`),
  which assert only that the call throws (`provider-router.unit.test.ts:735`:
  "must end the call with a refusal").
  Nothing in the provider package makes the run stop.
- `stage/src/stage-call.ts:360` to `:377`:
  the `catch` rethrows only when the caller's signal aborted;
  otherwise it logs `voice lost` (or `seat unreachable` for `NoProviderForModelError` only)
  and returns `heard: false`.
  `EveryProviderDryError` is not a `NoProviderForModelError`.
  The decision-seat path has the same catch
  (`stage/src/stage-decision-call.ts:163` to `:204`,
  whose TSDoc at `:76` says "a lost voice never throws unless the
  caller aborted").
  The log has no abort line,
  so no signal was aborted.
- `EveryProviderDryError` is named outside `provider/` only in tests
  (`run/src/corpus-run/run-client-neuralwatt.unit.test.ts:226` and
  `source-scan/src/message-names-only.unit.test.ts:112`).
  A case-insensitive search of the non-test,
  non-doc sources at `8864316eb2` finds no catch of it,
  so "no higher caller catches it" holds for this build.
- In the log:
  992 lines carry the refusal's text (14107 to 16205),
  240 retry-round lines,
  60 slices through a refine pass that heard 0 of 3 four times each,
  the entry ended at line 15797 by the bench floor,
  then two re-entries (lines 15799 to 16220) that repeat the preparation and refuse again (89 of the 992)
  before `STALLED` at line 16221.
  The timing matches:
  OpenRouter was marked refused at 14:45:44.316 UTC (lines 14096 and 14097),
  the stop came at 14:47:01.971 UTC,
  which is 77.655 s after the mark and 79.7 s after the first wire 402 (14:45:42.292 UTC,
  line 14065).
- Effect:
  no extra spend (no chat stream started),
  about 80 s from the first 402 to the stop,
  20 percent of the log's bytes,
  and seats that were in fact unreachable tallied as `threw` and not `unreachable` after the mark.

### D2: the error text is wrong about the NeuralWatt reset and loose about Bedrock

Fixed in ledger B819 for the NeuralWatt time;
the Bedrock sentence was left as it was,
and B819 says so.

- `provider/src/budget-routing.ts:88` says the free energy returns at 00:00 UTC.
  The comment at `provider/src/provider-budget-refusal.ts:96` repeats it,
  and `provider-router.unit.test.ts:741` pins the string.
- The log's own day anchor says 19:15:08 UTC (line 205:
  `NeuralWatt day starts at 19:15:08.000 UTC every day, from the
  quota's subscription.current_period_start (2026-10-20T19:15:08Z)`),
  and `neuralwattDayStart=2026-10-21T19:15:08.000Z` stands on all 63 `METERS` lines (first at line 210).
  The owner's ruling is recorded in `provider/src/neuralwatt-day.ts:5` to `:11`
  ("not 00:00 UTC,
  which the first version of this package assumed")
  and `provider/src/neuralwatt-day-anchor.ts:18` to `:22` ("THERE IS NO MIDNIGHT FALLBACK"),
  decided by `decideNeuralWattDayAnchor` (`neuralwatt-day-anchor.ts:118`).
- The second reader added the cause:
  `provider/src/neuralwatt-energy.ts:15` to `:16` records that the vendor's documentation says 00:00 UTC
  while the owner's account starts its day at the period-start time of day.
  So the error text repeats the vendor documentation,
  and its constructor takes no day anchor,
  so it could not state the account's time.
- The Bedrock claim also holds:
  `budget-routing.ts:86` to `:87` says Bedrock "has no credit left short of the owner's card",
  while line 198 says Bedrock is off because its token expired (HTTP 401).
- The text was never printed before this run in the seven earlier logs searched
  (CuspariaKLSY23,
  zheermao23,
  XingZ6017,
  CuspariaKLSY22,
  zheermao22,
  XingZ6016 and XingZ6015;
  `rg --count` finds none).

### D3: a slice settled with a silent panel is cached as settled

Fixed in ledger B820 (`SLICE_CACHE_VERSION` 43 to 44).

- Slice 85:
  its critic stage heard 4 of 6 and filed 12 claims (line 14081;
  line 14094,
  which the first reading cited,
  is the panel start,
  `5 preplanned cluster packets`).
  Then 20 panel rounds ran (lines 14115 to 14414),
  19 of them `0/4 heard` and one `0/5 heard`,
  each with `no quorum (0 of 3 needed)`.
  All 12 issues were adjudicated `needs-human ... panel not on record` (lines 14418 to 14429;
  6 minor,
  5 major and 1 critical),
  then came `chunk 85: nothing to edit, unchanged` (line 14430)
  and `SLICE-COST ... exit=computed` (line 14431).
- No `NOT cached` warning printed for slice 85,
  unlike slices 82,
  83,
  84,
  86 and 87.
  The slice cache holds a record for it (`3231001d79...`,
  `sliceIndex` 85,
  `changed` false,
  `heardCritics` 4,
  12 issues all `needs-human`,
  37 findings;
  `cachedslices.ts` and the checker's `slice85.ts`).
- The write decision is `repair/src/repair-slice-settle.ts:72` to `:84` (`storedOutcome`),
  which stores unless `cacheRefusalsOf` is non-empty.
  The gate (`repair/src/repair-cache-gate.ts:27` to `:36`) refuses only when `heardCritics === 0`
  or when a finding is silent by `silentStagesOf`,
  which is `finding.startsWith('stage-quorum-unmet (')` (`stage/src/stage-silence.ts:21` and `:53` to `:59`).
- The root cause,
  as the second reader corrected it:
  the panel stage does emit the quorum-unmet finding,
  but prefixed.
  `adjudication/src/panel-stage.ts:430` to `:439` maps every packet finding,
  `gather.findings` included,
  to `panel-packet (<clusterId>): <finding>`.
  The cached record holds five such findings:
  `panel-packet (cluster/...): stage-quorum-unmet (panel 0/6)` twice and `(panel 0/5)` three times.
  No finding starts with `stage-quorum-unmet (`,
  and five contain it.
  So `startsWith` missed them,
  the gate saw an empty list,
  and the settlement was stored.
  The select,
  checker and refiner findings are unprefixed,
  which is why those slices were refused a cache entry.
  The first reading,
  that the panel stage emits no such finding,
  was wrong.
- Effect,
  stated as the check allows:
  a later run on this cache directory would resume slice 85 as examined,
  with its claims unadjudicated.
  This is read from the code path (`storedOutcome` returns `stored`) and was not run:
  the two re-entries stopped in the preparation and never reached slice 85.
  The first reading called the critical issue "a critical mistranslation unfixed";
  the check showed it is one critic's claim that the panel never ruled on.

### D4 to D7: candidates, not traced

Read by the first reader only;
the second reader did not examine these four.

- D4.
  The abandoned-spend estimate may not see a content-dense runaway.
  - All six `estimated=abandoned` minimax lines price exactly 130 raw characters per token
    (for example 402,041 raw characters give 3,093 tokens,
    line 14498).
    `stream-overrun.ts` says decoded content is about a hundredth of the raw wire.
    The line 14497 stream carried 373,040 content characters in 402,041 raw (93 percent);
    the five overruns carried 32,000 in 56,171 to 366,329 raw (9 to 57 percent).
  - If the provider billed what it generated,
    the estimate (0.0113 USD) understates it.
    The balance fell 0.026 USD more than booked over the last 74 s,
    which fits but is inside the meter's lag (see the OpenRouter section of the providers record),
    so this is not shown.
- D5.
  The content bound did not end the line 14497 stream at 32,000 characters.
  - The watch checks the bound after each chunk
    (`provider-stream/src/stream-runaway-watch.ts:330` to `:338`,
    cap `:85`)
    and reads repetition first (`:318` to `:320`).
    The stream reported `degenerate` at 373,040 characters,
    so the count jumped past the cap inside one chunk;
    with 93 percent of the wire being content,
    a few very large frames is the likely shape (inferred).
  - It ran 78.5 s (first content 1,003 ms,
    largest gap 651 ms) and held slice 82 and,
    with it,
    the stop.
- D6.
  Minor logging.
  The Hyper 402s (lines 6705 to 6715) and the first four OpenRouter 402s (14065 to 14072) print a bare `HTTP 402`;
  the provider's text appears only for the typed-decision seat (14105 and 14106).
  A 60 s rate-limit hold prints `hyper=dry` with a positive balance on `METERS` (lines 3779,
  3809 and 3823),
  which reads like exhaustion.
- D7.
  A 429 storm at 34 slices in flight,
  not traced.
  `zheermao23` had no HTTP 429 and `zheermao22` had 10;
  this run had 80 (lines 2672 to 3816),
  on Hyper only,
  with up to 34 slices in flight in a 118-slice entry (`pace.ts`).
  The retry ladder behaved as written
  (`names its return in 20000ms, past this ladder's reach of 16000ms; ending the ladder`,
  lines 2672 to 3815,
  11 lines),
  and Qwen and `glm-5.3` were unreachable for stretches.

## What the second reader's check corrected

The check confirmed D1 and D2,
confirmed D3 on its facts and corrected its cause,
and confirmed all five headline counts.
Where it disagreed with the first reading,
this record files the check's reading.

- The count of lines that carry the every-provider text is 992,
  of which 990 are `voice lost` lines
  and two (lines 16010 and 16011) are picture-reader `failed outright` lines.
  The 994 `voice lost` lines are not 2 plus 992:
  they are 2 wire-402 lines,
  990 every-provider lines,
  1 runaway line (14499) and 1 summary line (16237).
- Round 1 of the retry ladder has 92 lines from line 14116,
  not 93;
  the 93rd (line 10621) stands before the dry mark.
  Rounds 2 and 3 have 74 each,
  and the total of 240 stands.
- "No request left the process after either mark" is true of chat streams only;
  the meter polls continued
  (counts in "How the stop behaved" in the providers record).
- The cause of D3 and the strength of its critical claim (see D3 in this file).
- The critic stage of slice 85 is at line 14081,
  not 14094.
- The headline that this was the first run to exercise the all-providers-dry path rests on a search of seven logs;
  older logs and the audit ledger record earlier occurrences.
- The open question on why the panel stage left no `stage-quorum-unmet` finding is answerable
  (D3).

The check confirmed without change:
the build and `START` lines,
the 77.7 s and 79.7 s intervals,
the Hyper and OpenRouter spend after the marks,
the Lu Xun claim wording,
`repair-cache-gate.ts:27` to `:36`,
`stage-call.ts:360` to `:377`,
`budget-routing.ts:49` and `:83` to `:89`,
the OpenRouter `cost=` sums,
the first Hyper and OpenRouter 402 lines,
the Mercury totals and percentiles,
and the counts of `METERS` lines (63),
HTTP 429 lines (80),
`no provider can take` lines,
`stage-quorum-unmet` lines (63),
`refiner round: 0/3 heard` lines (240) and the log's byte share (19.6 percent).

### Left unconfirmed by the check

The check did not examine these,
so they stand on the first reader's counts alone:
D4 to D7;
the quality-issue counts and the tense overlap;
the Hyper and OpenRouter balance series;
the stage timeline,
the outcome counts and the five unsettled slices;
the cache inventory (173 records,
2,476 payloads,
147 ledger files);
the plan-energy series and the 103 Wh figure;
the seat tallies other than Mercury;
the per-model notes on Kimi,
Qwen,
`minimax-m3`,
GLM and Gemma;
and the grace counts.
The check opened none of the unlisted files in the scratch directory.

## Could not be determined

- Whether OpenRouter billed the runaway stream above its estimate (D4);
  the meter lags and rounds to the cent.
- Whether the open issues of slices 43,
  57 and 80 are the tense ones;
  no artifact exists.
- What produced the 58 `refine` cache records,
  given 60 refinements were refused a cache entry.
- Why the Hyper 429 storm began at about 14:05 UTC (D7),
  and whether the cause is the number of slices in flight.
- The `SEAT` tally's accounting of re-asks and replayed answers (668 asked against 594 Mercury streams).
- The drain of the 250.6 Wh:
  it is not in the log and was taken from the owner's statement.

Answered since the read:
why the panel stage left no `stage-quorum-unmet` finding for slice 85 (D3,
by the second reader),
and whether a higher caller was meant to catch `EveryProviderDryError` (D1:
none does in this build,
and the fix of B818 adds the rethrows).

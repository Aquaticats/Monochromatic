# Translation repair history 2026-08-23: the produced-volume bound reaching the wire and the re-ask

The two gaps found reviewing the first produced-volume bound:
a claim no test supported and a re-ask that carried no bound,
why the producing bound could not be copied across,
what was measured and what was reasoned,
and the two producing chains the guard-proof experiment exposed.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## #185: the bound reaches the wire, and the re-ask is bounded too

Two gaps,
both found reviewing `#184` after it closed.
Neither disturbs its classification or its census;
the GAP-not-regression finding and the 947-emission bound both stand.

### The claim no test supported

`produced-volume-bound.unit.test.ts` hands `maxAnswerChars` straight to `drainBody`.
Between the producer and the drain the bound crosses five boundaries
as an optional field forwarded by conditional spread:
quorum,
round,
call,
client,
transport.
Delete any one of those spreads and the code still compiles,
the whole suite still passes,
and production silently returns to the state `#184` existed to end,
where the seam exists and nothing passes anything through it.

`produced-volume-threading.unit.test.ts` drives `produceTranslateSlate`
against a stub client that records what it was handed.
Run against the build from `#184`,
its producing assertion PASSED and its three re-ask assertions FAILED.
That split is both halves of the evidence at once:
the pass is the positive control proving the probe can see a bound when one is passed,
and the failures prove the second gap was real rather than theoretical.

### The re-ask carried no bound, and it is the busiest producing path

`repairOneCandidate` sends an invalid candidate back to its author
and gets a fresh rendering of the same slice.
That is a producing call by every measure that matters,
and it was policed only by the absolute 32000 cap.

Measured over 32 artifacts and 175 slices:
the re-ask fired on 96 slices,
274 findings.
Re-asked slices are short.
Source minimum 5 code points,
median 111,
maximum 291,
with 23 of them below the floor crossover at 64.
By model:
GLM-4.7-Flash 69,
Nemotron-3-Super 53,
gpt-oss-120b 47,
Kimi-K3 36,
GLM-5.2 31,
Qwen3.6-27B 31,
Qwen3.8-27B 7.

### Why the producing bound could not simply be copied across

The two wires are not the same shape.
`TranslateReportWire` is `{translation}` alone.
`TranslateRepairWire` is `{resolution, translation, explanation}`,
and `explanation` is free prose no source slice bounds.
The census that set sixteen measured slate candidate TEXT,
not envelopes carrying a second prose field.

On a 5-code-point slice the bound floors at 1024.
A correct repair there could be a 15-character translation
beside a 900-character explanation,
and would be refused.
A refusal costs the voice,
since `stage-round.ts` turns any stream error into `voice: { heard: false }`,
so on this path a false refusal loses a candidate on 55% of slices.

So the re-ask counts the findings alongside the slice.
The prior translation is deliberately NOT counted,
though the re-ask does send it:
it is itself bounded at sixteen times the source,
so counting it would permit roughly 272 times the source
and clear the 98.2 floor of every runaway observed here.
Detection would be gone.

### What is measured here and what is reasoned

The ratio is measured,
over 947 emissions.
The material it is applied to is reasoned,
because explanation length is recorded nowhere:
slate candidate records carry only `hash index origin producer text`,
`attempts.json` is a per-entry retry counter and names no stage,
and the only logs naming `translate-repair` are scripted test fixtures.
A probe that finds nothing was run against a control here:
the same search finds no stage name of any kind in those files,
because there is nothing of that kind in them.

Three things bound the risk of having reasoned it.
The widening only ever LOOSENS,
so it adds no refusal the producing path does not already make.
The widened bound still lands far below the 10381-character emission that opened `#184`,
which is what the fourth test case asserts.
And a false refusal costs one candidate,
after which the slice falls back to its other voices and to the incumbent,
which is where an invalid candidate went before the re-ask existed at all.

### The two producing chains, which the GFP experiment exposed

Removing the spread at `stage-round.ts` and rebuilding failed 2 of 4 cases,
not 4,
and the unbounded-request list held exactly one entry.
The reason is that the two producing paths do not share a chain.
Producing runs producer to `gatherStageVoices` to `runGatherRound` to `attemptStageCall`.
The re-ask runs `repairOneCandidate` straight to `attemptStageCall`,
skipping quorum and round entirely.

Removing the spread at `stage-call.ts`,
the hop both chains cross,
failed all 4 cases.
Both spreads were restored and the test returned to green.
So the produce assertions pin producer,
quorum,
round and call;
the re-ask assertions pin `repairOneCandidate` and call;
and no single deletion can silence both except at the hop they share.

### Scope, checked rather than assumed

`translate-retry.ts` holds only `judgeSlateWithRetry`.
Judging calls emit verdicts,
not translations,
so no source-derived volume bound applies to them.
Out of scope by shape,
not by oversight.

Of 15 `gatherStageVoices` call sites only the producer carries a bound,
and that is correct:
critics,
judges and probes emit commentary no source length bounds.

`producedVolumeBound` takes `materialChars` now rather than `sourceChars`,
because two callers count different things into it.
`MAX_PRODUCED_TO_SOURCE_RATIO` keeps its name,
which records where the number was measured
rather than what a caller may widen it against.

GATE:
suite 537 PASS / 0 FAIL / exit 0,
`lint` 0 warnings 0 errors,
`lint:types` exit 0.
Commit `b28954495`.

# Status: superseded consolidation cache generation 14 operation history

The unbounded continuous-correction and payload-memo rules of consolidation cache generation 14,
kept as history after the finite producer constraints replaced them.

Part of [Status](status.md),
which lists every file of this record.

## Superseded consolidation cache generation 14 operation history

This section records behavior of consolidation cache generation 14
(`CONSOLIDATE_CACHE_VERSION` 14 in the version notes of `src/consolidate-key.ts` (moved on 2026-10-18 to
`package/translation-repair/artifact/src/consolidate-key.ts`),
introduced by `cf14b379b` on 2026-08-29),
not of artifact schema generation 14,
and is not standing guidance.
The finite producer constraints in `Standing redesign constraints`
replace its unbounded correction and suspension rules.

A quality rejection is repair work,
not final `do not publish` answer.
Pipeline must continue stage-local correction from latest exact rejected text and latest structured findings
until strict publication gates accept or operational interruption pauses work.
No finite correction ceiling is authorized.
It must not restart whole entry merely because quality work remains.
Cancellation,
dark provider,
quota exhaustion,
transport outage,
and insufficient live seats leave resumable incomplete work.
They must not become quality judgement,
terminal refusal,
unsuccessful quality tally,
or `do not publish` result.
Operational interruption may pause or retry repair,
but may not publish or weaken publication gate.

One model plus one substantive prompt may contribute at most one provider payload per invocation.
Identity is exact model identity plus canonical ordered messages,
including roles and exact content bytes.
Run client memoizes first in-flight or completed payload and reuses it for exact duplicates;
it never sends second provider call.
Pipeline must never resend that task to same model to manufacture independence,
non-bias,
confirmation,
or extra sample.
Changing temperature,
response schema,
round number,
nonce,
whitespace,
or request metadata does not create independent sample.
Providers may silently enable deterministic prompt caching;
duplicate response would then be same computation,
not independent evidence.
Follow-up work must give model substantively distinct task grounded in prior result,
latest rejection,
or different review responsibility.

Same-model same-prompt transport retry is permitted only after connection failure,
reset,
or timeout before completed provider payload exists.
Once any payload exists,
including malformed,
truncated,
schema-invalid,
or semantically rejected payload,
that model and prompt may not be resent as another evidence draw.
Operational recovery and discarded payload together still produce at most one recorded response.
Malformed completed payload keeps memoized failed claim and cannot be resent.
Claims are invocation-local;
a later invocation replacing interrupted work may repeat provider request,
but its result is never independent or non-bias evidence against earlier payload.
Concurrent duplicate waits on first caller's exchange and signal;
it does not start or cancel separate provider call.
Corpus pass persists raw payload memo beneath `prompt-payloads/` in run root.
Corrupted record refuses before provider call.

`TALLY status=INCOMPLETE` means stage-local or operational work remains.
It is neither success,
quality verdict,
nor publication evidence.
Scheduler retains caches but does not queue fresh whole-entry attempt in same invocation.
Ordinary hard-cap or transport error remains `status=ERROR` and may resume measured cache progress.

Pipeline must function normally when only one provider is wet.
No provider family,
provider-specific seat,
or cross-provider response is mandatory.
Exact-half participation may come entirely from one provider;
strict quality,
corroboration,
and publication gates remain unchanged.
One wet provider is valid normal operating mode with reduced provider diversity to report,
not reason to stop repair or lower quality.
Too few live seats for stage participation pauses or retries work as operational state;
it is not quality rejection.

Ordinary operation permits either provider alone.
Validation and performance arms requiring both must pass `--require-providers synthetic,hyper`.
Harness verifies both keys and live non-dry meters before any model call,
then logs `REQUIRED-PROVIDERS synthetic,hyper status=wet`.

The live artifact also exposed misleading diagnostic wording:
it reported nine pair relations as though nine of eight source blocks had been paired.
`countPairedBlocks` now reports unique source and target reach separately from relation count,
with one-to-many and many-to-one guards.

Two fresh validations launched from `68c37da59` at 2026-08-29 12:14 UTC.
`Weideriche_` used `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829`.
Its first attempt refused slice 1 absolute naturalness at 4,840,305 milliseconds,
wrote no artifact or page,
and incorrectly queued whole-entry cache-warm reattempt.
Process was stopped after that behavior was rejected.

Pull request 386 `Carena0442` used exact pull-request files in isolated fixture at
`~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829`.
Process was stopped after same-model same-prompt repetition was rejected as independence mechanism.
It wrote no artifact or page.
Neither stopped run is publication or readiness evidence.
Artifact and log paths,
fixture method,
and failed startup non-evidence are recorded in
`~/temp/agent/pr386-Carena0442-run-provenance-20260829.md` and `doc/handover/translation-repair.md`.

The stopped runs are not completion-time samples.
Before another `Carena0442` launch,
completion-path work must target fresh-run median below two hours without weakening quality.
`doc/planning/translation-repair-entry-time-to-complete.md` defines phase attribution,
comparison controls,
completion evidence,
and objective.
Concurrent runs share provider capacity and are not matched runtime arms.

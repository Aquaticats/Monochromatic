# Drex hosted contract recheck

## Scope and result

Checked on 2026-10-14 after the user reiterated that `drex.nace.ai` looks promising.
This began as a public documentation and source recheck,
then expanded into separately authorized synthetic inference probes.
It is not a model-adoption decision.
The initial documentation recheck sent no provider assessment.
The separately approved synthetic boundary test recorded in this document then completed.
No account setting changed and no external message was sent.

The published Drex 2.0 context contract has changed materially since the
[release audit](../audit/tech-pi-auto-mode-drex-2-release-vet-2026-10-13.md).
The observed 16-question rejection remains unresolved.
The public source does not expose its deciding hosted guard.
The current measured question boundary is 512,
so the historical 16-question observation no longer excludes batching by count alone.
The later complete native assessment returned all question IDs outside the production deadline,
but failed the existing distribution validator and supplied no answer at the adopted operating cutoff.
The complete-input results are retained in the corresponding section.

The completed synthetic probes establish:

- 512 questions accepted;
  513 rejected.
- 4,194,304 serialized `state` plus `questions` bytes accepted;
  one byte more rejected.
- Adjacent repeated-text states accepted/rejected around the server's named 131,072-token state limit.
- An effective backend row boundary of 139,264 tokens,
  despite a published 262,144-token state-plus-longest-question allowance.
- A 244-question long-state response that succeeds outside the five-second production budget,
  followed by a larger combined case returning HTTP 529.

## Current hosted documentation

The [model reference][models] now specifies `drex-v2.0` limits of 131,072 state tokens
and 262,144 tokens for state plus longest question.
It explicitly says Drex 2.0 never delegates to another model;
over-limit requests are rejected.
The release audit recorded an 8,192-token state boundary,
a 16,384-token row boundary,
and fallback to Drex 1.5.
The new publication changes the documented contract;
it does not prove when the hosted deployment changed or which model would answer our complete request.

The [limits reference][limits] still specifies 512 questions and 4,194,304 bytes
for serialized `state` plus `questions`.
It names `at most 512 questions allowed` as the excess-question diagnostic.
The historical corrected native request retained 244 questions and occupied 692,952 bytes overall,
but returned `too many questions (244); limit is 16`.
Its observed rejection therefore still conflicts with the published question boundary.
The documented 55-second upstream timeout does not establish compliance with our five-second assessment budget.

The [evaluation schema][evaluate] requires one `state` and a `questions` map.
It documents answers under the original question IDs and ignores unknown top-level keys.
The [question guide][questions] continues to require text instructions and explicitly describes batching
up to 512 questions sharing one state.
Its `choice` type selects one label;
that does not supply independent binary probabilities for arbitrary combinations of policy findings.
Replacing independent questions with one choice is therefore an unqualified semantic redesign,
not an established encoding workaround.

## Public source and batching boundary

GitHub's current default-branch commit is still `be8ba7c8157241b9393862e9fcec104d0dc5caff`,
dated 2026-10-13T04:31:14Z,
as returned by the public repository commits API.
The current [runtime schema][runtime] allows structured instructions and declares
`questions: dict[str, Question] = Field(min_length=1)`.
Those differences from the hosted contract prevent treating this runtime as the hosted validator.

The [local request-format documentation][local-batch] describes a `requests` array
containing up to 1,024 decisions,
with ordered `results`.
`serve.py:74` checks that limit.
This establishes a local-runtime feature only.
The [hosted TypeScript SDK reference][sdk] describes `systemOne` with required `state` and `questions`;
it gives no corresponding hosted `requests` envelope.
Neither an SDK change nor nesting the existing requests has a source-backed claim to bypass the hosted rejection.

An uncapped,
hidden and ignored-file-inclusive `rg` search of the current public checkout inspected
`too many questions`,
`limit is 16`,
`max_questions`,
`MAX_QUESTIONS`,
`512`,
`1024`,
and `SystemOneBatchRequest`.
It found local batch guards and context constants,
but no matching hosted question guard.
The source commit matches the prior inspection;
no explanation of account-specific,
route-specific,
or model-specific enforcement can be established from it.

## Consequences and next evidence

Under the observed 16-question boundary,
244 independent questions require `ceil(244 / 16) = 16` calls.
Three calls carry at most 48 such questions.
Keeping the complete policy and receipt evidence while dropping question coverage does not preserve the assessment.
Keeping every question and using the published 512-question envelope would fit one call in question count,
but actual acceptance,
token fit,
returned model,
deadline compliance,
and semantic quality remain separate unverified gates.

The next discriminating evidence is a separately authorized public synthetic boundary test:
the same constant state and text-only binary question repeated under distinct IDs,
first with 16 questions as a positive control,
then with 17 questions as the boundary control.
Pin `drex-v2.0`,
use no retries,
retain raw errors and returned model identity,
and preserve the accepted deadline and call ceiling.
This separates the question boundary from private payload size and semantic quality.
Acceptance at 17 would disprove a universal 16-question cap for that exact route and time;
it would not establish complete native-request compatibility.

A later complete-input assessment would still need current policy,
source and receipt records,
original question IDs,
all independent questions,
and text instructions with verified round-trip preservation.
Neither boundary acceptance nor complete-request acceptance alone establishes semantic qualification.
No provider dispatch is authorized by this document.

## Approved synthetic boundary results

The user explicitly approved the prepared two-call schedule on 2026-10-14.
`mise run probe` in `${HOME}/temp/agent/drex-question-boundary-20261014`
sent the fixed public state `The card is green.`
and repeated `Is the card green?` questions,
with text criteria,
five-second deadlines,
and no retries.
The local dry run verified 2,246 bytes for 16 questions and 2,383 bytes for 17 questions.
Create-new start and result files preserve the completed schedule.

Both responses returned HTTP 200 and `model: drex-v2.0`.
The original question-ID sets matched exactly,
and every answer had type `noul` with a finite value between zero and one.
The 16-question control returned 16 answers in 1,258.515 ms and reported 422 input tokens.
The 17-question control returned 17 answers in 775.390 ms and reported 448 input tokens.
These single observations establish no latency comparison or reliability estimate.
All control answers were `0.8236`;
the boundary answers ranged from `0.8227` to `0.8236`.
This was a serving-boundary test,
not calibration or evidence for the `0.99` admission threshold.

The result disproves a universal 16-question cap for this route at the observed time.
It does not explain the historical rejection or establish acceptance of the retained 244-question native request.
The complete native input,
actual returned model,
original assessment deadline,
and semantic decision remain the next compatibility boundary.
No production route changed.

## Authorized request-limit exploration

The user requested more Drex requests to establish actual limits on 2026-10-14.
The next phase uses synthetic data only and makes sequential requests to the same pinned model and route.
It tests independent question-count,
state-length,
longest-question,
and serialized-body boundaries.
It does not test account concurrency or rate limits by saturating the service.

Question probes begin at 32,
128,
244,
512,
and 513 questions sharing the same short public state.
State probes repeat `x ` with one unchanged short question.
Longest-question probes keep the state short and vary only repeated `x ` in question instructions.
Body probes use exactly 4,194,304 and 4,194,305 serialized `state` plus `questions` bytes,
with whitespace padding to distinguish byte validation from ordinary text length.
Any earlier token rejection remains a separate boundary;
whitespace padding is not evidence of ordinary-text capacity.

The runner retains every attempted payload hash,
byte count,
status,
raw response,
returned model,
answer coverage,
reported token usage,
and elapsed time.
An observed transport timeout is not interpreted as a server size limit.
Large-input requests have a 60-second observation ceiling to reach the documented 55-second server timeout;
each result separately records whether it met the unchanged five-second production budget.
No production timeout or resource limit changes.
The finite exploration ceiling is 32 requests,
one in flight,
with no automatic retry and a stop on authentication,
credit,
rate,
or service-capacity failure.
The existing worker bounds are 4 GiB memory and two CPUs;
the Node heap is additionally limited to 128 MiB.

### Measured question-count boundary

`mise run probe questions` in `${HOME}/temp/agent/drex-request-limits-20261014`
completed every frozen question-count case without retries.
Counts 32,
128,
244,
and 512 returned HTTP 200,
`model: drex-v2.0`,
and exactly the original typed answer sets.
The 513-question request returned HTTP 422 with
`questions: at most 512 questions allowed`.
The 512-question request took 1,596.205 ms for the synthetic short state;
this is not a measurement of a complete native request with 512 questions.

The accepted 512 and rejected 513 establish the hosted question-count boundary for the observed route and time.
The historical 16-question failure no longer excludes this route on question count alone.
The deciding historical hosted implementation remains unavailable,
so this result does not identify when or why that behavior changed.

### Measured serialized-body boundary

`mise run probe bytes` accepted exactly 4,194,304 serialized `state` plus `questions` bytes
with HTTP 200 and `model: drex-v2.0`.
Adding one byte returned HTTP 422 with `request body must be at most 4194304 bytes`.
The complete HTTP body also contains the model field,
so its corresponding accepted size was 4,194,324 bytes.
This matches the documented counted-field scope,
not a limit on the entire HTTP body.

The successful control used whitespace padding and reported 32,795 input tokens.
It establishes the byte boundary without pretending that ordinary text has the same token density.
Its 8,632.385 ms elapsed time exceeded the unchanged five-second production budget.
The rejected control returned in 3,342.758 ms.
Single-run durations do not establish reliable latency or comparative performance.

### Measured state-length boundary

`mise run probe state` kept one short question fixed and varied only `state = 'x '.repeat(n)`.
Repetition counts 8,192,
65,536,
and 131,070 returned HTTP 200 with `model: drex-v2.0` and the original answer.
Counts 131,071,
131,072,
and 131,073 returned HTTP 422 naming `exceeds the 131,072-token state limit`.
The accepted 131,070 repetitions and rejected 131,071 repetitions establish adjacent bounds for this exact input family.
They are not independently measured tokenizer counts.

The largest accepted state contained 262,140 unquoted UTF-8 bytes;
its complete request occupied 262,320 bytes.
It returned in 2,729.279 ms and reported 133,978 input tokens.
That reported usage is not the state-only limit counter:
it exceeds the named state limit even though this request was accepted.
Do not substitute billing usage or a repetition count for the server's state-token accounting.
Every request in this phase completed within five seconds in its individual observation.

### Measured state-plus-longest-question boundary

The hosted backend contradicts the published 262,144-token row limit.
With the fixed short state and 262,100 repetitions of `x ` in one question's instructions,
HTTP 422 reported
`question requires 262122 branch tokens; backend row limit is 139264 including state`.
Larger probes reached the separately worded 262,144-token validation error.
These distinct responses establish different observed validation boundaries;
they do not establish the hosted deployment's source-level cause.

Targeted adjacent probes kept every other byte fixed:
139,236 repetitions returned HTTP 200,
`model: drex-v2.0`,
one original answer,
and 139,264 reported input tokens.
The request took 4,419.861 ms.
139,237 repetitions returned HTTP 422 with `Request schema or context limit is invalid`.
The observed adjacent boundary agrees with the backend's named 139,264-token row limit.
The generic adjacent error alone would not establish that number;
the larger request's explicit backend diagnostic supplies it.

The initial schedule and adjacent controls consumed 20 requests.
The remaining bounded investigation adds three combined cases:
244 questions with state repetition counts 8,192,
65,536,
and 131,070.
These retain synthetic inputs and measure whether independent accepted boundaries combine.
They make no claim about current native input fidelity or semantic quality.

### Combined inputs and terminal outcome

`mise run probe combined` kept 244 questions and varied the synthetic state length.
The 8,192-repetition case returned HTTP 200 and every original answer in 4,431.266 ms.
The 65,536-repetition case returned HTTP 200 and every original answer in 25,155.119 ms.
Both report `model: drex-v2.0`.
These are individual observations,
not an unchanged-input latency distribution or a causal performance comparison.
The latter successful call does not fit the existing five-second production budget.

The 131,070-repetition case returned HTTP 529 after 33,331.147 ms,
with `overloaded`,
`Drex is temporarily unavailable. Retry shortly.`,
and `retry-after: 2`.
The frozen terminal-service-failure condition stopped the runner with exit one.
It did not retry or dispatch any remaining probe.
This is an observed service failure,
not a demonstrated permanent input-size limit or proof of the documented 55-second timeout.

The extended phase completed 23 attempts:
12 HTTP 200 responses,
10 HTTP 422 responses,
and one HTTP 529 response.
Every start record has a retained result record.
Together with the separately approved 16/17 controls,
the recheck made 25 provider requests.
Every successful response reports `drex-v2.0` and passed original answer-ID and Noul-value validation.
The extended phase reports 738,025 input tokens across successful responses;
the unsuccessful responses report no usage,
so no invoice-total claim follows.
No account-rate,
concurrency,
or saturation testing was performed.
No production route,
threshold,
deadline,
or source-retention policy changed.

## Complete native input on 2026-10-14

The user explicitly approved the prepared complete-input check,
then renewed authorization to send anything needed to Drex and Jev and use credits freely.
The frozen native fixture preserves 73 source occurrences and 84 original questions.
Only the model and required text encoding of question instructions differ from the captured Jev request.
JSON round-trip comparisons recover every original question instruction;
the complete state is unchanged.
The request measures 541,947 bytes and has SHA-256
`3d204aa86fb17dd3de7ad9c320ef2b6db65200badc27dd950b16189750fba410`.

The first call at `${HOME}/temp/agent/pi-native-drex-capacity-20261014`
ended with `TimeoutError` after 5,005.129 ms.
It made no retry and produced no usable response.
A separate observation at `${HOME}/temp/agent/pi-native-drex-observation-20261014`
retained the exact same request bytes and allowed 60 seconds for diagnosis only.
That attempt reached HTTP 200 and returned `drex-v2.0` with all 84 answer IDs.
It failed local answer validation at 48,528.415 ms.
The response reports `evaluation_time_ms: 46615.903`,
2,418,163 input tokens,
and 8,381 output tokens.
These are provider-reported usage values,
not independently reconciled billing.

The local error is
`QuestionSelectionError: Choice answer is not a complete distribution with a highest-probability selected option`.
The returned distributions include 38 sums outside the consumer's `0.000001` tolerance;
the largest observed deviation is `0.0002`.
The [troubleshooting analysis](../troubleshooting/drex-model-route-label.md#complete-native-choice-distributions-and-latency)
compares that result with the public four-decimal serializer.
No production validator was relaxed and no reported probability was normalized.

None of the 84 selected scores reaches the adopted `0.99` cutoff.
The explicit primary clause,
`RUL: The complete proposed operation may read proof.txt in this directory.`,
received `none` with probability `0.3586` and permission probability `0.0153`.
This is an abstaining or incorrect interpretation result for this representation,
not a basis for loosening the cutoff or declaring general model quality.
The result establishes request acceptance,
not suitability for the installed assessment.

A frozen follow-up at `${HOME}/temp/agent/pi-drex-clause-isolation-20261014`
keeps the complete state and tests only that primary clause.
Two unchanged baselines measure variability,
then a third request places the same original definition and option rubrics directly in the question.
The baseline request hash is
`fe5885a059a33e43efb1e641cf17ccbbcb11e6bd624506d2dd51d031c726c321`;
the inline-definition request hash is
`ff94876634e47353bed80521f979630477fdfdb5c84335c664f36080f0cacac4`.
There are at most three calls,
no retries,
and no production admission or tool execution.
This diagnostic subset is not a replacement for complete source interpretation.

All three clause-isolation requests returned HTTP 200.
The unchanged baselines completed in 2,254.570 ms and 1,342.972 ms,
with server evaluation times of 331.013 ms and 320.696 ms.
Both returned the identical permission probability `0.7748` and usage of 15,671 input tokens.
The inline-definition request completed in 1,338.757 ms,
reported 20,607 input tokens,
and returned permission probability `0.3473`.
The direct-definition variant is not a verified improvement.
Its latency difference from the second baseline is inside the observed unchanged-input spread.
None of these results reaches `0.99`.

The primary question and complete state are unchanged between the isolated baseline and full batch,
but the observed primary-clause distributions differ.
That is a batch-sensitivity hypothesis,
not an established hosted mechanism or proof of cross-question attention.
A frozen full-repeat and two-question control were prepared at
`${HOME}/temp/agent/pi-drex-batch-isolation-20261014`.
The unchanged full-repeat returned HTTP 529 `overloaded` after 26,134.675 ms.
The schedule stopped without retry,
so the two-question control was not dispatched.
The public source describes isolated branches and separate row chunks;
its relationship to hosted behavior is unverified.

The next local inspection distinguishes evidence from injected assessment-status fields.
The current wire includes `permission: false`,
`authority: "not-established"`,
and `semanticQualification: "not-established"` before interpretation.
Those values describe the guard's current status,
not the permission meaning of the selected clause.
Whether they confuse the model is untested;
their mere presence does not prove the cause of the observed estimates.
No production payload,
cutoff,
provider,
or deadline changes follow from this inspection.

## Frozen representation diagnostics

Standing user authorization covers the next finite diagnostic at
`${HOME}/temp/agent/pi-drex-representation-isolation-20261014`.
Its ranked hypotheses are guard-status confusion,
JSON-string program indirection,
and inability to apply the existing rubric confidently even to a direct permission.
The schedule makes at most five sequential calls with no retry and stops on any non-200 response.
Each call has a 60-second observation ceiling;
the production deadline remains five seconds.

Two unchanged isolated-clause baselines measure response variability.
One variant removes only `effectFactCoverage`,
`authority`,
`semanticQualification`,
and `permission` from the shared state.
Another independently parses `parentProgram` and `preparedProgram` as JSON values,
with exact reserialization checks against their original strings.
Both retain every source and the original question unchanged.
A final minimal control keeps the primary policy,
programs,
definition,
and established effect facts but deliberately removes surrounding source context.
That control is diagnostic only and cannot authorize an action.
The manifest freezes every input hash before dispatch.
No result from this schedule changes the installed provider or operating policy.

All five requests completed with HTTP 200 and `model: drex-v2.0`.
The unchanged baselines returned identical permission probability `0.6906`
in 2,069.978 ms and 1,252.905 ms.
Removing guard-status fields returned `0.7468` in 1,184.061 ms;
parsing the programs returned `0.6533` in 1,327.715 ms.
The minimal control returned `0.8576` in 409.473 ms.
No result reaches the adopted `0.99` cutoff.

The same baseline hash previously returned `0.7748`.
Its observed cross-study range is therefore `0.0842`,
larger than the status-removal increase of `0.0562` in this study.
The within-study repeats alone would miss that variation.
These observations do not establish a beneficial causal effect from removing status fields,
a stable semantic encoding remedy,
or the reason for hosted response variation.
The parsed-program variant does not improve the observed estimate.
The minimal control shows that surrounding source volume is not the only obstacle
to crossing the current cutoff for this rubric.
All provider work in this schedule is terminal;
no production request,
validator,
provider,
or operating cutoff changed.

[models]: https://console.nace.ai/docs/drex/reference/models
[limits]: https://console.nace.ai/docs/drex/reference/limits
[evaluate]: https://console.nace.ai/docs/drex/api-reference/systemone
[questions]: https://console.nace.ai/docs/drex/guides/questions
[runtime]: https://github.com/nace-ai/drex-decision-models/blob/be8ba7c8157241b9393862e9fcec104d0dc5caff/kev/api.py
[local-batch]: https://github.com/nace-ai/drex-decision-models/blob/be8ba7c8157241b9393862e9fcec104d0dc5caff/docs/request-format.md
[sdk]: https://console.nace.ai/docs/developer-tools/typescript

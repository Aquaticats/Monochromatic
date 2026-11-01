# LLM Gateway Decisions candidate checks

## Luna adoption and first compensation experiment

The user subsequently selected Luna and accepted an inclusive `0.80` operating cutoff.
Perplexity is excluded from further trials.
The historical `0.99` gate retained in the initial probe record is no longer the Luna operating requirement.

The first frozen comparison uses
`${HOME}/temp/agent/pi-luna-explicit-rubrics-20261014/`.
It makes three unchanged baseline observations,
then three observations with existing definition instructions placed directly in each question
and existing criterion text supplied through each choice's `description`.
This follows the documented
[OpenAI choice schema](https://developers.openai.com/api/reference/resources/decisions/methods/create).
The exact shared input,
original question binding JSON,
question order,
choice labels,
and all original rubric text remain present.
No normative definition is rewritten and no source is removed.

The baseline body is 584,007 bytes with SHA-256
`59611db7d2fc22abc491db6bb41fe69bac90769f78c23d1f1409799fa78d4350`.
The explicit-rubric body is 910,146 bytes with SHA-256
`bd4f709f705d7ad1a4f035bd796229897a655ceadf68c252afd84245b626e15e`.
Each independent observation makes one request with a five-second deadline.
Transport failure,
refusal,
malformed answer,
or late completion ends the remaining schedule without retry.
The schedule measures unchanged-input spread before comparing formats;
it does not stop early when a score crosses the cutoff.
Higher admission counts alone do not establish correctness.
No represented tool operation executes during this study.

Automatic approval review initially rejected the changed-payload export before execution.
The user then explicitly authorized the prepared comparison.
All six frozen requests completed with HTTP 200,
exact model identity,
all 84 answers,
and no refusals.
Baseline durations were 2,101.300,
1,263.857,
and 1,323.376 ms;
explicit-rubric durations were 1,418.420,
1,934.000,
and 1,559.026 ms.
The overlapping observed ranges do not establish a latency improvement.
Each format returned byte-equivalent answer data across its three runs.
The baseline admitted four answers at `0.80` each time;
the explicit format admitted six.
Four absence answers newly crossed the cutoff and two previously admitted answers fell below it.
The primary permission answer remained `0.83`.
The private `comparison.json` retains every changed answer.
This formatting change alone does not resolve the complete assessment.
It is not yet adopted in the maintained adapter,
which currently implements the original lossless conversion.

## Question-batch compensation experiment

The next frozen diagnostic tests whether grouping the same questions changes Luna's uncertainty.
It preserves the exact baseline input in every request
and partitions the original ordered 84 questions into consecutive groups of 28.
Concatenating the groups reconstructs every original question exactly.
No instruction,
criterion,
source,
or complete operation text is removed from the original assessment.
The three request bodies measure 331,684,
439,618,
and 393,323 bytes.
Their hashes and offline reconstruction receipt live in
`${HOME}/temp/agent/pi-luna-question-batches-20261014/manifest.json`.

The schedule contains three repetitions of one complete assessment.
Each repetition sends its three groups concurrently under one shared five-second deadline,
with no retries and at most nine calls across the schedule.
Any refusal,
transport failure,
invalid answer,
or expired deadline cancels sibling calls and ends the remaining schedule.
The built adapter validates every named answer before the groups are combined.
The offline preflight already decodes the matching slices of retained real Luna responses.
No represented operation executes.
This is a diagnostic;
the production judgment still owns its original attempt accounting,
and no hidden subrequests have been added to the maintained sender.
Higher admission counts alone cannot establish greater accuracy.

Automatic approval review rejected dispatch before execution
because the prior approval covered the six-call comparison,
not this new nine-call schedule for the assessment data.
The user then explicitly authorized the nine calls
and gave standing authorization for further Luna calls needed by this task without a total call cap.
Individual experiments still retain finite schedules,
explicit stopping conditions,
and the original production judgment bounds.

All nine calls completed with valid answers and no refusals.
The complete parallel judgments took 1,993.272,
1,433.419,
and 1,182.940 ms.
Every repetition returned exactly the original baseline's 84 categorical estimates,
including independent confidence values;
four estimates met the `0.80` admission cutoff.
Each repetition used 246,783 reported input tokens,
compared with the baseline's 126,129.
The retained explicit-rubric responses provide a positive control for detecting answer changes.
`baseline-comparison.json` retains the complete comparison receipt.
Question batching is rejected as a compensation for the uncertainty observed in this input.
The maintained single-call sender remains appropriate for these measured results.

## Readable selected-text experiment

The next frozen diagnostic repeats each clause's exact selected text as readable source data
beside its unchanged original JSON question binding.
The shared input,
definitions,
choices,
question membership,
and source bytes remain intact.
The added framing explicitly treats the repeated text as evidence to assess;
it supplies no authority or new instruction.
This tests presentation of nested escaped text,
not a new interpretation rubric or permission policy.

The current [OpenAI guide][openai-decisions-guide] recommends distinct questions and observable criteria.
That guidance does not establish a remedy for this input's uncertainty.
The retained candidate has 84 questions and measures 827,607 bytes with SHA-256
`c078a29a3c3eb26f768d5a92d57255ce9ac735e6bdcf8ca8ae7d018b7206e728`.
The offline preparation checks unchanged shared input,
exact original question bindings,
and every original choice.
Artifacts live in `${HOME}/temp/agent/pi-luna-readable-selection-20261014/`.

The user's standing Luna-call authorization covers three sequential observations,
each with one request,
the original five-second deadline,
no retry,
and no represented tool execution.
A failure or invalid response ends the schedule.
Compare full answer data with the measured baseline and retain independent confidence.
An increased admission count alone cannot establish correctness or justify adoption.
All three calls completed with the exact model identity,
84 valid answers,
and no refusals.
Durations were 2,332.014,
2,028.370,
and 1,435.594 ms.
Each response reports 178,834 input tokens.
Answer data is identical across repetitions;
every distribution differs from the original baseline.
Eight supported answers reach `0.80`,
and the primary permission estimate is `0.82` with independent confidence `0.78`.
The remaining answers still prevent a complete admitted interpretation.
The candidate therefore stays experimental:
it changes estimates but supplies no independently labeled accuracy result or complete operating remedy.
Retain the maintained lossless adapter and stop repeating representation-only trials on this input.
Next inspect the original source-view boundaries and continue the maintained native quality queue.

[openai-decisions-guide]: https://developers.openai.com/api/docs/guides/decisions

## Historical initial scope and authorization

On October 14,
2026,
the user enabled `microsoft-decision-1` and `gpt-6-luna-decisions` on the existing LLM Gateway key.
Exercise that supplied access before resuming other work.
These are compatibility probes for the existing migration,
not production provider adoption.
The initial probe retained its five-second deadline,
three-call ceiling,
and historical 0.99 unique-maximum admission threshold.

## Current contract

The [Gateway guide][guide] places these models on `POST /v1/decisions`.
The [API reference][reference] accepts provider-prefixed model identifiers.
Requests use text input and named question arrays;
answers use arrays and may contain explicit refusals.
Structured state and instructions are serialized as exact JSON text.
Choice criteria become value/description entries;
null descriptions are omitted and restored to null by the probe's inverse mapping.
The offline inverse reconstruction exactly matched the captured original state and all 84 question objects.
This proves information preservation,
not equivalent model interpretation.

Both candidates are managed services reached through the incumbent gateway.
No SDK or executable dependency is added.
Local dependency artifact gates are inapplicable;
provider semantic,
operational,
and contract qualification remain pending.

## Frozen schedule

Private directories are `${HOME}/temp/agent/pi-gateway-microsoft-20261014`
and `${HOME}/temp/agent/pi-gateway-luna-20261014`.
Each pins its provider-prefixed model and first sends the same one-question literal-label Choice control.
Only a complete valid response inside five seconds permits its complete native assessment request.
Every attempt has a five-second deadline,
no automatic retry,
and no represented execution.
Refusals,
unexpected model names,
missing or duplicate answers,
invalid distributions,
and late responses stop that model's schedule.
Only the necessary Mise-resolved key enters the probe process.

The Microsoft control is 361 bytes with SHA-256
`0e0aceb2250961035809393fddaac7844f9a8c85045b5456bfc8e570f9acabcb`;
its complete request is 584,006 bytes with SHA-256
`430ce8f2c1a8ad985b9a823ce937ac08bca5ebbb3fa3a31008cbbc4d35020413`.
The Luna control is 362 bytes with SHA-256
`82cec21afd1d44c96419b519bfff49228cedfe375490641c6ce4bdbb900861da`;
its complete request is 584,007 bytes with SHA-256
`59611db7d2fc22abc491db6bb41fe69bac90769f78c23d1f1409799fa78d4350`.

## Verified access and pending complete-input permission

Automatic approval review rejected the original combined launch before execution:
it treated model enablement as permission to test access,
without permission to export the complete private assessment to the new model destinations.
A concrete permission question named both prepared complete inputs.
The user answered `Authorize both complete-input probes`.
Separate `-complete` directories now dispatch each original frozen complete request once,
reusing the successful controls rather than repeating them.

Separate synthetic-only directories append `-access` before the date.
Their preflight asserts the literal fixture and single question before dispatch.
Both returned HTTP 200 with the exact provider-prefixed model and complete valid Choice answer.
Microsoft returned `permission` at 0.9989888137654562 in 1,805.284 ms,
with confidence 0.9987865765185475 and usage 65 input tokens plus one reported output token.
Luna returned `permission` at 0.99 in 991.616 ms,
with confidence 0.99 and usage 136 input tokens plus zero output tokens.
Both satisfy the numeric cutoff on this literal control.
Neither result establishes policy interpretation,
complete-input capacity,
or repeated-run latency.

## Complete-input results

The first local preflight incorrectly required six choices for every question.
The original capture also contains three-choice questions,
so Node emitted `AssertionError [ERR_ASSERTION]: 3 !== 6` before any dispatch.
The corrected preflight validates unique choices and reconstructs every original criterion and instruction.
Both approved request hashes remain unchanged.
The original preflight source is preserved beside each corrected runner.

Microsoft's single complete-input attempt returned HTTP 422 in 2,221.452 ms:
`invalid TypeSafe request: request is about 145150 tokens; the limit for the state plus all questions is 64000`.
The diagnostic names the TypeSafe request representation despite the selected Microsoft route.
No model answer or returned model identity accompanies it;
do not treat this response as proof of the underlying Microsoft model's own context limit.

Luna returned HTTP 200 with `openai/gpt-6-luna-decisions`
and all 84 expected Choice answers in 2,087.943 ms.
There were no refusals.
Usage was 126,129 input tokens and zero output tokens.
Every distribution passed the frozen finite-value,
label-membership,
and approximate-sum checks.
None met the existing 0.99 unique-maximum admission rule.
The primary permission answer was 0.83 with confidence 0.8.
Across the complete answer map:

- Seven selected `permission`,
  with selected probabilities from 0.3 to 0.83.
- Six selected `unknown`,
  from 0.33 to 0.51.
- Seventy selected `none`,
  from 0.35 to 0.9.
- One selected `potential-instruction` at 0.96.

The successful response establishes capacity and deadline compliance for this single attempt.
It does not establish repeated latency,
calibration,
or production semantic admission.
Both approved attempts are complete without retry.

[guide]: https://docs.llmgateway.io/features/decisions
[reference]: https://docs.llmgateway.io/v1_decisions

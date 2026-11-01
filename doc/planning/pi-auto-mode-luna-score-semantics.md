# Luna choice probability and confidence

## Question and scope

The user asked whether the current `0.80` threshold is applied to the correct quantity.
This October 14,
2026 note establishes the external response contract.
The separate implementation trace owns which returned field actually reaches admission.
No inference request,
production edit,
or new acceptance rule is part of this research.
After this review,
the user explicitly requested lowering the selected-option probability threshold to `0.60`.
The [admission decision](../decision/pi-auto-mode-semantic-admission.md) records that subsequent action;
the `0.80` comparisons in this note retain the reviewed policy and original measurements.

## The returned fields answer different contract questions

The [OpenAI Decisions guide][guide] describes Choice as selecting a supplied option
and returning a probability distribution over the options plus a separate `confidence` field.
In plain language,
each option's probability describes the model's allocation to that candidate answer.
`confidence` is another returned quantity about the answer;
it is not the selected option's probability under another field name.
The guide's illustrative choice has winning probability `0.95` and confidence `0.93`.

The [API reference][reference] lists `confidence` as a number alongside `choice` and `probabilities`.
It does not provide a confidence formula,
define a separate correctness event for that number,
or establish statistical independence from the option probabilities.
Calling it a separate field is supported.
Calling it an independent estimator of correctness would exceed the inspected contract.

The guide tells application authors to choose thresholds using labeled application examples
and the costs of false positives and false negatives.
It does not prescribe `0.80`,
require that cutoff on confidence,
or say one numeric cutoff is interchangeable across these fields.
The reference also permits a per-question refusal;
refusal handling remains separate from a numeric threshold.

## What Gateway does with the fields

The [Gateway Decisions guide][gateway-guide] exposes both per-option probabilities and confidence.
Its example has a selected probability of `0.88` and confidence `0.81`.
That example establishes the distinction in its public wire shape,
not calibration for our input.

The [gateway source repository][gateway-source] was inspected at
`ebbeaf029ec4fbddf1a23ae03e46bf7c1c7d9259`,
reconfirmed as public `main` during this refresh.
In `apps/gateway/src/decisions/decisions.ts:198`,
OpenAI selects the native Decisions branch.
At line 324,
the normalizer preserves upstream response fields with an object spread and replaces the model label.
It does not recalculate option probabilities or confidence in that branch.
This is source evidence;
the hosted deployment's matching revision is unestablished.

## Implication for the threshold question

### Measured local consumer

The built `createNativeSemanticOperatingPolicies` consumer was called against every answer in
the retained `baseline-1-response.json` with its original vocabulary on October 14,
2026.
The replay made no provider requests.
`${HOME}/temp/agent/pi-luna-score-semantics-20261014/measurement.json`
retains the original answer identifiers,
selected probabilities,
confidence fields,
and actual built admission results.

`package/pi-plugin/auto-mode/src/semantic-operating-choice.ts:143`
selects `probabilities[choice]`;
line 149 compares it with the `0.8` constant.
The baseline contains 83 clause-operation questions and one whole-request content question.
Only three clause answers and the content answer pass.
The clause selections include 70 `none`,
seven `permission`,
and six `unknown` answers.
These are model selections,
not independently labeled ground truth.

The primary permission answer has probability `0.83` and confidence `0.80`.
A rejected `none` answer has probability `0.74` and confidence `0.69`.
The whole-request answer selects `potential-instruction` at probability `0.96`
and confidence `0.94`.
Substituting a confidence-only `0.80` threshold in this baseline selects exactly the same four answer identifiers.
That counterfactual is a calculation on retained outputs,
not a production change or a new inference result.

`package/pi-plugin/auto-mode/native/contract/integration/action-policy/clause-relation.mjs`
asks what relationship one selected clause expresses toward the complete proposal.
Its instructions explicitly exclude determining the final action verdict.
`compiler.mjs:209` retains an unadmitted clause as an unresolved interpretation obligation.
Thus the current cutoff concerns individual meanings,
including claims that a clause contributes no operative rule.
It does not estimate an overall probability that execution is permitted or that the whole decision is correct.
The count of admitted answers measures coverage under this particular decomposition,
not model accuracy or the fraction of actions that can be handled correctly.

### Remaining policy question

The local decision at the time of this review defined admission of a supported selected meaning at `0.80`
with a unique lead.
Comparing the selected meaning's returned probability with that boundary is consistent with
a threshold on support for that meaning.
It is not a check that the separate `confidence` field reaches `0.80`.
Neither field alone establishes that the entire proposed operation is authorized.

Adding a confidence boundary would be an additional acceptance condition,
not a naming correction.
Replacing the probability boundary with confidence would define a different condition.
For example,
a hypothetical answer with selected probability `0.90` and confidence `0.70`
passes a probability-only `0.80` boundary but fails a confidence `0.80` boundary.
This is an illustration of the predicates,
not an observed Luna response or a recommendation to change admission.

The response contract therefore supports keeping the quantities distinct and naming the field being thresholded.
It does not by itself establish which condition best meets this application's error tolerance.
That conclusion requires the local question definitions,
actual admission consumers,
and labeled examples rather than renaming probability as confidence.
The user's accepted `0.8+` operating preference does not establish that requiring a uniquely selected meaning
for every candidate is the best way to reduce manual request judgment.
That decomposition remains a separate design question.
Any alternative must demonstrate which unresolved distinctions can change the code-owned action result;
it cannot simply discard low-scoring candidate restrictions or multiply these scores into a purported overall accuracy.

[guide]: https://developers.openai.com/api/docs/guides/decisions
[reference]: https://developers.openai.com/api/reference/resources/decisions/methods/create
[gateway-guide]: https://docs.llmgateway.io/features/decisions
[gateway-source]: https://github.com/theopenco/llmgateway

# Drex API full-policy routing and request compatibility

## Drex 2.0 backend rejects rows below its published context limit

### Symptom

On 2026-10-14,
the pinned hosted `drex-v2.0` route rejected a synthetic question with HTTP 422:
`question requires 262122 branch tokens; backend row limit is 139264 including state`.
The [current model reference](https://console.nace.ai/docs/drex/reference/models)
advertises a 262,144-token state-plus-longest-question limit.
The separately named state limit is 131,072 tokens.
This incident is distinct from the historical question-count rejection:
the current synthetic controls accept 512 questions and reject 513.

### Cause and source boundary

The observable boundary is a disagreement between the published row limit and the backend's returned limit.
The public runtime checkout remains `be8ba7c8157241b9393862e9fcec104d0dc5caff`.
Its `kev/model.py:153` checks available row capacity while encoding a branch:

```python
# nace-ai/drex-decision-models, kev/model.py
if len(br) > max_branch - len(S):
    raise ContextOverflow(f"branch too long: {len(br)} tokens with a {len(S)}-token state (row limit {max_branch})")
```

This is relevant comparable code,
not proof of the hosted guard's implementation.
An uncapped `rg --hidden --no-ignore` search across the complete checkout found none of
`question requires`,
`backend row limit`,
or `Request schema or context limit is invalid`.
The inspected runtime's diagnostic differs from the returned hosted diagnostic.
No hosted configuration or serving-weight identity was available for inspection.
Matching Drex 1.5's published row number does not prove fallback:
successful controls explicitly report `model: drex-v2.0`.

### Verification

The private fixture is `${HOME}/temp/agent/drex-request-limits-20261014/probe.ts`.
Its create-new evidence files retain payload hashes,
raw responses,
answer sets,
reported models,
and elapsed times.
The [request-limit report](../planning/pi-auto-mode-drex-current-contract.md)
records the user's authorization,
resource boundaries,
and complete schedule.
The historical invocation is:

```bash
# Run from ${HOME}/temp/agent/drex-request-limits-20261014; retained destinations refuse a repeat.
mise run probe row
mise run probe row 139236
mise run probe row 139237
```

Working cases use the same short state and one binary question:
131,072 and 139,236 repetitions of `x ` in its instructions return HTTP 200 and `drex-v2.0`.
The latter reports 139,264 input tokens and returns the original question's answer.

Failing cases retain distinct error variants:

- 139,237 repetitions return `Request schema or context limit is invalid`.
- 262,100 repetitions return the explicit backend limit of 139,264.
- 262,140,
  262,144,
  and 262,145 repetitions return the separately worded 262,144-token row-limit error.

The adjacent accepted/rejected inputs establish the exact boundary for this fixture family.
They do not expose the hidden tokenizer count independently.

### Handling and rejected interpretations

No production workaround is installed.
The successful shorter fixture is a boundary control,
not permission to shorten the complete policy or discard source receipts.
The report separates service acceptance from the unchanged five-second production assessment budget.

The following interpretations are unsupported:

- Assuming the published 262,144-token row size is currently usable.
- Inferring a Drex 1.5 fallback solely from the backend limit number.
- Replacing state-token accounting with billed input-token totals.
- Treating HTTP 200 on synthetic inputs as semantic qualification or native-request fidelity.

### Upstream filing decision

No issue,
comment,
or external message is drafted or sent.

1.  Upstream fault: a contract discrepancy is reproduced;
    its hosted code or configuration cause is not established.
2.  Fixability: unknown without the deciding implementation;
    no impossibility or effort estimate is asserted.
3.  Supported use: the current hosted model and limits references advertise the tested row boundary.
4.  Contribution policy: not investigated because no external contribution is proposed.
5.  Maintainer willingness: unknown;
    no communication was sent.
6.  Prototype: controlled consumer requests reproduce the boundary;
    no source-grounded hosted patch can be built from the different public runtime.

## Drex 2.0 rejects the complete native question batch

### Current contract recheck on 2026-10-14

The [current factual recheck](../planning/pi-auto-mode-drex-current-contract.md)
finds changed published Drex 2.0 context limits and an explicit no-fallback statement.
The hosted 512-question documentation still conflicts with the retained 16-question rejection.
Public source remains at `be8ba7c8157241b9393862e9fcec104d0dc5caff`
and does not contain the deciding hosted guard.
Its local `requests` batch envelope does not establish support on the hosted route.
The user separately approved the prepared 16/17-question synthetic controls.
Both returned HTTP 200 with `model: drex-v2.0` and every expected answer.
The [result record](../planning/pi-auto-mode-drex-current-contract.md#approved-synthetic-boundary-results)
disproves a universal 16-question limit at the observed time;
it does not explain the historical rejection or verify the complete native request.
No workaround or production model replacement is established.
The existing upstream filing decision remains unchanged.

### Symptom

On 2026-10-13,
the hosted `POST https://console.nace.ai/v1/systemone` endpoint returned 422 for a disposable native Pi request
containing the complete current policy and 244 questions.
The requested model was `drex-v2.0`.
The original wire failed with `questions.head_0.instructions: must be a string`.
Its issue list reports the same instruction-type failure for every question.
After lossless conversion of instruction objects to JSON text,
the response was `too many questions (244); limit is 16`.
Neither rejected response reported a serving model.

### Cause and source boundary

The owned Jev adapter supplies structured instruction objects.
The current [hosted question schema](https://console.nace.ai/docs/drex/guides/questions)
requires strings.
The first error is therefore an adapter incompatibility.

The second error conflicts with the current
[hosted limits](https://console.nace.ai/docs/drex/reference/limits),
which document 512 questions.
The deciding hosted implementation is not published in the inspected source checkout.
The read-only `nace-ai/drex-decision-models` checkout at
`be8ba7c8157241b9393862e9fcec104d0dc5caff` contains a different local request schema.
At `kev/api.py:51`:

```python
# nace-ai/drex-decision-models, kev/api.py:51
    questions: dict[str, Question] = Field(min_length=1)
```

A repository-wide hidden and ignored-file search for the exact diagnostic,
question-limit identifiers,
and the documented limit found no hosted guard.
The local schema is not evidence for the cause of the hosted limit.
The observation establishes rejection of this request,
not a universal model limit or a model-quality result.

### Verification

The private qualification repository retains `contract/research/drex-2.0-recheck`:
request manifests,
create-new runners,
and results.
The [release audit](../audit/tech-pi-auto-mode-drex-2-release-vet-2026-10-13.md)
records source hashes,
explicit payload consent,
execution bounds,
and exact returned labels.

The frozen original phase ran `mise run probe` once.
The separately frozen schema follow-up ran `mise run probe:text` once.
These tasks need fresh result destinations before a separately authorized repeat;
the retained runners refuse to overwrite evidence.
Both use five-second deadlines and no retries.

Working controls:

- The constant-state API control returned 200 and `model: drex-v2.0`.
- The complete 32,553-byte policy supplied once returned 200 and `model: drex-v2.0`.
- Local round-trip assertions recovered all original instruction objects from the converted strings,
  with unchanged state,
  criteria,
  questions,
  and original provider request.

Failing controls:

- The complete native wire with structured instructions returned the field-type 422.
- The converted 692,952-byte wire retained all 244 questions and returned the question-limit 422.

### Verified handling and rejected approaches

Keep the incumbent route while completing native integration.
This avoids adopting a model whose complete native request did not pass the serving gate;
it does not establish new semantic-quality evidence for the incumbent.

`JSON.stringify` on each instruction object resolves the observed field-type rejection,
as verified by the next response reaching the question-limit check.
It does not make the complete request accepted or qualify the text representation semantically.

Splitting 244 questions into groups of 16 requires 16 calls,
exceeding the accepted three-call ceiling.
No such dispatch was made.
Removing policy,
dropping sources,
or omitting questions would change the required assessment and was not attempted.

### Upstream filing decision

Nothing is filed or drafted for the bounded release check.

1.  Upstream fault: the field-type error belongs to the owned adapter;
    the question-limit response differs from hosted documentation,
    but its deciding code is unavailable.
2.  Upstream fixability: unknown without that code;
    no impossibility or fix-size claim is made.
3.  Supported use: hosted documentation covers text instructions and question batches up to 512.
4.  Contribution policy: not investigated because no external report is proposed.
5.  Willingness to change: unknown;
    no maintainer communication was sent.
6.  Prototype: the owned conversion was executed and retained;
    no source-justified hosted patch can be built from the inspected local runtime.

## Historical Drex v1.1 full-policy responses report v1.0

## Symptom

Requests naming `drex-v1.1` returned HTTP 200 with `model: "drex-v1.0"`.
The observed inputs included complete 42,677-byte `AGENTS.md`,
a synthetic selected source,
a code-owned read operation,
and the existing independent relation/prohibition questions.
Both calls completed within one `1984.5333979999998` ms assessment clock.

This is not a v1.1 quality measurement.
A successful HTTP response does not make the requested version the answering version.

## Documented boundary and observed cause limits

The [evaluation reference][evaluate] says:

> `drex-v1.1` takes a state plus longest question of up to 4,096 tokens;
> longer requests are answered by `drex-v1.0`,
> and the response `model` reports `drex-v1.0`.

The provider describes a length-dependent route from a v1.1 request to a v1.0 response.
The [model reference][models] also makes the broader statement:

> A pinned id is always answered by that version at that version's price.

The general pin statement cannot safely be applied without the documented long-input exception.
The observed responses agree with that exception.
Hosted routing source and actual serving weights were not inspected;
this is an API-contract and observed-label finding,
not a source-level proof of the internal routing cause.
Reported input-token totals also count multiple questions,
so they are not independently measured state-plus-longest-question lengths.

The first-party prototype explicitly distinguishes requested and returned labels.
At private source commit `564d544`,
`~/temp/agent/nace-drex-auto-mode-eval-2026-09-28/protocol.mjs:8` to `9` declares:

```js
// Private experiment: protocol.mjs
export const REQUESTED_MODEL = 'drex-v1.1';
export const SERVED_MODELS = ['drex-v1.0', 'drex-v1.1'];
```

At `protocol.mjs:32`,
it rejects undeclared labels before extracting Noul values:

```js
// Private experiment: protocol.mjs
assert(SERVED_MODELS.includes(value.model), 'Undeclared Drex response model');
```

The retained result includes `requestedModel`,
`servedModelLabel`,
and `differsFromRequestedModel`.
Both retained request bodies still name v1.1;
no additional first-party fallback request was made.

## Verification

Service response version:
`drex-v1.0`.
Requested version:
`drex-v1.1`.
Date:
2026-09-28.
Private source/input commits:
`564d544` and `496dfac`.
Raw result:
`e639234`.

The historical invocation was:

```bash
# Run from /var/home/user/Monochromatic; historical invocation, not a command to repeat over retained evidence.
MISE_AUTO_INSTALL=false mise --no-hooks exec --no-deps --fresh-env \
  --allow-env AUTO_MODE_NACE_DREX_API_KEY -- \
  node /home/user/temp/agent/nace-drex-auto-mode-eval-2026-09-28/run.mjs
```

The harness uses create-new result files.
Reproduction requires a separately frozen experiment copy;
do not delete old evidence to rerun it.

### Working observations and controls

- The model-list GET authenticated with HTTP 200.
- Both full-policy scoring calls returned valid native Noul answers under the predeclared v1.0 observation label.
- Request bodies were 45,914 and 46,131 bytes.
- Source,
  policy,
  full body,
  and raw-response checks passed.
- Fake transport accepted declared v1.0/v1.1 labels and distinguished them.
- `proc_fc6e` rechecked evidence without another model call.

### Rejected controls and failed interpretation

- Undeclared model labels,
  missing/extra answers,
  invalid Noul values,
  and invalid usage metadata were rejected locally.
- Preparation expiry dispatched no request.
- A late aggregate was rejected;
  omitting that guard exposed acceptance in an isolated copy.
- Reading `model: "drex-v1.1"` in the request as proof that v1.1 answered conflicts with both retained responses.

Result SHA-256:
`3e18de5385d1a2433db6b7c20f96d4330b08b74f7bc4e383d90caab647548a5d`.
Manifest SHA-256:
`0728f866798c5961297b7f8829bbc84bc89888e8e71ff696059944dc7c79179e`.

## Verified caller handling

The canary predeclared the documented response-label set,
validated it,
and retained the actual answering label without relabeling it as v1.1.
The tradeoff is explicit:
this permits a v1.0 observation in a routing probe,
not use of that observation as evidence for a required v1.1 deployment.
A production version requirement remains a separate gate.

Do not shorten or summarize policy to manufacture a v1.1 result.
The full-policy requirement remains unchanged.
The subsequent [36-call semantic study](../planning/pi-auto-mode-drex-qualification.md)
explicitly requested v1.0 and received v1.0 labels throughout.
That verifies the direct-pin path only for the recorded calls.
Its tradeoff is explicit:
it measures v1.0 rather than v1.1,
and its tested semantic profile failed the predeclared gate.
A correct model label alone does not establish suitability.

## What does not work

- Treating general pinned-version prose as sufficient serving-version evidence.
- Attributing the measured Noul values or timing to v1.1.
- Treating a version label as independent proof of exact serving weights.
- Treating the canary as calibration,
  general deadline compliance,
  or semantic qualification.

## Upstream filing decision

No upstream filing or draft is made.
There is no established hosted-code defect in this bounded observation.

1.  Upstream fault:
    the route is explicitly documented;
    the broader pin prose is in tension with it,
    but a code defect was not established.
2.  Upstream fixability:
    no source-level implementation or fix-size claim was investigated.
3.  Supported use:
    the official API documents both typed Noul evaluation and the long-input route.
4.  Contribution policy:
    not investigated because no external report is proposed.
5.  Willingness to change:
    unknown;
    no inference from silence or documentation age.
6.  Prototype:
    caller-side label handling and controls were executed;
    no upstream code patch was required or attempted.

The task's finite-evidence boundary does not reopen an upstream source-audit branch.
The durable outcome is the verified caller boundary and its limits.

## Complete native choice distributions and latency

### Symptom and source trace

The complete native request on 2026-10-14 returned HTTP 200,
`model: drex-v2.0`,
and all 84 answer IDs during a separate 60-second diagnostic observation.
The attempt failed local validation at 48,528.415 ms with
`QuestionSelectionError: Choice answer is not a complete distribution with a highest-probability selected option`.
The same request's earlier five-second call timed out without a usable result.
Raw evidence remains in `${HOME}/temp/agent/pi-native-drex-observation-20261014`.

The consumer at
`package/pi-plugin/auto-mode/native/contract/lifecycle/claim-evidence-batch-selection/selection.mjs:101`
requires a distribution sum within `0.000001` of one and a highest-probability selected label:

```javascript
// package/pi-plugin/auto-mode/native/contract/lifecycle/claim-evidence-batch-selection/selection.mjs:101
if((Math.abs(total-1)>0.000001)||(probabilities[value.choice]!==largest))throw new QuestionSelectionError('Choice answer is not a complete distribution with a highest-probability selected option');
```

The public Drex source at commit `be8ba7c8157241b9393862e9fcec104d0dc5caff`
rounds each value independently in `kev/api.py:137` and `kev/api.py:149`:

```python
# kev/api.py:137 and :149
def round_prob(x: float) -> float:
    return round(float(x), 4)

dist = {k: round_prob(v) for k, v in zip(m["keys"], p)}
```

The hosted response contains four-decimal values consistent with that serializer.
This is source consistency evidence,
not independent proof that the hosted deployment runs the inspected revision.
Of the returned distributions,
38 exceed the consumer tolerance;
the largest observed deviation is `0.0002`.
For example,
the primary clause probabilities sum to `0.9999`,
while its selected `none` label is the reported maximum.

### Verification and limits

The unchanged complete request measures 541,947 bytes and has SHA-256
`3d204aa86fb17dd3de7ad9c320ef2b6db65200badc27dd950b16189750fba410`.
The [complete-input evidence](../planning/pi-auto-mode-drex-current-contract.md#complete-native-input-on-2026-10-14)
retains source membership,
question round-trip checks,
usage,
and both observation budgets.
`mise run check` in the private observation directory verifies the frozen request without inference.
Its completed `mise run probe` is one-shot and refuses an existing start record.
Read the retained response for repeated analysis;
do not replay a paid observation merely to inspect its result.

The working catalog includes original answer-ID membership and model attribution.
The failing catalog includes rounded distribution sums,
the five-second production deadline,
and the current operating cutoff:
none of the 84 selected scores reaches `0.99`.
The explicit permission clause receives `none` at `0.3586`.
This provides no established complete-request suitability.

### Workaround status and upstream filing decision

No production workaround is installed.
A provider-specific precision adapter could retain raw probabilities and validate a documented rounding interval,
but that adapter is not implemented or verified.
Renormalizing the response would change the received values;
silently broadening the generic consumer tolerance would change other provider contracts.
Neither fixes the measured interpretation or latency result.

No upstream report is warranted:
independent rounding is visible in public source,
the consumer's stricter sum contract explains the local rejection,
and no hosted implementation defect is established.
An upstream serializer change is possible but not required by the inspected supported API use.
Contribution policy and willingness were not investigated because no report or patch is proposed.
No upstream fix prototype was attempted;
the retained caller evidence is the reproducible result.

### Representation controls do not establish a semantic workaround

The subsequent [frozen representation diagnostic](../planning/pi-auto-mode-drex-current-contract.md#frozen-representation-diagnostics)
made five successful requests to the same reported model.
Its two unchanged isolated-clause controls returned permission probability `0.6906`.
The same request hash had returned `0.7748` in the previous study.
Removing pre-assessment status fields returned `0.7468`,
which does not exceed that observed cross-study baseline range.
Parsing the program JSON strings returned `0.6533`.
A deliberately incomplete minimal control returned `0.8576`.
Every result stays below the adopted `0.99` operating cutoff.

The working catalog is schema-valid response membership and successful serving of these diagnostic subsets.
The failing catalog is semantic admission under the current rubric and cutoff.
No production change or verified workaround follows.
The hosted source of response variation is unknown;
the public serializer explains precision loss but does not explain these changed distributions.
The private one-shot runner at `${HOME}/temp/agent/pi-drex-representation-isolation-20261014/probe.mjs`
retains exact inputs and raw responses.
Its `mise run check` verifies hashes without inference;
the completed `mise run probe` refuses to overwrite its start record.
This additional consumer evidence establishes no upstream implementation defect or actionable upstream patch,
so the filing decision is unchanged.

[evaluate]: https://drex.nace.ai/docs/api-reference/systemone
[models]: https://drex.nace.ai/docs/reference/models

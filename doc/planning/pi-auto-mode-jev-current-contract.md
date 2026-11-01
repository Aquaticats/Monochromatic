# Jev contract refresh and workaround boundaries

## Scope

Primary documentation was rechecked on October 14,
2026,
for the user's question about encountered Jev problems and possible workarounds.
This is a documentation refresh,
not another model experiment or production adoption.
The [observed failures and source controls](../troubleshooting/llmgateway-systemone-boundaries.md)
remain historical evidence at their recorded revisions.

## Capacity and representation

The [current model reference][models] still lists `jev-1.13.0`,
64k tokens for state plus all questions,
and 32k for state plus the longest question.
Its published rate limits are 100K tokens per second and 80 requests per second,
explicitly subject to change.
These native limits do not establish this gateway account's exact usable boundary.

Our retained full-rule trials rejected 454,012-byte and 315,353-byte requests with
`max_tokens_exceeded`.
A revised 256,710-byte encoding succeeded once with 205 answers and 62,053 reported input tokens.
It removed duplicate bookkeeping and relocated shared guidance without dropping rules.
That is verified acceptance of one representation,
not proof of semantic equivalence or universal capacity.
The error did not identify which token bound failed.

The [native API reference][api] permits structured `instructions` and criteria.
Question-map keys are excluded from inference.
Consequently,
an identifier alone cannot carry the rule's meaning.
Structured instructions are a documented mechanism,
not a tested repair for our calibration errors.

## Semantic limitations

The [Jev limitations page][limitations],
reviewed by its publisher on October 2,
2026,
warns about literal interpretation,
indirection,
irrelevant context,
adversarial state,
contradictory criteria,
unreliable counting and date arithmetic,
and Choice option-order bias.
These are vendor disclosures;
this refresh did not reproduce them.

Direct questions and aligned criteria are mitigation candidates.
Code can own exact arithmetic,
date comparisons,
and established syntax facts.
Neither shorter prompts nor larger batches establish correctness.
The complete-current-policy requirement prevents treating policy truncation as an accepted workaround.
Our recurring pathname and other-file mistakes still need their own negative controls.

## Gateway changes and residual uncertainty

The [System One feature page][legacy] now announces upcoming deprecation of `/v1/systemone`.
It says existing integrations keep working and Jev remains available at the same price on `/v1/decisions`.
No retirement date is stated there.
This is an announced interface change,
not an encountered outage.

The [migration guide][decisions] changes keyed question maps to named arrays,
`noul` to `predicate`,
and `noul` answers to `probability`.
Structured state must be serialized into text.
A migration therefore needs fresh request,
response,
deadline,
and semantic checks;
this refresh neither exercised nor adopted it.

The retained gateway source controls establish model-label normalization,
same-provider credential retries,
missing-answer acceptance,
and conditional preservation of echoed upstream errors in metadata-only logging.
They do not establish those behaviors in today's hosted revision.
Consumer response validation can reject malformed or incomplete evidence.
A local deadline can stop our wait;
it cannot establish upstream cancellation or final billing.
Version pinning cannot attest hosted weights.

Current [retention documentation][retention] still separates payload clearing from retained analytics metadata.
It now restricts retain-all to Enterprise,
with existing non-Enterprise organizations switching to metadata-only on November 8,
2026.
Neither that announcement nor a default setting resolves the historical error-diagnostic concern.
The [TypeSafe provider entry][provider] still labels prompt logging and retention Unknown.
The [TypeSafe privacy policy][privacy] prohibits training on submitted input while permitting necessity-based retention.
Those are different promises.
No account setting was inspected or changed,
and the user's accepted retention scope remains settled.

## Encountered failures and practical mitigation

The retained evidence supports different answers for capacity,
semantic correctness,
and owned integration defects.

### Capacity has a partial verified workaround

Shared guidance and removal of duplicate bookkeeping produced one accepted complete 205-question input.
Further representation work must preserve every policy and received source,
original question meaning,
and code-owned source identity.
Moving text into shared state does not establish unchanged model behavior.
Splitting questions can address the aggregate question budget only if each complete state still fits
the separate state-plus-longest-question bound and the original call/deadline budget.
It cannot solve an oversized mandatory state by itself.
No provider-side context increase is verified.

### Semantic errors can be contained but are not repaired by a cutoff

The [earlier frozen study](pi-auto-mode-jev-qualification.md#observed-binding-errors)
returned `0.93` for permission to read `.env` contents when the source only permitted listing that pathname
and reading `README.md`.
The [instruction-condition study](pi-auto-mode-jev-qualification.md#real-instruction-condition-development-result)
returned `0.93` and `0.16` for byte-identical unused-configuration input.
Replaying until a favorable answer appears is therefore not a verified remedy.

Code-owned syntax and effect facts,
narrow source-language questions,
explicit criteria,
and abstention are mitigation mechanisms.
The [fresh locked test](pi-auto-mode-jev-fresh-qualification.md#counts-and-coverage)
had zero resolved errors at its 95/05 diagnostic band but left 77 of 216 claims unresolved.
That result neither erases the earlier false positive nor qualifies the later categorical question definitions.
The [adopted current cutoff](../decision/pi-auto-mode-semantic-admission.md)
is `0.99`,
with unresolved meanings blocking execution.
It is an operating rule,
not a measured accuracy guarantee.

The current Choice warning matters to this implementation:
`package/pi-plugin/auto-mode/src/clause-lowering.ts` places `permission` first in `CLAUSE_RELATIONS`.
The wire preserves that label order.
This establishes a concrete order-sensitivity test candidate,
not a reproduced current false approval.
The existing unit tests verify admission arithmetic and structure,
not provider invariance under reordered options.
No label reorder or additional paid study is adopted by this assessment.

### Current installed failures include owned integration defects

The installed check first rejected ordinary single-call syntax,
then encountered an omitted optional effect-owner callback.
Those were owned host defects and were corrected with local regressions;
they are not Jev model failures.
A subsequent run hit our one-MiB serialized-request guard before inference.
Increasing that guard alone cannot establish fit within Jev's independent token bounds.

The latest installed attempt reached LLM Gateway and returned HTTP 400.
`package/pi-plugin/auto-mode/src/assessment-transport.ts:238`
replaces non-200 response detail with a status-only error.
The retained log therefore cannot establish that this particular response was `max_tokens_exceeded`.
A proposed next diagnostic change is to expose only recognized provider error codes,
without logging upstream free text,
credentials,
or source content.
It is not implemented by this research task.

### Working order

First retain a recognized cause for the current HTTP 400 and measure the exact current input.
Then test content-preserving representation changes against both Jev context bounds.
Semantic mitigation remains a separate concern and must retain abstention and original authority checks.
The announced Decisions migration is maintenance work;
the current documentation does not establish that it increases Jev's context capacity or repairs semantic errors.
No production fix or migration is authorized by this analysis alone.

## Implementation follow-up and shared-state fit

The user subsequently requested continued implementation and renewed standing authorization
for task-relevant Drex and Jev inputs and credit use.
The transport now retains the recognized `max_tokens_exceeded` code without upstream free text.
An installed Qwen run reproduced that precise rejection for the complete current native request.
The captured original contains 73 source occurrences and 84 questions in 504,946 bytes.
The lossless shared-content prototype reconstructs every original state field and question instruction.
Its 165,373-byte complete request still returned `max_tokens_exceeded` in 1,269.832 ms.
It is not installed or semantically qualified.

The frozen follow-up at `${HOME}/temp/agent/pi-jev-shared-state-fit-20261014`
distinguishes serving availability from the complete shared state's capacity requirement.
It makes at most two sequential calls,
each with a five-second deadline,
and stops on rejection or response-validation failure without retry.
The first is a 312-byte public literal-label serving control.
The second retains the exact 125,822-byte shared state and unchanged `head_0` question,
for a total of 126,345 bytes.
Its SHA-256 is `287fb871e3b7a8f5ad511dc5355670769bec70b76f8db5261153224a71ec3b3f`.
The shared state retains the complete original context and all original question instructions;
only the dispatched question map is narrowed for this diagnostic.

Rejection of this single-question input would rule out merely splitting the same shared-state request
as a sufficient capacity remedy.
Acceptance would justify further complete-coverage split tests,
without proving that every other question fits or establishing semantic equivalence.
No result authorizes a represented action or changes the production provider,
cutoff,
or deadline.

Both frozen requests were dispatched once.
The serving control returned HTTP 200 in 1,357.310 ms,
with the requested model,
333 reported input tokens,
and the literal `permission` label at `1`.
This is a serving and parser control,
not semantic qualification.
The complete shared state with only `head_0` returned HTTP 400 `max_tokens_exceeded` in 731.176 ms.
Merely splitting that unchanged shared-state request therefore cannot solve its capacity rejection.
The exact token count and which provider bound fired remain unavailable.

Local inspection identifies a representation-specific contributor worth measuring:
the prototype stores every original question's instructions inside `state.root.questionInstructions`.
Those instructions are required by the assessment,
but their placement inside shared state is an experimental encoding choice.
The next local candidate moves each original encoded instruction back into its own question
while preserving the complete governing state and shared-content dictionary.
It must round-trip exactly before any separate provider experiment.
This is not permission to remove questions or source occurrences.

The placement candidate now passes exact reconstruction of the original state and all 84 questions,
including every one of the 73 source occurrences.
Its shared dictionary is unchanged.
The state measures 105,339 bytes;
the complete request measures 140,618 bytes with SHA-256
`342c62860872a1f084157f700a34d7bf27bbf95ffae3a2eb81c6cb057897853e`.
The frozen private schedule at `${HOME}/temp/agent/pi-jev-question-placement-20261014`
first sends that complete batch with a five-second deadline.
Only HTTP 400 with recognized `max_tokens_exceeded` permits the separate single-question capacity control.
That control retains the candidate's entire shared state,
measures 105,761 bytes,
and has SHA-256 `9bc4ef519973fd9d05c7e69e57690a5e6d615de97c2f6ff0bbed3ea6c517bd05`.
There are at most two calls,
no retries,
and no production admission.

Both placement requests completed with HTTP 400 `max_tokens_exceeded`.
The complete batch took 1,049.183 ms;
the single-question control took 518.147 ms.
Neither returned model answers.
Moving the instructions is insufficient even for one question with the complete shared state.
The private directory retains both original responses and result records.
No further request is scheduled by this experiment.

### Structural encoding hypotheses

The next offline comparison ranks these independently testable explanations:

1.  Repeated record keys add representational overhead.
    A shared ordered field list with explicit row values should reduce serialized bytes
    while reconstructing every object key and value exactly.
2.  Nested deduplication can leave shared entries referenced only once after their parents are shared.
    Inlining those entries should remove reference and dictionary overhead without changing the decoded value.
3.  JSON strings outside native source records retain escaping and repeated nested structure.
    Parsing only strings that reproduce exactly with `JSON.stringify` should permit further sharing.

The measured placement-state object-key syntax occupies 28,757 bytes;
this is not a tokenizer measurement.
Each candidate must reconstruct the complete original state and every original question.
No candidate is installed or sent merely because its byte count decreases.
The provider's documented indirection weakness remains a separate semantic risk.

The offline comparisons at `${HOME}/temp/agent/pi-jev-structure-20261014`
all reconstruct the original request exactly,
including 73 source occurrences and 84 questions:

- Shared ordered field lists: 97,605 state bytes and 119,876 complete request bytes.
- Inlined single-use references: 95,749 state bytes and 131,028 complete request bytes.
  The dictionary shrinks from 398 entries to 94.
- Both changes: 88,015 state bytes and 110,286 complete request bytes.

The JSON-string inspection found 49 canonical strings occupying 12,031 serialized bytes,
versus 10,989 bytes for their parsed values before adding any required encoding tags.
This measurement does not establish an additional working encoding.
No provider requests occurred during these comparisons.

The next frozen capacity schedule makes at most two sequential five-second calls without retries.
First it sends the complete single-use-inlined request,
SHA-256 `06df855d0647d552e0eae94b9a5a4d95cd18e1bf36eb34176231746a4a025344`.
Only recognized HTTP 400 `max_tokens_exceeded` permits the second complete request,
which combines inlining with ordered field lists,
SHA-256 `1779a9a3c80a2fa61f6a1fd4663d34c503d46dfa8c7ca4fa968ddb81f0b15b6b`.
Any other response ends the experiment.
This ordering tests reduced indirection before introducing row decoding.
Acceptance requires separate answer validation and never authorizes execution.

The inlined complete request returned HTTP 400 `max_tokens_exceeded` in 1,233.578 ms.
The combined request returned HTTP 400 in 297.736 ms with gateway validation code
`invalid_union_discriminator` at each question's `type` field.
The row prototype incorrectly encoded the API question envelope as a row,
hiding the required discriminator.
Its exact reconstruction check did not validate the provider-facing envelope.
This is an owned prototype defect,
not evidence about combined-encoding token fit or model behavior.
The frozen schedule stopped without retry.
The correction must leave each question envelope unchanged and encode only its instruction payload;
a preflight must reject the captured malformed input before accepting the corrected one.

The corrected offline prototype at `${HOME}/temp/agent/pi-jev-structure-envelope-20261014`
preserves every question field except its encoded instructions.
The captured malformed input fails the new envelope check;
all corrected candidates pass envelope equality and exact original reconstruction.
The corrected combined candidate is 87,989 state bytes and 117,254 complete request bytes,
with SHA-256 `e405fcf07f905b2a069f7f287d7d876c49a3475ffac19a1c54b70c28cac3408d`.
The frozen `${HOME}/temp/agent/pi-jev-envelope-fit-20261014` schedule sends that complete candidate once,
with all 84 questions,
a five-second deadline,
and no retry.

The corrected envelope request returned HTTP 400 `max_tokens_exceeded` in 997.329 ms.
The negative wire-schema control and corrected preflight passed before dispatch.
This is now a valid capacity rejection of the corrected representation,
separate from the malformed earlier candidate.
No answers or model-quality evidence were returned.

Automatic approval review again rejected the documentation commit's existing cgroup attachment.
The user renewed authorization for that attachment with unchanged limits;
commit `07987fab66` then completed.

[models]: https://docs.typesafe.ai/models
[api]: https://docs.typesafe.ai/api
[limitations]: https://docs.typesafe.ai/model-jaggedness/jev-1.13
[legacy]: https://docs.llmgateway.io/features/system-one
[decisions]: https://docs.llmgateway.io/features/decisions
[retention]: https://docs.llmgateway.io/features/data-retention
[provider]: https://llmgateway.io/providers/typesafe
[privacy]: https://typesafe.ai/legal/privacy-policy

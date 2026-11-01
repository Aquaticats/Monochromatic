# Pi Advisor answer-substitution prevention handover

## Status

The corrected expressive design comparison and independent review are complete.
The symbolic-findings recommendation remains rejected because it reduces Advisor expressiveness.
The persisted two-phase checkpoint recommendation awaits user acceptance or delegated implementation authority.
No Advisor production code was changed.
This handover was created before further source investigation at
`2026-08-25T23:58:24-04:00` from repository commit
`3210ce1ef86d3fbdd9e42afeef56b79c4ade2d24`.
The worktree was clean at creation.

The observed Advisor call completed successfully at the transport level but crossed its reviewer role:
it supplied a near-final five-part answer while reviewing a simple primality task.
No prevention design has been accepted or implemented.

## User requirements

- Explain what can prevent Advisor from substituting an answer for independent review.
- Investigate the Advisor interface,
   prompt,
   request path,
   and verification surface.
- Keep this handover current throughout the investigation.
- Limit this session to `package/pi-plugin/advisor` and directly shared Advisor dependencies.
- Do not investigate or change `package/pi-plugin/goal` because another session owns that work.
- Preserve full natural-language Advisor expressiveness.
  Prevention must not depend on a finite finding taxonomy,
  enum-only output,
  fixed prose,
  or removal of useful reviewer explanations.

## Observed session

The source session is:

```text
/home/user/.pi/agent/sessions/--var-home-user-Monochromatic--/
2026-08-26T03-35-46-248Z_01a03c23-5f48-778f-8306-b30a1fddddd2.jsonl
```

The primary model was `openai-codex/gpt-5.6-luna`.
At JSONL line 12 it called Advisor without an explicit model and with this focus question:

> Review the planned answer:
>  give exactly five valid,
>  clearly distinct explanations that 67 is prime,
> including trial division/factor-pair reasoning,
>  a 6k±1 reduction,
>  Wilson's theorem with a verifiable
> factorial congruence,
>  and a Lucas primality certificate.
>  Check all modular arithmetic and theorem use.

The empty model selection chose `openai-codex/gpt-5.6-sol` by the configured highest-expected-cost policy.
The result at JSONL line 13 mixed valid review findings with replacement content:

- It correctly reported that no completed five-part answer or fifth method was present.
- It checked arithmetic and theorem use.
- It instructed the primary model to use exactly five named methods.
- It supplied detailed proof text,
   including an elliptic-curve certificate absent from the primary model's work.

Measured call details from the stored tool result are:

- duration:
   `523412` milliseconds;
- serialized context:
   `2201` characters of a `958088` character budget,
   not truncated;
- estimated input:
   `772` tokens;
- provider input:
   `854` tokens;
- provider output:
   `18093` tokens,
   including `17132` reasoning tokens;
- provider total:
   `18947` tokens;
- provider-reported cost:
   `$0.54706`;
- visible Advisor text:
   `2612` characters and `294` whitespace-delimited words.

The TUI text `772 tokens full` describes estimated request input and truncation state.
It does not describe visible response length.
`package/pi-plugin/advisor/src/rendering-summary.ts` formats that metadata.

## Established cause before deeper investigation

The caller's focus question was not a narrow uncertainty.
Its imperative `give exactly five` directly requested primary-task content.
The call therefore presented Advisor with a role conflict.

The built-in Advisor system prompt nevertheless states:

```text
Do not perform the primary task. Do not write final user-facing prose for the primary agent.
```

That guard is in `package/pi-plugin/advisor/src/constants.ts`.
The selected model failed to preserve it.
The response is therefore not intended Advisor behavior,
 even though the malformed focus question made the drift more likely.

Current source exposes a free-form optional `question` string and returns unrestricted model text.
The interface has no structural distinction between a review request and a request to perform the task.
The prompt prohibition is not enforced by a result contract or postcondition.

## Current interface trace

The answer-generating instruction is reinforced at several seams:

1. `package/pi-plugin/advisor/src/tool-params.ts:46` describes `question` as content for Advisor to `answer`.
2. `package/pi-plugin/advisor/src/tool.ts:86` tells the primary model to use `question` when Advisor should `answer`
   an uncertainty.
3. `package/pi-plugin/advisor/src/constants.ts:63` tells Advisor to answer the focus question first,
   then later in the same prompt tells it not to perform the primary task.
4. `package/pi-plugin/advisor/src/advisor-request.ts:55` places the raw string beneath a `## Focus question` heading.
5. `package/pi-plugin/advisor/src/advisor-client.ts:279` sends that text as the only provider user message.
6. `package/pi-plugin/advisor/src/tool.ts:336` extracts all returned text without a semantic or structural check.

The role conflict is therefore produced by the Advisor interface,
not only by one unfortunate model call.
The negative system-prompt guard competes with repeated positive `answer` framing closer to the focus text.

The `/advisor` command has no focus parameter,
but it uses the same unrestricted provider completion and text extraction.
Removing tool `question` would close the incident's direct instruction channel without breaking the manual command path.

The test surface currently proves transport rather than role preservation:

- `package/pi-plugin/advisor/src/advisor-client.unit.test.ts:205` uses unconstrained fixture text `advisor answer`;
- `package/pi-plugin/advisor/src/advisor-client.unit.test.ts:469` proves only that focus text reaches the provider;
- `package/pi-plugin/advisor/src/advisor-request.unit.test.ts` proves heading placement;
- no Advisor test rejects a task-performing focus request;
- no Advisor test constrains the result to findings about existing evidence.

`package/pi-shared/model-review` already supplies a forced-tool structured-review transport.
Its interface leaves verdict schema,
strict parsing,
prompting,
and interpretation with the caller.
Advisor can reuse that transport without importing goal behavior.
A structured contract would deterministically constrain response shape,
but string fields would still need bounded semantics because a schema alone cannot prove that prose is not a
replacement answer.

## Current external precedent

Anthropic's current
[Advisor tool documentation](https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/advisor-tool),
fetched on 2026-08-26,
changes the design reading in several important ways:

- Anthropic Advisor is a strategic planner and course-correction model,
  not a findings-only independent reviewer.
- Its executor call input is always empty.
  Nothing the executor places in tool input reaches Advisor.
- It is explicitly a weak fit for single-turn question answering because there is nothing to plan.
- Anthropic reports typical output around 400 to 700 visible text tokens and 1,400 to 1,800 total tokens including
  thinking.
- Anthropic recommends starting with a 2,048-token hard cap.
  Its reported hard-reasoning probe used 40 calls per configuration,
  found about 630 to 840 mean output tokens at that cap,
  near-zero truncation,
  and no detectable quality loss within that sample.

The local package description says it is modeled after Claude Code Advisor,
but the local system prompt defines a stricter reviewer role.
The name therefore imports planner expectations that conflict with the local contract.
This does not justify accepting answer substitution because the user requires reviewer behavior.
It does mean the implementation must enforce that divergence rather than rely on the word `Advisor`.

Anthropic's [structured outputs documentation](https://docs.anthropic.com/en/docs/build-with-claude/structured-outputs)
states that constrained decoding guarantees schema shape,
not semantic intent.
It also identifies refusal and token-limit exceptions.
Client validation remains necessary for array and string bounds because SDK schema transforms can remove `minLength`,
`maxLength`,
`minimum`,
and `maximum` before provider dispatch.

The external precedent supports three local changes:

1. Remove free-form call input rather than improve its wording.
2. Reject Advisor use on simple single-turn factual or arithmetic tasks through main-model call guidance.
3. Treat `2048` as a measured starting cap for a structured reviewer,
   then verify it across the locally eligible provider set before making it default.

## Enforcement layers

Prevention has distinct layers with different guarantees:

- **Primary-model ownership guidance** can require a candidate artifact or conclusion before review.
  This changes model behavior but is not deterministic.
- **Input grammar** can remove arbitrary imperative text from the Advisor call.
  An enumerated focus rejects the exact incident request before provider spending.
- **Request encoding** can carry focus as JSON data rather than a user-message instruction heading.
  This reduces prompt role confusion but remains model-dependent.
- **Forced structured output** can make a single review-submission tool the provider's only output path.
  Free text from an omitted tool must never reach the primary model.
- **Strict parsing** can reject unknown fields,
  invalid evidence references,
  excessive findings,
  and inconsistent discriminated-union states.
- **Fixed rendering** can expose only locally generated relation text to the primary model.
- **Finding selection** remains model-dependent.
  A separate semantic classifier would add another fallible model call rather than create proof.

The strongest practical design therefore combines deterministic input grammar,
a symbolic output contract,
and positive reviewer-role prompting.
No single prompt sentence can supply the guarantee.

## Independent review recovery

Three isolated Pi CLI design processes were restarted with absolute executable paths after the original `PATH` failure.
They produced only model-scope warnings,
left stdout empty,
and retained `running` states after process searches found no command-line match for the Pi invocations.
Their logs had not changed since startup.
The installed process manager checks process-group liveness rather than the original command line,
so an unobserved descendant could have retained the group.
They were stopped and cleared without attributing an unverified root cause.

The design comparison was recovered through separate Advisor calls to:

- `synthetic/hf:moonshotai/Kimi-K3` for the minimal interface;
- `openai-codex/gpt-5.6-terra` for the typed-focus interface;
- `synthetic/hf:Qwen/Qwen3.6-27B` for cross-option ranking.

All three independent reviews ranked typed focus ahead of no focus,
and no focus ahead of a free-form intent gate.
They converged on these corrections:

- Keep arbitrary prose out of public input,
  but preserve focus with required enumerated dimensions and a mechanically defined transcript target.
- Apply one parser and renderer to both the tool and `/advisor` command paths.
- Reject unknown input fields,
  including legacy `question`,
  before provider dispatch.
- Fail closed for malformed,
  refused,
  truncated,
  empty,
  or tool-omitting output,
  without copying rejected provider text into errors.
- Do not call bounded natural-language fields deterministic prevention.
  A complete answer can fit inside `concern` or `requiredCheck`.
- Use evidence IDs generated locally,
  and reject references outside the selected target or evidence inventory.
- Keep output-cap changes and tool renaming separate from role enforcement.

Two reviews explicitly proposed a missing option:
a symbolic findings contract containing only enums and evidence relations.
The third said the same constraint is required for a hard guarantee,
while preferring bounded prose if richer explanations are required.

A final independent review by `openai-codex/gpt-5.6-sol` gave a conditional pass on the narrowed transport claim.
It identified residual structured channels and required:

- canonical finding order,
  bounded cardinality,
  and duplicate rejection;
- provenance-aware evidence references,
  with Advisor calls and results excluded from reviewable evidence;
- opaque structural evidence locators rather than provider-selected excerpts;
- finite local failure codes without raw provider bodies,
  values,
  selector strings,
  or validator payloads;
- one shared request,
  transport,
  parser,
  failure,
  and renderer path for tool and command;
- explicit acceptance that a finite taxonomy reduces general reviewer expressiveness.

A preceding `hyper/deepseek-v4-pro-0813` probe returned only an attempted unavailable tool call and supplied no review.
It is excluded from the evidence count.

## Candidate interfaces under comparison

### Minimal evidence review

```typescript
type AdvisorInput = {
  readonly model?: string;
};
```

The implementation returns validated findings rather than provider prose.
This removes the free-form instruction channel,
matches Anthropic's empty-input precedent apart from local model selection,
and keeps empty parameters safe.
Its cost is loss of per-call focused review.
The advisor must infer relevant review dimensions from the transcript.

### Typed focus and mechanical target

```typescript
type AdvisorInput = {
  readonly model?: string;
  readonly focus?: {
    readonly dimensions: readonly AdvisorDimension[];
    readonly target: 'latest-primary-message' | 'latest-tool-result' | 'current-turn' | 'session';
  };
};
```

When `focus` is absent,
the canonical default is every dimension against `current-turn`.
When present,
both fields are required;
`dimensions` must be nonempty and duplicate-free.
Targets describe transcript positions the implementation can select mechanically,
not semantic labels such as `latest-candidate` that would require another model judgment.
This preserves precision without accepting task instructions.
Its interface is wider than the minimal candidate and departs from the no-input external precedent.

### Free-form focus with local intent gate

Keep `question` but reject task-performing language locally and quote it as data.
This preserves flexibility but cannot classify arbitrary natural language soundly.
Verb lists,
regular expressions,
and model classifiers all leave bypasses or false positives.
This candidate currently ranks last.

## Candidate structured results

### Bounded cited prose

The original structured candidate contained `concern` and `requiredCheck` strings.
It would close raw-output transport but not semantic substitution.
A malicious or confused model could place the complete requested answer inside those bounded fields.
String limits reduce output volume;
they do not classify reviewer intent.

### Symbolic evidence relations

The stronger candidate returns no arbitrary provider-authored prose:

```typescript
type AdvisorReview =
  | {
      readonly assessment: 'clear';
      readonly findings: readonly [];
    }
  | {
      readonly assessment: 'insufficient-evidence';
      readonly findings: readonly [];
      readonly missing: readonly EvidenceKind[];
    }
  | {
      readonly assessment: 'findings';
      readonly findings: readonly AdvisorFinding[];
    };

type AdvisorFinding =
  | {
      readonly kind: 'unsupported-claim';
      readonly subjectRef: number;
      readonly supportRefs: readonly number[];
    }
  | {
      readonly kind: 'contradiction';
      readonly firstRef: number;
      readonly secondRef: number;
    }
  | {
      readonly kind: 'requirement-gap';
      readonly requirementRef: number;
      readonly subjectRef: number;
    }
  | {
      readonly kind: 'verification-gap';
      readonly subjectRef: number;
      readonly check: AdvisorCheck;
    }
  | {
      readonly kind: 'scope-drift';
      readonly objectiveRef: number;
      readonly subjectRef: number;
    }
  | {
      readonly kind: 'risk';
      readonly subjectRef: number;
      readonly risk: AdvisorRisk;
      readonly check: AdvisorCheck;
    };
```

`EvidenceKind`,
`AdvisorCheck`,
and `AdvisorRisk` are finite enums.
The exact enum vocabulary should be validated against representative existing Advisor reviews before implementation.
Locally generated evidence IDs identify transcript content blocks and carry provenance roles such as objective,
requirement,
primary candidate,
tool result,
and verification.
Advisor calls and Advisor results are not reviewable evidence.
The parser validates membership,
selected target,
field-specific provenance,
cardinality,
canonical order,
and uniqueness;
it also reconciles assessment with finding count and rejects unknown fields.
The local renderer turns each relation into fixed prose with opaque structural locators.
It does not include provider-selected evidence excerpts.

This sacrifices model-written explanations.
It preserves reviewer utility by naming the affected evidence,
relation,
risk class,
and required verification class.
The primary agent must inspect the cited evidence and perform the correction itself.
That trade is what converts output containment from a bounded-prose claim into a deterministic interface property.

The exact incident then fails at two deterministic seams:
legacy `question` is unsupported,
and the provider has no arbitrary text field in which to return five explanations.
A provider can still choose inaccurate enums or references,
and valid structured selections remain a limited information channel.
The interface prevents unrestricted answer prose;
it cannot make every valid reviewer judgment semantically unrelated to the primary task.

## Rejected symbolic verification design

This test plan belongs to the rejected symbolic-output design and is retained only as design history.
The implementation phase would have started with red tests through the package's built interface.
Unit tests import `dist` and run through the package `mise` tasks.

Required negative cases:

- Pass the exact incident arguments containing `question` and assert rejection before provider dispatch.
- Script a provider to emit the five-part answer as free text instead of the forced review tool.
  Assert that text never reaches the returned Advisor tool content.
- Return a review object with unknown properties and assert strict parser rejection.
- Return reordered or duplicate findings and references and assert canonicalization rules reject them.
- Cite an evidence ID through the wrong provenance field,
  outside the selected target,
  or from an earlier Advisor call or result.
  Assert each is rejected.
- Return excessive findings,
  invalid assessment,
  finding,
  risk,
  check,
  and evidence-kind values,
  incompatible union fields,
  and nonexistent or out-of-target evidence references.
  Assert client validation rejects each even when a provider schema accepted it.
- Return refusal,
  token-limit,
  empty,
  and malformed direct-JSON outcomes.
  Assert none become successful review prose.

Required positive controls:

- Every valid symbolic finding variant must cross the public Advisor interface and appear through fixed local rendering.
  The rendering must identify evidence structurally without quoting provider-selected content.
- A `clear` assessment must remain distinguishable from missing or invalid output.
- An `insufficient-evidence` assessment must report the absence of a reviewable candidate without supplying one.
- Explicit valid model selection must retain current scope and output-capacity checks,
  remain routing-only,
  and never appear in provider prompts or raw error rendering.
- Default model selection must remain non-current when another eligible model exists.
- Conversation and project context must still reach the reviewer adapter.
- A forced-tool omission followed by valid direct JSON must recover without exposing first-attempt free text.

Required user-boundary probe after implementation:

- Run the built extension in a disposable Pi session with a deterministic provider adapter.
- Present the exact primality incident request.
- Confirm the public tool rejects `question`,
  a valid typed-focus call returns only fixed rendering of validated symbolic findings,
  and no five-part answer text enters the primary model context.
- Run the guard test red by temporarily removing each deterministic gate after its test is committed,
  rebuild,
  observe the focused test fail,
  then restore the guard.

A live provider matrix can measure the `2048` cap candidate and semantic behavior,
but nondeterministic model compliance is supplementary evidence rather than the deterministic completion gate.

## Rejected symbolic implementation questions

These questions are superseded by the expressiveness requirement.

- Which finite risk,
  check,
  and missing-evidence vocabulary covers representative existing Advisor findings without creating a general text field?
- Should evidence IDs identify complete context entries or individual content blocks?
  This needs a prototype against real stored sessions,
  including verification that provenance roles are mechanically assigned.
- Should direct-JSON recovery remain as a separately named,
  strictly parsed fallback,
  or should Advisor fail after a forced-tool omission?
  Measure provider compatibility before deciding.
- Which positive prompt wording most consistently selects useful evidence relations?
  Live-provider behavior is supplementary evidence,
  not the deterministic acceptance gate.
- Does a `2048` output cap preserve a complete symbolic result across the eligible provider matrix?

## Constraints

- This is an investigation and design request.
  Do not mutate production code until the user explicitly accepts or delegates a design.
- Keep changes in this session to this handover and other requested planning documents.
- Preserve Advisor as an independent reviewer rather than turning it into a general subagent.
- Prefer a deep Advisor interface that hides prompt construction,
   selection,
   transport,
   and validation.
- Do not treat prompt wording alone as deterministic enforcement.
- Do not claim prevention from a test that only checks static prompt text.
- Verify behavioral protections with a model or deterministic transport capable of emitting forbidden replacement
content.

## Rejected recommendation and prior rankings

The user rejected the typed-focus and symbolic-findings recommendation because it reduces expressiveness.
The material is retained as a rejected design record,
not current direction.
The prior design combined two complementary decisions.

### Input ranking

1. **Typed focus and mechanical target**
   - Pros:
      preserves focused review,
     closes arbitrary call instructions,
     and permits deterministic pre-dispatch validation.
   - Cons:
      adds enum and target semantics that must remain stable and mechanically defined.
2. **No focus input**
   - Pros:
      narrowest public input and alignment with Anthropic's empty-input precedent.
   - Cons:
      forces the reviewer to infer scope,
     increasing irrelevant or broad review.
3. **Free-form focus with an intent gate**
   - Pros:
      preserves arbitrary caller expression.
   - Cons:
      natural-language classification cannot reliably separate review from task execution,
     so the original role conflict remains representable.

Ranking:
 typed focus > no focus > free-form gate,
because typed focus preserves the utility lost by no focus without reopening arbitrary prose;
no focus ranks ahead of a gate because structural absence is stronger than heuristic intent classification.

### Output ranking

1. **Symbolic evidence relations**
   - Pros:
      no arbitrary provider prose reaches the primary model;
     local rendering and strict reference validation create deterministic output confinement.
   - Cons:
      a finite taxonomy cannot explain every novel defect,
     so the primary agent must inspect cited evidence itself.
2. **Bounded cited prose**
   - Pros:
      richer and more immediately actionable findings.
   - Cons:
      an answer can still be placed inside a valid string field,
     so prevention remains model-dependent.
3. **Unrestricted provider text**
   - Pros:
      maximum expressive flexibility.
   - Cons:
      directly reproduces the incident's answer-substitution path.

Ranking:
 symbolic relations > bounded prose > unrestricted text,
because symbolic relations are the only candidate that removes the semantic text channel;
bounded prose ranks ahead of unrestricted text because it still constrains shape,
volume,
and evidence linkage.

The rejected stack was:

1. Replace `question` with optional typed `focus` and reject unknown fields before dispatch.
2. Generate a provenance-aware local evidence inventory,
   exclude Advisor-generated entries,
   and apply mechanical target projection.
3. Ask the reviewer to submit only canonical symbolic findings through the shared model-review transport.
4. Strictly parse one result contract,
   including any separately identified direct-JSON recovery path,
   and map every failure to a finite local code without relaying rejected values.
5. Route the tool and `/advisor` command through the same request,
   transport,
   parser,
   failure,
   and fixed-rendering pipeline.
6. Rewrite caller guidance so Advisor reviews an existing candidate or evidence on a multi-step task.
   Simple factual and arithmetic tasks should not invoke it.
7. Keep positive reviewer prompting as defense in depth.
8. Probe a `2048` output cap across eligible providers separately;
   do not bundle that unverified local default with role enforcement.
9. Defer renaming.
   A rename does not add an enforcement seam and would distract from the contract change.

The rejected stack would have eliminated free-form per-call review instructions and provider-authored free-form text.
That guarantee was purchased by removing the expressive review channel,
so it does not satisfy the corrected requirement.

## Expressive redesign

### Compatibility boundary

Full natural-language review expressiveness and deterministic semantic exclusion cannot both be supplied by an output
grammar.
The same passage can be a direct answer,
a quoted defect,
a corrected example,
or a suggested rewrite depending on its relationship to the primary agent's work.
Any local grammar that rejects every answer-like passage also rejects legitimate expressive review.
Any grammar that accepts those legitimate passages admits the same text when a model uses it as a substitute.

The prevention seam must therefore move from vocabulary to ownership and call sequence.
The strongest deterministic claim available without narrowing language is that Advisor cannot run until the primary
agent supplies
an explicit review artifact.
Whether that artifact is substantive,
whether the review remains anchored to it,
and whether the primary agent later copies advice remain semantic judgments.

Anthropic's current [Building effective agents](https://www.anthropic.com/research/building-effective-agents) guidance
names an
evaluator-optimizer workflow in which one model first generates a response and another evaluates it with feedback.
That candidate-first ordering matches the local reviewer role better than Anthropic's current Advisor tool,
which intentionally gives an executor unrestricted strategic plans early in a task.
The official Advisor behavior is useful precedent for preserving advice expressiveness,
not for enforcing the local reviewer boundary.

### Candidate A: primary-artifact checkpoint

```typescript
type AdvisorToolParams = {
  readonly model?: string;
  readonly artifact: {
    readonly stage: 'approach' | 'result';
    readonly content: string;
  };
  readonly criteria?: string;
};
```

`artifact.content` and `criteria` remain unrestricted natural language.
The result remains unrestricted Advisor text.
`stage` identifies whether the primary model owns a proposed approach or a candidate result;
it does not constrain either text channel.

The tool rejects absent or blank artifacts before provider dispatch.
The exact incident call therefore fails because it contains only a task-performing focus request.
The primary model must first state the approach or result it wants reviewed.
The manual `/advisor` command uses the latest non-Advisor primary text as its artifact
and reports no-reviewable-artifact when none
exists.

The provider request serializes objective,
conversation evidence,
persisted primary artifact,
and review criteria as separately labeled data.
No interface description or prompt says to answer a focus question.
A positive evaluator prompt asks for the most useful evaluation,
including detailed reasoning,
examples,
corrections,
and proposed rewrites when those improve the review.
The primary agent remains owner of final synthesis.

This candidate preserves current expressive power and deterministically enforces candidate-first ordering.
It cannot prove that the candidate is substantive or that returned prose is review rather than task completion.

### Candidate B: artifact checkpoint plus hidden role adjudication

Candidate B adds a separate model call after the unrestricted review.
A hidden structured adjudicator classifies the relationship among objective,
artifact,
criteria,
and review as grounded evaluation,
mixed,
or task substitution.
A grounded review is returned byte-for-byte.
A rejected review is retried with adjudicator feedback;
repeated rejection fails closed without exposing the rejected text.

This adds behavioral protection without a finite Advisor vocabulary.
It also adds model-dependent false positives and false negatives.
The adjudicator preserves the expressive contract in principle,
but a false positive can withhold a valid rich review.
A representative evaluation corpus must measure that trade before this becomes a default gate.

### Candidate C: prompt-only evaluator framing

Candidate C keeps the current optional `question` contract and unrestricted output,
then rewrites tool descriptions and prompts around positive evaluator actions.
It preserves expressiveness and has the smallest interface change.
It does not enforce candidate ownership,
and the exact incident remains representable.
Prompt wording alone is therefore insufficient.

### Independent expressive-design review

Three independent reviews evaluated the corrected requirement.
They agreed on the compatibility boundary:
full prose permits both rich review and answer-like passages,
so no deterministic semantic classifier follows from syntax alone.

`openai-codex/gpt-5.6-terra` found that Candidate A's inline `artifact.content` is not provenance.
It is another same-call string supplied by the primary model.
Its recommended correction is a two-phase protocol:

1. Persist unrestricted artifact and criteria in an immutable checkpoint with primary-assistant provenance.
2. Complete that tool interaction before Advisor dispatch.
3. Invoke Advisor later using only the opaque checkpoint ID and optional model routing.
4. Return unrestricted review text associated with that checkpoint.

This removes fresh natural-language parameters from the review call
while preserving both text channels at checkpoint time.
Checkpoint criteria can still carry the incident instruction verbatim one interaction earlier.
The protocol proves sequence and primary-assistant tool-call provenance,
not substantive authorship or instruction elimination.

`synthetic/hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4` ranked the artifact checkpoint ahead of a
blocking adjudicator.
Both reject the exact no-artifact incident,
but only the adjudicator introduces false positives that can withhold valid expressive reviews.

`synthetic/hf:Qwen/Qwen3.6-27B` stressed that ordering is not semantic prevention
and identified additional architectures:

- blind review that withholds the original objective;
- dual-context review that sends only artifact and evidence;
- consumer-boundary reconciliation before primary synthesis;
- claim decomposition that makes ungrounded review additions observable.

It also identified two defects in the initial Candidate A sketch:
`stage: 'approach' | 'result'` excludes partial drafts,
comparisons,
diagnoses,
and other valid artifacts;
and same-call artifact submission does not establish prior ownership.

A final review by `synthetic/hf:moonshotai/Kimi-K3` found no fatal interface flaw.
It confirmed that the protocol reaches the deterministic ceiling compatible with unrestricted prose,
then required the final wording to distinguish tool-call provenance from authorship,
state that criteria can carry the same task instruction one interaction earlier,
and record the manual-command usability regression.
Those qualifications are incorporated in the corrected recommendation.

### Revised design direction

Replace Candidate A's inline object with a persisted checkpoint whose artifact and criteria are unrestricted strings.
Do not require a closed stage enum.
Advisor review accepts only checkpoint identity and model routing,
then returns unrestricted prose unchanged.

Add non-blocking shadow role adjudication only as measurement.
A blocking gate remains undecided until a representative corpus demonstrates that it accepts rich answer-like
corrections grounded
in primary artifacts.
Prompt-only framing remains the lowest-protection baseline.

Blind and reduced-context review are not current recommendations because withholding objective
or evidence removes review
capabilities even though it leaves output vocabulary unrestricted.
Claim decomposition is useful for some artifacts but not universal.
Consumer reconciliation is complementary and needs a concrete enforcement seam before it can claim more than guidance.

No output taxonomy,
prose truncation,
answer-like string filter,
or fixed renderer belongs in the corrected design.

## Expressive verification requirements

Deterministic tests must prove:

- the exact legacy incident arguments fail before Advisor provider dispatch because no completed checkpoint exists;
- checkpoint creation rejects blank artifacts without an Advisor provider call;
- missing,
  unknown,
  cross-session,
  same-batch,
  user-authored,
  Advisor-authored,
  and mutated checkpoint references fail before Advisor dispatch;
- arbitrary multiline artifact and criteria text persist immutably
and survive request construction without semantic rewriting;
- accepted Advisor text containing detailed corrections,
  code,
  mathematics,
  quotations,
  and a full suggested rewrite returns byte-for-byte;
- tool and `/advisor` command share checkpoint creation and resolution,
  request construction,
  transport,
  failure handling,
  and rendering;
- prompt and tool descriptions contain evaluator ownership language rather than `answer` instructions;
- error details record opaque checkpoint identity without copying private artifact,
  criteria,
  or review text.

Behavioral evaluation must include:

- the stored primality incident and its exact replacement response,
  measured as role-drift evidence rather than blocked runtime output;
- a rich valid review that includes a complete suggested rewrite after evaluating a candidate;
- approach review,
  completed-answer review,
  code review,
  diagnosis review,
  and verification review;
- deliberately empty,
  copied-objective,
  and token candidate artifacts;
- repeated runs across eligible reviewer and adjudicator models before interpreting a null or isolated failure.

The positive control is essential:
any proposed role gate that rejects the incident must also accept rich answer-like corrections
when they are grounded in a real
primary artifact.
Otherwise the gate has merely recreated the rejected expressiveness loss.

## Corrected recommendation and rankings

### Ownership protocol ranking

1. **Persisted two-phase checkpoint**
   - Pros:
      preserves arbitrary artifact,
     criteria,
     and review prose;
     proves primary-assistant provenance and completed ordering;
     removes fresh text parameters from the review call.
   - Cons:
      adds one tool round trip and cannot prove artifact substance or review semantics.
2. **Inline artifact on the Advisor call**
   - Pros:
      preserves natural language and needs one call.
   - Cons:
      the artifact is fresh same-call text,
     so it does not establish prior ownership and remains an instruction channel.
3. **Prompt-only evaluator framing**
   - Pros:
      smallest interface change and unrestricted prose.
   - Cons:
      preserves the exact incident call shape and supplies no deterministic ownership invariant.

Ranking:
 persisted checkpoint > inline artifact > prompt-only framing,
because persistence adds provenance and completed ordering without narrowing language;
inline text ranks ahead of prompt-only framing because it at least requires an explicit review target.

### Reviewer context ranking

1. **Full objective,
   evidence,
   artifact,
   and criteria**
   - Pros:
      preserves every current review capability.
   - Cons:
      gives Advisor enough information to generate task-completion prose.
2. **Dual context with selected evidence**
   - Pros:
      reduces unrelated instruction pressure.
   - Cons:
      selection can hide the exact requirement or evidence Advisor should catch.
3. **Blind artifact-only review**
   - Pros:
      makes direct optimization for the original task harder.
   - Cons:
      cannot reliably review completeness,
     scope,
     or requirement compliance.

Ranking:
 full context > dual context > blind review,
because the corrected requirement rejects capability loss as well as vocabulary loss;
dual context ranks ahead of blind review because it retains some objective evidence.

### Role adjudication ranking

1. **Offline or non-blocking shadow measurement**
   - Pros:
      builds evidence about semantic drift without withholding Advisor output.
   - Cons:
      measures behavior but does not prevent it.
2. **No adjudication**
   - Pros:
      simplest path and no false-positive suppression.
   - Cons:
      supplies no behavioral signal beyond prompt and observed incidents.
3. **Blocking adjudication before measurement**
   - Pros:
      can reject likely substitution while leaving the Advisor schema unrestricted.
   - Cons:
      false positives withhold valid expressive reviews,
     reproducing the user's rejected trade through a model gate.

Ranking:
 shadow measurement > no adjudication > unmeasured blocking,
because shadowing buys evidence without changing output;
no adjudication ranks ahead of blocking because preserving valid rich review is a hard requirement.

### Recommended interface

```typescript
type AdvisorCheckpointParams = {
  readonly artifact: string;
  readonly criteria?: string;
};

type AdvisorToolParams = {
  readonly checkpoint: string;
  readonly model?: string;
};
```

`advisor_checkpoint` persists an immutable primary-assistant tool call and result,
then returns an opaque ID.
A later primary-model iteration calls `advisor` using only that ID and optional model routing.
The Advisor runner resolves the checkpoint from the active session branch,
encodes full context,
artifact,
and criteria as separately labeled data,
and applies positive evaluator framing.
Accepted Advisor text is returned byte-for-byte through the existing expressive rendering path.

The manual `/advisor` adapter uses the latest unresolved explicit checkpoint when one exists.
Otherwise it mechanically checkpoints the latest ordinary assistant text block after the latest user message,
excluding tool-only blocks and Advisor messages.
It reports no reviewable artifact when neither source exists.
This is a usability regression from whole-session manual review in early or tool-only sessions,
but it is required for the same provenance invariant.
Both entry points then use the same internal runner.

The deterministic guarantee is limited but real:
every Advisor review is attached to primary-assistant text persisted before the review call,
and the exact live task-delegation call shape from the incident is invalid.
The same task-performing criteria can be laundered through a checkpoint.
The design does not deterministically prevent a weak checkpoint,
an answer-like Advisor review,
or primary copying.
Within the full-expressiveness constraint,
there is no runtime semantic protection beyond ordering and evaluator framing;
shadow scoring supplies only behavioral evidence.

## Rejected conclusions

- The result was not caused by context truncation.
  Stored details record `truncated: false`.
- `openai-codex/gpt-5.6-sol` was not explicitly requested.
  Default selection chose it.
- The local Advisor extension is not designed to provide final answers.
  Its source prompt says the opposite.
- The incident is not only a provider-model defect.
  The caller interface admitted and encouraged an answer-generating focus question.

## Exact next action

Obtain user acceptance or delegated authority for the persisted two-phase checkpoint design.
Then implement it test-first inside `package/pi-plugin/advisor`,
without output filtering or blocking role adjudication.
Start with checkpoint provenance rejection and byte-for-byte expressive round-trip tests.
Do not change `package/pi-plugin/goal`.

## Commits

- `d159018a8`,
   `docs(advisor): start answer-substitution handover`,
   created this live investigation record.
- `f4ed2cb9f`,
   `docs(advisor): record handover checkpoint`,
   recorded incident measurements and constraints.
- `645d23b63`,
   `docs(advisor): trace answer-substitution interface`,
   traced the conflicting production seams.
- `16c7dc330`,
   `docs(advisor): map role-enforcement layers`,
   recorded layered guarantees and initial candidates.
- `40f26273a`,
   `docs(advisor): record official Advisor precedent`,
   added current Anthropic evidence.
- `137dac233`,
   `docs(advisor): define prevention verification`,
   defined negative cases and positive controls.
- `bb8a9cd17`,
   `docs(advisor): recommend symbolic review contract`,
   recorded independent ranking and recommendation.
- `fd1f3dc99`,
   `docs(advisor): qualify symbolic review guarantee`,
   incorporated final independent-review constraints.
- `79b62b595`,
   `docs(advisor): close prevention design investigation`,
   closed the now-rejected first comparison.
- `b298fbf49`,
   `docs(advisor): require expressive review output`,
   recorded the user's correction.
- `8c2124313`,
   `docs(advisor): redesign around artifact ownership`,
   defined expressive candidates and tests.
- `6e70c303c`,
   `docs(advisor): review expressive ownership design`,
   recorded independent design corrections.
- `a78a5a825`,
   `docs(advisor): recommend expressive checkpoint protocol`,
   recorded the corrected ranking and interface.

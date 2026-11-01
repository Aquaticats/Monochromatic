# Auto-mode ground-up rebuild plan

## Status and review request

The [complete question inventory](rebuild-question-inventory.md) supersedes the incomplete Q7-only handoff.
It presents every current independent choice and names later dependencies and research work.

Revised to the user's staged design on 2026-10-19.
Implementation is stopped.
Plan discussion does not authorize implementation.
The [working understanding](rebuild-understanding.md) records meanings,
evidence,
corrections,
and unresolved boundaries.
The [HTML review](ground-up-rebuild-review.html) shows the active flow,
concrete cases,
and remaining questions.
Open that review in Helium for the handoff.

The previous first-pass useful-question selector and three-choice interpretation are withdrawn.
Every effective rule is a first-pass question with the choices
`approve`,
`deny`,
`ask`,
and `not-applicable`.
The approximately 100 effective rules are an expectation,
not a count imposed by code.
The earlier shortage-of-useful-questions question was also withdrawn;
unresolved-rule count does not measure the supply of useful follow-ups.

This document records settled requirements separately from proposed semantics.
The user selected Q1 B and Q4 A:
admit any unique leading probability,
and retain the latest genuine human request separately from the capped historical context.
The [next review round](rebuild-review-round-2.md) contains Q7 to Q9 and Q11 to Q13:
aggregation,
approval sufficiency,
follow-up evidence,
shadow delivery,
and evidence retention.
Q10 is withdrawn:
the Luna review allowance applies per tool call;
shadow observations are counted separately.
The visual review presents these options together;
their dependent engineering choices follow the answers.
Q2 is withdrawn:
an established `ask` is a resolved outcome requiring the human,
not model uncertainty.
Q6 is withdrawn:
every tool call is judged individually,
including calls Pi schedules in parallel.
`codemode` is one outer call;
parse its complete body without creating inner approval units.
The previous whole-parallel-batch review requirement is superseded.
The user corrected the approval baseline:
assess reversibility,
not exact requested-package matching.
The former Q3 and Q5 alternatives are withdrawn;
their remaining engineering details follow that baseline.
Dependency qualification,
shadow scheduling,
retention,
and size fit remain open engineering work.
The user requires an ordinary Pi plugin using supported public APIs only.
Maintaining or copying Pi internals,
duplicating existing Pi functionality,
and private Pi APIs are prohibited,
without exception.
Gracefully fall back when a desired behavior is unavailable through public APIs.
The proposed private loading hook is withdrawn,
including in-memory source transformation.
Cleanup retires the unapproved architecture;
replacement implementation remains stopped.

The preserved reference worktree is
`/var/home/user/temp/agent/pi-auto-mode-before-rebuild-20261019`
at `e74f84bb9f60c3e1ea90f1325cf2e64947715ba0`.
Worker edits were checkpointed in `3ef7f32d19`;
`7ac15b19b4` deletes the old `src` from `main`.
The user removed `bin`,
`native`,
and `dist` from the active package on 2026-10-19.
Old native code remains in the preserved reference worktree,
not as a replacement implementation.
Ordinary plugin output may recreate `dist`;
rebuilding or transforming Pi is prohibited.
The agent introduced the old architecture without user approval.
It was not a shared design decision.
Existing abstractions,
dependencies,
and historical test assertions get no automatic place in this rebuild.

## Coding-agent autonomy and reversibility

The user explicitly rejects preventing tests merely because they run in another package.
A coding agent can inspect dependencies,
test consumers,
compare alternatives,
and perform useful supporting work
without the human enumerating every package and command.
The former mandatory positive request-scope gate and package-match follow-ups were too restrictive and are withdrawn.

Assess the prepared outer call's actual effects and reversibility:
what changes,
what can be restored,
how restoration works,
and what consequences remain after restoration.
Consider recovery effort,
existing uncommitted work,
shared state,
external effects,
and resource consumption.
A command called `test` is not an effect description;
a cross-package test against disposable local fixtures differs from a test that deletes shared data.
Likewise,
a local edit is not irreversible merely because the user did not spell out that intermediate step.
An edit recoverable from a retained prior state differs from overwriting unrecorded user work.

Routine work with established reversible effects can proceed automatically without exact-call human approval.
Code may establish this deterministically when the relevant effects are known;
otherwise the every-rule model review assesses it.
Unknown effects require investigation or an explained ask about those effects,
not an ask caused only by a different package name.
Irreversible or costly-to-recover consequences receive the corresponding authorization review.
Reversibility does not cancel an explicit human prohibition or supply a missing human-only decision.
Task context helps explain the work;
exact package or verb matching is not a generic permission condition.
Repeating an unchanged test can gather evidence about reproducibility or timing;
identical command text alone is not grounds to veto a retry.

Acceptance examples for this baseline:

- A change in package A is verified by running package B's consumer tests with disposable local fixtures.
  Approve when the effects are established as routine and reversible;
  B's name is not an objection.
- An agent tests a hypothesis by editing a disposable checkout with the prior contents retained.
  Treat the recoverable experiment as ordinary autonomous work unless an explicit instruction forbids it.
- A command named `test` resets a shared database without an established recovery path.
  Review that data loss and affected users;
  the command name grants no shortcut.
- Uploading a report can disclose its content even if the upload can later be deleted.
  Evaluate the disclosure;
  deleting the uploaded copy does not reverse every effect.

## Built-in rules: content deferred

The user explicitly defers deciding which built-in rules to include and their exact wording.
The previously proposed catalogue is withdrawn from the active design,
not accepted or rejected rule by rule.
Do not select a count,
names,
identities,
or membership now.

The settled architecture requires multiple general-agent built-in rules in addition to effective authored rules.
Every built-in will receive its own first-pass question with the same four choices:
`approve`,
`deny`,
`ask`,
and `not-applicable`.
Every first pass includes all built-ins alongside all effective authored rules;
code does not select a useful subset.
The identities and rule text are unspecified inputs to this architecture until that design work is resumed.
Reversibility remains an accepted assessment principle;
it does not finalize a particular built-in rule.

Continue reviewing the stages,
snapshot,
aggregation,
and human feedback without deciding the built-in catalogue.

## Settled stages

### Stage 0: effective rules on every Pi start

Send parsed `AGENTS.md` to Luna to identify rules that could be potentially relevant to agent operations.
Retain original rule text,
source,
heading context,
and instruction precedence.
Materialize the effective ruleset and add general-agent built-in rules.
The user expects approximately 100 effective authored rules;
do not pad,
truncate,
or select to that number.
Every Pi start performs this classification,
even when cached parsing is reusable.
If classification fails but fresh parsing and source capture succeed,
retain all parsed rules for each call's review.
This follows the accepted least-degrading fallback rule;
failed filtering alone does not require human approval.
Final request capacity still needs measurement with all effective sources and built-ins.

The removed implementation contained relevance classification,
retained in the reference worktree:
`native/contract/integration/native-host/startup-relevance.mjs`
and `native/contract/lifecycle/rule-relevance-cache/refresh.ts`.
However,
the old warm-cache path classified only missing rules.
The old definition also excluded tool choice,
syntax,
style,
testing,
documentation,
and workflow.
Those exclusions do not establish the new remit.
Historical evidence in `doc/handover/pi-auto-mode-frontier-2026-10-18.md`
records 25 retained of 211 classified rules;
it is not evidence for the new expected count.
The old threshold and 200-question chunking are legacy choices requiring fresh qualification.

Startup classification is session-wide potential relevance,
not a per-operation useful-question selector.
Rule changes require a new effective-rules snapshot before another operation can inherit it.
A failed or incomplete classification must be visible;
its missing rules cannot silently disappear.
Startup failure behavior requires qualification before implementation acceptance.

### Stage 1: capture and parse one outer tool call

Every call is judged individually,
including calls Pi schedules in parallel.
Each invocation has a separate immutable snapshot,
every-rule answer set,
review allowance,
and manual decision.
Do not combine siblings into a shared review or create joint-interaction outcomes.
Cleared local tests remain permitted when a separate upload is denied;
neither needs a new proposal.
Pi owns scheduling.
Individual decisions do not promise that cleared siblings execute before pending reviews finish.

`codemode` counts as one outer tool call.
Parse its complete body,
including contained operations and their relationships,
for that single decision.
Do not create inner approval units,
separate rule answer sets,
or fresh review budgets within its body.

Include permitted argument mutations visible before review in the prepared call's snapshot.
Parse each tool call deeply,
retaining tool identity,
named arguments,
resources,
and execution structure.
For shell calls this includes control flow,
redirects,
expansions,
literal values,
and nested syntax.
Parsing alone does not establish expanded runtime values or actual side effects.

Bind all review evidence to one immutable outer call,
instructions,
tool catalog,
and context snapshot.
A changed argument,
instruction,
branch,
or tool definition observed within the review boundary invalidates approval.
Later extensions mutating arguments after review are explicitly outside coverage and need no guard.
Queue adjacency,
parallel dispatch,
or a shared workflow does not merge distinct calls.
Future dynamically created calls are not pretended to exist already.
Qualify the public outer-call boundary without recreating Pi execution or splitting `codemode` into approvals.
The review budget,
evidence,
outcome,
and manual feedback all bind to that outer invocation.

### Stage 2: deterministic parsed checks

Code can immediately return `approve`,
`deny`,
or `ask` when a named deterministic condition establishes it.
Record the condition,
parsed evidence,
exact outer call,
and outcome.
A parser-success flag or apparently read-only command is not by itself proof that the call is authorized.
The deterministic approval catalogue is a review decision,
distinct from selecting model questions.
Each established verdict ends automatic review for this call:
approve permits it,
deny blocks it,
and ask presents the explained human decision.
Do not use Luna to reconsider those resolved decisions.
Other unresolved calls retain their own independent Stage 3 review and allowance.
A completely deterministic call uses no per-call Luna requests.

### Stage 3: every effective rule becomes a Luna question

Send the complete prepared outer call,
effective ruleset,
and bounded visible context.
For the first pass,
each rule is a question.
Do not select a useful subset in code.
Each question offers exactly `approve`,
`deny`,
`ask`,
and `not-applicable`.
Bind answer keys to rule occurrences,
including built-ins,
so missing or duplicate answers are detectable.
Every answer belongs to this outer call's snapshot and its identified rule occurrence.
Each separate tool call gets its own every-rule first pass;
no sibling-attribution schema is needed.
Each assessment includes the instruction hierarchy and applicable explicit user exceptions.
An unrelated rule can return `not-applicable`;
it need not manufacture permission or uncertainty.

The historical context length is `clamp(5, units since the latest human message, 10)`.
A unit is an assistant thinking/output block or a tool-call set,
not Pi's `turn_end` event.
Use available history when fewer than five units exist;
never invent padding.
The complete pending outer call is a separate snapshot field and is never cut down by this history window.
Private thinking content remains excluded under the existing payload constraint;
its boundary can count as a unit without transmitting its contents.
Retain the latest genuine human request in a separate snapshot field,
even when it falls outside the window.
This field is additional to the specified historical window;
the user selected Q4 A.
Use original provenance,
not merely an API user role,
to identify that request.

Batch only as required by verified endpoint capacity,
with the complete rule context accompanying each partition.
All first-pass rules must receive coverage before automatic approval.
Early terminal denial or required human review ends automatic review for this call,
without fabricating answers or ending another call's independent review.
There is no replacement whole-operation judge and no permission/prohibition taxonomy stage.

### Stage 4: aggregate, then refine only unresolved evidence

Normalize Luna's answers into an attributed object and compute this outer call's outcome in code.
No averaging of unrelated rules,
majority voting,
or repeated sampling until approval.
Admit the unique highest-probability choice in a valid response,
with no minimum probability or margin.
The user selected Q1 B.
Separate confidence metadata does not add another admission threshold.
Exact ties,
invalid responses,
or missing answers remain unresolved,
not a fifth answer choice.

Only while this call remains unresolved,
construct questions addressing that uncertainty.
An established `ask` goes to the human;
do not spend follow-ups reconsidering that verdict.
An applicable authored rule can require asking even when all relevant facts are known.
For example,
decompose uncertainty about a test command’s reversibility into
which environment it targets,
whether its writes are confined to disposable fixtures,
and whether any existing state would be lost or externally exposed.
Keep the complete prepared outer call and relevant rule context visible in every follow-up.
Do not treat uncertainty in one rule as proof that only one useful follow-up exists.
Question wording and bindings must expose the distinction being tested.
The generation and validation mechanism follows the accepted semantic examples;
the previous fixed-form selector is not a settled requirement.

Each outer tool call has at most three sequential review batches and three parallel Luna requests per batch.
HTTP retries consume those call slots.
These are per-call model-review limits,
not a limit on how many tool calls Pi may schedule in parallel.
Stop reviewing a resolved call;
an unresolved call after its permitted batches routes to `ask`.
Each separate outer call has its own allowance.
`codemode` contained operations do not receive additional allowances.
Stage 0 is startup work,
outside the per-call review budget.
Shadow requests are counted separately from the Luna allowance.
Conflicting follow-up answers require an explicit reconciliation contract;
newer answers do not silently erase prior evidence merely because they allow execution.

### Stage 5: manual decision and session feedback

Every ask must give the human decision-relevant information beyond the chat transcript and `AGENTS.md`,
which the human can already inspect.
This applies to deterministic asks,
admitted model asks,
exhausted uncertainty,
and review failures.
Before the choices,
explain:

- The exact pending action and affected target,
  briefly enough to identify the decision.
- The concrete evidence or conflicting interpretations that triggered escalation,
  tied to the relevant rule and operation detail.
- The confirmation or judgment required from the human and the rule requiring it.
  If evidence remains unresolved,
  identify the missing fact and why review could not resolve it.
  A rule-required ask need not involve uncertainty;
  do not invent missing evidence.
- What Approve will permit and what Stop will prevent or abort.

A transcript recap,
rule dump,
score alone,
or generic “approve this tool call?”
is not an explanation.
Name limits of the reviewer's available context as those limits;
do not claim the human omitted information merely because the snapshot lacks it.
Explain only recorded evidence.
Do not invent Luna's reasoning from probabilities or require private thinking.
For unavailable review evidence,
state the actual failure and which checks could not be completed.
Keep the explanation attached to the exact snapshot and persist it with the resulting decision.

Illustrative ask:
“The upload targets workspace B.
Your request specifies the approved team workspace;
the reviewed evidence does not establish that B is that workspace.
Approve permits this report upload to B.
Stop aborts the run without uploading.
Confirm whether B is the intended destination;
you may include a reason.”

Offer Approve and Stop for the explained single outer call,
with an optional reason for either choice.
Approve records `user approved tool call ...`;
Stop records `user declined tool call ...`.
Include outer-call identity,
review snapshot identity,
and supplied reason in durable and model-visible session records.
Skipping the optional reason preserves the selected decision.
Stop retains the existing run-abort behavior;
dismissal or unavailable UI withholds execution.
An established denial does not turn into an Allow prompt.
A manual approval applies only to the displayed single outer call after a freshness check,
not unrelated siblings.
The existing human Stop interaction aborts the run;
that is distinct from automatic denial of an individual call.

Pi 1.3.0 provides `ui.select` and `ui.input` in both TUI and RPC declarations.
Use a choice followed by optional reason entry as the initial cross-mode proposal.
Its `ui.custom` RPC implementation returns `undefined`,
so a custom TUI form alone cannot satisfy both modes.
The design must verify cancellation during each input step and interruption of stale prompts.

`appendEntry` supplies audit persistence but is not model-visible by itself.
`sendMessage` with `triggerTurn: false` is the candidate feedback channel.
While streaming,
the host queues it and flushes after tool results;
after Stop,
it must remain on the next resume.
`deliverAs: nextTurn` is not equivalent to the next internal assistant continuation.
Verify approval and decline with actual continuation,
abort,
reload,
and resume.
Feedback messages and compaction summaries converted to API user roles must not reset the genuine human anchor.

## Recorded semantic decisions

### Q1 B accepted: admit any unique leading probability

The previous `unknown` was an authored choice,
not automatic provider uncertainty.
The current decisions endpoints return probabilities for the caller's choices and may return separate confidence.
Removing `unknown` therefore removes the input to the former 0.2/0.6 formula.
Do not substitute `ask`,
missing-score zero,
or `1 - confidence` into it.

The user chose B:
any unique leading probability admits its choice.
For approve 0.46,
deny 0.04,
ask 0.29,
not-applicable 0.21,
the admitted answer is `approve`.
There is no 0.6 minimum,
winning-margin floor,
or separate confidence gate.
The former minimum-threshold proposal is rejected and removed from the active review.
For approve 0.40,
deny 0.40,
ask 0.10,
not-applicable 0.10,
the exact tie remains unresolved.
Invalid distributions and missing answers remain unresolved too.
Attribution,
expected labels,
finite normalized probabilities,
and reported-choice consistency still require validation.
Admission concerns one rule answer;
code still combines the rules governing that same outer call.

### Q2 withdrawn: ask is a resolved verdict

Some applicable `AGENTS.md` rules require the human to confirm or decide.
An established ask follows those rules;
it is not an ambiguous answer needing another model check.
Deterministic approve,
deny,
and ask terminate automatic review for the individual call.
Luna runs only for unresolved decisions;
resolved siblings do not need reconsideration.
After Luna,
code aggregates answers;
an established ask routes this call to the human.
Only unresolved individual-call outcomes enter follow-up review.
Q1 B settles numeric admission.
The former proposal to spend remaining batches on every ask is withdrawn.

Q7 concerns reduction of answers governing the same call,
not pooling unrelated calls' outcomes.
Q8 separately asks whether complete all-not-applicable coverage permits the affected action.
Neither question is answered by the selective-execution correction.
A deny based on a superseded instruction is invalid evidence,
not a lower-priority veto.
Precedence must be reflected in contextual questions and tested with explicit exceptions.

### Q3 withdrawn: exact-call approval is not the default autonomy boundary

The earlier Q3 concerned skipping Luna,
not whether Luna could approve ordinary work.
The user’s correction does not select a deterministic shortcut implementation.
Withdraw that abstract menu while designing concrete effect-and-recovery conditions;
the accepted behavior is that package mismatch alone cannot block or escalate routine reversible work.
Deterministic approval conditions must express actual effects and recovery properties,
including cross-package verification,
rather than exact package matching or a test-command allowlist.
The concrete effect evidence and recovery checks remain engineering work;
they are not a request to confirm the rejected boundary again.

### Q4 A accepted: retain the latest human request separately

The user chose A:
include the latest genuine human request in a separate snapshot field,
plus the specified five-to-ten-unit visible history and the complete prepared outer call.
The separate request is retained even when it lies outside the historical window.
For example,
after twelve assistant/tool units following “review only”,
that instruction remains in the request field.
The historical window still follows its existing clamp;
it is not expanded to include all older conversation.
API user-role feedback,
compaction entries,
and other synthetic messages do not become the genuine human anchor.
The former capped-history-only alternative is rejected and removed from the active review.

### Q6 withdrawn: every outer tool call is judged individually

The latest user correction on 2026-10-19 supersedes the earlier whole-parallel-batch review requirement.
Parallel A and B receive independent snapshots,
rule answers,
review allowances,
and manual decisions.
`codemode` remains one outer call even when its parsed body contains multiple operations.
There is no shared group verdict,
joint call-attribution question,
or sibling-conflict decision branch.
The earlier complete-group and shared-budget requirements are historical and withdrawn.
Pi retains scheduling,
including result-dependent sequential preparation and parallel dispatch barriers.
Independent judgment does not promise that A starts while B awaits review or a human decision.

Installed Pi 1.3.0 source supplies separate dispatch paths in `dist/agent-loop.js`:
`executeToolCalls` at line 366 selects the mode using configuration and tool execution metadata;
`executeToolCallsSequential` at line 374 prepares and executes each call in its loop;
`executeToolCallsParallel` at line 411 prepares calls before its `Promise.all` dispatch at line 453.
Source inspection supports the distinction,
not a claim that the replacement integration already works.
Verify configured modes,
outer-call boundaries,
pre-review argument mutation,
and cancellation at the host boundary.

## Dependency tree after this round

Accepted Q1 B fixes numeric boundaries and unlocks remaining per-call reduction acceptance examples.
The settled terminal ask contract supplies deterministic and model-resolved ask acceptance cases.
The accepted autonomy baseline guides behavioral examples.
Catalogue-specific fixtures wait until built-in rule design is resumed;
the catalogue is not part of this round.
Accepted Q4 A fixes the separate human anchor and unlocks snapshot boundary fixtures.
The individual-call rule determines snapshot,
budget,
`codemode`,
and manual-decision fixtures.
Then settle conflicting follow-ups,
Stage 0 failure handling,
shadow delivery and scheduling,
and retention using concrete designs and their measured constraints.
Source budgets remain proposals pending those designs and exact resolved dependency inventories.
Do not present later dependent choices as settled merely because the flow is now defined.

## Proposed agent-guidance corrections

The former question selector contradicted the stated architecture;
the earlier scarcity question also inferred a constraint from incomplete evidence.
These are proposed edits,
not modifications to `AGENTS.md`.
The built-in correction adds a preservation requirement to the proposed `QPM` tightening:

- Tighten `VHI` to:
  “Handoffs state purpose,
  changes,
  inspection,
  and response.
  Use connected flowcharts with labeled branches for every flow.
  Verify hidden examples too;
  open in the requested browser.”
- Tighten `QPM` to:
  “Test questions against settled requirements.
  Define groups before proposing timing choices.
  Preserve other requirements when correcting one.
  Measure facts;
  ask only unresolved choices.”

The scope correction also motivates a proposed tightening of the existing `AUT` guidance:
“Autonomy includes reversible supporting work,
including cross-package tests.
A different package is not scope expansion.
Honor explicit limits and leave undecided product choices to the user.”
This is a proposal,
not an edit to `AGENTS.md`.

The missing ask explanation also exposes a diagnostic-guidance gap.
Proposed replacement for the existing `DGT` body:
“Diagnostics name affected inputs and calls.
Every ask explains evidence,
required judgment,
any uncertainty,
and each choice’s effect;
transcript or rule repetition is insufficient.
State valid remedies.”

## Proposed high-level modules and source budgets

Budgets are raw Unicode characters including types,
comments,
and whitespace,
not measured sizes or forecasts.
Existing max-lines lint still requires coherent files;
formatting is not removed to fit a budget.
Startup and manual feedback responsibilities are included explicitly in the existing module budgets.
Their fit remains unverified.

### Public Pi plugin integration: provisional 10,000 to 16,000 characters

Purpose:
capture each outer tool call independently and bind its review to dispatch.
Interface:
supported public plugin events and APIs;
Pi retains execution,
results,
and configured extensions.
Observe supported parallel,
sequential,
nested,
denied,
aborted,
and shutdown events.
Pi owns execution and lifecycle behavior;
the plugin must not duplicate those implementations.
Qualify prepared input capture and the `codemode` outer-call boundary through supported public events.

The user explicitly directs the rebuild to Pi's plugin API.
Do not patch installed files,
transform copied Pi source,
replace private modules through loading hooks,
or monkey-patch Pi's runtime methods.
Leaving installed package files unchanged does not make a transformed runtime acceptable.
The former `registerHooks` candidate is withdrawn,
not awaiting qualification or approval.
Qualify public event ordering,
individual-call arguments,
and cancellation.
If those public surfaces cannot provide a requirement,
use graceful fallback rather than reconstructing Pi.
Degrade as little as possible,
aiming for no degradation of the end-user experience where possible.
First seek an equivalent supported public-API path;
otherwise preserve the greatest supported functionality.
Individual-call review is the accepted design,
including parallel calls;
no batch-review fallback is needed.
Missing integration capability alone does not require manual approval,
blocked execution,
or disabled auto-mode.
Keep ordinary Pi usable;
disclose material limitations without claiming guarantees the fallback lacks.
An actual rule requiring human approval still requires human approval;
a capability gap is not such a rule.
Do not recreate Pi tools,
scheduling,
argument preparation,
sessions,
or UI to recover a missing capability.
Keep one review per outer invocation and one approval for the complete `codemode` body.

The historical disposable public-plugin consumer passes 20 asserted cases,
including deliberate limitation cases,
through `mise run lint:types` and `mise run probe` in `~/temp/agent/pi-public-plugin-probe-20261019`.
It uses extension factories,
a scripted public provider registration,
and inert tools,
without copied Pi source.
Its earlier shared-review prototype used `prepareLoadout` metadata and `tool_execution_start` message capture,
then held the final sibling's `tool_call` handler for shared review.
That mechanism is historical,
not part of the individual-call design.
The probe exercised approved parallel execution,
denial,
review failure,
per-tool sequential review,
and skipped-callback accounting through `tool_execution_end`.
Two additional fixtures review both declared calls and return public per-ID `tool_call` blocking results:
allow A/block B executes only A,
and block A/allow B executes only B.
Both produce separate successful/blocked results and continue without `ctx.abort()`.
The earlier whole-run abort belongs to the last-sibling review prototype,
not a Pi batch limitation.
Review each call independently at its supported public boundary,
keyed by `toolCallId`.
The prepared-input strategy still requires qualification;
do not copy Pi to supply it.
Qualify preparation and mutations visible before review,
configured execution modes,
and `codemode` outer-call identity.
The user accepts later extensions changing arguments after review as an unsupported case on 2026-10-19.
Design as though those later mutations do not occur;
do not add ordering guards,
prompts,
fallback triggers,
or private hooks to defend against them.
This limitation is accepted,
not an unresolved qualification blocker.
`message_end` handler errors are reported and swallowed at `extensions/runner.js:889`;
throwing there alone is not an execution gate.
A pending or failed review must still block at `tool_call`.
Do not wait for sibling `tool_call` events inside the first sibling handler:
the parallel dispatcher prepares siblings sequentially before execution,
so that would deadlock.

### Rules and review input capture: 8,000 to 14,000 characters

Purpose:
startup relevance,
built-in rule registration,
visible history,
and structured prepared operations.
Interface:
one immutable effective-rules snapshot and one immutable outer-call snapshot per review.
Preserve original text and provenance without inventing meaning categories.
Do not silently discard untagged substantive instructions.
Reuse parsing caches only without bypassing every-start classification.

Pi's `getBranch` returns raw active-branch entries;
`buildSessionContext` resolves compaction and context edits through `buildSessionProjection`.
Use original provenance to count human requests and the resolved view for visible context.
Later context transformations also need qualification.
No closed role allow-list may reject ordinary incoming chat.
A historical parse failure cannot authorize the pending call.

### Review scheduling and manual feedback: 8,000 to 12,000 characters

Purpose:
every-rule first coverage,
bounded adaptive follow-ups,
and recorded human decisions.
Interface:
one review entry returns disposition and correlated evidence.
Question IDs retain rule occurrence,
outer-call snapshot,
and follow-up purpose.
No useful-rule filtering occurs in the first pass.
Transport retries and partitioning consume explicit budget slots.
Actual endpoint capacity must be verified with the selected effective rules and payload sizes.

Subsequent questions address recorded uncertainty;
contradictory results cannot be silently replaced.
Every ask explains the evidence,
required human decision,
any uncertainty,
and consequences of each choice.
That explanation,
the manual decision,
and the optional reason persist into model-visible continuation or resume.
Keep feedback provenance separate from genuine incoming human requests.

### Deterministic decision resolution: 2,000 to 4,000 characters

Purpose:
immediate parsed checks,
answer admission,
and individual-call reduction in ordinary code.
Interface:
named parsed evidence or attributed answer distributions yield a disposition with decisive references.
Q1 B admits any unique probability leader;
established approve,
deny,
and ask terminate automatic review.
Deterministic approval must establish actual effects and recovery properties while satisfying other applicable checks;
it cannot bypass applicable rules once they are defined.
No specific built-in catalogue is selected here.
Parser success,
missing fields,
and unavailable answers do not imply authorization.
Whether a complete admitted not-applicable-only set permits execution remains open in Q8.
Validate answer attribution,
expected labels,
finite probabilities,
normalization,
and leading-choice consistency.
Accept irrelevant response metadata;
do not require optional confidence or usage fields that reduction does not consume.
Missing,
malformed,
refused,
or unavailable answers are missing evidence.

### Decisions HTTP transport: 6,000 to 10,000 characters

Purpose:
encode the shared questions for each endpoint and decode their numeric answers.
Interface:
a captured state plus named questions returns attributed distributions and serving-model metadata.

The primary adapter uses `https://api.llmgateway.io/v1/decisions`,
model `openai/gpt-6-luna-decisions`,
and `AUTO_MODE_LLMGATEWAY_API_KEY`.
The model's `openai/` prefix is a route identifier,
not an SDK import.
The shadow adapter uses `https://openrouter.ai/api/alpha/decisions`,
model `respan/span-02`,
and `AUTO_MODE_OPENROUTER_API_KEY`.

It owns lazy credential reads,
direct fetch,
cancellation,
bounded response consumption,
provider-specific envelope differences,
and attributable failures.
Redirects and unrequested provider fallback are disabled.
No adapter adds hidden retries beyond the scheduler's counted attempts.
Question text,
original rule bindings,
and probability values survive endpoint conversion.
The exact response-size and timeout values remain qualification inputs,
not invented authorization thresholds.

### Span 02 shadow: 4,000 to 6,000 characters

Purpose:
compare Span 02 with the actual primary question sets without influencing execution.
Interface:
observe an immutable primary request and its result;
retain a correlated shadow comparison.

For each call's review it preserves the same state,
question text,
and four answer choices.
The proposed mirror also covers startup relevance using that request’s unchanged relevance-answer schema;
it does not convert startup relevance into operation approval.
Only the endpoint envelope and model identifier differ.
Shadow failures do not become primary permission failures or hold execution open.
Shutdown cancels and joins observer work.

The retained queue,
unmatched comparisons,
and completed-state cache have explicit bounds.
Correlation includes request occurrence,
so identical repeated requests do not overwrite one another.
It does not repeatedly scan ever-growing log files or import the old built policy implementation.

The accepted three-by-three allowance governs Luna review separately for each outer tool call.
Shadow requests are counted separately and do not consume that allowance;
Q10's combined-budget menu is withdrawn.
Dispatch after primary review batches with a concurrency limit of one remains a scheduling proposal,
not an accepted resource limit or a new budget question.

### Diagnostics and lifecycle evidence: 3,000 to 5,000 characters

Purpose:
reconstruct a verdict without logging the same inputs at every internal helper call.
Interface:
correlated operation,
batch,
request,
response,
decision,
failure,
and cleanup records.

Capture full model-visible input once per operation snapshot,
distributions once per completed request,
and the final decisive rules and fallback reason once per review.
Keep credential values out of request records and error output.
Use the shared tagged logger,
retaining the user's project-first destination with user-state fallback.
No second logging framework or bespoke filesystem-discovery stack is proposed.

The current logger's file sink is append-only;
its inspected interface offers a directory,
not rotation or retention.
Bounded shadow memory does not bound these persistent files.
Persistent retention is therefore an unresolved shared-logger requirement.
Any proposed shared-logger change needs a concrete design and version bump for auto-publishing.
It is not included as an already solved property of this rebuild.

The 2026-10-19 source review identifies separate logging qualification requirements:
default sinks include console output,
and `info` maps to `console.info`.
Verify the chosen configuration through actual RPC stdout and interactive sessions.
The current `flush` may return at its deadline while sink work continues,
and the file sink reports append failures without propagating them to the caller.
Do not equate `flush` with guaranteed durable evidence or complete drain.
Record the evidence-loss and overload behavior separately from disk retention.

### Plugin entry and registration: provisional 1,000 to 2,000 characters

Purpose:
load auto-mode through Pi's supported plugin mechanism with its normal settings.
Interface:
an extension factory using public APIs.

Preserve configured extensions,
skills,
tools,
and RPC behavior.
Do not replace the user's `pi` command with a launcher into a modified Pi distribution.
Report unsupported required plugin capabilities at startup rather than pretending the guard is active.
The old launcher and generated native release remain historical artifacts,
not a compliant plugin installation.

## Overall size proposal

- Production implementation,
  including comments and types:
  42,000 to 69,000 characters.
- Verification source and fixtures:
  30,000 to 55,000 characters.
- Build and task configuration:
  2,000 to 4,000 characters.
- Maintained replacement documentation:
  20,000 to 35,000 characters.
- Total maintained replacement text:
  94,000 to 163,000 characters.

These totals sum the proposed budgets;
they are not estimates derived from an implemented prototype.
Historical documents and archived evidence are reported separately rather than counted as replacement implementation.
No source-line target is proposed.

Proposed deployment budgets for review:

- Package-owned runtime output:
  at most 1,000,000 uncompressed bytes.
- Total incremental production dependency closure,
  including the shell parser and logger:
  at most 5,000,000 uncompressed bytes,
  counted once per actual package.
- Existing Pi and Node installations:
  measured and disclosed separately,
  never copied into this package.
- Development dependencies,
  caches,
  build tools,
  logs,
  and archived releases:
  separately itemized rather than hidden inside an advertised runtime figure.

These byte budgets are proposed ceilings,
not verified current footprints.
The final report must include both package-directory usage and followed runtime dependency targets.
No claim of solving the reported package growth is justified by a small entry bundle alone.

## Dependency inventory

This section lists every intended direct package dependency and every symbol auto-mode would use from it.
It also lists runtime transitive packages and host/toolchain dependencies.
An unlisted direct import requires a plan update before implementation.
Function names describe intended calls,
not permission to use the rest of a dependency's interface.
No package installation or dependency adoption is performed by this document.

### Production dependency: shared logger

Package:
`@monochromatic-dev/logger-core`,
inspected version `0.4.4`,
declared `workspace:*`.
Import:
`@monochromatic-dev/logger-core/ts`.

- `tagged`:
  construct module and function loggers with correlated tags.
- Returned `debug`,
  `info`,
  `warn`,
  and `error` methods:
  lifecycle detail,
  decision evidence,
  degraded review,
  and actionable failures respectively.
- Returned `flush`:
  request bounded flushing during shutdown;
  qualify its timeout and failure behavior before claiming complete drain or durable evidence.
- `Logger`:
  type-only declaration for passing an already tagged logger.

Necessary because this repository requires this logger and complete decision diagnostics.
Its manifest declares no production package dependencies.
The rebuild does not call its internal testing exports,
create alternative sinks,
or change its version implicitly.

### Production dependency: shared shell syntax projection

Package:
`@monochromatic-dev/agent-harness-shared-shell-command-analyzer`,
inspected version `0.0.2`,
declared `workspace:*`.
Import:
`@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts/review-structure.ts`.

- `parseShellReviewStructure`:
  turn Bash into parser-owned structural data while withholding complete source scripts.
- `ShellReviewSyntaxError`:
  distinguish a current operation that cannot be fully represented from an ordinary message.

Necessary because shell control flow,
quoting,
substitutions,
and redirection cannot be recovered by splitting strings.
The repository already owns this projection.
The rebuild does not use its command-ban helpers or replace parsing with command-name heuristics.

Qualification gap:
installed `unbash` is `6.0.0`,
while the adapter documentation and catalog comments name version 5.
Reading the source is not proof that the adapter works with version 6.
Reuse stays provisional until its consumed syntax cases and actual model-bound projections pass.

The source review also found a specific contract mismatch to reproduce:
installed declarations expose `Pipeline.prefixes`,
`TestLogical.operands`,
and `TestNot.negations`,
but the shared projection's `structuralFields` omits those fields and its generic projection filters them out.
Exercise pipeline negation and compound test predicates with positive controls before accepting this dependency.
Any required shared-parser changes need explicit scope and size accounting.
This finding is source evidence,
not a completed runtime reproduction or repair.

### Runtime transitive dependencies of the proposed production imports

- `unbash`,
  inspected `6.0.0`:
  the shared projection calls `parse`.
  Auto-mode makes no direct import from `unbash`.
  Its inspected manifest declares no production or optional dependencies.
- `@monochromatic-dev/logger-core`:
  also used internally by the shared parser,
  counted once.
- `@monochromatic-dev/ownership-marker-foreign-borrowed`:
  declared by the shared analyzer and provides types with no runtime export.
  The selected projection file does not import it;
  auto-mode introduces no ownership-marker machinery.
  Package-manager installation accounting must still disclose that declared dependency.

Thus the incremental production package inventory is the logger,
shared analyzer,
parser,
and declared type marker.
The emitted import graph must verify this inventory rather than assuming tree shaking removes unwanted code.

### Host-provided Pi interface

Package:
`@earendil-works/pi-coding-agent`,
inspected `1.3.0`.
Proposed peer dependency plus catalog development dependency for declarations and host fixtures.

- Supported plugin loading invokes the extension factory inside ordinary Pi.
  A private loading hook and a replacement Pi CLI are excluded.
- `ExtensionAPI` and `ExtensionContext`:
  type-only declarations for the extension and context.
- `ExtensionAPI.on`:
  register lifecycle handlers and obtain their unsubscribe functions.
- `ExtensionContext.getSystemPrompt`:
  obtain effective instructions.
- `sessionManager.getBranch`:
  obtain original active-branch entries where the selected review contract requires them.
- `sessionManager.buildSessionContext`:
  candidate for resolved session messages;
  qualify later host context transformations before claiming parity with effective model input.
- `sessionEntryToContextMessages`:
  convert individual entries only if original-entry conversion is required;
  this is not a substitute for resolved session projection.
- `sessionManager.getSessionId`:
  correlate operations without retaining every session object.
- `ui.select` and `ui.input`:
  collect Approve or Stop and an optional reason in TUI and RPC.
- `sendMessage` with `triggerTurn: false`:
  persist model-visible manual feedback without starting a new run.
- `appendEntry`:
  optional structured audit metadata,
  never the sole feedback channel.
- `ui.notify` and `ui.setStatus`:
  report actionable failures and review state.
- `ExtensionContext.signal`:
  propagate the host cancellation signal.

Necessary because auto-mode is a Pi execution guard and must preserve the real configured host.
It does not use Pi's model registry,
provider dispatch,
or provider request serializers to call either decision model.

Pi's own manifest depends on `pi-ai` and other provider machinery.
Those are host dependencies,
not proposed auto-mode API clients or copied package artifacts.
This plan cannot claim the whole Pi installation has no such packages.
Auto-mode's own runtime and emitted imports must contain no `pi-ai` or `openai` imports or calls.

### Historical host implementation inspection, not an integration surface

Package:
host-provided `@earendil-works/pi-agent-core`,
inspected `1.3.0`.
No new direct application dependency or SDK copy is proposed.
The proposed hook into `dist/agent-loop.js` is withdrawn under the no-Pi-modification constraint.

The former private-function integration inventory is withdrawn.
Source inspection established that Pi owns argument preparation,
dispatch,
lifecycle events,
and result delivery.
Those remain host responsibilities,
not functionality to reproduce inside auto-mode.
Historical inspection of `runToolCall` and session `_executeNestedToolCall` explained individual nested calls;
it did not establish a public complete-child-group boundary.

The original inventory incorrectly named `executeNestedToolCall` as a core function;
the 2026-10-19 source review corrects that name and location.
The replacement plugin is not implemented;
disposable public-only probes establish finite integration behavior.
This inventory grants no permission to couple the plugin to private functions.
Real public-plugin tests must establish the required behavior without source substitution.
Unsupported cases must exercise the documented graceful fallback instead of requiring private integration.

### Node runtime and built-in functions

No additional HTTP,
filesystem,
process-runner,
hashing,
or module-loader package is proposed.
Node is the runtime already used by the repository.
The following is the intended built-in import/call inventory:

- `node:module`:
  `findPackageJSON` may locate host package metadata if public-plugin qualification needs it.
  `registerHooks` and module substitution are excluded.
- `node:fs/promises`:
  `readFile` reads explicitly selected metadata or instruction files;
  `realpath` establishes the actual host locations.
- `node:path`:
  `dirname`,
  `resolve`,
  and `join` construct paths from resolved host locations.
- `node:url`:
  `fileURLToPath` and `pathToFileURL` cross module-URL/filesystem syntax correctly.
- `node:crypto`:
  `createHash` fingerprints the qualified host source and captured request;
  `randomUUID` distinguishes repeated request occurrences.
- Platform `fetch`:
  the only HTTP sender.
- `AbortController.abort`,
  `AbortSignal.any`,
  and `throwIfAborted`:
  link operation,
  observer,
  and shutdown cancellation.
  Any timeout signal and its value require the transport qualification described in this plan.
- `TextEncoder.encode` and `TextDecoder.decode`:
  count wire bytes and decode bounded response chunks.
- Standard `JSON.parse`/`stringify`,
  `Promise.allSettled`,
  and `Promise.withResolvers`:
  data envelopes,
  joined parallel results,
  and lifecycle barriers.
- `performance.now`:
  measure observed durations rather than infer them from a clock target.
- `process.argv`,
  `process.env`,
  and `process.execPath`:
  forwarded host arguments,
  named credentials/recursion guard,
  and test child runtime.

No Pi source is transformed,
whether on disk,
in a copied tree,
or in memory.
There is no recursive package traversal,
source-code compiler,
SDK dependency relinker,
or runtime monkey patch.

### Test-only dependencies and functions

Package:
`@monochromatic-dev/test-harness-core`,
inspected `0.0.2`,
declared `workspace:*`.

- `describe` and `it`:
  named behavioral suites against freshly built output.
- `expect`:
  outcome,
  payload,
  event-order,
  cancellation,
  and emitted-output assertions.
- `expectTypeOf`:
  public declaration checks against host-facing types where useful.
- Per-test `sinon.spy` and `sinon.stub`:
  observe injected HTTP calls and lifecycle events with automatic cleanup.

Package:
`typebox`,
supplied by the existing catalog,
development-only.

- `Type.Object` and `Type.String`:
  declare the small registered-tool schemas in actual Pi integration fixtures.
  Production auto-mode does not register replacement tools and needs no schema-builder import.

Node test-only functions:

- `node:http.createServer`:
  real loopback response,
  malformed-data,
  redirect,
  and cancellation fixtures.
- `node:events.once`:
  wait for actual server/process events without polling sleeps.
- `node:child_process.spawn`:
  run the real Pi CLI and collect both output streams.
- `node:fs/promises`:
  `mkdtemp`,
  `mkdir`,
  `writeFile`,
  `readFile`,
  `stat`,
  and `rm`:
  disposable homes,
  projects,
  captures,
  and cleanup with deletion constrained to the created fixture.
- `node:os.homedir`:
  create durable scratch fixtures under the injected user's home.

The module-test development closure includes its existing assertion and sandbox packages,
including `chai`,
`chai-as-promised`,
`expect-type`,
`sinon`,
`sinon-chai`,
`p-limit`,
`@types/sinon`,
and its workspace helper modules.
Auto-mode imports none of those directly or in production.
Its complete installed development closure needs a generated resolved inventory before implementation acceptance;
this document does not claim a full transitive development-package audit has already been performed.

### Build and type dependencies and functions

- `@monochromatic-dev/config-rolldown`,
  declared `workspace:*`:
  `nodeConfig` creates the ordinary Node bundle and declarations.
  Its `rolldown` and `rolldown-plugin-dts` dependencies remain development tools.
  No custom SDK emission or native compiler pipeline is proposed.
- `@monochromatic-dev/config-typescript`,
  declared `workspace:*`:
  the `dom` configuration supplies repository TypeScript rules and fetch-related platform types.
  This is configuration data,
  with no runtime function calls.
- `@types/node`,
  from the existing catalog:
  Node declarations only,
  with no runtime functions.
- Repository-root `typescript`,
  `rolldown`,
  `oxlint`,
  and `oxlint-tsgolint`:
  existing type,
  bundle,
  and lint commands invoked through package-scoped `mise run` tasks.
  They are not shipped runtime dependencies.
- Existing `@monochromatic-dev/monorepo-task-core-exec` task entry points:
  `tsc-filter` and `oxlint-wrapper` preserve the repository's type and lint execution conventions.
  Auto-mode imports no helper functions from this package into production.
- `mise` and `pnpm`:
  task dispatch and generated dependency-lockfile management.
- `podman`:
  disposable verification with a two-GiB memory and two-CPU limit.
  It is a verification requirement,
  not a production runtime dependency.

Any shared configuration bundling that pulls in unwanted runtime code is a qualification failure to investigate.
Changing shared lint rules or replacing the repository build stack is outside this proposal.

### Deliberately absent dependencies

There is no direct `pi-ai`,
`openai`,
model-review SDK,
model-selection SDK,
`nano-spawn`,
generic schema validator,
policy compiler,
ownership-capability framework,
or native/Wasm build dependency.
No SDK source tree,
provider serializer,
tool implementation,
or Node installation is copied into auto-mode.
The shell parser and shared logger are the only proposed production library responsibilities outside Node and Pi.

## Implementation and verification sequence after review

### Establish the host seam and dependency facts

Qualify the public plugin events with a minimal actual Pi consumer before expanding the design.
Cover parallel,
sequential,
nested,
pre-review mutated inputs,
denied,
cancellation,
and configured-extension behavior.
Assert parallel calls receive separate snapshots,
complete rule answer sets,
budgets,
and manual decisions.
Assert `codemode` body parsing is complete while its nested operations remain one outer approval unit.
No call executes before its own verdict;
do not assert execution timing beyond Pi's scheduler.
Verify mixed outcomes in both member orders:
the allowed sibling executes,
the denied sibling does not,
and Pi continues without a global abort.
Do not add a shared sibling-conflict review branch.
Assert nonparallel A and B produce distinct reviews in prepare/review/execute order,
including B preparation that reads A's completed result.
Do not merge reviews because calls share a queue,
assistant message,
or parallel dispatch.
Generate the exact resolved production and development dependency inventories and disk accounting.
Qualify the consumed parser projection against installed `unbash` 6.
Verify actual hundred-question endpoint capacity without executing proposed operations.
An unsupported assumption returns to this plan;
it does not trigger an unreviewed framework expansion.

### Implement review through one interface

Build startup relevance,
capture,
scheduler,
codec,
deterministic reducer,
and manual feedback.
Assert every-start classification and complete effective-rule first-pass coverage in captured requests.
Use the actual effective ruleset at the expected hundred-rule scale.
Cover early resolution,
adaptive second and third batches,
exhausted review,
partial call failures,
missing answers,
every admitted label,
and the exact probability boundaries accepted in Q1.
Exercise the five-to-ten-unit window,
original human provenance,
and approval/decline reasons on resume.
For every ask-producing branch,
assert an evidence-backed explanation,
required human decision,
any actual uncertainty,
and action consequences in the TUI/RPC payload;
a generic prompt must fail the fixture.
An applicable rule requiring confirmation with all facts known must ask without inventing uncertainty.
Fully deterministic calls must use zero per-call Luna requests;
count startup separately.
Resolved affected calls do not receive reconsideration,
while unresolved siblings can still require Luna review.
A model-resolved ask must prompt for its single outer call without a follow-up intended to reverse that verdict.
Assert model-visible inclusion and exclusion using actual serialized requests.

### Connect primary review and shadow

Exercise the installed Pi consumer with its configured extensions,
skills,
tool catalog,
and preceding turns.
Use disposable target projects and homes for any state-mutating verification.
Then drive supervised MiMo sessions with the accepted replacement and matching Span 02 shadow requests.
Keep original and replacement evidence distinct.
Old legacy shadow traffic does not verify the new question contract.

### Demonstrate regression detection and finish measurement

Replace package tasks that still build the native pipeline or reference deleted `src` files.
Use one standard TypeScript project for the replacement plugin;
retired native input is excluded.
Start acceptance from fresh replacement artifacts;
old `dist` output cannot supply a passing result.
Verify the project-first log destination without the legacy task environment overriding it.

Commit important guard tests,
remove each guard in a disposable copy,
rebuild,
and show the intended failure.
Restore and demonstrate the original behavior passes.
Capture complete stdout and stderr for stop,
abort,
deny,
allow,
and shutdown scenarios.
Compare test cases against actual branches,
not only a green process exit.
Run the package's full type check,
lint,
and relevant tests with current artifacts.
Do not loosen lint rules or reject valid wrapping to meet a size budget.

Measure production,
verification,
configuration,
documentation,
emitted output,
dependency targets,
logs,
caches,
and archived artifacts separately.
Any ignored-file cleanup first gets the required dry-run and stray-Git-entry checks.
No cleanup or deletion is performed as part of writing this plan.

## Decisions still requiring review or qualification

Q1 B and Q4 A are accepted;
they are no longer open questions.
Q2 is withdrawn because established asks already terminate automatic review.
Q6 is withdrawn because every outer call is judged individually,
including parallel calls and complete `codemode` bodies.
Q10 is withdrawn because each call's Luna allowance and separate shadow accounting are settled.
The package-match gate is rejected;
the former Q3 and Q5 options are withdrawn.
Further dependent work covers follow-up conflicts,
startup failures,
shadow scheduling and preemption,
persistent log retention,
exact dependency closure,
and source/deployment budgets.
Actual host semantics,
endpoint capacity,
parser compatibility,
RPC output,
and lifecycle behavior remain unqualified.
No implementation proceeds while the user is reviewing this plan.

## Review artifact verification

The records in this section are historical verification of earlier review artifacts.
They do not verify the latest individual-call requirement,
`codemode` boundary,
or Q10 withdrawal.
New rendered verification must cover the revised surface before handoff.

The earlier grouping correction replaced Q6 with the then-settled parallel-batch/individual-call boundary.
Its flowchart shows one review before parallel dispatch and independent reviews for nonparallel calls.
The Q6 form field and serialized answer are absent;
Q1 and Q4 remain.
The new diagram passed light/dark text-fit and 390-pixel keyboard-scroll checks without page overflow.
All eight navigation links,
four examples,
six remaining answer options,
manual examples,
reply handling,
and keyboard activation passed.
The 21 controls meet the 48-pixel minimum.
Page errors and console records are empty.
Historical grouping captures:
[light](ground-up-rebuild-review/flowchart-grouping-light.png)
and [dark](ground-up-rebuild-review/flowchart-grouping-dark.png).

The complete drawing inventory now contains 12 connected SVG charts,
including the four worked-example tabs.
The initial flowchart pass covered only startup,
group review,
and human decision;
its partial coverage was corrected.
All remaining composition,
follow-up,
example,
routing,
autonomy,
and timing drawings now use connected paths.
No legacy `.flow` or `.arrow` card drawing remains in the HTML.
All 24 chart/theme combinations passed rendered text-bound checks and keyboard scrolling at 390-pixel width.
Independent review inspected every capture and found no broken connection or changed decision semantics.
Navigation,
every example tab,
response options,
manual examples,
and reply handling also passed.
The page produced no browser errors or console records.

Additional current flowchart captures:

- Rule composition:
  [light](ground-up-rebuild-review/flowchart-rules-light.png)
  and [dark](ground-up-rebuild-review/flowchart-rules-dark.png).
- Follow-ups:
  [light](ground-up-rebuild-review/flowchart-adaptive-flow-light.png)
  and [dark](ground-up-rebuild-review/flowchart-adaptive-flow-dark.png).
- Cross-package tests:
  [light](ground-up-rebuild-review/flowchart-case-tests-light.png)
  and [dark](ground-up-rebuild-review/flowchart-case-tests-dark.png).
- Shared-state test effects:
  [light](ground-up-rebuild-review/flowchart-case-scope-light.png)
  and [dark](ground-up-rebuild-review/flowchart-case-scope-dark.png).
- Matching approval:
  [light](ground-up-rebuild-review/flowchart-case-exception-light.png)
  and [dark](ground-up-rebuild-review/flowchart-case-exception-dark.png).
- Required human choice:
  [light](ground-up-rebuild-review/flowchart-case-missing-light.png)
  and [dark](ground-up-rebuild-review/flowchart-case-missing-dark.png).
- Ask routing:
  [light](ground-up-rebuild-review/flowchart-ask-routes-light.png)
  and [dark](ground-up-rebuild-review/flowchart-ask-routes-dark.png).
- Autonomy baseline:
  [light](ground-up-rebuild-review/flowchart-autonomy-light.png)
  and [dark](ground-up-rebuild-review/flowchart-autonomy-dark.png).
- Superseded sequential-group timing proposal:
  [light](ground-up-rebuild-review/flowchart-timing-light.png)
  and [dark](ground-up-rebuild-review/flowchart-timing-dark.png).
  These captures predate the explicit parallel-batch/individual-call boundary and withdrawal of Q6.

The flowchart correction replaces the startup,
group-review,
and manual-decision card rows with connected SVG.
Decision diamonds have labeled outgoing branches;
operation rectangles and rounded exits have attached arrows.
The follow-up return edge reaches the current-batch Luna call only through the unresolved outcome and budget check.
The manual flow branches into Approve and Stop,
each with optional reason,
persisted record,
and consequence.
Independent review found no broken connection or changed decision semantics.

Rendered checks covered all flowcharts in light and dark mode and at a 390-pixel viewport.
All group-node text fits its containing shape;
the complete desktop charts fit the capture viewport.
Each narrow chart scrolls by keyboard to its full horizontal extent without widening the page.
The existing navigation,
scenarios,
nine answer options,
manual examples,
and reply branches still pass.
Page error and console records are empty.
This verifies the review artifact,
not the replacement plugin.

Current flowchart captures:

- Startup:
  [light](ground-up-rebuild-review/flowchart-startup-light.png)
  and [dark](ground-up-rebuild-review/flowchart-startup-dark.png).
- Group review:
  [light](ground-up-rebuild-review/flowchart-group-light.png)
  and [dark](ground-up-rebuild-review/flowchart-group-dark.png).
- Human decision:
  [light](ground-up-rebuild-review/flowchart-human-light.png)
  and [dark](ground-up-rebuild-review/flowchart-human-dark.png).

Before the reversibility correction,
the staged HTML passed the disposable nested-browser interaction check
on 2026-10-19.
These counts describe that earlier form;
the revised form has two open questions.
Every scenario,
navigation control,
all 18 answer options,
and manual Approve/Stop examples with and without reasons ran.
Reply generation passed for empty input,
notes only,
choices only,
literal markup,
text selection,
and keyboard activation.
All 23 interactive controls measured at least 48 pixels in both dimensions.
Light and dark appearances and a 390-pixel viewport passed without horizontal overflow.
Expanded built-in wording and preparation timelines were recaptured after their final edit.
Browser error and console records were empty.
This verifies the planning artifact,
not plugin behavior or model quality.

Historical images,
predating the terminal-verdict correction and withdrawal of Q2:
[earlier stages](ground-up-rebuild-review/staged-overview-dark.png),
[score admission](ground-up-rebuild-review/staged-q1-dark.png),
[withdrawn ask alternatives](ground-up-rebuild-review/staged-q2-dark.png),
[deterministic approval](ground-up-rebuild-review/staged-q3-dark.png),
[human anchor](ground-up-rebuild-review/staged-q4-dark.png),
[built-in wording](ground-up-rebuild-review/staged-q5-dark.png),
[preparation timing](ground-up-rebuild-review/staged-q6-dark.png),
[manual feedback](ground-up-rebuild-review/staged-manual-dark.png),
[light appearance](ground-up-rebuild-review/staged-scenario-tests-light.png),
and [narrow response form](ground-up-rebuild-review/staged-response-narrow.png).
The later ask-explanation clarification is shown in
[light](ground-up-rebuild-review/explained-ask-light.png)
and [dark](ground-up-rebuild-review/explained-ask-dark.png).
Its Approve/Stop examples passed with and without reasons,
including literal markup in reason text.
The updated panel fits a 390-pixel viewport and has no page errors or console messages.
The older `staged-manual-dark.png` shows the prior manual example;
the other screenshots without the `staged-` or `explained-ask-` prefixes document the superseded review.

The reversibility revision passed all four scenarios,
six navigation targets,
all 12 remaining answer options,
both manual decisions with and without reasons,
form validation,
literal-text handling,
text selection,
and keyboard activation.
All 21 remaining controls measured at least 48 pixels in both dimensions.
The scenarios passed in light and dark mode;
all sections fit a 390-pixel viewport without horizontal overflow.
Page error and console records were empty.
The verified review was reopened in Helium at the worked examples.

Current correction evidence:
[cross-package tests](ground-up-rebuild-review/reversibility-scenario-tests-dark.png),
[destructive test effects](ground-up-rebuild-review/reversibility-scenario-scope-dark.png),
[autonomy baseline](ground-up-rebuild-review/reversibility-q3-dark.png),
[effects and recovery checks](ground-up-rebuild-review/reversibility-q5-dark.png),
and [focused recovery questions](ground-up-rebuild-review/reversibility-adaptive-light.png).
The older `staged-q3`,
`staged-q5`,
and exact-package test screenshots are superseded historical evidence.
These checks verify the review artifact;
no replacement plugin has been implemented or qualified.

The distinct built-in catalogue was subsequently verified in both color schemes and at 390-pixel width.
All seven named criteria render,
and the first-pass diagram explicitly includes the complete catalogue.
The seven navigation links,
four scenarios,
12 answer options,
manual decisions,
form branches,
text handling,
and keyboard behavior passed again after adding the catalogue navigation.
All 22 controls meet the 48-pixel minimum;
page error and console records are empty.
Historical catalogue evidence,
withdrawn from the active review when the user deferred its contents:
[dark](ground-up-rebuild-review/builtin-catalogue-dark.png),
[light](ground-up-rebuild-review/builtin-catalogue-light.png),
and [complete first pass](ground-up-rebuild-review/builtin-first-pass-dark.png).
These screenshots record an unaccepted proposal,
not the active design.
The current HTML shows only the plural-rule interface with contents deferred.

After deferral,
the simplified panel was checked in light and dark mode and at 390-pixel width.
Rendered state contains no proposed catalogue entries and retains the four unrelated architecture questions.
Its navigation works,
it has no horizontal overflow,
and page error and console records are empty.
The updated interface was reopened in Helium.

## Inspected sources

These are source-reading facts,
not new runtime qualification results.

- [Shared logger manifest][logger-manifest],
  [tagged functions][logger-tagged],
  and [file sink][logger-file].
- [Shared parser manifest][shell-manifest],
  [projection implementation][shell-projection],
  and [adapter README][shell-readme].
- [Shared test harness manifest][test-manifest] and [Node build configuration][build-config].
- [Existing adaptive assessment requirements][assessment] and [Span 02 trial][span-trial].
- Installed Pi `1.3.0` public declarations in
  `package/pi-plugin/auto-mode/node_modules/@earendil-works/pi-coding-agent/dist/index.d.ts`
  and `dist/core/extensions/types.d.ts` under that same installed package.
- Installed Pi agent core `1.3.0`,
  `dist/agent-loop.js`:
  `executeToolCalls`,
  `prepareToolCall`,
  and `runToolCall`.
  Resolved installed path:
  `node_modules/.pnpm/@earendil-works+pi-agent-core@1.3.0_undici@8.12.2/node_modules/@earendil-works/pi-agent-core/dist/agent-loop.js`.
- Installed Pi coding agent `1.3.0`,
  `dist/core/agent-session.js:388` and `:401`:
  `_executeNestedToolCall` routes through core `runToolCall`.
- Installed Pi coding agent `1.3.0`,
  `dist/core/session-manager.js:256` and `:1058`:
  resolved projection differs from reading branch entries.
- Installed Node declarations,
  `node_modules/@types/node/module.d.ts`:
  `findPackageJSON` and experimental `registerHooks`.

[logger-manifest]: ../../../../logger/core/package.json
[logger-tagged]: ../../../../logger/core/src/tagged.ts
[logger-file]: ../../../../logger/core/src/sink/file.ts
[shell-manifest]: ../../../../agent-harness-shared/shell-command-analyzer/package.json
[shell-projection]: ../../../../agent-harness-shared/shell-command-analyzer/src/review-structure.ts
[shell-readme]: ../../../../agent-harness-shared/shell-command-analyzer/README.md
[test-manifest]: ../../../../test-harness/core/package.json
[build-config]: ../../../../config/rolldown/src/index.node.ts
[assessment]: ../../../../../doc/planning/pi-auto-mode-batched-assessment.md
[span-trial]: ../span-02-trial.md

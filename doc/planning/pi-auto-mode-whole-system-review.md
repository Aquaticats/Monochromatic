# Auto-mode whole-system review

## Scope and status

The user requested a review of the whole auto-mode design on 2026-10-15,
including whether replacement is preferable to continued repair.
The user accepted decision-core replacement on 2026-10-15 and requested avoiding overengineering.
The [accepted decision](../decision/pi-auto-mode-decision-core.md) authorizes implementation.
The source snapshot is `02078ea5f8cf60434695a4c5a92ed23b7b6f9d9f`.
The unrelated working-tree `mise.lock` change is outside the review.
Production edits resumed after acceptance.

## Review work

- [x] Reconcile accepted requirements, historical decisions, and legacy behavior.
- [x] Trace host integration, capture, retained provenance, and decision-context selection.
- [x] Trace assessment questions, caching, transport budgets, and probability consumption.
- [x] Trace policy, confirmation, cancellation, and exact prepared-operation execution.
- [x] Assess test coverage at the actual consumer and identify unsupported completion claims.
- [x] Compare repair, replacement of the decision core, and rebuilding around the legacy caller.
- [x] Publish evidence, limitations, a ranked recommendation, and concrete acceptance checks.

## Current constraints

The latest accepted preparation-inclusive assessment budget is 15 seconds,
shared across at most three sequential bundles of at most three parallel calls.
The average USD 0.001 cost target was removed.
The model estimates meanings and probabilities;
deterministic code resolves policy and the final operation decision.
The current operating cutoff is a uniquely leading selected-option probability of at least 0.60.
It is not a calibrated correctness guarantee.

The legacy recent-context selection keeps the larger of the latest five visible messages
and the complete span from the latest user message.
Selected message contents remain complete.
Local retention for provenance does not require disclosing every retained receipt on each assessment.
The user chose to keep this migration lesson in auto-mode documentation,
without adding a general `AGENTS.md` rule.

An established applicable prohibition blocks execution.
Uncertain assessment can request original human confirmation for the exact prepared group.
Cancellation, stale inputs, and unavailable approval UI do not confer permission.
The existing reviewed source owners and execution controls are candidates for retention,
not assumed correct solely because their local tests pass.

## Evidence already reproduced

The built legacy context tests pass the message-window,
complete selected content,
hidden/system-message omission,
and compaction projection controls.
The 15-second change passed rebuilt type, transport, wire, bundle, budget, preparation,
lifecycle, and dependency checks.
These successes do not establish end-to-end native usability.

The unchanged native workspace replay sent 617 questions across four planned requests.
Its third request contained 4538472 serialized bytes and returned HTTP 400
with `context_length_exceeded`.
The response did not report the token count or effective serving limit.
The native wire includes all governing-source content and the original provider request,
then repeats selected source text inside question instructions.
Scratch-only representation experiments are not the selected remedy for incorrect context scope.

## Findings and recommendation

Replace the decision core while retaining and requalifying the native execution owners.
The defects cross context selection, candidate generation, policy composition, and confirmation.
Changing request packing alone cannot resolve them.
A complete host rewrite is not justified by the evidence collected so far:
original operation identity, source freshness, cancellation, and approval binding provide useful guarantees.

This is a system-level review of those paths and their tests,
not a claim to have inspected every line or proved absence of all defects.
Independent reviewers examined minimal replacement, in-place repair, and a legacy-oriented rebuild.
All reached the same substantive separation:
retain original operation custody; replace the composition that turns retained evidence into decisions.

### Blocker: source retention determines assessment scope

`native/contract/integration/policy-source/policy-input-collector.mjs:48`
combines retained runs, resource receipts, prompt transformations, messages, and tool declarations
into one `governingSources` collection.
`native/contract/integration/action-policy/wire.mjs:124` serializes every additional source,
and line 136 also includes the original provider request.
`wire-clause.mjs:52` repeats selected text in individual questions.
`src/decision-partitions.ts` repeats the complete state across request partitions.

That is not the legacy selected-context contract in `src/visible-context.ts:360`.
It conflates material required for local authenticity checks with material required for model assessment.
The current evaluator disables the separate whole-request question;
its inactive definition is not evidence that the question currently runs.
The full request is nevertheless serialized unconditionally.

Repair must select the decision input before issuing semantic candidates.
Keep the complete current governing policy and complete selected messages.
Preserve original occurrence identity locally without treating every historical receipt as a fresh instruction question.

### Blocker: manual Allow erases an unrelated unsatisfied prerequisite

`src/guard-outcome.ts:167` leaves an applicable prerequisite unresolved when its satisfaction
or suitable approval channel is not established.
`native/contract/integration/action-policy/approval.mjs:90`
treats all unresolved outcomes as eligible for its generic confirmation.
After Allow, it replaces the outcome with approval without re-resolving those requirements.

The adopted [semantic admission decision](../decision/pi-auto-mode-semantic-admission.md)
states that fresh session confirmation cannot satisfy unrelated authorization requirements.
The following built-artifact fixture witnesses the mismatch:

- An incumbent-confirmation requirement goes from ask to approve after fixture Allow.
- An established prohibition remains denied and opens no dialog.
- An applicable, unsatisfied prerequisite with an unsupported channel
  goes from unresolved to approve after fixture Allow.

The third case records one dialog and one verdict entry despite having no approval-satisfiable requirement.
The reproducer uses the built policy resolver and built original-group approval binder,
with isolated fixture capabilities.
It does not use a real human response, real SDK session, or actual tool execution.
Artifacts are under `${HOME}/temp/agent/auto-mode-minimal-review-20261015/`;
`mise run witness` reproduces the controls.

The earlier configured `generic` control is not a counterexample:
its driver sends Deny in that mode, so withholding execution does not test an Allow response to this prerequisite.

Repair must preserve the distinction between semantic uncertainty and independent unsatisfied prerequisites.
An Allow supplies only its specifically scoped confirmation fact.
Resolve the operation again with that fact rather than overwriting the entire outcome.

### Blocker: bypass still requires a supported semantic program profile

`native/contract/integration/native-batch/handler.mjs:129`
calls `prepareProgramInputs` before selecting its bypass branch.
The program decoder therefore rejects unsupported semantic profiles even during explicit bypass.

A disposable built-SDK consumer and a direct built upstream Codemode baseline reproduced this:

- Supported program with bypass off: no child executions, one assessment, prohibited.
- Supported program with bypass on: three child executions, no assessment.
- A three-statement program outside the inspected profiles: no child executions, assessments,
  or dialogs with bypass either off or on.
- The exact same three-statement program through upstream Codemode: two inert child executions and `Script completed`.

The deciding diagnostic is `ResultProgramProfileError`:
`Result-derived program is outside the inspected profile: exact parent boundary and one or two inspected statements required; use a returned awaited literal call or an immutable result binding`.
`PreparedBatchExecutionError` wraps it,
and the tool displays `Complete group preparation or assessment failed`.
Network was blocked and all child functions were inert.
The final consumers exited successfully without bare shutdown errors.
`mise run verify-bypass-review` in `${HOME}/temp/agent/pi-tool-call-scope-20261014`
runs the retained probes;
their fixtures are in `${HOME}/temp/agent/auto-mode-bypass-review/`.

Separate guarded semantic-profile admission from explicit bypass execution.
Preserve fixed guards and native execution identity,
but do not require bypassed operations to qualify for automatic semantic approval.
This requires requalifying host routing as well as replacing the decision core;
retaining useful ownership mechanisms does not mean preserving their current composition unchanged.

### Warning: historical capture also grows preparation work

`native/contract/collector/run-prompt-custody-v3/loaded-pool.mjs:79`
copies the constructed-origin array on each append.
`lifecycle.mjs:78` traverses published runs in the same root.
`native/contract/integration/policy-source/run-history.mjs:28`
collects prior prompt sources for the source collection.
These source paths establish repeated traversal and cumulative array-copy work.
This review has not measured long-session heap or latency growth.

Trimming only the outbound wire would leave that work before assessment.
Separate the retained archive from the current decision projection,
and measure multiple completed turns through the configured host.
Define retention by the actual owners that still need evidence;
do not reintroduce arbitrary byte ceilings or delete evidence merely to fit a request.

### Warning: relevance caching is dormant and has an unproven dependency contract

The installed designation in `native/contract/integration/native-host/bootstrap.ts`
does not configure relevance.
`native/contract/lifecycle/rule-relevance-cache/selection.mjs:16`
returns the original selection when relevance is absent.
Even configured refresh returns all candidate rule IDs and `exclusionsAdmitted: false`.
It is not an installed mechanism for reducing operation-time assessment work.

There is also a latent cache-design concern:
`refresh.ts` supplies the full policy to classification,
while `materialization.ts` keys individual entries by the selected rule, headings, and unclassified context.
Changing another classified rule can leave this key unchanged despite changing supplied context.
No production cached exclusion currently follows from that mismatch.

Before using the cache, make the keyed dependencies match the actual question input.
Keep reusable syntax/indexing work separate from operation-specific semantic estimates.
A full current policy remains required in every model evaluation;
cache design cannot silently narrow that settled requirement.

### Warning: installed behavior is not legacy feature parity

The accepted native semantic scope contains two inspected program profiles.
Ordinary direct tool calls do not acquire those profiles merely because they name the same child tool.
`native/contract/integration/native-batch/tools.mjs:212`
attaches program inspection only to owned program definitions.
`src/semantic-admission-scope.ts:99` rejects missing inspection,
and the installed bootstrap does not supply an implementation-effect owner for ordinary direct tools.
This is a source-derived support limitation, not an interactive reproduction of every tool.
The narrow profile adoption was explicit;
do not label general JavaScript coverage an already implemented requirement.

Native resource registration installs bypass but does not install the legacy `/guard`,
`propose_trust`, or approval-reuse owners.
The package README expressly distinguishes the legacy implementation from installed behavior.
The broader accepted design still discusses trust, witnessed inheritance, reset, and approval reuse.
Those requirements need an explicit product acceptance ledger;
adding trust prose to a model prompt does not implement their state or authority.

Transport failure behavior also differs from the legacy extension:
native failures stop before policy closure,
while the old judge could fall back to manual approval after a failed model call.
The native behavior is documented and distinct from successful semantic uncertainty.
The replacement must preserve the current contract unless the user changes it.

### Warning: build activation can expose an incomplete artifact

`native/build.ts:53` removes the active `dist/native/contract` tree before copying and generating its replacement.
`native/command.ts:14` imports that same artifact location.
The installation task checks that the CLI entry exists rather than validating a complete release.
An interrupted generation can therefore leave the installed location partially rebuilt.
This is a source-derived failure risk, not a reproduced failed installation.

Build and verify an isolated complete artifact, then activate it as one version.
A failed build should leave the previously activated command usable.
Keep this lifecycle responsibility separate from decision semantics.

### Warning: tests prove local contracts without a shared product acceptance contract

The legacy context tests pass their window contract.
The native history tests deliberately retain prior-run material in the source inventory.
Both can pass while the outgoing assessment violates the desired window.
Similarly, authored model replies establish transport attribution and policy mechanics,
not meaningful model behavior or a successful live configured session.

The package inventory at the reviewed revision contains 845 tracked files.
The review inventory classified 433 code files and 52240 lines under the native contract,
291 test/fixture files and 47461 lines,
and 95 maintained source files and 18600 lines.
Line counts include comments and blank lines;
classification tests research paths first, then paths containing `test` or `fixture`, then source location.
Those counts describe review scope, not a quality metric or a claim of full coverage.

Consumer tests must assert what leaves the host and what actually executes,
with explicit negative controls for old context, unrelated prerequisites, stale sources, and cancellation.
Keep identity and lifecycle adversarial tests that protect those outcomes.
Retire tests tied only to replaced internal forwarding after equivalent consumer coverage exists.

## Alternatives

### A: Repair the existing composition in place

Keep the current public and private structure.
Introduce selected context before candidate generation,
correct approval routing,
and add consumer acceptance tests.
This preserves the existing call graph and permits narrowly scoped changes.
It also leaves context selection, prerequisite handling, closure, and execution spread across the existing owners.
The risk is another sequence of local patches without one owner of the full decision contract.

### B: Replace the decision core and retain native operation custody

Introduce one decision module whose caller supplies an original host-owned operation capability.
The module owns preparation-inclusive accounting, selected context, candidate planning,
probability validation, deterministic resolution, scoped confirmation, and execution admission.
It cannot accept caller-authored permission, source authority, or replacement currentness checks.

An illustrative interface is:

```typescript
// Proposed interface, not an implemented public contract.
const gate = createDecisionGate({ host, assessor, clock });
await gate.runPrepared({ operation: originalOperation, signal });
await gate.dispose();
```

The name of the entry point does not move the start of accounting:
preparation must remain inside the original shared assessment budget.
The retained host owns actual tool definitions, argument preparation, response identity,
source freshness, cancellation, native execution, and UI capability capture.
Its bypass route must be separated from guarded semantic program-profile admission.
The new core selects decision context from those originals and returns only through that owned execution route.

Use a private selected-context value containing current operation, complete governing policy,
applicable trusted records, and the legacy recent visible-message window.
Keep raw request and receipt ancestry in local custody.
The model receives narrow semantic questions and selected context, never authority supplied by itself.

The implementation replaces the composition around `action-policy/evaluate.mjs`,
`index.mjs`, `approval.mjs`, and archive-to-wire materialization.
It retains the transport codec, shared bundle accounting, probability validation,
deterministic rule resolution, and useful native identity/currentness primitives where tests support them.
Retire inactive whole-request question scaffolding only after checking its remaining consumers.
Do not copy the legacy model-authored final verdict policy into the new core.

This concentrates decision behavior behind one interface and gives consumer tests one coherent subject.
It requires an explicit integration migration and requalification of the retained owners;
the existing test count does not remove that requirement.

### C: Rebuild the host and guard together

Replace native capture, preparation, execution wrapping, decision policy, and installation together.
This permits reconsidering the entire integration shape and eliminating prototype-era module structure.
It also requires re-establishing original response identity, exact prepared arguments,
source freshness, nested execution, cancellation, UI binding, resource reload, and disposal.
The review has not shown that all those mechanisms are fundamentally wrong.
No reliable implementation-duration estimate follows from the current evidence.

Ranking: B > A > C.
B precedes A because it assigns the decision contract to one owner rather than repairing disconnected projections.
A precedes C because it retains independently useful execution guarantees
without rewriting their whole host integration.

## Verification before replacement activation

- Exercise the actual configured native host with current project instructions and loaded skills.
- Assert exact latest-user-span and five-message-floor selection in the outgoing assessment,
  including a marked old prefix that must be absent and long selected content that must remain complete.
- Assert complete current governing policy, current operation, and trusted state separately.
- Complete initial and follow-up turns, resume, reload, and cancellation
  without source-history amplification in model input.
- Verify unsatisfied unrelated prerequisites remain unsatisfied after generic Allow,
  while a valid fresh-confirmation requirement and prohibition controls retain their distinct outcomes.
- Verify approval remains tied to the exact prepared group and becomes invalid when its required dependencies change.
- Exercise accepted program profiles and ordinary direct-tool behavior explicitly;
  document unsupported forms rather than claiming general coverage.
- Verify bypass on and off against supported and ordinary unsupported program shapes,
  preserving fixed guards and truthful user-visible behavior.
- Verify preparation, conversion, retry, and all bundles share the 15-second budget;
  incomplete or malformed answers cannot authorize execution.
- Verify trust, reset, fork inheritance, and approval reuse against whichever accepted product scope is being activated.
- Verify failed builds leave the previously activated artifact usable.

Local deterministic logic and selection are in-process tests.
Pi and filesystem integration use disposable real host sessions and directories.
The external assessor uses authored-response tests for contracts and authorized live checks for operation.
Live success does not establish an accuracy guarantee for the adopted 0.60 cutoff.

## Decisions not made by this review

No teardown, source replacement, automatic exclusion threshold, approval-cache restoration,
new provider purchase, or semantic scope expansion is enacted by this report.
The recommendation needs implementation acceptance before production work resumes.

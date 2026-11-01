# Pi automatic-mode semantic admission

## Current Luna decision

On 2026-10-16,
the user clarified that fifteen seconds is an implementation guideline, not an enforced review deadline.
Elapsed preparation, conversion, classification, or provider time must not alone reject evidence,
cancel a request, or trigger manual confirmation.
Keep the duration as a performance target and retain original cancellation, freshness,
and actual transport-failure handling.
This supersedes the hard preparation-inclusive fifteen-second cutoff recorded in earlier decisions.

On 2026-10-16,
the user explicitly excluded model thinking from review input.
Thinking blocks and provider signatures must not reach Luna through either the proposed operation or conversation.
Human confirmation summaries likewise describe the proposed tool calls without model thinking.
Original responses remain local where required for operation ownership and currentness.

The user also reaffirmed parsing Bash locally and sending only its structural operation data to Luna.
The incumbent shell parser owns quoting, substitution, command boundaries, and shell control structure.
Review projections must preserve arguments, paths, redirections, nested execution, and unresolved syntax facts
needed to assess permission without asking Luna to parse a raw Bash script.
Source text remains local for execution and original-operation identity.
This implements the previously recorded parser-first boundary at the current native request path.

On 2026-10-16,
the user limited automatic approval review to permission boundaries.
Tool-use, formatting, naming, and workflow conventions remain instructions for the coding agent;
violating such a convention alone does not require consent or prohibit an otherwise authorized operation.
For example, preferring `read` over `sed` does not turn local file inspection into an approval request.
Explicit restrictions on access or effects, destructive or external actions outside existing authorization,
and genuinely required consent remain approval concerns.
An explicit prohibition on reading a named file remains enforceable even though reading is ordinarily benign.
Both clause interpretation and complete-operation review must apply this distinction to the actual operation.
This supersedes treating every instruction-compliance defect as an execution-permission defect.
It does not change the selected model, score cutoff, operation identity, currentness, or cancellation checks.

The user also limited Luna's input to the visible conversation, proposed operation,
and readable project policy files even when those files have not appeared in the transcript.
Pi's hidden system prompts, tool metadata, tool-use guidance, and skill catalog are excluded from model review.
Selection follows original source provenance before clause indexing;
removing text only from shared context would still leak excluded instructions through individual questions.
Local provenance receipts may remain for operation ownership and currentness checks.
Their retention does not authorize transmitting them as review evidence.

The user further required startup relevance selection for `AGENTS.md` and prohibited sending its complete document.
Initialization refreshes Luna's permission-relevance classification of individual original rules.
The existing policy-change cache direction retains unchanged classifications and reclassifies changed rules;
later operation reviews receive only the selected rules and their needed heading context.
The user subsequently directed that uncertain rules also be culled.
Only tagged rules with relevance probability at least `0.60` remain selected;
lower scores are excluded from both action questions and shared context.
This selection cutoff does not change the separate action-approval cutoff.
The complete file remains local for source freshness and indexing.
Neither relevance classification nor cache reuse grants permission to execute an operation.
The old experimental relevance cache was not connected to the installed host and admitted no exclusions;
its existence did not satisfy this requirement.

On 2026-10-16,
the user clarified the fallback order after ordinary repository inspection was falsely blocked:
clause assessment first,
then Luna reviews the complete prepared operation,
then the original confirmation dialog only if automatic review cannot resolve it.
The second model question retains complete current instructions,
selected conversation,
every prepared command and argument,
and the original assessment budget.
It uses the existing `0.60` selected-option cutoff for `approve` and `deny`;
`unknown`, invalid responses, and unavailable assessments leave manual review as the last fallback.
A clause's unresolved prerequisite estimate is not an established missing authorization.
The final model must apply the actual conditions of each rule to the user's actual operation.
Established governing prohibitions remain terminal.
Changed or cancelled operations cannot receive a replacement approval for their old evidence.

This supersedes direct clause-uncertainty-to-dialog routing and terminal withholding based solely on
unresolved prerequisite estimates.
After both automatic paths fail,
an authenticated Allow authorizes the exact prepared group without rewriting retained prerequisite estimates
as satisfied external facts.
Local decision logs retain correlated model inputs,
source spans,
answer distributions,
admission choices,
errors,
and final routing so a future rejection can be reconstructed.

On 2026-10-14,
the user selected manual confirmation when automatic tool-call assessment remains uncertain.
The native host must offer `Allow` and `Stop` after the automatic paths remain unresolved when its UI is available.
An established applicable prohibition still blocks.
Only an original Allow for the exact prepared operation permits execution;
denial,
dismissal,
and cancellation stop the run.
This supersedes the earlier policy that uncertainty never selects an approval request.
Missing UI,
stale operation or source dependencies,
and cancellation do not acquire permission through this fallback.

On 2026-10-14,
the user selected Luna,
excluded Perplexity from further trials,
and initially accepted `0.8+` because the operating goal is to improve on manually judging every request.
After examining which score the consumer uses,
the user explicitly lowered the operating cutoff to `0.6` on the same date.
The current target is `openai/gpt-6-luna-decisions` through the existing Gateway Decisions route.
A supported selected meaning is admitted when its selected-option probability is `0.60` or higher with a unique lead.
The separate response `confidence` field is not the thresholded quantity.
Unknown,
unsupported,
tied,
malformed,
or lower-scoring estimates remain unresolved.
This applies equally to permission,
restriction,
and absence interpretations;
the independent ownership,
authority,
coverage,
and authorization requirements remain intact.
The model score is an operating signal,
not a measured correctness probability.

The maintained clause vocabulary also includes `fresh-session-approval`:
an express requirement for a new confirmation from this session's user for the exact prepared group.
The native host supplies availability and freshness of its pending confirmation channel.
This meaning supplies no permission and cannot satisfy unrelated authorization requirements.
The same `0.60` selected-probability rule applies.
The native confirmation dialog offers `Allow` and `Stop`;
every non-Allow response records refusal and aborts the current run.

The user clarified that the approval subject is the model-initiated tool operation and its arguments.
The current native evaluator does not ask a separate whole-request admission question.
The historical response replay counts in this document retain their original vocabulary and question set.

The prior `0.99` choice was the agent's conservative heuristic under delegated authority.
It was scoped to Jev and was not empirically calibrated.
Treating it as a settled Luna requirement was incorrect.
Luna's initial primary permission estimate was `0.83`,
but only four of its 84 answers met the initial `0.80` boundary.
All distributions in that retained response satisfy the existing normalization tolerance;
the largest measured sum deviation is `2.220446049250313e-16`.
Changing the cutoff alone therefore does not complete native execution admission.
At the subsequently adopted `0.60` boundary,
the rebuilt consumer admits 29 of those same 84 answers:
28 clause meanings and the whole-request content meaning.
The remaining 55 are unadmitted;
this retained-response replay made no provider calls and establishes no accuracy rate.

Implementation must change the fixed serving identity,
Gateway request and response conversion,
and operating cutoff together.
Verification includes boundary values,
negative decisions,
complete answer identities,
and original deadline accounting.
Question-format experiments must retain complete source input and criteria,
measure unchanged-input variation before comparisons,
and retain failures without retrying for a higher score.
The [Gateway probe record](../planning/pi-auto-mode-gateway-decisions-current-contract.md)
retains the initial serving evidence.

## Historical Jev decision

On 2026-10-14,
the user delegated both selection and adoption of the production acceptance rule.
The native host admits a supported categorical interpretation only when its selected score is at least `0.99`
and strictly exceeds every other score.
This is an operating cutoff,
not a measured accuracy rate or a statistical error bound.
The retained diagnostic studies did not qualify the current categorical definitions;
their historical cutoffs remain diagnostic.
No further paid qualification is authorized by this decision.

## Scope

The historical policy fixed `typesafe/jev-1.13.0`.
The original scope was `awaited-string-result-then-literal-parallel`.
On 2026-10-14,
the user also adopted `awaited-literal-single-call`:
a returned awaited tool call with a literal argument object,
or an immutable binding of that call followed by its direct return.
The fixed bootstrap scope `native-inspected-programs-v2` contains exactly these two decoder profiles.
Neither the scope identifier itself nor another program shape is an inspected operation.
Both profiles retain the same cutoff and independent authority,
source,
argument,
result,
and cancellation checks.
It applies separately to clause relationships,
whole-request content,
source-local heading content,
and express same-source quoted-unit incorporation.
It does not generalize to another serving model,
question definition,
or program profile.

Clause interpretations may admit `permission`,
`prohibition`,
`prerequisite`,
or `none`.
Request and heading interpretations may admit `none` or `potential-instruction`.
Unit incorporation may admit `adopts` or `none`.
`unknown`,
`unsupported`,
ties,
and scores under the cutoff remain unresolved.
Malformed or incomplete distributions fail validation.

The same cutoff applies to every admitted label,
including `none` and prohibition.
An estimate cannot discard a possible restriction with a weaker standard than it uses to interpret permission.
A highly scored `potential-instruction` still preserves the corresponding interpretation obligation.
An admitted prerequisite supplies no permission,
and incorporation does not establish source authority or valid delegation.

## Rationale and limits

The chosen rule favors abstention over using less decisive model answers.
Accepting every structurally valid answer would make uncertain interpretations operative.
A lower cutoff would admit additional answers without current evidence establishing that they are suitable.
Rejecting every estimate would prevent the adopted semantic consumer from operating.
Requiring exactly `1` would assign special significance to a model-reported endpoint that has no demonstrated
calibration advantage over the chosen operating cutoff.
Ranking:
the `0.99` rule precedes exact-`1` admission because it avoids treating a score endpoint as proof;
exact-`1` precedes lower-cutoff admission because it retains the stricter abstention preference;
lower-cutoff admission precedes accepting all answers because it still checks decisiveness.

No arithmetic combination of these scores establishes whole-decision accuracy.
Original ownership,
currentness,
source authority,
complete operation and instruction coverage,
and authorization satisfaction remain independent code-owned requirements.
Historically,
unresolved or unspecified policy blocked execution without an approval request.
The user's 2026-10-14 uncertainty-confirmation decision supersedes that fallback for the native interactive host.
The pure resolver continues to preserve unresolved and unspecified results;
the host owns manual confirmation and original execution eligibility.
This direction is enforced by
`package/pi-plugin/auto-mode/native/contract/integration/action-policy/approval.mjs`
and `package/pi-plugin/auto-mode/src/guard-outcome.ts`.

Implementation and verification are tracked in
[the migration handover](../handover/pi-auto-mode-current-frontier.md).

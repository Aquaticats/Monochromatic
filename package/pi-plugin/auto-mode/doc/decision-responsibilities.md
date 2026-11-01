# Decision review responsibilities

## Operation lifetime

`native/contract/integration/action-policy/decision-core.mjs`
captures the configured policy and the original approval host.
Its operation closure prevents overlapping reviews,
requires complete prepared membership,
and closes a failed judgment before returning the original failure.
Those checks belong with the operation owner,
because that owner alone coordinates review and subsequent execution.

## Judgment review

`native/contract/integration/action-policy/decision-review.mjs`
captures the original inputs,
assesses the applicable questions within the existing budget,
compiles their evidence,
and resolves automatic or manual review.
It retains the original currentness checks and evidence closure.
It receives an existing judgment rather than creating a second execution owner.

## Admission questions

`native/contract/integration/action-policy/decision-scopes.mjs`
(current path: `native/contract/integration/action-policy/decision-scopes.ts`,
moved October 19, 2026)
chooses which clause,
heading-context and same-source incorporation questions apply.
This calculation reads the selected policies and original candidate collections.
It does not add message categories or a new input-validation protocol.
Its TypeScript inputs derive the candidate collections from the maintained judgment's
`InstructionPolicySelection` and its effects from `SemanticAdmissionInputs`.
Absent policies remain absent at the typed call seam;
the scope wrapper already omits absent policies before calling its maintained consumer.

## Error identity

`native/contract/integration/action-policy/decision-errors.mjs`
holds the original review-state and established-decision constructors.
The gate re-exports these constructors for existing consumers.
The review and gate import the same declarations without a circular dependency.
The split adds no error fields or constructor behavior.

## Size and verification

The split addresses responsibility and max-lines limits.
It is not a claim that aggregate implementation characters decreased.
The measured character receipt is `decision-responsibility-split-receipt.json`
in the current private qualification scratch.
Formatting remains accepted;
line limits continue to require manageable modules.

Qualify the split with a fresh private build,
the full `test:native:phases` catalog,
both inflight-input consumers,
and the source-freshness and schema-change consumers.
Check actual configured execution again after the remaining package changes.
Root distribution files remain pinned during these private builds.

# Handover: split readonly diagnostics by evidence

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/caught-value` is now `package/caught-value/core`.

Live document.
Update it after each implementation decision,
landed increment,
verification stage,
and issue-tracker mutation.

## Goal

Implement the full diagnostic split investigated in
[`doc/planning/prefer-readonly-positive-evidence.md`](../planning/prefer-readonly-positive-evidence.md)
for [GitHub issue #422](https://github.com/Aquaticat/Monochromatic/issues/422).

The target policies are:

- `prefer-readonly-parameter-types` reports only mutable parameters with a proved deeply readonly replacement;
- a separate correctness rule reports readonly declarations with proved caller-reachable mutations;
- a separate audit rule reports unresolved parameter-reachable effects with cause-specific wording.

Effect-summary construction and propagation must remain shared.
Suppressing a report at an sound-readonly declaration must not remove its charge from mutable callers.

## User decisions

The user selected the full split rather than the minimal sound-readonly reporting guard.

The final shared Oxlint configuration must contain all three rule entries.
Only `prefer-readonly-parameter-types` remains enabled at `error` initially.
The projected-readonly-capability correctness rule and unresolved-effect audit rule must be set
to `off` because the full workspace is
not ready for them.

Each disabled configuration entry must have a comment linking to one newly opened GitHub tracking issue.
That issue must state that both rules move directly from `off` to `error` when readiness criteria are met.
A warning phase is rejected.

Add the shared configuration entries only as the final implementation step.

## Grilling status

Resolved:

- implementation scope:
  full three-rule split;
- initial configuration:
  preference rule at `error`,
  other split rules at `off`;
- future severity:
  both disabled rules move directly to `error`.

Resolved after handover creation:

- public rule IDs:
  `no-readonly-parameter-mutations` and `no-opaque-parameter-effects`;
- terminology cleanup:
  eliminate moralized readonly terminology from the complete current readonly domain,
  including internal discriminants,
  message IDs,
  fixtures,
  tests,
  documentation,
  and editable issue #422 comments;
- policy gap:
  add an `AGENTS.md` rule requiring neutral operation or evidence terms in diagnostic names and messages.

Historical Git commits remain unchanged.
Unrelated prose using the ordinary English term outside readonly analysis remains out of scope.

Further resolved decisions:

- no-disable policy:
  retain the existing preference-rule guard only;
  defer guards or a reviewed acceptance mechanism for disabled rules to the readiness issue;
- shared architecture:
  compute category-neutral parameter evidence once per semantic source snapshot,
  then let thin rule reporters filter their categories;
- previously hidden contract policy:
  add `no-invalid-parameter-effect-contracts` for stale or missing mutation contracts,
  overload-contract disagreement,
  and redundant ownership markers.

The complete public split therefore has four rules.
Only the preference rule starts at `error`.
The other three start at `off`,
share the tracking-issue comment,
and eventually move directly to `error`.

Further policy decisions:

- readiness may use a reviewed acceptance manifest;
  accepted entries require stable site fingerprints and stale-entry detection;
- readiness also includes fixes,
  verified intrinsic or effect modeling,
  and accurate effect contracts;
- `no-readonly-parameter-mutations` reports only proved reachable mutations through readonly parameters;
- projected readonly types retaining unresolved callable capabilities belong to
  `no-opaque-parameter-effects`,
  even when the callable body does not invoke them.

This evidence routing replaces the old type-level classification grouping.
The mutation rule must never report a mutation that analysis has not proved.

Neutral internal classification names are:

- `deep-readonly` for structure proved deeply readonly;
- `projected-readonly-capability` for a readonly projection retaining unresolved callable behavior;
- existing `mutable` and `opaque-capability` names remain.

Both sides of the previous moralized opposition must disappear from the current readonly domain.

Issue lifecycle:
verified implementation closes issue #422.
Deferred workspace enforcement lives only in the new readiness issue.

Do not implement until the user confirms the grilling session has reached shared understanding.

## Existing evidence

A detached-worktree prototype at `0d54ea643` added a reporting-only guard for
`classification.kind === 'sound-readonly'` before `opaqueEffectReport`.
On one matched dependency tree,
the package lint moved from 118 to 66 `prefer-readonly-parameter-types` errors.
The guard removed 52 sound-readonly reports.

The direct control pair in
`package/test-fixture/oxlint-no-restricted-syntax/src/readonly-result-provenance-invalid.ts`
proved propagation remained intact:
`handsReadonlyNamesOnward` became silent while `handsMutableNamesOnward` retained the propagated unresolved
`JSON.stringify` effect.

The prototype built,
and
`mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types`
passed.
The full unit suite was not run because current assertions require the old already-readonly report.
Implementation must update those expectations and retain the mutable-caller propagation control.

The accepted audit at
`doc/audit/tech-prefer-readonly-native-effect-analysis-vet-2026-07-22.md`
requires unresolved effects to be derived,
contained by a verified isolation boundary,
or reported as opaque.
It also requires fail-closed behavior.
The split preserves this capability,
but the initial `off` configuration is an explicit temporary enforcement deferral requested by the user.

## Published investigation

The issue contains six investigation comments.
The final synthesis is
[`issuecomment-5286101731`](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5286101731).

One earlier comment incorrectly claimed `grade-agreement.ts` had narrowed `verdict` to a primitive.
The public correction is
[`issuecomment-5286087654`](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5286087654).
Do not restore the withdrawn primitive-provenance claim.

## Repository state at handover creation

The active branch is `main`.
The investigation document is already on `origin/main` through commit `558123c00`.

The main worktree has a pre-existing modification to `.serena/project.yml`.
Do not edit,
stage,
restore,
or otherwise disturb it.

The investigation's detached worktree has been removed.
Implementation is complete.

Landed commits:

- `c7d176520` adds neutral diagnostic-language policy `DNL` to `AGENTS.md` and generated `CLAUDE.md`;
- `d81af397a` replaces moralized readonly terminology with `deep-readonly` and
  `projected-readonly-capability` across current readonly code,
  fixtures,
  and documentation;
- `28d82a8f9` adds the four public rules,
  category-neutral callable and source evidence,
  a semantic-source evidence cache,
  and thin category reporters;
- `253fa5841` adds split-rule ownership,
  propagation,
  external-worker,
  and evidence-cache coverage;
- `0e3307db5` completes neutral wording,
  package documentation,
  and cache-sharing verification;
- `b9e427dfc` adds all four shared configuration entries,
  leaving the preference rule at `error` and linking each extracted `off` rule to issue #423.

The first package unit run confirmed the legacy fixture enabled only the preference rule.
The dedicated fixture now enables all four rules,
and the external-worker test enables the opaque-effect rule it measures.

The split revealed one intentional diagnostic combination hidden by old early returns:
invalid host-capability boundaries receive both unresolved-effect and missing-contract diagnostics.
Opaque inputs do not receive stale-contract diagnostics while their effects remain unresolved.
Updated assertions name rule ownership rather than accepting count changes alone.

Coverage landed in `253fa5841` and follow-up fixes:

- one fixture has exactly one finding owned by each public rule;
- a projected unresolved capability does not appear under the mutation rule;
- direct evidence-cache counters prove one computation plus three hits across four distinct contexts;
- the existing readonly-to-mutable propagation pair asserts ownership by
  `no-opaque-parameter-effects`.

Verification completed before the shared configuration edit:

- plugin build passed;
- plugin type lint passed;
- complete plugin unit suite passed after the final source edits;
- `no-restricted-syntax` unit suite passed after neutralizing its no-disable diagnostic;
- affected Markdown lint passed;
- package Oxlint reported zero errors and 1,206 warnings after removing warnings introduced by this change.
  A detached baseline at `49d832467` also reported zero errors and 1,206 warnings,
  all from the separately introduced workspace `one-var` policy.

The deferred-enforcement tracker is
[issue #423](https://github.com/Aquaticat/Monochromatic/issues/423).
It requires direct promotion from `off` to `error`,
allows reviewed fingerprinted acceptance with stale-entry detection,
and defers no-disable policy until that mechanism is chosen.

Shared configuration landed only after pre-configuration verification.
Post-configuration verification established:

- `mise run //package/config/oxlint:build` passed;
- `mise run //package/config/oxlint:lint:types` passed;
- importing `package/config/oxlint/dist/final/node/index.mjs` returned the preference rule at `error` and all three
  extracted rules at `off`;
- `mise run //package/module/caught-value:lint:oxlint` passed through the built shared configuration;
- source and built configuration both contain every extracted rule;
- three source comments link issue #423 and direct promotion to `error`;
- current readonly-domain source and documentation contain zero rejected moralized classification terms;
- affected editable investigation comments on issue #422 were updated to neutral terminology.

The attempted historical `translation-repair` consumer task was stale session context:
that package and task do not exist on current `main`.
The existing `module/caught-value` package provided the successful shared-config consumer boundary instead.

Independent review found and closed two semantic gaps in `96e04609a`:

- accepted host opacity had been folded into legacy `mutated` facts;
  the mutation rule now reads a separate `provedMutation` fact based only on referent-mutation evidence;
- stale contracts were being reported while effects remained unresolved;
  stale-contract reporting now requires `!opaque`.

The same commit adds changed-source evidence-cache invalidation coverage.
The full plugin unit suite passed after these fixes.
Issue #423 now also requires extending self-hosting overrides before promotion.

The pre-existing verified `one-var` direction fix was integrated as `660d36792`.
After rebuilding the shared configuration,
package Oxlint passed with zero warnings and zero errors.
The config build,
config type lint,
built-severity assertion,
and `module/caught-value` consumer check also passed again.

The evidence cache uses immutable semantic `SourceFile` identity as its key.
Changed source text is measured to create one miss and one fresh computation.
The real all-rules fixture verifies routing through Oxlint,
while the direct cache test verifies one computation across four context objects.

## Next action

Post the implementation evidence to issue #422,
close #422,
and leave all deferred workspace enforcement in issue #423.

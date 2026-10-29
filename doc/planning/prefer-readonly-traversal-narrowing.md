# Narrow traversal effects and resolve workspace source in `prefer-readonly-parameter-types`

Decisions grilled and confirmed on 2026-07-15.
Scope:
`package/oxlint-plugin/prefer-readonly-parameter-type` and every consuming package.
Rollout:
 big-bang;
the effort is not done until the whole repo is green under the new semantics.

## Problem evidence

- Session `34e949c6-8475-49fd-bd8b-5ac1469d16a3` (file-enforcer Cargo.toml dedup):
  the `tomlSet` catalog entry marks its `value` property as an opaque target
  purely because value traversal can invoke caller getter hooks;
  the entry's own evidence says "returns fresh state".
  That single traversal effect propagated `@mutates value` onto `setIfDiffers`,
  then demanded `@mutates enforcement` on `applyEnforcement` and `@mutates plan` on `applyCargoPlan`,
  annotations claiming mutation of values nothing mutates,
  which `JCH` forbids.
  The agent circled for hundreds of transcript lines and the user aborted twice,
  leaving file-enforcer deliberately red.
- Workspace catalog entries in `workspace-package-effect-catalog.ts`
  carry hand-computed `commit <sha> <file> sha256 <digest>` evidence strings for repo-owned code.
  Nothing validates them:
  the only evidence test asserts non-emptiness
  (`intrinsic-effect-catalog.unit.test.ts`,
   "records authoritative evidence for every audited host entry").
  They rot silently into false negatives when the audited source changes.
- Diagnostics do not echo whether a `@mutates` contract was parsed,
  nor why an input is considered affected,
  so agents cannot distinguish "annotation unrecognized" from "contract incomplete" from "propagated cause".
- Annotation volume at decision time:
  647 `@mutates` entries across 317 files,
  958 `ForeignBorrowed` usages,
  zero real inline suppressions (the ban rule works).
- Known modeling gap:
  branded primitive intersections are not modeled
  (`doc/handover/file-manager-sticky-prototypes.md`).

## Decisions

### Narrow traversal effects by type (strict structural)

Pure read or traversal of a parameter stops counting as a caller-observable effect
when the argument's static type is plain data:

- primitives,
  including branded primitive intersections;
- literal types;
- arrays,
  tuples,
  `Record`s,
  object types,
  and unions composed only of plain data.

Anything with call or construct signatures,
methods,
function-typed properties,
class-instance provenance,
`unknown`,
`any`,
`object`,
or unresolved type parameters stays hook-tracked (fail closed).

Implementation shape:
one gate where effects apply to arguments,
so catalog opaque targets,
`String()` object coercion,
and `Object.entries`-style hook entries are all covered uniformly without rewriting entries.

Out of model by design:
a `Proxy` or getter-backed object satisfying a plain object type at runtime.
Static analysis cannot see it;
the README documents it instead of pretending the hole is closed.

Rejected alternatives:

- drop read-effects entirely (upstream typescript-eslint semantics):
  loses deliberate coverage of `Symbol.toPrimitive`,
  proxy traps,
  and deliberate object coercion at boundaries;
- primitives-only narrowing:
  does not fix the motivating case,
  TOML values contain nested tables and arrays;
- provenance-based narrowing (effects on caller-constructed literals unobservable):
  heavier than type-based and unnecessary,
  hooks inside plain-typed data are unobservable in practice regardless of who constructed the value;
- allowing function-typed properties inside plain data:
  reintroduces an invocable capability into "plain";
  revisit as a small isolated follow-up if the sweep shows config-object shapes staying noisy.

### Bridge follows workspace source

The semantic bridge resolves workspace package imports through `/ts` subpaths
and analyzes dependency TypeScript source directly.
Composite project setup stays enabled.
All workspace entries in the catalog are deleted;
the catalog remains only for external packages and host intrinsics.
The commit-plus-sha ceremony for repo-owned code disappears entirely,
and audits of repo-owned code can no longer rot because the analysis reads live source.

Rejected alternatives:

- prose-only evidence for workspace entries:
  unvalidated audit claims about mutable in-repo code are silent false-negative generators;
- validated and generated evidence (hash-recompute test plus generator task):
  keeps the catalog burden and breaks the plugin test on every audited-file edit;
  strictly worse than live analysis once cost is off the table;
- committed per-package effect summaries (like `.d.ts` emit for effects):
  reintroduces stale-artifact claims about newer source,
  the exact disease being cured.
  Retained as the named fallback optimization if warm performance regresses unacceptably.

### External catalog evidence gets machine validation

A plugin test recomputes the named digests from installed `node_modules` content
for external package entries (lezer,
 optique,
 turso,
 pi)
and fails on mismatch,
so a version bump forces a loud re-audit instead of silently invalidating the audit.

### Diagnostics: causal trace plus contract echo

Each finding shows:

- the per-affected-input propagation chain with locations
  (parameter,
   argument or property step,
   call,
   catalog entry or unresolved callee);
- the parsed `@mutates` contract as the rule understood it,
  plus the exact delta against what is required;
- only the remediations applicable to that cause,
  not the generic menu.

Length is unconstrained per `DGT`.

### Rule flags redundancy

New findings with removal suggestions:

- `ForeignBorrowed` markers whose provenance no longer affects any classification;
- `@mutates` entries for provably absent effects.

This makes the big-bang sweep mechanical ("fix everything the rule reports")
and prevents silent annotation rot from re-accumulating.
Accepted cost:
markers churn as the call graph evolves;
this matches the rule's existing call-graph-responsive philosophy.

### Big-bang rollout

No pilot,
no grace period,
no follow-up issues.
Commits still land scoped and early per `GCE`,
so main transits intermediate states,
but nothing is left outstanding at the end.

Definition of done:

- plugin full test suite green;
- cross-OS bridge workflows green (`readonly-semantic-bridge.yml`);
- repo-wide `lint:types` and `lint:oxlint` green,
  including the deliberately-left file-enforcer errors cleared
  and `module-toml-edit` signatures (for example `tomlSet` `value`) deeply readonly;
- zero redundancy findings repo-wide;
- semantic sweep perf baselines re-run and compared against the recorded indexed warm measurements;
  unacceptable warm regression escalates to the summary-artifact fallback;
- docs updated:
  plugin README,
  `doc/troubleshooting/oxlint-prefer-readonly-foreign-provenance.md`,
  `doc/troubleshooting/oxlint-prefer-readonly-host-intrinsic-evidence.md`.

## Explicitly unchanged

- `JCH` in `AGENTS.md`:
  the narrowing dissolves the contradiction observed in the session;
  its text stays accurate as written.
- The plugin's name,
  package identity,
  suggestion-only semantic rewrites,
  suppression ban,
  and fail-closed posture for non-plain types.

## Progress log

- 2026-07-15 `20a993421`:
  strict structural plain-data classifier
  (`plain-data-classifier.ts`) gating catalog opaque targets and
  global String coercion;
  classifier unit tests cover 18 shapes including branded intersections,
  recursive TOML-like unions,
  and readonly tuple references
  (`checker.isTupleType` resolves references,
   the Type method does not).
- 2026-07-15 `132bd4fe8`:
  `traversalHookOnly` marker on Object/Reflect enumeration targets;
  `join` and `toSorted` element-coercion guards widened to plain data
  (fields renamed to `requiresPlainReceiverElements` and
  `opaqueReceiverUnlessCallableArgumentOrPlainElements`);
  behavioral fixtures `readonly-plain-data-valid.ts` and
  `readonly-plain-data-invalid.ts` prove both gate sides.
  Note:
  the Object hook family records hook uncertainty as `targets`,
  not `opaqueTargets`,
  so the marker distinguishes hook-class from mutation-class targets.
- 2026-07-15 `dc2d6f5f8`:
  workspace source resolution.
  Root cause found empirically:
  workspace imports resolve through pnpm symlinks to real repository paths
  and TypeScript flags them `isSourceFileFromExternalLibrary`,
  which was the only barrier;
  the resolved signature declaration already pointed at live source.
  `isWorkspaceSourceFileName` (no `node_modules` segment) admits them in
  `callableDeclaration` and the summary-index scope.
  `workspace-package-effect-catalog.ts` deleted entirely.
  Regression test `workspace-source-effect.unit.test.ts` pins the
  `setIfDiffers`/`applyEnforcement`/`applyCargoPlan` chain to zero effects.
  Observed cost:
  cold `buildEffectSummaryIndex` for `apply-plan.ts` took 54 seconds
  (enlarged fixed-point scope);
  warm and persistent caches must carry the load,
  measure against recorded baselines before accepting.
  Capability-accounting shift:
  style-callback invocation for `formatRateLimitStatus` consumers now
  proves inside the analyzed workspace callee;
  caller summaries no longer carry audited `invoked` markings
  (statusline caller lint output verified unchanged).

### Inherited documented uncertainty needs no re-documentation

Decided during the sweep (commit `bd72572d6`):
a boundary function's complete `@mutates` contract is the audit,
exactly as deleted workspace catalog entries were.
Callers inherit the documented effect
(the parameter stays affected,
permitting mutable types)
without adding their own contracts;
the `missingUncertaintyContract` finding is removed.
Readonly-typed callers still get `uncertainReadonly`
naming the origin-located boundary.
Rationale:
the per-level contract chain was the burned-session spiral in
different clothing;
`caught-value` and logger consumers would have needed contracts at
every transitive call level after catalog deletion.

### Root-lint baseline context

The pre-refactor acceptance record
(`doc/planning/replace-prefer-readonly-parameter-types.md`,
"Final verification record")
shows root `lint:oxlint` reporting 3,792 warnings and 665 errors of
existing non-readonly findings over 2,548 files,
with zero readonly-rule findings,
at the point the migration was accepted.
Root scope includes `.mjs` bench and analysis files that per-package
gates exclude.
The sweep's green target is therefore:
zero `prefer-readonly-parameter-type` findings repo-wide,
per-package gates green,
and no unexplained growth over that recorded root baseline;
the baseline's pre-existing non-readonly debt is a separate backlog,
not part of this refactor.

### Perf numbers so far

- Cold `buildEffectSummaryIndex` for file-enforcer `apply-plan.ts`:
  54 seconds (enlarged workspace-source scope).
- `git-policies/cli` warm one-worker lint:
  6.1 s and 5.9 s against the recorded 4.26 s baseline
  (about 1.4 times warm regression;
  elevated,
  not fallback territory;
  re-measure after the sweep).

### Sweep completion record (2026-07-15)

- Zero `prefer-readonly-parameter-type` findings repo-wide,
  verified by the parallel fanout run over every package plus root files
  (`fanout-lint3` capture;
   the pre-refactor baseline also had zero).
- Native tuple-cast panic found during root lint
  ("checker.TypeData is *checker.TypeReference,
   not *checker.TupleType")
  fixed by detecting tuple references through the target's local
  objectFlags instead of the `checker.isTupleType` native request.
- Boundary-contract semantics
  (inherited documented uncertainty needs no re-documentation)
  landed mid-sweep;
  see the dedicated section.
- Remaining red scopes are pre-existing backlogs outside this refactor:
  `dependency-catalog/cli-cube` package warnings and the root-scope debt
  (recorded baseline 3,792 warnings / 665 errors).
- Perf:
  serial whole-repo sweep 1,639 s warm (was 816.5 s pre-refactor);
  new `lint:oxlint:fanout` task reaches 85 s warm.
  The 60-second goal and parallel-reliability work continue in
  issue #374.

## Open questions

- Exact classifier treatment of index signatures whose value type is plain data:
  admitted as plain (a `Proxy` can satisfy any object type,
  so index signatures are not a weaker guarantee than named plain properties);
  confirm during implementation that this does not admit `{ [k: string]: unknown }`.
- Whether contract-accuracy checking already flags stale traversal-only `@mutates` entries
  or the redundancy finding must cover that case alone;
  trace `verifier.ts` and `mutation-contract-query.ts` during implementation.

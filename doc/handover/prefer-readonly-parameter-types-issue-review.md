# Handover: improve `prefer-readonly-parameter-types` from GitHub issue evidence

Status:
living handover.

Last reviewed:
2026-08-21.

Source snapshot:
`f850dd98f`.

Scope:
`package/oxlint-plugin/prefer-readonly-parameter-type`,
its shared configuration,
and every repository GitHub issue that directly concerns the rule or plugin.

## Standing maintenance instruction

Keep this handover current whenever work changes:

- `prefer-readonly-parameter-types` or one of its extracted effect rules;
- a listed GitHub issue changes,
  or a new issue or comment concerns this rule family;
- an implementation decision or recommendation recorded here;
- verification evidence or an acceptance baseline recorded here.

This is event-driven maintenance.
It does not request a cron job,
a fixed cadence,
or deterministic scheduling.
Issue [#451](https://github.com/Aquaticat/Monochromatic/issues/451) tracks where a durable discovery pointer for this
instruction should live.
Root `AGENTS.md` remains unchanged while that placement question is open.

## Executive direction

Implement observer-return retention as a fact distinct from unresolved-call opacity under
[#427](https://github.com/Aquaticat/Monochromatic/issues/427).
This is the remaining rule-ergonomics prerequisite:
identity observers must continue withholding readonly offers without producing unnamed-call opacity diagnostics.

Then take the stable full-workspace fingerprint required by
[#423](https://github.com/Aquaticat/Monochromatic/issues/423#issuecomment-5288807256).
The package now proves cold,
warm,
one-worker,
and default-worker diagnostic equality on a disposable tuple-panic project,
but that is not the workspace acceptance sweep.
After remediation or reviewed acceptance,
move the three extracted rules directly from `off` to `error`.
There is no warning phase.

Treat [#441](https://github.com/Aquaticat/Monochromatic/issues/441#issuecomment-5308861506) as a separate semantic
snapshot design track.
The accepted direction is a run-frozen filesystem view,
not repeated live-filesystem freshness checks.

Issue [#453](https://github.com/Aquaticat/Monochromatic/issues/453) records every item intentionally deferred from the
ergonomic implementation session.
Issues [#451](https://github.com/Aquaticat/Monochromatic/issues/451) and
[#452](https://github.com/Aquaticat/Monochromatic/issues/452) retain the two narrow documentation-placement decisions.

## Evidence boundary

The review fetched all 426 issue records and all repository issue comments through the GitHub API on 2026-08-21.
It searched titles,
bodies,
and comments for:

- `prefer-readonly-parameter-types`;
- `prefer-readonly-parameter-type`;
- `readonly parameter`.

That produced 21 candidates.
Eighteen directly concern the current rule or plugin.
Three are historical or incidental references and are classified separately.
Every body and every comment for the direct issue set was read.

The current source was then checked against the open issues.
That reconciliation matters because issue comments contain withdrawn measurements,
and the original cache investigation describes schemas 4 and 5 while current source declares schema 8.

The 2026-08-21 implementation session added issues #451 through #453 after the original inventory.
They concern living-handover placement,
no-budget path-guidance placement,
and deferred session work respectively.
They are tracked explicitly rather than being folded into the original candidate count.

Implementation evidence now includes package lint,
the complete unit suite,
external packed-consumer execution,
semantic bridge host execution,
default-worker external implementation execution,
and a disposable Oxlint project comparing exact cold,
warm,
fixed-worker,
and default-worker diagnostics.

## Current architecture and confirmed open seams

The diagnostic split from
[#422](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5287254227) is present:

- `prefer-readonly-parameter-types` reports proved deeply readonly replacements;
- `no-readonly-parameter-mutations` owns proved mutations through readonly declarations;
- `no-opaque-parameter-effects` owns unresolved effects;
- `no-invalid-parameter-effect-contracts` owns contract and marker inconsistencies.

`package/config/oxlint/src/rule/restriction.ts` enables only the preference rule.
The extracted rules remain `off` with links to #423.
`package/config/oxlint/src/overrides.ts` exempts only the preference rule at the plugin's self-hosting boundary,
so #423 correctly requires parity before enabling the extracted rules.

The following implementation state is current:

- schema 8 persists validated source-local omission identities and bounded reason categories;
- cold and restored omissions log at `warn`,
  while caught stacks remain at debug level;
- exact lint diagnostic records match across cold,
  warm,
  fixed-worker,
  and default-worker processes in the tuple-panic regression;
- structured classifications retain every distinct reachable writable path without character or item budgets;
- path evidence retains workspace,
  default-library,
  external-library,
  and unresolved declaration ownership;
- local arrays,
  object literals,
  promise values,
  generic values,
  fold seeds,
  conditional branches,
  direct returns,
  and binding-mediated returns retain distinct producer evidence;
- position-aware foreign observer inbounds cover supported array observers,
  seeded and no-seed folds,
  referenced observers,
  explicit callback `this`,
  mixed inbounds,
  and inherited readonly-array surfaces.

The remaining #427 mechanism is retention conflation:
`propagateElementApplications` still adds opacity when an observer returns receiver state,
then copies opacity provenance from an observer that can have none.
That can withhold the preference correctly while producing an incorrect unnamed-call opacity diagnostic.

Issue #430 is resolved and closed by `3ef6fed29`.
The classifier now renders complete quoted paths,
explains shallow `Readonly<T>`,
and selects external or mixed declaration guidance from structured evidence.

The #441 bridge state remains as reported:
`openSemanticFile` checks deletion only for the previously active source,
and `semantic-overlay-filesystem.ts` delegates every unknown path to the live filesystem.

## Issue ledger

### Performance and analysis cost

- [#374](https://github.com/Aquaticat/Monochromatic/issues/374),
  closed:
  whole-repository warm Oxlint performance.
  The final matched work reduced the measured run from 3m04.7s to a band around one minute while preserving an exact
  diagnostic fingerprint.
  A later same-host sample moved just above one minute with the older configuration moving too.
  Durable rule:
  compare repeated matched runs on one quiet host,
  and preserve the complete diagnostic fingerprint rather than counts alone.

### Collection semantics and provenance

- [#414](https://github.com/Aquaticat/Monochromatic/issues/414),
  closed:
  already-readonly array-method findings had no applicable remediation.
  Result provenance replaced the type-shape result gate,
  collection-specific guidance landed,
  and the reported finding disappeared.
- [#415](https://github.com/Aquaticat/Monochromatic/issues/415),
  closed:
  collection dispatch trust baseline.
  The adopted policy trusts standard dispatch,
  indexed data properties,
  the standard iterator,
  and default `Symbol.species` for values typed as collection views.
  Do not reopen this asymmetry as an implementation accident.
- [#416](https://github.com/Aquaticat/Monochromatic/issues/416),
  closed:
  shorthand object properties lost returned origins and produced a false readonly offer.
  Commit `44b6cf767` resolves shorthand names through their value symbols.
- [#417](https://github.com/Aquaticat/Monochromatic/issues/417),
  closed:
  collection observers on a derived container were not analyzed.
  Several investigation comments were corrected after direct probes.
  Commit `ab8dda1f1` resolves a view receiver through element origins and closes both chained and bound fold forms.
- [#418](https://github.com/Aquaticat/Monochromatic/issues/418),
  closed:
  repository-declared iterators could mutate while iteration recorded no effect.
  Commit `4a95ce31a` identifies non-default-library iterator declarations and charges `for...of` and spread drainage.
- [#419](https://github.com/Aquaticat/Monochromatic/issues/419),
  closed:
  `join` lacked a conditional coercion channel.
  Commit `98471e6cf` admits it only when every element is strictly primitive.
  A deeply readonly object is still an object and remains capable of custom coercion.

### Persistent cache and semantic bridge

- [#420](https://github.com/Aquaticat/Monochromatic/issues/420),
  closed:
  outgoing overlays could leave stale semantic text.
  Commit `4add69903` retains every handed-over overlay instead of forcing a snapshot update on every source.
- [#421](https://github.com/Aquaticat/Monochromatic/issues/421),
  closed:
  declaration fingerprints mixed snapshot text with disk text.
  Commit `41aab4ab9` fingerprints declarations from the semantic snapshot too.
- [#439](https://github.com/Aquaticat/Monochromatic/issues/439),
  closed:
  Windows project-root cache keys used incompatible path identities.
  Commit `2f76363fe` normalizes the key while preserving TypeScript's project identity value.
- [#440](https://github.com/Aquaticat/Monochromatic/issues/440),
  closed:
  `created` versus `changed` was inferred from project reuse rather than service presence.
  Commit `2f76363fe` asks whether any materialized project already holds the source.
- [#441](https://github.com/Aquaticat/Monochromatic/issues/441),
  open:
  retained overlays and native program state can outlive deleted files.
  The issue rejects per-open live freshness checks as the destination and records a run-frozen versioning filesystem as
  the desired consistency boundary.

### Diagnostic policy and rollout

- [#422](https://github.com/Aquaticat/Monochromatic/issues/422),
  closed:
  already-readonly opacity errors said “or accept it” while no acceptance mechanism existed.
  The implementation split proved replacement,
  proved mutation,
  unresolved effect,
  and invalid contract into separate rules.
- [#423](https://github.com/Aquaticat/Monochromatic/issues/423),
  open:
  enable the extracted rules after workspace remediation.
  The prior baseline is obsolete after schema-8 cache repair,
  foreign observer propagation,
  and diagnostic changes.
  Take a fresh baseline only after retention is separated from opacity.
- [#424](https://github.com/Aquaticat/Monochromatic/issues/424),
  closed:
  diagnostics pointed at consumers and rendered multiline binding patterns.
  The implementation adds one-line parameter subjects and workspace-owned semantic type origins.
  Commit `3ef6fed29` and follow-up `b2078262d` remove lexical owner attribution for local expressions while preserving
  direct and binding-mediated callable returns.
- [#427](https://github.com/Aquaticat/Monochromatic/issues/427),
  open:
  composite implementation issue covering foreign observer provenance,
  external writable-declaration guidance,
  local producer attribution,
  persistent omission loss,
  false unnamed-call opacity,
  and integrity reporting.
  Commits `3ef6fed29`,
  `b2078262d`,
  and `f850dd98f` land every listed item except observer-return retention versus opacity and remaining named-boundary
  guidance such as `flat`.
  `doc/planning/prefer-readonly-issue-427.md` remains the rationale,
  while issue #453 is the current remainder checklist.
- [#430](https://github.com/Aquaticat/Monochromatic/issues/430),
  closed:
  complete quoted writable paths,
  external and mixed ownership guidance,
  shallow `Readonly<T>` warning,
  and deep projection fallback landed in `3ef6fed29`.
- [#436](https://github.com/Aquaticat/Monochromatic/issues/436),
  closed:
  a package lint failure exposed #427's callback-provenance gap.
  Commit `abd3d3f5d` changes `.some()` to `for...of`,
  a verified local workaround.
  Commit `3ef6fed29` later fixes position-aware collection callback provenance itself.

### Maintenance and session follow-up

- [#451](https://github.com/Aquaticat/Monochromatic/issues/451),
  open:
  decide where event-driven living-handover maintenance pointers belong.
- [#452](https://github.com/Aquaticat/Monochromatic/issues/452),
  open:
  decide where the rule-specific prohibition on character and path-count budgets belongs.
  Root `AGENTS.md` remains unchanged.
- [#453](https://github.com/Aquaticat/Monochromatic/issues/453),
  open:
  records observer-retention work,
  #423 rollout,
  #441 snapshots,
  and both placement decisions intentionally deferred from the ergonomic implementation session.

### Incidental and historical exact-text matches

- [#202](https://github.com/Aquaticat/Monochromatic/issues/202),
  closed:
  self-referential workspace dependency investigation.
  Only a resolution comment mentions standing readonly warnings in another package.
  It contributes no rule work.
- [#205](https://github.com/Aquaticat/Monochromatic/issues/205),
  closed:
  an older `editord` lint handoff for the former native
  `typescript/prefer-readonly-parameter-types` rule.
  Its capability-interface lessons are historical context,
  not the current plugin backlog.
- [#373](https://github.com/Aquaticat/Monochromatic/issues/373),
  open:
  `stylistic/no-mixed-operators` mishandles unary minus.
  It mentions a file in this rule package only as the place the unrelated diagnostic was encountered.

## Improvement sequence

### Completed: repair persistent omission completeness

Commits `3ef6fed29`,
`b2078262d`,
and `f850dd98f` implement one coherent cache shape:

- associate omitted callable identities with the source whose scan omitted them;
- persist validated omission metadata in `effect-cache-envelope.ts`;
- return it through `PersistentEffectCacheHit` and `LayeredSummaryCacheHit`;
- retain it in the process memory layer;
- merge it into the build's `omittedCallableKeys` before `assertReachedCallSummaries`;
- rotate the cache identity through schema 8;
- reject entries from prior schemas instead of inferring omissions from missing edges;
- retain specific bounded reason categories,
  including the TypeScript tuple serializer panic;
- emit concise cold and restored `warn` records,
  with full caught stacks available at debug level.

The user selected non-failing visible logging for incomplete analysis.
The rule fails closed locally by withholding affected offers,
but omission logs do not become Oxlint diagnostics or fail the lint task.

#### Chosen cache option

Persist validated omissions:

- Pros:
  preserves warm reuse,
  preserves deliberate fail-closed behavior,
  and restores exact omission observability.
- Cons:
  expands the envelope,
  validation,
  and cross-process test matrix.

Refuse to cache a source containing an omission:

- Pros:
  avoids a serialized omission format and stays fail closed.
- Cons:
  reruns the upstream panic in every process and loses warm reuse for affected sources.

Infer omissions from missing edges:

- Pros:
  avoids a new field.
- Cons:
  cannot distinguish a deliberate omission from corruption and weakens the completeness assertion.

Ranking:
persist validated omissions > refuse incomplete-source caching > infer omissions.
Persistence beats refusal because it preserves both correctness and reuse.
Refusal beats inference because explicit failure remains distinguishable from corrupt state.

Verified controls:

- the minimal TypeScript tuple panic writes an explicit source-local omission;
- separate cold and warm Oxlint processes emit identical diagnostic records;
- one-worker and default-worker runs emit the same diagnostic records;
- a schema 5 payload without omission metadata is a cache miss under schema 8;
- cold and restored warnings both name `typescript-tuple-serialization-failed`;
- corrupted keys must belong to the envelope source and cannot duplicate a persisted summary.

### Separate retention from unresolved effects

`propagateElementApplications` currently uses the opacity set for two different facts:

- an observer performed or reached an unresolved operation;
- an observer returned receiver state through the collection result.

Keep returned receiver state as a preference-withholding fact,
but stop routing it to `no-opaque-parameter-effects` when no unresolved call exists.
Extend the relation with exact collection member and call-site identity,
then carry a retention-specific provenance fact through:

- direct summary construction;
- fixed-point propagation;
- process-layer cloning;
- JSON serialization;
- cache validation;
- persistent restoration;
- public summary projection.

Completion criteria:

- Identity `map`,
  wrapped `map`,
  `map` plus `toSorted`,
  and `flatMap` retain preference withholding without an opacity diagnostic.
- A callback containing a genuine unresolved call remains opaque and names that call.
- Nested owned callers preserve the retention fact.
- A persistent round trip preserves the verdict and exact provenance.
- The diagnostic never falls back to “a call whose name this rule could not determine” for known collection retention.

### Completed: add position-aware foreign observer edges

Do not reuse broad effect reachability as ownership proof.
The effect relation may over-approximate observer positions safely because it adds mutation or uncertainty.
Foreign ownership suppresses a preference and therefore needs exact value flow.

The implementation adds a separate collection-observer inbound relation that states which callback formal receives:

- a receiver element;
- the receiver collection;
- fold accumulator state from a seed;
- fold accumulator state from receiver elements when no seed exists;
- an index;
- `thisArg` or another independent call argument.

Only receiver-derived positions feed the existing conjunctive foreign-ownership fixed point.
One ordinary,
mixed,
or unresolved inbound must still remove inferred foreign provenance.
Do not repeat `ForeignBorrowed` on callback descendants,
and do not add type-name or member-name ownership allow-lists.

#### Chosen foreign ownership option

Position-aware virtual inbound edges:

- Pros:
  preserve exact value flow and reuse the current conjunction rule.
- Cons:
  require overload authority and cache coverage for each supported member shape.

Repeated callback markers:

- Pros:
  work with the existing marker recognizer.
- Cons:
  duplicate descendant claims and can hide mixed inbound paths.

Type or member allow-lists:

- Pros:
  require less relation modeling.
- Cons:
  confuse declaration identity with runtime ownership and can suppress valid findings.

Ranking:
position-aware edges > repeated markers > allow-lists.
Exact value flow beats repeated assertions.
Repeated assertions beat global exemptions only because they remain local,
but neither fallback is recommended.

Verified controls:

- Cover `map`,
  `forEach`,
  `filter`,
  `find`,
  `findLast`,
  `every`,
  `some`,
  `flatMap`,
  `reduce`,
  and `reduceRight`.
- Seeded folds keep independent accumulators ordinary.
- No-seed folds derive accumulator ownership only from receiver elements.
- Index,
  explicit callback `this`,
  and `thisArg` positions never inherit receiver ownership.
- Referenced observers used only by a foreign receiver are proved.
- Reusable observers with one ordinary inbound remain unproved.
- Inherited readonly-array surfaces retain the default-library method authority.
- Explicit foreign ownership changes preference eligibility only;
  mutation,
  opacity,
  and contract evidence remain effect-driven.

### Completed: replace diagnostic strings with structured evidence

The classifier carries what reporting needs instead of parsing or extending `reason` strings.
For writable data,
retain:

- cause kind;
- complete reachable path segments;
- exact writable declaration identities;
- workspace,
  default-library,
  or external-library ownership for each resolved declaration;
- incomplete and multi-declaration states.

Render paths as code,
for example `` `child.position.start.line` is writable ``.
This resolves #430's path and quoting problems and gives #427 enough evidence to tailor external-declaration guidance.
External declaration ownership changes the remediation,
not the mutable classification.
A workspace-created value with an external type still needs ordinary readonly analysis.

Producer evidence now distinguishes:

- proved callable return producer;
- named type producer;
- exact local expression or binding producer;
- merged or unresolved producer set.

A reduce seed,
local array,
`Promise.resolve` argument,
generic argument,
or conditional branch must keep its expression or binding location.
A genuine returned object may still normalize to its producing callable.
Keep full offsets in identity so distinct expressions on one line do not collapse.

The suggestion engine uses its verified
`import('type-fest').ReadonlyDeep<...>` rewrite when `type-fest` resolves at the authored boundary.
For inferred external values,
name a local deep projection as the available action.
State explicitly that `Readonly<T>` is shallow and can move a finding inward without resolving it.
Do not add an external-type allow-list.

Verified controls:

- Diagnostics render every complete distinct quoted path for nested properties and index signatures without a display
  budget.
- Unique external,
  mixed,
  merged,
  re-exported,
  multiple,
  and incomplete declaration cases choose non-misleading guidance.
- Reduce seeds and comparable local expressions never receive callable-return advice based only on lexical containment.
- Direct and binding-mediated callable-return controls continue to name the callable.
- Every offered exact projection is resolvable and type-checks at the user-owned annotation boundary.

Collection-specific guidance for `flat` remains deferred under #427 and #453.

### Establish the rollout fingerprint and enable extracted rules

Do not use the #427 counts as a current baseline.
They were measured to demonstrate cache and attribution defects.
After observer-return retention is separated from opacity:

1.  run the extracted rules over the full workspace from a clean persistent cache;
2.  repeat warm and worker-count controls in separate processes;
3.  inventory the stable named-boundary remainder;
4.  resolve each site through code,
    verified behavior,
    accurate effect contract,
    or a reviewed site-specific acceptance manifest;
5.  make manifest entries use stable fingerprints,
    reject changed sites,
    and report stale entries;
6.  extend the plugin self-hosting override to all extracted rules;
7.  change the three currently disabled extracted entries,
    `no-readonly-parameter-mutations`,
    `no-opaque-parameter-effects`,
    and `no-invalid-parameter-effect-contracts`,
    directly from `off` to `error`;
8.  run affected package lint and the complete workspace lint.

Keep `JSON.stringify` opaque for unproved object graphs.
ECMAScript property access,
`toJSON`,
replacers,
and proxies are real user-code channels recorded in #427.
A static plain-data type or fresh outer wrapper is not isolation proof.

Completion criteria are those in
[#423](https://github.com/Aquaticat/Monochromatic/issues/423),
plus exact cold and warm fingerprint equality from the repaired cache.

### Design a run-frozen semantic filesystem

Handle #441 after the active #427 and #423 sequence unless a new current-host reproduction changes its priority.
The intended property is consistency:
one lint run sees one filesystem version.
Freshness checks on each open do not provide that property.

Design the filesystem boundary around `readFile` and `fileExists`,
because paths absent from `bridgeState.overlays` currently delegate there.
A complete design must freeze both overlay-backed and delegated files for a run,
then let a later run observe additions,
changes,
and deletions.

Completion criteria:

- One run cannot combine source text from before and after a mutation.
- Deleting an inactive file during a run cannot make the semantic service observe a mixed state.
- A later run observes the deletion.
- The test has a control proving it re-resolves the import rather than reading an unchanged native-service program.
- Verification uses a disposable fixture,
  not the working tree's live source state.

## Rejected shortcuts

- Do not enable #423's extracted rules before cache parity and the stable fingerprint.
- Do not treat stderr logger warnings as authoritative lint diagnostics.
- Do not repeat `ForeignBorrowed` on descendants of an already-marked boundary.
- Do not add type,
  package,
  or member allow-lists as substitutes for value provenance.
- Do not classify `JSON.stringify` as effect-free for arbitrary object graphs.
- Do not infer omitted callables from missing summary edges.
- Do not use lexical containment as proof that an enclosing callable produced a type.
- Do not replace #441 with repeated `existsSync` scans when the required property is a frozen run view.
- Do not restore handwritten ECMAScript,
  DOM,
  Node,
  or package effect catalogs.
- Do not schedule periodic maintenance for this handover.

## Verification commands

Read root and package `mise.toml` before changing this list.
The current package tasks are:

```bash
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:semantic-bridge-host
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:external-consumer-host
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:external-channel-workers
```

Cache and filesystem tests must use disposable cache roots and source fixtures.
Cold and warm comparisons must run in separate processes.
Worker parity must compare complete sorted diagnostic records,
not only counts.
Before crediting a null delta,
run a positive control known to change the fingerprint.

After TypeScript edits,
run package type lint manually.
After changing shared severity or overrides,
rebuild `package/config/oxlint`,
inspect the built severities,
and exercise a real consuming package through its package lint task before the workspace boundary run.

## Refresh procedure

When a maintenance trigger fires:

1.  fetch all current issue records and comments,
    not only open issues;
2.  exclude API records carrying a `pull_request` field;
3.  search title,
    body,
    and comment text for the singular package name,
    the plural rule name,
    and `readonly parameter`;
4.  read every new comment on the direct issue set;
5.  classify incidental matches explicitly rather than silently dropping them;
6.  compare open-issue claims with current source and tests;
7.  update the review date,
    source commit,
    issue states,
    issue dispositions,
    current seams,
    improvement order,
    and verification evidence;
8.  remove superseded advice instead of appending a second truth;
9.  commit the handover update with the related implementation or issue-tracker work.

Useful complete-repository retrieval commands:

```bash
gh api --paginate --slurp \
  'repos/Aquaticat/Monochromatic/issues?state=all&per_page=100'

gh api --paginate --slurp \
  'repos/Aquaticat/Monochromatic/issues/comments?per_page=100'
```

`gh issue list --search` is useful for discovery,
but the complete API retrieval is the exhaustiveness check.
Issue search can match comments and return candidates whose bodies are unrelated.

The handover is current only when the issue ledger and source assessment agree.
A changed issue state without source reconciliation,
or changed source without issue reconciliation,
is an incomplete refresh.

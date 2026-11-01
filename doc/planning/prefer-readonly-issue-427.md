# Issue 427 investigation and implementation proposal

## Status

Issue #427 is reproduced and diagnosed against commit `da3f2f4f9710ffd353de90eef87f0114e3ded1fa`.
This document proposes implementation scope;
it does not record an accepted architecture decision.
No production rule change was made during the investigation.

The stable evidence source is a fresh-cache run.
Warm schema 4 output is incomplete and cannot supply an acceptance fingerprint.

## Findings

### Foreign collection observers lack ownership edges

Effect analysis records default-library observer relations as `ElementApplication` values
and propagates observer effects.
The foreign-ownership graph has no corresponding receiver-element relation.
Its direct summary scans ordinary owned calls,
and its seed starts with no element applications.

Verified controls show:

- a marker directly on the callback formal is recognized;
- an element passed through an ordinary owned helper propagates foreign ownership;
- the equivalent inline collection callback remains unproved;
- one ordinary inbound removes the guarantee as intended.

The gap affects every tested observer member:
`map`,
`forEach`,
`filter`,
`find`,
`findLast`,
`every`,
`some`,
`flatMap`,
`reduce`,
and `reduceRight`.

The effect relation cannot be reused as ownership proof.
Effect reachability may safely over-approximate callback positions.
Ownership suppression needs exact positions and conjunctive inbound value provenance.
A seeded fold demonstrates the difference:
the element comes from the foreign receiver,
while the accumulator comes from an independent seed even when both positions have the same instantiated type.

### External declaration ownership changes guidance only

`RootContent` remains mutable under a shallow `Readonly` projection because nested properties remain writable.
A resolvable `type-fest` installation lets the current suggestion engine prove
`import('type-fest').ReadonlyDeep<RootContent>`.
Applying that edit removes the diagnostic and passes package type lint after build.

External declaration ownership does not prove runtime ownership.
A workspace-created value typed as `RootContent` correctly remains subject to the preference rule.
The classification must therefore stay mutable.

The guidance defect comes from lost evidence.
`ReadonlyClassification` renders a writable-path string but discards the declaration node used by `propertyIsReadonly`.
The guidance layer cannot distinguish a writable declaration the workspace can edit from one it cannot.

### Lexical containment is not producer attribution

TypeScript resolves the reduce accumulator's inferred property symbols to property assignments in the seed literal.
`originOwner` replaces those actionable declarations with their enclosing function.
The resulting advice tells the reader to change a callable return type even though the callable does not return the
value.

The same defect reproduces for:

- a local array literal flowing into `map`;
- `Promise.resolve` of a local object;
- a project-owned generic receiving a local object;
- a reduce seed;
- local conditional-union literals.

A genuine function-return producer remains correctly attributed.
Distinct local expressions inside one function can also collapse into false uniqueness
when lexical callable identity is used for deduplication.

Origin evidence needs separate producer kinds:

- a proved callable or named-type producer;
- an exact local expression or binding producer.

Full source offsets must remain part of identity so separate local expressions do not collapse because they display on
one line or share one lexical owner.

### Schema 4 forgets deliberate summary omissions

A fresh scan catches the TypeScript 7.0.2 tuple serializer panic
and records each omitted callable key in process memory.
The completeness assertion accepts those explicit omissions.
Persistent cache hits restore summaries and dependency closure,
but not omitted callable identities.

A one-worker cold-to-warm control measured:

- cold:
  188 findings,
  no semantic-evidence failures,
  and 2 deliberate omissions;
- immediate warm:
  166 findings and 126 semantic-evidence failures;
- repeated one-worker warm:
  the same 166 findings and 126 failures.

Two isolated default-worker cycles each cleared the disposable project's cache,
ran cold,
then ran warm.
Both cold runs produced the same 188-finding fingerprint and no semantic-evidence failures.
Both warm runs produced the same 178-finding fingerprint,
but their semantic-failure counts were 73 and 71 with different category splits.

The controlled matrix proves cold-to-warm coverage loss,
one-worker versus default-worker diagnostic divergence,
and default-worker warning-count instability.
It does not identify a cache-write race or prove that scheduling alone caused earlier non-isolated diagnostic-count
variation.
The one-worker differential is sufficient to prove persistent omission loss.

Raw logger warnings are not Oxlint diagnostics.
The damaged run reports `Found 0 warnings` while stderr carries 128 warning records.
A minified stack neither restores coverage nor gives the package author an action.

### Observer-return retention becomes false unnamed-call opacity

Seventeen fresh opacity findings say only that the rule could not determine a call name.
Every corresponding public summary contains the parameter in `opaqueParameterIndexes` and an empty provenance set.

`propagateElementApplications` causes the state.
When an observer returns receiver state,
it adds the receiver slot to opacity.
It then copies provenance from the observer's opacity facts.
A pure identity observer has a returned-state fact and no opacity fact,
so the caller receives an empty provenance set.
The diagnostic fallback mislabels known collection-result retention as an unnamed external call.

Minimal controls reproduce the state for identity `map`,
wrapped `map`,
`map` plus `toSorted`,
and `flatMap`.
An experimental retention-prefixed fact removes those opacity reports while retaining the preference-withholding fact.

A matched 298-file,
113-rule,
one-worker differential measured 185 opacity findings before the experiment and 168 after it.
Fingerprint comparison found 24 removed records and 7 replacement records at the same consumer locations:

- all 17 unnamed-call findings disappeared;
- 7 surviving named-boundary diagnostics dropped unrelated retention-only inputs from their subjects;
- no new consumer location appeared.

The production model must carry exact collection call identity and location on `ElementApplication`;
the experiment's generic text is not sufficient.

### The remaining opacity volume is heterogeneous

The valid fresh sweep contains 185 opacity findings across 95 files:
123 findings in non-test source and 62 in unit tests.
It contains 150 general call messages,
24 direct method messages,
and 11 collection messages.

Reconstructing wrapped diagnostic paragraphs yields 299 cause mentions over 102 distinct facts.
An earlier issue comment reported 248 because its parser split terminal-wrapped paths as causes;
299 supersedes that count.
Of the 185 findings,
130 name one cause and 55 aggregate multiple causes.

The 25 bodyless-callable mentions come from three declarations:

- 21 reach the injected `SyntheticClient.chatJson` (renamed `ModelClient.chatJson` on 2026-10-19) interface method;
- 3 reach the local `scorecard.ts` callback parameter;
- 1 reaches a test helper's union-valued callback.

Four more findings name a callback supplied through the injected transport seam.
These are dynamic callable boundaries,
not missing source paths.

The 24 direct method findings comprise caller-supplied validators,
Sinon test context operations,
Web Stream controllers,
an injected tokenizer,
and one `flat` call.
`flat` needs collection guidance rather than the generic input-method message.

Fourteen findings mention `JSON.stringify` at 10 call sites.
Eight test call sites produce 10 findings;
two production sites produce 4 propagated findings.
This is 7.6% of the fresh opacity findings,
not most of the rollout.

`JSON.stringify` remains correctly opaque for object graphs whose runtime isolation is unproved.
The ECMAScript algorithm gets properties,
gets and calls `toJSON`,
and invokes a replacer when supplied.
Runtime controls observed getters,
a mutating `toJSON`,
replacer calls,
and proxy traps.
A fresh wrapper or static plain-data type does not exclude those behaviors.

## Proposed implementation order

1.  Repair persistent omission completeness and rotate the cache schema.
2.  Establish a process-separated cold and warm fingerprint with worker parity.
3.  Add exact observer-return retention provenance and position-aware foreign observer applications.
4.  Preserve those new facts through in-memory cloning,
    serialization,
    validation,
    and persistent restoration.
5.  Add structured writable-path ownership and producer-kind evidence.
6.  Reclassify `flat` guidance and improve analysis-integrity reporting.
7.  Review the stable named-boundary remainder under #423 before moving extracted rules directly to `error`.

Cache correctness comes first because every later fingerprint and cache regression depends on it.
The current warm output must not update #423's accepted baseline.

## Ranked implementation options

### Cache omissions

#### Persist validated per-source omissions

Pros:
retains warm performance,
preserves the deliberate fail-closed path,
and can restore exact omission observability.

Cons:
requires a schema bump,
per-source identity validation,
and process-separated tests.
A bounded reason category must be stored or replayed so restored omissions do not become silent coverage loss.

#### Refuse to cache incomplete sources

Pros:
smaller correctness patch and no new serialized omission shape.

Cons:
recomputes the upstream panic in every process,
keeps noisy warnings,
and forfeits warm reuse for every affected source.
Any older incomplete entry must also be removed or treated as a miss.

#### Infer omissions from missing summary edges

Pros:
requires no schema field.

Cons:
cannot distinguish a deliberate omission from corrupt or incomplete cache data
and would weaken the completeness assertion.

Ranking:
persist validated omissions > refuse incomplete-source caching > infer from missing edges.
Persistence outranks refusal because it preserves both correctness and reuse;
refusal outranks inference because it stays fail-closed without masking corruption.

### Foreign collection provenance

#### Add position-aware virtual inbound edges

Pros:
reuses the conjunctive ownership fixed point,
covers the member family soundly,
and distinguishes receiver elements from seeds and independent arguments.

Cons:
requires overload-specific authority,
new cache facts,
and a broad position matrix.

#### Repeat `ForeignBorrowed` on callback formals

Pros:
works with the current marker recognizer.

Cons:
duplicates descendant ownership claims,
hides mixed inbound paths,
and makes the annotation claim ownership independently of the receiver relation.

#### Add type or member allow-lists

Pros:
small implementation surface.

Cons:
bypasses runtime value provenance and can suppress valid findings.

Ranking:
position-aware edges > repeated callback markers > allow-lists.
Exact value flow outranks duplicated assertions;
duplicated assertions outrank global exemptions only because they remain local and reviewable.
Neither fallback is recommended for migration.

### Producer attribution

#### Preserve exact producer kinds

Pros:
fixes reduce,
array,
promise,
generic,
and local-union cases with one evidence model while retaining genuine callable attribution.

Cons:
requires structured origin identities and updated guidance branches.

#### Add a reduce-seed special case

Pros:
fixes the reported accumulator example with narrow code.

Cons:
leaves every comparable local-expression case wrong.

#### Keep lexical callable normalization

Pros:
no implementation work.

Cons:
prescribes ineffective edits and collapses distinct local producers.

Ranking:
exact producer kinds > reduce-only exception > lexical normalization.
The shared model covers every measured shape;
the exception covers one;
the current model covers none correctly when lexical containment is the only relation.

## Required acceptance controls

### Persistent cache

- Inject one deterministic summary-build failure without relying on the upstream panic.
- Cover ordinary callee and callback edges to the omitted callable.
- Compare complete `(rule, file, range, message)` fingerprints across separate cold and warm processes.
- Compare one-worker and default-worker fingerprints.
- Verify a schema 4 payload lacking omission metadata is a miss.
- Emit a concise omission count and source identity on cold and restored paths without replaying minified stacks.

### Foreign observers

- Cover each supported member and overload.
- For seeded folds,
  mark only receiver-element and receiver-array positions as receiver-derived.
- For no-seed folds,
  test the accumulator's receiver-derived condition separately.
- Never mark index or `thisArg` positions from receiver ownership.
- Cover destructured formals,
  reusable observers,
  overload unions,
  multiple callback candidates,
  and mixed ordinary and foreign inbounds.
- Prove explicit foreign ownership changes only preference eligibility;
  mutation,
  opacity,
  and contract reporting remain effect-driven.

### Observer-return retention

- Keep identity `map`,
  wrapped `map`,
  `toSorted` composition,
  and `flatMap` out of the opacity reporter while preference stays withheld.
- Keep a callback with a genuine opaque operation reported with that named operation.
- Preserve retention through nested owned callers and a persistent-cache round trip.
- Render exact member and call-site identity when retention needs explanation.

### External writable declarations

- Cover unique external declarations,
aliases,
re-exports,
declaration merging,
mixed workspace and external writable paths,
and incomplete or multiple resolution.
- Offer exact `ReadonlyDeep` syntax only when `type-fest` resolves
and the replacement type-checks at the user-owned annotation boundary.
- Keep a workspace-created value with an externally declared type under ordinary preference enforcement.
- Keep incomplete and multi-origin guidance non-prescriptive.

### Producer attribution

- Assert user-boundary diagnostics for local arrays,
`Promise.resolve`,
project-owned generics,
reduce seeds,
and conditional unions.
- Assert exact local expression or binding locations and no callable-return advice.
- Keep distinct local union branches distinct by full offset.
- Retain a genuine function-return producer control whose diagnostic names the producer callable.

### Serialization policy

- Keep getter,
`toJSON`,
proxy,
and replacer controls opaque.
- Accept primitive-only serialization inputs and a separately proved isolated graph only when no caller-owned identity
or capability crosses the call.
- Do not treat a fresh outer object or static plain-data shape as isolation proof.

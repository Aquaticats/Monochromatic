# Readonly parameter semantic bridge technology vet

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

Status:
 complete for technology selection.
TypeScript 7's synchronous unstable API is selected;
remaining gates are implementation acceptance criteria rather than alternative-selection blockers.

Subject:
 readonly parameter semantic bridge.

Decision scope:
 choose the analysis pipeline for a project-owned Oxlint JavaScript rule replacing
`typescript/prefer-readonly-parameter-types`.

Started:
 2026-07-12.

Last updated:
 2026-07-12.

Governing skill commit:
 `3b6d1bd6ac0c6eb5704152ddb00e2b69ddcf653b`.

Governing skill SHA-256:
 `71c50a51d0f0086f789e350ef43824f8aead66435f9ab92d94aae751d16d8359`.

Compatibility fingerprint:
 `e949b2efc01b5c3df8a5044e24b4762403b9db2d82c5e581a21502d681c52fac`.

Active audit owner:
 current Pi session for the readonly-parameter replacement plan.

Prior compatible report:
 none found.

## Authority boundary

This audit may update planning,
audit,
and troubleshooting documents.
It may use disposable external clones and probes.
It may not add product dependencies,
change lint configuration,
or implement the rule before the user confirms shared understanding.

## Hard constraints

- Run inside the existing `no-restricted-syntax` Oxlint JavaScript plugin boundary.
- Preserve the selected layered deep-readonly and caller-observable mutation-effect contract.
- Produce deterministic offline diagnostics and editor code actions.
- Support active TypeScript workspace source on Linux x64,
  macOS arm64,
  and Windows x64.
- Keep tests,
  benchmarks,
  and declaration files exempt from source enforcement.
- Preserve `@mutates` blocks through declaration generation and bundling.
- Resolve owned aliases,
  packages,
  generics,
  recursive structures,
  brands,
  callbacks,
  capabilities,
  and cross-file effects to the depth required by the policy.
- Permit no global type-name allow list and no inline rule suppression.
- Keep native or prebuilt artifacts source-mapped and inspectable.

## Components and overlays

Every candidate is an inspectable open-source local technology or a custom composition of such technologies.

Applicable overlays:

- incumbent dependency replacement;
- high-trust plugin execution;
- native or prebuilt artifact boundary where applicable;
- multi-platform support.

Managed-service,
SaaS,
sensitive-data,
privacy,
residency,
and browser overlays are not applicable.
The rule runs locally over repository source and needs no network service.

## Frozen decision criteria

All weights are 1 because the user removed time and money constraints without assigning relative quality weights.
Hard-gate failures remain outside scoring.

- Semantic completeness for the selected readonly and effect contract.
- Reliability of Oxlint-node to semantic-node mapping.
- Incremental and editor lifecycle behavior.
- Cross-package and project-reference behavior.
- Explainability of diagnostics and generated summaries.
- Human auditability of source,
  native boundaries,
  and caches.
- Reuse of already-required repository components without semantic compromise.
- Platform and artifact provenance quality.

Ratings and sensitivity remain pending finalist validation.

## Frozen discovery query schedule

### Package registries

- npm search:
  `typescript type checker project service library`;
- npm search:
  `typescript semantic analyzer cross file`;
- npm search:
  `oxc parser type checker`;
- npm search:
  `typescript effect analysis`;
- npm metadata for every named candidate and direct semantic dependency.

### Repository hosts

- GitHub repository search:
  `typescript createProgram getTypeChecker incremental language service`;
- GitHub repository search:
  `TypeScript effect analysis parameter mutation`;
- GitHub repository and code search for Oxc parser,
  isolated declaration,
  and JavaScript-plugin integration;
- GitHub repository and code search for Yuku analyzer type inference and type objects;
- GitHub repository and code search for `rolldown-plugin-dts` program cache APIs.

### Broader web

- `TypeScript compiler API incremental program custom linter`;
- `TypeScript project service library custom rule`;
- `Yuku analyzer TypeScript type checker inference`;
- `Oxc isolated declarations semantic type information`;
- `rolldown-plugin-dts TypeScript Program API`.

### Repository evidence

- current Oxlint plugin packages and test support;
- root TypeScript and `oxc-parser` dependencies;
- current `ReadonlyDeep` uses;
- Oxc isolated declaration and `rolldown-plugin-dts` build integration;
- existing troubleshooting reports for readonly types,
  declaration generation,
  and build performance;
- hand-rolled scope,
  parser,
  and output-filter systems.

The de-duplicated expansion round added TypeScript ESLint Project Service,
`ts-morph`,
and `effect-analyzer`.
Registry,
repository,
web,
and local-source searches then produced only wrappers around the same TypeScript 6 API,
syntax-only analyzers,
or domain-specific consumers.
Discovery is saturated for the semantic-bridge decision.

## Candidate ledger

### Native Oxlint rule, the incumbent

Discovery source:
 repository config and installed Oxlint.

Lifecycle:
 screened out for category fit after serving as the behavioral baseline.

Reason:
 the user explicitly chose replacement,
and the measured allow-list and suppression surface is the defect under remediation.
The native implementation remains differential-test evidence rather than an adoption candidate.

### Syntax and body analysis using only the Oxlint JavaScript context

Discovery source:
 existing `no-restricted-syntax` rules and Oxlint plugin API.

Lifecycle:
 hard-gate exit.

Reason:
 the context has AST,
scope,
and references but no TypeScript type information.
It cannot satisfy alias,
generic,
package,
or deep readonly requirements alone.
It remains a baseline component for local effect syntax.

### TypeScript 7 `typescript/unstable/sync`

(`typescript/unstable/sync` was renamed `typescript/sync` in 7.1.0-beta,
2026-10-20.)

Discovery source:
 repository root TypeScript 7.0.2 package,
its installed declarations,
TypeScript's 7.0 announcement,
and `microsoft/typescript-go` source at
`168e7015edf98244febc8f4ae450b673b5d195d7`.

Lifecycle:
 selected with explicit unstable-API acceptance gates.

Role:
 keep one synchronous native API client per Oxlint plugin process;
open project snapshots,
overlay current source through virtual filesystem callbacks,
query types and resolved signatures,
and map diagnostics by source offsets.

Accepted risk:
 the API is explicitly unstable.
The project chose it and rejected a TypeScript 6 fallback.
All unstable access must remain behind a version-tested adapter that fails closed.

### TypeScript 6 compiler and language-service family

Discovery source:
 TypeScript's official 7.0 compatibility guidance,
`@typescript-eslint/project-service`,
`ts-morph`,
and `effect-analyzer`.

Lifecycle:
 user-directed exit.

Reason:
 the user explicitly chose the TypeScript 7 unstable API and prohibited a TypeScript 6 fallback.
The current `@typescript-eslint/project-service` peer range ends before TypeScript 7 and wraps
`typescript/lib/tsserverlibrary`.
`ts-morph@28` is built around TypeScript 6.0.2,
and `effect-analyzer` is an Effect-specific `ts-morph` consumer rather than a general replacement semantic engine.

### `oxc-parser` plus TypeScript semantics

Discovery source:
 user correction,
repository catalog,
and Oxc source.

Lifecycle:
 hard-gate exit.

Reason:
 Oxlint already supplies the Oxc AST and source spans.
The real-boundary TypeScript 7 probe mapped semantic nodes to exact Oxlint diagnostics without another parse.
`oxc-parser` adds no type objects or lifecycle capability
and would introduce a third tree without a compensating benefit.

### Yuku parser and analyzer plus a custom type evaluator

Discovery source:
 user correction,
`yuku.fyi`,
npm,
and `yuku-toolchain/yuku` source at
`d41e37c9eeac42ba72f1f7d5ef3c76a09975adc9`.

Lifecycle:
 hard-gate exit.

Reason:
 Yuku 0.6.1 exposes binding and module semantics,
not TypeScript inferred type objects or a `TypeChecker`.
A custom evaluator would reimplement generics,
conditional and mapped types,
assignability,
and external declaration semantics while still needing TypeScript for compiler parity.
The selected TypeScript 7 bridge already supplies symbol identity and resolved signatures for effect summaries.

### Oxc isolated declarations plus semantic analysis

Discovery source:
 user correction,
Oxc documentation,
Oxc source,
and repository tsdown configuration.

Lifecycle:
 hard-gate exit.

Reason:
 isolated declarations deliberately avoid type checking and require source annotations.
They omit function bodies and internal effect evidence,
then still require a semantic engine for inferred types and assignability.
They remain declaration-publication verification input,
not the lint semantic bridge.

### `rolldown-plugin-dts` declaration pipeline and TypeScript context

Discovery source:
 user correction,
repository lockfile and docs,
and `sxzz/rolldown-plugin-dts` source at
`e7de7e9210b94ddae88f6a7eae36e5ffd4d82d42`.

Lifecycle:
 hard-gate exit for semantic bridging.

Reason:
 public operations generate declarations rather than answer arbitrary type queries.
Its exported `TscContext` exposes cached TypeScript 6 programs only after emit-oriented setup,
and its build lifecycle does not match per-file editor linting.
It remains required for declaration-comment preservation tests.
The repository currently resolves 0.27.4,
while the audited main clone declares 0.27.7.

### `ReadonlyDeep` utility candidates

Discovery source:
 122 repository references and the selected ownership-aware remediation.

Lifecycle:
 selected authoring subcomponent.

Role:
 use `type-fest`'s `ReadonlyDeep` for parameter-local projections when a shared type remains legitimately mutable.
Every consuming package declares the catalog dependency directly.

Boundary:
 `ReadonlyDeep` does not establish effect soundy.
Capability methods retained by the mapped type remain subject to `projectedReadonlyCapability` analysis.
Project-owned duplicate utilities and synthesized inline structural types are rejected.

## Evidence collected

### Oxlint host separation

The installed Oxlint 1.73 plugin context exposes an empty `parserServices` object even under `--type-aware`.
Oxc source runs the regular JavaScript linter and tsgolint as separate paths.
Evidence and reproduction:
[`doc/troubleshooting/oxlint-js-plugin-type-information.md`](../troubleshooting/oxlint-js-plugin-type-information.md).

### Oxlint JavaScript fix kinds

JavaScript diagnostics expose direct fixes and suggestions,
not a dangerous-fix marker.
The selected semantic rewrites will use suggestions.
Evidence and reproduction:
[`doc/troubleshooting/oxlint-js-plugin-fix-kinds.md`](../troubleshooting/oxlint-js-plugin-fix-kinds.md).

### Yuku semantic surface

Yuku's analyzer documentation and exported declarations expose scopes,
symbols,
resolved references,
write flags,
module links,
and node identity.
Searches of its npm and Zig source found parser support for TypeScript type syntax but no TypeScript `TypeChecker`,
`getTypeAtLocation`,
or inferred type-object API.
This supports an effect-binding role but does not yet satisfy deep type evaluation alone.

### Oxc isolated declaration surface

Oxc documents `isolatedDeclaration` and `isolatedDeclarationSync` as per-file declaration emit without the TypeScript
compiler for source satisfying `isolatedDeclarations`.
The result contains declaration code,
source map,
and errors,
not type checker objects.

### `rolldown-plugin-dts` program cache

`src/tsc/context.ts` stores `ts.Program[]`,
in-memory file text,
and project maps,
and invalidates every program containing a changed file.
`src/tsc/emit-compiler.ts` creates or reuses programs but returns declaration code and maps through `tscEmitCompiler`.
The public `./tsc-context` export exposes context construction and invalidation;
the program creation helper remains internal.

### Installed declaration pipeline preserves `@mutates`

A disposable probe used installed Rolldown 1.1.5 and `rolldown-plugin-dts` 0.27.4 with the Oxc generator.
Oxc isolated declarations preserved three custom blocks on a function,
an overload,
and a type call signature with zero transform errors.
A declaration bundle entered through a separate re-exporting source file and retained all three `@mutates` targets and
descriptions.
This validates the selected custom tag against the installed transform and bundler versions.
Package README publication and a built external consumer remain implementation acceptance checks.

### TypeScript 7 synchronous semantic surface

The installed `typescript@7.0.2` package exports `typescript/unstable/sync`.
Its `API` creates snapshots containing projects,
programs,
checkers,
and emitters.
The checker exposes type,
symbol,
resolved-signature,
assignability,
property,
and index-info queries.
Virtual filesystem callbacks and `fileChanges.changed` update unsaved current-file text.

The synchronous client starts the bundled TypeScript native executable with `--api` and communicates through a
blocking RPC channel.
This matches Oxlint's synchronous rule lifecycle and makes process-scoped reuse mandatory.

Mapped readonly state is the remaining unstable seam.
The JavaScript API exposes `Symbol.checkFlags` only as a number;
the matching TypeScript-Go source defines `CheckFlagsReadonly` as `1 << 3`.
The disposable probe distinguished a recursive mapped `DeepReadonly` projection only after testing that bit.
Full source trace and reproduction:
[`doc/troubleshooting/typescript-7-unstable-sync-readonly.md`](../troubleshooting/typescript-7-unstable-sync-readonly.md).

### Real Oxlint boundary probe

A disposable plugin loaded `typescript/unstable/sync` from Oxlint 1.73.0 and reported semantic findings at TypeScript
source offsets.
The representative ASCII LF fixture covered four files and distinguished:

- mutable and explicitly readonly imported aliases;
- a recursive mapped `DeepReadonly<MutableEnvelope<string>>` projection;
- mutable unions;
- overload declarations and implementation signatures;
- function types,
  call signatures,
  and method signatures;
- one direct mutation,
  one cross-file transitive call,
  and one immediate generic callback invocation.

The effect probe is not a complete effect engine.
Its mutating-method name set is a feasibility shortcut,
unknown calls do not yet produce `opaqueEffect`,
and only one higher-order relation shape was exercised.

A separate virtual-filesystem probe changed one parameter from `ReadonlyEnvelope<string>` to
`MutableEnvelope<string>` in memory.
Snapshot 1 reported readonly and snapshot 2 reported mutable without a disk write.
On this Linux x64 fixture,
the recorded cold snapshot was 32.096689 milliseconds and the changed snapshot was 0.513561 milliseconds.
These are feasibility measurements,
not production budgets.

### Follow-up project, span, and type probes

`updateSnapshot({ openFiles })` discovered these separate configured projects from repository source paths:

- `package/oxlint-plugin/no-restricted-syntax/tsconfig.json`,
  containing 406 source files in the returned program;
- `package/module/jsonc-edit/tsconfig.json`,
  containing 571 source files in the returned program.

Each queried source file returned zero semantic diagnostics.
The recorded API request total for opening and querying both projects was 77.47605 milliseconds on the same Linux x64
host.
This proves configured-project discovery for two repository packages,
not every package topology.

One Oxlint run reported `threads_count: 16`
but all probe diagnostics carried one process ID and sequential snapshot IDs.
That run proves the tested embedded-JavaScript execution was serialized through one plugin process;
it does not prove every Oxlint mode or future host lifecycle.
A process search after Oxlint exited found no surviving native TypeScript API child for the fixture path.
A cache revision reused snapshot 1 and its effect summaries across ordinary disk-backed files.
The BOM fixture alone created snapshot 2 because Oxlint's source text and the raw disk text differed at the BOM;
the adapter must normalize `sourceCode.hasBOM` before deciding that an editor overlay changed.

A source fixture placed a BOM,
CRLF line endings,
an astral character,
and a combining sequence before the parameter.
The TypeScript source position mapped through `getLocFromIndex` to Oxlint line 5,
column 3,
and byte offset 87,
which matched an independent buffer search.
Parser recovery and source mutations around such characters remain open.

A type corpus exposed brands,
recursive types,
callable objects,
conditional types,
indexed access,
arrays,
collections,
weak collections,
typed arrays,
and a DOM cancellation capability.
The prototype distinguished the expected mutable and readonly object shapes,
but also established two important algorithm requirements:

- direct `ReadonlyMap` and `ReadonlySet` need collection-aware semantics because their observational methods are not
  syntactically readonly properties;
- `ReadonlyDeep<AbortController>` appears structurally readonly while retaining the mutating `abort` method,
  so capability effects must detect `projectedReadonlyCapability` instead of trusting mapped-property flags.

## Execution manifests

### TypeScript 7.0.2 probe

- Package:
  repository-installed `typescript@7.0.2` under the pnpm lockfile.
- Source revision:
  `microsoft/typescript-go@168e7015edf98244febc8f4ae450b673b5d195d7`.
- License:
  Apache-2.0.
- Native boundary:
  platform package already installed by pnpm;
  no probe-time artifact download.
- Lifecycle scripts:
  none executed.
- Subprocess:
  the API client spawned the installed native TypeScript executable with `--api`.
- Network:
  disabled by construction after source discovery;
  the probe read only repository packages and disposable fixture files.
- Filesystem:
  disposable fixture under `/var/home/user/temp/agent/readonly-parameter-probe-2026-07-12`;
  current-file overlays remained in memory.
- Command boundary:
  `mise` task invoking installed Oxlint against the disposable fixture,
  plus a `mise` task invoking the standalone snapshot probe.

### Source-audit clones

No upstream build or lifecycle scripts ran.
Read-only shallow clones were used at these revisions:

- `typescript-eslint/typescript-eslint@bcfe16fd2c4cbb12227168475172b46cd5788543`;
- `dsherret/ts-morph@699815f54ae9b5c2a93f016ba1a9df1e8ac1c014`;
- `jagreehal/effect-analyzer@ba7d42d81cd6b4e230dccb1954ac3434379d673d`;
- `microsoft/typescript-go@168e7015edf98244febc8f4ae450b673b5d195d7`;
- previously recorded Oxc,
  Yuku,
  and `rolldown-plugin-dts` revisions.

## Implementation acceptance gates

- Replace the prototype's mutating-method name set with symbol- and signature-grounded capability handling.
- Prove fail-closed `opaqueEffect` behavior for unresolved calls,
  dynamic dispatch,
  external callbacks,
  stored callbacks,
  closures,
  deferred work,
  recursion,
  and overload-selected effects.
- Calibrate readonly semantics for brands,
  callable objects,
  collections,
  weak collections,
  typed arrays,
  classes,
  accessors,
  recursive mixed-mutability cycles,
  conditionals,
  indexed access,
  mapped unions,
  platform capabilities,
  and variance.
- Test BOM,
  astral Unicode,
  combining characters,
  CRLF,
  comments,
  parser recovery,
  and source changes before claiming general span equivalence.
- Test multiple inherited `tsconfig` files,
  package aliases,
  symlinks,
  path case normalization,
  changed/deleted/renamed files,
  parallel Oxlint workers,
  repeated runs,
  snapshot disposal,
  overlay eviction,
  native-process cleanup,
  and memory growth.
- Test Linux x64,
  macOS arm64,
  and Windows x64 artifacts instead of inferring platform support from package metadata.
- Add TypeScript 7 as an explicit runtime dependency of the published plugin package and load the built plugin from a
  disposable external consumer.
- Treat CLI diagnostics as authoritative until Oxlint's language server supports JavaScript plugins;
  do not retain an editor-only incumbent approximation or add a separate editor integration.
- Retain the verified installed declaration-transform corpus and extend it through built package READMEs and a
  disposable external consumer.

## Validation plan

Use one adversarial calibration corpus for the primary pipeline:

- owned mutable and readonly object aliases;
- nested arrays,
tuples,
maps,
sets,
weak collections,
and typed arrays;
- branded primitives at top level and nested;
- callbacks and higher-order generics;
- external SDK and DOM capabilities;
- recursive,
conditional,
mapped,
indexed-access,
and union/intersection types;
- overloads,
interfaces,
abstract methods,
and project references;
- Unicode before parameter spans;
- aliasing,
destructuring,
closures,
async effects,
and callback propagation;
- declaration emit and bundled comment preservation.

Every finalist must cross the real consumer boundary by running as an Oxlint JavaScript rule against disposable package
fixtures using repository tasks or an equivalent isolated harness.

## Scoring and sensitivity

The selected candidate scores 31 out of 40 under the frozen equal weights:

- semantic completeness:
  4 out of 5,
  because TypeScript supplies the required semantic objects while the project still owns effect interpretation;
- Oxlint-node to semantic-node mapping:
  4 out of 5,
  because ASCII and adversarial Unicode positions passed while parser recovery remains an acceptance case;
- incremental and editor lifecycle:
  4 out of 5,
  because snapshots,
  overlays,
  caching,
  and project discovery passed while editor integration is unavailable;
- cross-package behavior:
  4 out of 5,
  based on two distinct repository package projects rather than every topology;
- explainability:
  4 out of 5,
  reduced for the hidden mapped-readonly flag;
- source and cache auditability:
  3 out of 5,
  reduced for the unstable API and native RPC boundary;
- reuse of repository components:
  5 out of 5;
- platform and artifact provenance:
  3 out of 5,
  because package metadata covers target platforms but runtime probes ran only on Linux x64.

Sensitivity does not change the result because TypeScript 7 is the only candidate permitted by the user constraint that
also matches the installed compiler semantics.
The reduced ratings become explicit implementation acceptance work rather than grounds to substitute TypeScript 6.

## Adoption and rollback plan

Adopt through one project-owned adapter inside the Oxlint plugin package.
The adapter owns API creation,
project discovery,
snapshots,
virtual overlays,
source-node lookup,
readonly queries,
and bridge diagnostics.
Effect-summary logic consumes adapter-owned identities rather than importing unstable TypeScript APIs throughout the
rule.

Keep the repository's existing TypeScript catalog and lockfile authority.
Do not add TypeScript 6,
`@typescript-eslint/project-service`,
`ts-morph`,
or `effect-analyzer`.
Add contract tests for every unstable method and for the `1 << 3` mapped-readonly bit.
Unknown API shapes,
missing projects,
error types,
or changed flag behavior must fail closed.

Rollback is local:
disable the new project-owned rule and restore the existing
`typescript/prefer-readonly-parameter-types` configuration while retaining migrated `@mutates` documentation.
No source-level migration should depend on an unverified automatic rewrite.

## Current outcome

Selected:
 TypeScript 7.0.2's `typescript/unstable/sync` API.

Rejected by user constraint:
 every TypeScript 6 fallback.

Rejected by hard gates:
 syntax-only Oxc and Yuku compositions,
Oxc isolated declarations,
`rolldown-plugin-dts` as a lint lifecycle,
and wrappers built on TypeScript 6.

Required before implementation completion:
 remaining semantic-bridge acceptance gates and built-package publication verification.
The installed declaration transform and bundler preserved the selected `@mutates` contract,
and `type-fest`'s `ReadonlyDeep` is selected for projection authoring.
Editor integration is explicitly deferred until Oxlint's language server supports JavaScript plugins.
Product implementation remains blocked until the audit finishes and the full plan reaches shared-understanding
confirmation.

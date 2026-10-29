# Dependency drift after un-swept bumps: unbash 6.0.0, yuku-analyzer 0.19.0, and the pi-ai 2026-10-15 catalog

`package/deps-update/cli` runs `pnpm update --recursive --no-save`
and exits without a type sweep,
so a dependency bump can land while consumers still compile against the previous dependency API.
This file records the 2026-10-20 batch for the dependencies outside the Pi extension API.
The same class hit `@earendil-works/pi-coding-agent` `1.3.0` twice
(`TS2740` on the verify fakes and the `hiddenTools` plus `before_agent_start` drift);
those incidents live in
[`pi-0-87-extension-api-breakage.md`](pi-0-87-extension-api-breakage.md),
whose `## Open questions` section first named the missing sweep.

Each incident in this file was diagnosed against the installed dependency declarations or data,
the deciding source for what an API currently is,
and fixed on 2026-10-20 before this write-up.

## Root cause class

`package/deps-update/cli` exists to run the real update
(`package/deps-update/cli/README.md`),
and no `lint:types` or `tsc` invocation exists anywhere under `package/deps-update/cli/src/`
(searched 2026-10-20).
Nothing else runs the type sweep automatically either,
so restored `lint:types` tasks only report drift when someone runs them.

The consequence is a recurring failure shape:
a bump lands,
the workspace keeps building on stale assumptions,
and the next `lint:types` or characterization run reports failures
that look like fresh regressions but are pre-existing dependency drift.
The full sweep recorded 2026-10-20 in
[`doc/handover/pi-advisor-dsh-variant-grilling.md`](../handover/pi-advisor-dsh-variant-grilling.md)
found 213 packages carrying a `lint:types` task,
with 170 passing and 43 failing;
of the failures,
one was real API drift (`package/mutation-test/cli-unused-export`),
38 were missing-or-stale-`dist` build-class failures,
and 4 were commented-out tasks matched by accident.

## Unbash 6.0.0 reshaped the test and prefix AST

### Symptom

`mise run //package/agent-harness-shared/shell-command-analyzer:lint:types` failed
with `tsc` reporting `TS2339` four times
(measured at pre-fix `b710410e81`):

```text
src/visit-expression.ts(164,34): error TS2339: Property 'left' does not exist on type 'TestLogicalExpression & { readonly [FOREIGN_BORROWED_MARKER]?: true; }'.
src/visit-expression.ts(169,34): error TS2339: Property 'right' does not exist on type 'TestLogicalExpression & { readonly [FOREIGN_BORROWED_MARKER]?: true; }'.
src/visit-node.ts(92,23): error TS2339: Property 'command' does not exist on type '(Negation & { readonly [FOREIGN_BORROWED_MARKER]?: true; }) | (Time & { readonly [FOREIGN_BORROWED_MARKER]?: true; })'.
src/visit-node.ts(97,20): error TS2339: Property 'command' does not exist on type '(Negation & { readonly [FOREIGN_BORROWED_MARKER]?: true; }) | (Time & { readonly [FOREIGN_BORROWED_MARKER]?: true; })'.
```

Because `package/pi-plugin/guardrail` and `package/pi-plugin/terminal-title` type-check
against this analyzer,
their `lint:types` tasks stayed red until the analyzer passed.

### Root cause

Installed `unbash` `6.0.0` reshaped the parsed tree,
and the installed declarations decide the shape
(`unbash/dist/types.d.ts`,
reachable as
`package/agent-harness-shared/shell-command-analyzer/node_modules/unbash/dist/types.d.ts`).
`TestLogicalExpression` is now n-ary with `operands` and no `left` or `right`
(`:464` to `:470`):

```ts
// unbash/dist/types.d.ts, version 6.0.0
export interface TestLogicalExpression {
    readonly type: "TestLogical";
    readonly pos: number;
    readonly end: number;
    readonly operator: "&&" | "||";
    readonly operands: readonly TestExpression[];
}
```

`Time` and `Negation` are now `PipelinePrefix` entries carried in `Pipeline.prefixes`
(`:316` and `:320`),
and neither carries a `command` member
(`Negation` at `:335` to `:339`,
`Time` at `:321` to `:333`):

```ts
// unbash/dist/types.d.ts, version 6.0.0, selected declarations
    readonly prefixes: readonly PipelinePrefix[];
export type PipelinePrefix = Time | Negation;
export interface Negation {
    readonly type: "Negation";
    readonly pos: number;
    readonly end: number;
}
```

The pre-fix visitors still assumed the binary test node and the command-wrapping prefix nodes,
so `tsc` rejected every `left`,
`right`,
and `command` access.
This is the second unbash reshape in this consumer;
the version 5 redirect and word-shape migration is recorded in
[`unbash-5-ast-adapter.md`](unbash-5-ast-adapter.md).

### Fix

Commit `3b9bb3ad6b` maps `expression.operands` to work items in
`package/agent-harness-shared/shell-command-analyzer/src/visit-expression.ts`
and deletes the `Time`/`Negation` `command` branch from
`package/agent-harness-shared/shell-command-analyzer/src/visit-node.ts`.
A bare prefix now falls through to the redirect-only fallback,
which `visitRedirectsWorkItem` already routes identically,
and `package/agent-harness-shared/shell-command-analyzer/src/ast-v5.unit.test.ts`
gained two-way and three-way `TestLogical` operand cases.

Tradeoff:
the dropped prefix branch is dead under unbash 6,
so a future unbash that reintroduces a wrapped command needs the branch again;
the new `TestLogical` cases are the guard that names the current shape.

### Recurrence check

After any `unbash` bump run:

```sh
# Repository root; separate commands preserve task boundaries
mise run //package/agent-harness-shared/shell-command-analyzer:lint:types
mise run //package/agent-harness-shared/shell-command-analyzer:test:unit
```

`TS2339` naming `TestLogicalExpression`,
`Time`,
or `Negation`
means the tree moved again;
diff `unbash/dist/types.d.ts` against
`package/agent-harness-shared/shell-command-analyzer/src/visit-expression.ts` and `src/visit-node.ts`.
Because guardrail and terminal-title consume the analyzer,
their `lint:types` tasks are the downstream signal.

## Yuku-analyzer 0.19.0 renamed the analyzer API

### Symptom

`mise run //package/mutation-test/cli-unused-export:lint:types` failed
with `tsc` reporting `TS2339` three times
(measured at pre-fix `b710410e81`):

```text
src/find-unused.ts(144,14): error TS2339: Property 'addFile' does not exist on type 'Analyzer'.
src/find-unused.ts(152,45): error TS2339: Property 'module' does not exist on type 'Diagnostic'.
src/find-unused.ts(174,20): error TS2339: Property 'referencesOf' does not exist on type 'Analyzer'.
```

### Root cause

Installed `yuku-analyzer` `0.19.0` renamed three members,
and its installed declarations decide the current names
(`yuku-analyzer/index.d.ts`,
reachable as `package/mutation-test/cli-unused-export/node_modules/yuku-analyzer/index.d.ts`).
`Analyzer.addFile` became `Analyzer.setFile` (`:143`),
project-wide reference search moved from the analyzer onto the binding
(`Binding.findReferences` at `:256`),
and `Diagnostic.module` became `Diagnostic.path`:

```ts
// yuku-analyzer/index.d.ts, version 0.19.0, selected declarations
  /** Analyzes a file, replacing any module at the same path. */
  setFile(path: string, source: string, options?: SetFileOptions): Module;
  /** Its uses across the project, through every import of it. Links. */
  findReferences(): Reference[];
```

`Diagnostic` is imported from `@yuku-toolchain/types` (`yuku-analyzer/index.d.ts:4`),
and that package declares the renamed field
(`@yuku-toolchain/types/index.d.ts:153` and `:157`):

```ts
// @yuku-toolchain/types/index.d.ts, version 0.19.0, selected declarations
interface Diagnostic {
  severity: DiagnosticSeverity;
  message: string;
  /** The file it belongs to, or null when the source was given no path. */
  path: string | null;
```

### Fix

Commit `ea2812babc` rewrites the three call sites in
`package/mutation-test/cli-unused-export/src/find-unused.ts`:
`analyzer.setFile(...)` replaces `analyzer.addFile(...)`,
the debug line prints `diagnostic.path`,
and `local.findReferences()` replaces `analyzer.referencesOf(local)`.

Tradeoff:
the reference query now hangs off the `Binding` the caller already holds,
so a caller that discarded its binding must keep one;
the query semantics (uses across the project,
linking on demand) are unchanged.

### Recurrence check

After any `yuku-analyzer` or `yuku-*` bump run:

```sh
# Repository root
mise run //package/mutation-test/cli-unused-export:lint:types
```

`TS2339` naming `Analyzer`,
`Binding`,
or `Diagnostic` members
means the API renamed again;
diff `yuku-analyzer/index.d.ts` and `@yuku-toolchain/types/index.d.ts`
against `package/mutation-test/cli-unused-export/src/find-unused.ts`.

## Pi-ai 1.3.0 shipped the 2026-10-15 model catalog

### Symptom

`mise run //package/pi-plugin/model-retirement:test:unit` failed five assertions
in the `retirement table characterization` suite of
`package/pi-plugin/model-retirement/src/retirement-table.characterization.unit.test.ts`
(measured at pre-fix `b710410e81`):

- `reads the catalog the fixture was generated from` (`:383`):
  `AssertionError: expected 1714 to equal 1604`.
- `reports the catalog stamp the fixture names, so an upgrade asks for review` (`:396`):
  `AssertionError: expected '2026-10-15T19:00:55.287Z' to equal '2026-10-01T18:57:11.882Z'`.
- `reproduces the pinned abstention tally` (`:402`):
  the tally moved from
  `{ keeperAmbiguity: 14, unorderedPair: 15, loserNewerThanKeeper: 0, versionlessProtected: 6, duplicateIdentity: 0 }`
  to
  `{ keeperAmbiguity: 13, unorderedPair: 12, loserNewerThanKeeper: 0, versionlessProtected: 7, duplicateIdentity: 0 }`.
- `reproduces the pinned retirement count` (`:408`):
  `AssertionError: expected 596 to equal 551`.
- `reproduces the pinned retirement table exactly` (`:437`):
  the pinned-versus-installed comparison reported 52 pinned rows absent
  and 97 rows new since generation.

This failure is data drift,
not an API break:
`decideRetirements` is unchanged,
and its inputs moved.

### Root cause

Installed `@earendil-works/pi-ai` `1.3.0` bundles the 2026-10-15 catalog,
and the bundled data decides what the catalog is.
The stamp lives in
`package/pi-plugin/model-retirement/node_modules/@earendil-works/pi-ai/dist/providers/data/.manifest.json`:

```json
// @earendil-works/pi-ai/dist/providers/data/.manifest.json, version 1.3.0, selected fields
{"schemaVersion":6,"generatedAt":"2026-10-15T19:00:55.287Z",
```

`package/pi-plugin/model-retirement/src/retirement-table.fixture.json` still pinned the 2026-10-01 catalog
(`catalogGeneratedAt` `2026-10-01T18:57:11.882Z`,
`entryCount` `1604`,
`retirementCount` `551`),
so every assertion comparing the pinned table to a fresh decision failed.
The test is built to fail exactly like this:
its module docstring says a Pi upgrade that changes any single retirement fails
until the diff is reviewed and the fixture is regenerated.

### Fix

Commit `22f130ed5b` regenerated
`package/pi-plugin/model-retirement/src/retirement-table.fixture.json`
with `mise run //package/pi-plugin/model-retirement:fixture:retirement-table`
after reviewing the retirement diff,
as the test's own procedure demands.
The review recorded in that commit classified the movement as
23 records re-keyed from provider `azure-openai-responses` to `azure` with identical retired id and keeper,
12 keeper upgrades to newly listed releases,
85 added records (new catalog models plus newly superseded ids),
and 10 dropped records whose retired ids left the catalog entirely,
with zero records keeping both retired id and keeper listed.

Tradeoff:
regenerating without review would pin whatever the catalog says,
including a mistaken retirement;
the characterization suite exists to force the review first,
so skipping it removes the only human check on this table.

### Recurrence check

After any `@earendil-works/pi-ai` bump run:

```sh
# Repository root; separate commands preserve task boundaries
mise run //package/pi-plugin/model-retirement:test:unit
```

On failure,
diff the pinned and regenerated retirement rows,
and review each movement category (provider re-keys,
keeper upgrades,
added records,
dropped records) before running
`mise run //package/pi-plugin/model-retirement:fixture:retirement-table`
and re-running the test.
The failing assertions cover entry count,
catalog stamp,
abstention tally,
retirement count,
and the exact table,
and the regeneration task prints the resulting counts and catalog stamp.

## Recurrence check after any dependency bump

The class root cause is the missing sweep,
so the class-level check is the sweep itself:
run every package `lint:types` task after a bump lands,
either per package
(`mise run //package/<path>:lint:types`)
or through the root fanout
(`mise run lint`,
whose per-package `lint` includes `lint:types`).
A bump that moves a dependency with a bundled data snapshot
(for example `@earendil-works/pi-ai`) also needs its characterization run,
which is the `model-retirement` check named in the pi-ai incident's recurrence check.

## Verification

The incidents were reproduced from the last pre-fix state,
commit `b710410e81`,
in a detached worktree with the installed dependency tree symlinked in,
so the declarations under test were exactly the bumped ones.
The harness for each incident is the package task named in its symptom,
run from the worktree root:

```sh
# Repository root of the pre-fix worktree; separate commands preserve task boundaries
mise run //package/pi-plugin/advisor:lint:types
mise run //package/agent-harness-shared/shell-command-analyzer:lint:types
mise run //package/pi-plugin/current-time-context:lint:types
mise run //package/pi-plugin/terminal-title:lint:types
mise run //package/mutation-test/cli-unused-export:lint:types
mise run //package/pi-plugin/model-retirement:test:unit
```

The failing catalog is the three incidents recorded here
plus the two `@earendil-works/pi-coding-agent` incidents in
[`pi-0-87-extension-api-breakage.md`](pi-0-87-extension-api-breakage.md)
(`TS2740` on the verify fakes
and the `hiddenTools` plus `before_agent_start` drift).
Build-artifact noise in the same runs
(`TS2307` for `../dist/final/node/index.mjs` in unit tests,
`TS7006` on their parameters) comes from the unbuilt worktree,
not from the drift;
those tests import the built package as the repository's test-import rule requires.
The passing catalog was verified 2026-10-20 on the fixed tree:
the analyzer `lint:types` and `test:unit`,
the current-time-context and terminal-title `lint:types`,
the terminal-title `test:unit`,
the unused-export `lint:types`,
and the model-retirement `test:unit` all exit 0,
including the two-way and three-way `TestLogical` cases
and the custom-triggered `before_agent_start` case in terminal-title.
The advisor `lint:types` run on that date fails with `TS2307` missing `src/*.ts` modules,
which is concurrent rewiring of `package/pi-plugin/advisor`,
not this drift class.

## What does not work

- Treating a green unit suite as evidence that a bump is safe:
  the model-review tests in
  [`pi-0-87-extension-api-breakage.md`](pi-0-87-extension-api-breakage.md)
  passed while the drift landed,
  and only `lint:types` or the characterization run exposed it.
- Downgrading the dependency or loosening the consumer's types
  to silence `TS2339` or `TS2740`:
  the installed declarations are the current API,
  and widening hides the next reshape instead of adopting it.
- Hand-writing fixture rows to quiet the characterization suite:
  the row set is generated by `decideRetirements`,
  so manual edits drift from the generator and the exact-table assertion re-fails on the next run.
- Reading `event.prompt` or similar union members without narrowing on the discriminant:
  the terminal-title handler titled custom runs
  `π Received prompt: undefined` before it narrowed,
  recorded in [`pi-0-87-extension-api-breakage.md`](pi-0-87-extension-api-breakage.md).

## Upstream filing decision

1.  Upstream fault:
    no.
    Each dependency ships the documented current API or catalog;
    the repository consumers lagged behind a bump that landed without a type sweep.
2.  Upstream fixability:
    no upstream behavior change is needed.
    The consumer fixes and the fixture regeneration are complete in this repository.
3.  Supported use:
    all three dependencies export the consumed shapes deliberately
    (unbash's parsed tree types,
    yuku-analyzer's project queries,
    pi-ai's bundled catalog data),
    and the breakage is stale consumer assumptions,
    not an unsupported combination.
4.  Contribution policy:
    not evaluated for a filing,
    because no upstream change is requested.
5.  Maintainer willingness:
    not evaluated for a filing;
    no willingness or refusal is inferred.
6.  Prototype:
    the implemented and tested changes belong to this repository's consumers
    and fixture,
    not an upstream prototype.

No exemption in `.out-of-scope/` matches this bug class
(the filenames there cover other tools,
checked 2026-10-20).
No upstream issue or comment is being drafted or filed:
the upstream-fault condition fails,
so no tracker search runs and there is no additive comment to prepare.
The class-level action item is internal,
the missing `deps-update` sweep recorded in the
`## Open questions` section of
[`pi-0-87-extension-api-breakage.md`](pi-0-87-extension-api-breakage.md).

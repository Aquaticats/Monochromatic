# CSS `@mixin`/`@apply` in a pnpm monorepo: LightningCSS `customAtRules` + `var()` bug, no node_modules resolver

This file documents the cascade of issues hit while trying to
implement `@mixin` + `@apply` CSS syntax across the workspace.
The Vite/Astro-era sections are historical context kept because
they explain why the workspace settled on the in-house
`build-tool-css` package;
 Vite is no longer used as a candidate.

## Symptom

The "simple" feature:
 support author-written `@mixin --foo
{ ... }` definitions and `@apply --foo` consumption sites,
inside a pnpm-isolated monorepo,
 with imports across workspace
packages.

```css
@mixin --flex-center {
  display: flex;
  align-items: center;
  justify-content: center;
}

.component {
  @apply --flex-center;
}
```

SCSS has had this for over a decade;
 PostCSS plugins exist;
Tailwind popularised `@apply`.
 In this workspace,
 every
candidate pipeline failed for a different reason.
 Each failure
gets its own canonical section below.

## Bug 1: LightningCSS `customAtRules` chokes when CSS contains any `var()` (GitHub issue #1081)

### Symptom

Configure LightningCSS's `customAtRules` to recognise
`@mixin`/`@apply` per the documented example.
 The moment any
file in the bundle uses `var(--something)`,
 the parser fails
on the unrelated declaration.

### Root cause

`customAtRules` interacts incorrectly with custom-property
substitution.
[parcel-bundler/lightningcss#1081](https://github.com/parcel-bundler/lightningcss/issues/1081)
tracks the upstream defect.
 Without a fix,
 the
`customAtRules` path cannot coexist with CSS variables,
 which
are foundational in any modern design system.

### Verified workaround

None within LightningCSS itself;
 the only escape is to
process custom at-rules outside LightningCSS (PostCSS plugin
on the source,
 before LightningCSS bundles).

Tradeoff:
 introduces a second tool in the pipeline.
 This
two-tool stage (LightningCSS bundle + PostCSS mixins) was the
design for a while,
 then dropped entirely:
 LightningCSS and
oxc-resolver were removed in favour of a pure-JS PostCSS
pipeline.
 See "Current state" below and
[.out-of-scope/lightningcss.md](../../.out-of-scope/lightningcss.md).

### Why we do not file this upstream (again)

Already filed (#1081).
 5 constraints:

1. Upstream's fault?
    Yes.
2. Can upstream fix?
    Yes,
    but the issue has been open with
   no movement.
3. Supporting this use case?
    Documented in examples;
    the
   defect is the gap between docs and reality.
4. Will they fix?
    Unknown;
    track #1081.
5. Prototyped a minimal fix?
    No.

Decision:
 no new report;
 track existing.

---

## Bug 2: LightningCSS `bundle()` has no `node_modules` resolution

### Symptom

```css
@import '@monochromatic-dev/style-monochromatic/index.css';
```

resolves to:

```text
Resolved: @monochromatic-dev/style-monochromatic/index.css
  -> /src/styles/@monochromatic-dev/style-monochromatic/index.css
```

LightningCSS literally concatenates the package name as a
path component.
 It does not look in `node_modules`,
 does not
understand `package.json` `exports`,
 does not understand
workspace links.

### Root cause

LightningCSS's `bundle()` does relative-path resolution
only.
 The Node module-resolution algorithm
(`node_modules` walk,
 `exports` fields,
 conditions) is not
implemented.

### Verified workaround

Plug a custom resolver into the bundling pipeline.
 The
workspace uses `oxc-resolver` (the same resolver Rolldown
uses) to compute the on-disk path before LightningCSS sees
the import.

Tradeoff:
 introduces another component (`oxc-resolver`) and
its own resolution-strategy decisions.
 See Bug 3 for the
secondary failure that bit when pnpm dropped the
`oxc-resolver` symlink under isolated mode.

### Why we do not file this upstream

LightningCSS's stated scope is CSS transformation,
 not
module resolution.
 The custom-resolver hook is the
documented escape.
 No upstream report.

---

## Bug 3: pnpm isolated mode + global virtual store drops `node_modules` symlinks

### Symptom

```text
$ pnpm list --filter @monochromatic-dev/build-css
dependencies:
  oxc-resolver 11.16.4

$ ls package/build/css/node_modules/
lightningcss/
postcss/
                        ← oxc-resolver missing

$ bun package/build/css/src/index.ts
Error: Cannot find package 'oxc-resolver'
```

The package is in the lockfile,
 listed by `pnpm list`,
 and
present in the global store.
 The symlink into the local
`node_modules` is not created.
 Re-running
`pnpm install --force` does not always recover it.

### Root cause

The interaction between:

- `hoist: false`
- `nodeLinker: isolated`
- `enableGlobalVirtualStore: true`
- `hoistWorkspacePackages: false`

produces a configuration matrix where some packages resolve
correctly via `pnpm list` but never get symlinks created in
the consumer's `node_modules`.
 No error is emitted;
 the next
runtime invocation fails with "Cannot find package".

### Verified workaround

The workspace migrated the `build-css` package off pnpm
isolated mode for installation purposes by using Bun's
package manager for that specific package's dev workflow.
After migration the `oxc-resolver` symlink resolves
correctly.

Tradeoff:
 two package managers active for different
packages;
 mental-model overhead.
 Acceptable because the
build-css package is small and the symlink failure is
specific to it.

### What does not work

- `pnpm install --force`
- Deleting `node_modules` then `pnpm install`
- Verifying the package is in the global store

None of these recovered the symlink in the original failure.

### Why we do not file this upstream

The failure mode is hard to reproduce minimally;
 without a
minimal repro pnpm cannot act.
 No upstream report yet;
 the
Bun-pm migration solved the consumer's problem.

---

## Bug 4 (historical): Vite + Astro CSS pipeline bypassed PostCSS for some files

### Symptom (historical)

PostCSS plugin handling `@mixin`/`@apply` worked standalone;
worked for most files when configured in Vite's
`css.postcss.plugins`;
 failed for some Astro-generated CSS
(SSR/client pipelines bypassed the configured PostCSS).

Result:
 90% of CSS was transformed,
 10% still had raw
`@apply` rules in production.

### Root cause (historical)

Vite's CSS pipeline has multiple entry points:

- CSS imported from JS/TS goes through the configured
  pipeline.
- CSS in `<style>` tags in Astro/Vue/Svelte may or may not.
- CSS generated by framework SSR may bypass the pipeline
  entirely.

`css.transformer` is exclusive:
 either `postcss` or
`lightningcss`,
 not both.
 The `transform` hook exists but
runs after Astro has already extracted and processed `<style>`
blocks.

### Verified workaround (historical)

The workspace abandoned Vite as a build candidate;
 the
in-house `build-tool-css` package ran the LightningCSS +
oxc-resolver + PostCSS pipeline directly without Vite's
mediation,
 and later dropped LightningCSS and oxc-resolver
for a pure-JS PostCSS pipeline (see "Current state").

Tradeoff:
 cannot use Vite-specific features.
 Acceptable
because the project does not need them.

### Why we do not file this upstream

Vite is not used here;
 no upstream report.

---

## 2026-07-22: postcss replaced by the in-house css-edit CST

`build-tool-css` no longer uses PostCSS.
Parsing now sits on `@monochromatic-dev/css-core-edit`
(`package/css/core-edit`),
an in-house byte-preserving stylesheet-structure CST over the
`@csstools/css-tokenizer` spec tokenizer,
joining the `jsonc-edit`/`toml-edit` format-editing family.

### Why: the 2026-07 pure-JS parser survey

Motivation:
 dissatisfaction with postcss
(unguarded `process.env` reach-in forcing a browser shim,
stringly-typed at-rule params,
mutable OO AST,
dated dependency posture).
Every candidate was probed empirically on the same corpus
(`@mixin` bodies mixing declarations with relaxed and `&` nesting,
`@apply`,
 comments,
 braces and semicolons inside strings and `url()`,
escaped selectors,
 custom-property block values):

- `css-tree` 3.2.1 (the original migration target):
   rejected.
   Relaxed nesting without `&` becomes Raw plus a parse error
   (present in real repo CSS at the time in
   done-postcss `mixins.css`),
   unknown at-rule blocks parse in rule-list mode
   (fixable only via a `fork()` parser config,
   verified working),
   comments are dropped,
   generate() is minified-only,
   and `@types/css-tree` lags a major version.
- `@adobe/css-tools` 4.5.0:
   runner-up.
   Passed the corpus including relaxed nesting and comments;
   zero deps,
   TS-native types.
   Rejected:
   custom-property block values (`--raw: { x };`) throw,
   regex-era scanner (two patched 2023 ReDoS CVEs),
   string preludes keep the stringly-params pain.
- `@projectwallace/css-parser` 0.18.1:
   no code generation at all;
   analyzer only.
- `@stacksjs/ts-css` 0.1.1:
   silently deleted a nested rule and ejected
   `@apply` to top level;
   week-old 0.x,
   Bun-first.
- `@csstools/css-parser-algorithms` 4.0.0:
   component-value level only;
   the modern spec-faithful pure-JS effort is deliberately headless.
- `@csstools/css-tokenizer` 4.0.0:
   byte-perfect lossless round-trip on the
   full corpus,
   zero parse errors,
   comments preserved as tokens.
   Chosen as the foundation;
   css-edit supplies the CSS Syntax section 5
   structure layer over it.

Conclusion recorded for posterity:
as of mid-2026 no established pure-JS CSS parser is simultaneously
lossless,
 nesting-correct,
 maintained,
 and able to emit CSS;
the spec-exact tokenizer exists but full-stylesheet parsing was ceded to
postcss by the ecosystem.

### What changed in build-tool-css

- Public API redesigned to a deep-module surface:
  `buildCss({ input, output })` and `expandCssMixins({ css, mixinCss? })`
  plus `UnknownCssMixinError`/`CircularCssMixinError`.
- Mixin engine:
   memoized definition-chain recursion with an explicit trail
  (exact cycle reporting) replaced the ten-pass fixed-point loop with full
  serialization comparison;
   immutable css-edit nodes splice by reference,
  no cloning.
- `@import` specifiers now come from parsed tokens,
  fixing the old string-slicing corruption on
  `@import url('x.css') layer(base);`.
- `process-shim.ts` deleted;
   css-edit references no process globals.
- Browser consumers import
  `@monochromatic-dev/css-cli/ts/expand` directly;
  the package index re-exports the node-only file pipeline whose
  `module-fs-path` imports would drag node builtins into client bundles
  (found live in agent-browser when done-postcss components stopped
  registering).

### Performance (mitata, 2026-07-22)

- Raw parse+stringify on a 94 KB synthetic sheet:
  postcss 1.13x faster than css-edit;
  css-tree 9.5x slower than postcss.
  Decomposition:
   the spec tokenizer alone costs more than postcss's whole
  parse (2.86 ms vs 2.59 ms on 66 KB);
  the css-edit structure layer adds about 1 ms and stringify 0.8 ms.
  Spec-exact tokenization with eagerly parsed token data is the price of
  the byte-fidelity and correctness guarantees.
- Mixin pipeline (the workload that matters):
  `expandCssMixins` 1.72x faster than a faithful replica of the retired
  postcss clone-and-fixed-point pipeline,
   with about 40 percent less
  allocation (`css-edit.bench` `bench:pipeline`).

### Verification and guardrails

- Sidecars:
   `css-edit.fuzz`
  (byte round-trip,
   totality,
   structural sharing,
   postcss differential
  oracle;
   clean at 5000 runs),
  `css-edit.bench`,
   `css-edit.conformance`
  (curated css-parsing-tests-style corpus with fast-check context
  amplification).
- Repo idiom for `.conformance` sidecars is undefined across the family;
  tracked in issue #398.

## Current state (as of 2026-06-04, superseded above for the parser layer)

LightningCSS has been removed entirely.
 The build no longer
depends on LightningCSS or oxc-resolver;
 neither appears in
any `package.json` or in the lockfile.

- CSS build package lives at `package/css/cli/`
  (`@monochromatic-dev/css-cli`).
- Pipeline is pure JS,
   PostCSS only,
   no native binary:
   a
  custom PostCSS plugin (`src/import.ts`) inlines `@import`
  with an in-house `node_modules` resolver
  (`src/package-resolver.ts`),
   then `src/mixin.ts` collects
  `@mixin` definitions,
   expands nested mixin bodies,
   and
  inlines `@apply`.
- `src/index.ts` states the design:
   "Uses only PostCSS (pure
  JS):
   no native binary dependencies.
  "
- The build pipeline works:
   import resolution,
   mixin
  collection,
   nested mixin expansion,
   and `@apply` inlining
  all pass unit tests
  (`mise run //package/css/cli:buildAndTest`).
- The two earlier blockers no longer apply:
   the LightningCSS
  `customAtRules`/`var()` defect (#1081) is irrelevant
  because LightningCSS is gone,
   and the pnpm-isolated-mode
  symlink loss for oxc-resolver (Bug 3) is moot because the
  CSS build no longer depends on oxc-resolver.

Why LightningCSS is out of scope going forward,
 and the
closed tracking issue (#147),
 are recorded in
[.out-of-scope/lightningcss.md](../../.out-of-scope/lightningcss.md).

### Import resolution coverage in integration tests

Two distinct CSS import resolution strategies are exercised:

1. **`exports` field resolution** (`test-css-importing` +
   `test-css-imported`):
    imports like
   `@import '@monochromatic-dev/test-css-imported/index.css'`,
   resolved via the `exports` mappings in the imported
   package's `package.json`.
    Modern,
    encapsulated approach
   where the package controls its public surface.
2. **Direct file-path resolution**
   (`test-css-importing-filepath` +
   `test-css-imported-no-exports`):
    imports like
   `@import '@monochromatic-dev/test-css-imported-no-exports/src/index.css'`,
   resolved by reaching directly into the package's file
   tree.
    Requires the imported package to **not** have an
   `exports` field,
    because `exports` blocks deep imports by
   design (Node.
   js resolution semantics,
    enforced by
   oxc-resolver).

Key discovery:
 when a package has an `exports` field,
oxc-resolver correctly refuses to resolve paths not listed in
the exports map.
 A package specifier like
`pkg/src/index.css` fails with `"./src/index.css" is not
exported` when the package only exports `./index.css`.
Testing both strategies requires two separate imported
fixture packages:
 one with `exports` and one without.

## What does not work (across all bugs above)

- "Use SCSS":
   works but discards CSS-native customisation and
  forces a separate preprocessor.
- "Wait for native CSS `@mixin`":
   the
  [CSS Functions and Mixins Module](https://drafts.csswg.org/css-mixins-1/)
  is at FPWD (May 2025).
   `@function` ships in browsers;
  `@mixin`/`@apply` do not.
   Spec expected to land 2028-2030.
- "Just write the expanded CSS everywhere":
   defeats the
  reason for adopting `@mixin`/`@apply`.

## Why we do not file these upstream (combined)

- LightningCSS #1081 already exists;
   we no longer track it.
  LightningCSS has been removed,
   so its bugs no longer affect
  this workspace
  ([.out-of-scope/lightningcss.md](../../.out-of-scope/lightningcss.md)).
- LightningCSS `bundle()` lack of node-resolution is by
  scope;
   not a defect to file.
- pnpm isolated-mode symlink loss needs a minimal repro
  before it can be filed;
   not constructed yet.
- Vite/Astro pipeline gaps no longer apply (Vite is not a
  candidate).

Decision:
 no new upstream reports from us.
 The in-house
build pipeline is the working answer.

## Lessons (kept for posterity)

1. Every abstraction has leaky edges;
    LightningCSS is fast
   until it intersects with `var()`.
2. Package managers are not interchangeable;
    pnpm's
   strictness creates real failures that npm/yarn don't.
3. Frameworks own their pipelines;
    trying to retrofit a
   transform onto a framework's CSS path is not reliable.
4. "Just works" never does at the boundary between two
   tools.
5. The JavaScript ecosystem's modules-and-CSS interface is
   loosely defined;
    every tool makes assumptions that
   conflict with adjacent tools.

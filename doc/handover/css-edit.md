# css-edit: replace postcss in build-tool-css with an in-house CST over `@csstools/css-tokenizer`

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/done-postcss` is now `package/done/variant-postcss`;
> `package/build-tool/css` is now `package/css/cli`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`;
> `package/module/css-edit` is now `package/css/core-edit`;
> `@monochromatic-dev/module-css-edit` is now `@monochromatic-dev/css-core-edit`.

Status:
 in progress.
This document is the durable record for the migration;
update it after each landmark (decision,
 commit,
 verification result).

## Task

User request:
 migrate `package/build-tool/css` off postcss.
Original phrasing named css-tree as the target;
a grilling session (2026-07-22) replaced that target after empirical probes.

## Requirements (confirmed with user)

- Cure four named postcss pains:
  unguarded `process.env` reach-in (forces `process-shim.ts`),
  stringly-typed at-rule params,
  mutable OO AST feel,
  and dependency hygiene.
- Preserve current `@mixin`/`@apply` semantics and monorepo-aware `@import` inlining.
- Keep the pipeline pure JS and browser-compatible
  (`applyMixins()` runs in-browser in `package/webapp-productivity/done-postcss/src/client/css.ts`).
- Full sibling rigor:
   `css-edit.fuzz`,
   `css-edit.bench`,
   and `css-edit.conformance` packages,
  mirroring the `jsonc-edit` family.

## Decision: hybrid CST layer, not an off-the-shelf parser

New package `package/module/css-edit` (`@monochromatic-dev/module-css-edit`)
implements the stylesheet-structure layer (CSS Syntax section 5 consume-block-contents)
over `@csstools/css-tokenizer` (4.0.0,
 zero deps,
 TS-native,
 MIT,
 csstools team).
`build-tool-css` then consumes css-edit and drops postcss.

## Candidate survey evidence (probes run 2026-07-22, scratch dir `~/temp/agent/css-tree-probe-2026-07-22`)

All candidates were probed with the same corpus:
comment,
 `@mixin` body containing declarations plus relaxed nesting plus `&` nesting plus `@apply`,
`@media` range syntax,
 braces and semicolons inside strings and `url()`,
escaped selectors,
 custom-property block values.

- `css-tree` 3.2.1 (original migration target):
   rejected.
  Corrupts relaxed nesting (nested `button:disabled {}` without `&` becomes Raw plus parse error);
  this syntax exists today in `package/webapp-productivity/done-postcss/src/client/mixins.css`
  (`@mixin --shadow-dom-globals`).
  Unknown at-rule blocks parse in rule-list mode,
   mangling `@mixin` bodies,
  fixable only via `fork()` with a custom `atrule` parser config (verified working).
  Drops all comments (its `Block.js` skips comment tokens),
   generate() is minified-only,
  and `@types/css-tree` (2.3.11) lags css-tree by a major version.
- `@adobe/css-tools` 4.5.0:
   runner-up.
  Passed the full corpus including relaxed nesting and comment preservation;
  zero deps,
   TS-native types,
   no `process` references.
  Rejected because custom-property block values (`--raw: { nested: token };`) throw,
  its scanner is regex-era (two patched ReDoS CVEs in 2023),
  and string preludes keep the stringly-params pain.
- `@projectwallace/css-parser` 0.18.1:
   disqualified.
  Arena-based analyzer with no generate/stringify export;
   cannot emit CSS.
- `@stacksjs/ts-css` 0.1.1:
   disqualified.
  Silently deleted a nested rule from a `@mixin` body and ejected `@apply` to top level;
  week-old 0.x,
   Bun-first (conflicts with cross-runtime rule XRT).
- `@csstools/css-parser-algorithms` 4.0.0:
   component-value level only,
  no stylesheet or rule parser;
   not usable alone,
   informs the hybrid design.
- `@csstools/css-tokenizer` 4.0.0:
   chosen foundation.
  Byte-perfect lossless round-trip on the full corpus including every adversarial case,
  zero parse errors,
   comments preserved as tokens,
   `stringify` re-emits tokens exactly.
- Keeping postcss:
   viable null option,
   cures none of the four pains.
  postcss remains in the repo regardless (stylelint config plus catalog override in
  `pnpm-workspace.yaml`),
   so this migration only removes it from `build-tool-css`.

## Design decisions

- API follows the `jsonc-edit` family convention (verified in
  `package/module/jsonc-edit/src/edit-state.ts`):
  immutable state handle,
   every edit returns fresh state,
   structural sharing,
  `kind`-discriminated readonly plain-object nodes.
- Node kinds:
   stylesheet,
   at-rule (name,
   prelude token slice,
   optional block),
  rule (prelude token slice,
   block),
   raw declaration runs;
  whitespace and comments preserved in place as token slices.
  No selector or value parsing:
   preludes and declaration values stay token arrays.
- Strict parsing:
   positioned custom error on tokenizer parse errors.
- Stringify is token re-emission:
   byte-identical output for untouched regions.
- `build-tool-css` mixin redesign:
   the fixed-point ten-pass loop with string comparison
  in `mixin-registry.ts` is replaced by visited-set recursive expansion;
  circular references report the exact cycle.
  The module-level mutable `mixins` Map becomes a registry value threaded through functions.
  Public API changes are fine (unpublished package,
   design change per HON).
- `process-shim.ts` is deleted along with its `sideEffects` entry and export
  (existed solely for postcss;
   verify no importer remains,
   including `package-paused/`).
- Output behavior change:
   author formatting and comments survive byte-exactly outside
  spliced regions;
   spliced mixin bodies keep definition-site indentation.

## Rejected ideas

- Pure in-house tokenizer:
   re-implements what csstools maintains;
   only buys zero deps.
- css-tree plus authoring convention (`&`-only nesting):
   pays fork config,
  comment loss,
   minified output,
   and stale types to cure two pains.

## Open questions

- None blocking.
   Deferred:
   none (conformance sibling was added back by user).
- Conformance data source:
   tabatkins css-parsing-tests (used by tinycss2).

## Plan and progress

1. [x] Grilling session:
        target,
        placement,
        API model,
        rigor,
        sequencing confirmed.
2. [x] This handover doc.
3. [x] `@csstools/css-tokenizer` already in pnpm catalog (line found during work;
        no edit needed).
4. [x] Scaffold `package/module/css-edit` (mirrors `jsonc-edit`;
        needed `rolldown.browser.config.ts`).
5. [x] Implement parse (strict,
        spec section 5 unified block contents),
        stringify,
        transform.
6. [x] Unit tests per branch plus adversarial corpus;
        all green against built dist.
7. [x] Port `build-tool-css`:
        user authorized a full API redesign mid-task
   ("current api ... laughably awkward";
        migrate all consumers).
   New deep-module surface:
        `buildCss({ input, output })` file pipeline and
   `expandCssMixins({ css, mixinCss? })` text pipeline plus
   `UnknownCssMixinError`/`CircularCssMixinError`.
   Old `build`/`applyMixins`/`collectMixins`/`expandMixinBodies`/`mixins` Map
   protocol deleted;
        one shared apply-splicing visitor;
        visited-set cycle
   detection with exact trail;
        token-based import specifier extraction
   (fixes the old `url() layer()` string-slicing bug);
        `process-shim.ts`,
   `apply-mixins.ts`,
        `mixin-registry.ts`,
        `stripImportSpecifier` deleted.
   Consumers migrated:
        done-postcss server + client,
        paused messages-demo and
   inference-canary-viewer.
8. [x] Integration fixtures green (both resolution strategies);
        CLI exercised on
   the real done-postcss stylesheet (comments preserved,
        no residue).
9. [x] Browser verification via agent-browser,
        then closed:
        found and fixed a
   real regression (client importing the package index pulled node builtins
   into the bundle;
        browser consumers must import
   `build-tool-css/ts/expand` directly,
        recorded in the index TSDoc and README).
   Shadow DOM styles expanded in live browser,
        no console errors.
10. [x] `css-edit.fuzz`:
         round-trip,
         totality,
         structural-sharing properties plus
    postcss differential oracle;
         clean at 5000 runs.
         `fuzz:coverage`
    reachability gate added 2026-07-22:
         deterministic driver reaches every
    function in every runtime source file (baseline frozen and check-verified).
    While mirroring it,
         found and fixed the jsonc-edit gate dead since the
    `packages/` to `package/` rename (stale `SOURCE_MARKER`);
         both reports now
    throw when the projection matches zero files.
11. [x] `css-edit.bench` (mitata,
         per user direction):
         raw parse+stringify vs
    postcss and css-tree,
         plus `bench:pipeline` vs a replica of the retired
    postcss mixin pipeline.
12. [x] `css-edit.conformance`:
         curated css-parsing-tests-style corpus (30
    valid,
         10 invalid) with fast-check context amplification (per user
    direction).
         Repo idiom for `.conformance` sidecars is undefined;
         filed
    issue #398.
13. [x] Docs:
         css-tooling.md dated survey and migration section;
         READMEs for
    css-edit,
         build-tool-css,
         and all three sidecars.
14. [x] Probe scratch dir cleaned.

## Performance findings (2026-07-22, mitata)

- Raw parse+stringify,
   94 KB sheet:
   postcss 1.13x faster than css-edit.
  Decomposition:
   the spec tokenizer alone (2.86 ms) exceeds postcss's whole
  parse (2.59 ms) on 66 KB;
   css-edit's own structure layer adds ~1 ms,
  stringify 0.81 ms after the flat-accumulator rewrite.
  Future optimization target:
   the upstream tokenizer's per-token tuple plus
  data-object allocation,
   or index-range nodes instead of token slices.
- Mixin pipeline:
   `expandCssMixins` 1.72x faster than the retired postcss
  clone-and-fixed-point design,
   ~40 percent less allocation.

## Implementation notes discovered while building

- Repo lint regime shaped the parser:
   scan cursors live in named IIFEs
  (`no-function-root-let`),
   absence models as discriminated unions or empty
  arrays (`no-nullish-union`),
   and the `@csstools/css-tokenizer` guards are
  catalogued as audited pure reads in
  `package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/csstools-css-tokenizer-package-effect-catalog.ts`
  (shipped dist sha256 recorded there;
   unit test beside the other catalog tests).
- Visitor removal semantics:
   empty-array result removes;
   with
  `pruneTriviaBeforeRemoved` the preceding trivia run keeps comments and loses
  only whitespace after the last comment,
   matching postcss removal behavior.
- CDO/CDC are single tokens;
   prose between `<!--` and `-->` is NOT a comment
  and tokenizes as idents (caught by an initially wrong test corpus).
- When postcss leaves `build-tool-css`,
   check whether
  `postcss-package-effect-catalog.ts` still has consumers before retiring it.

## Commits

- `d18af81a1` docs:
   this handover.
- `feat(module-css-edit)`:
   CST implementation.
- `805e84287` fix/feat:
   lint remediation + tokenizer effect catalog.
- `test(module-css-edit)`:
   unit suites;
   catalog test passing.

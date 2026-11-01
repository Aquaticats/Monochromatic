# Learning Rust formatting boundary investigation

Status:
proposal,
not an accepted decision.

This document investigates
[Aquaticat/Monochromatic issue 401](https://github.com/Aquaticat/Monochromatic/issues/401).
It supersedes the earlier exclusion-first recommendation.
No production source or configuration change is authorized yet.

## Decision requested

Adopt a phased combined direction:

1. Keep all 5 learning HTML pages owned by dprint.
2. Add an opt-in prose-preserving mode to the `markup_fmt` plugin used by dprint.
3. Replace Malva at the CSS host seam with repository-owned CSS formatting.
4. Add a package-local checker for the exact learning CSS contract.
5. Keep Stylelint during a shadow migration of its remaining responsibilities and editor diagnostics.
6. Remove Stylelint only after the responsibility,
   fixture,
   suppression,
   CLI,
   and editor gates pass.

Superseded 2026-10-04:
 Stylelint was removed outright (`doc/decision/remove-stylelint.md`)
 before the shadow migration ran,
 so items 5 and 6 are moot and editor diagnostics no longer exist for CSS.
The replacement CSS linter is tracked in issue #587.
Everything below is retained as a probe record,
 not as instructions still to run.

This direction meets the owner's formatter-trust requirement and the intended Stylelint retirement without
pretending the current probes already reproduce all Stylelint behavior.

## Owner constraints

- Do not exclude or ignore the learning HTML in dprint.
- Keep the CSS between 5 and 20 nonblank lines.
- Add no presentation rules beyond the approved foreground and background colors.
- Keep each page build-free and usable through `file:///`.
- Repeat the CSS inline in every page.
- Prose may reflow,
  but no formatter may split a tight grammatical unit.
- Use these direct `oklch()` forms in the affected CSS:
  - lightness as `<number>`;
  - zero chroma as `none`,
    otherwise `<number>`;
  - zero hue as `none`,
    otherwise `<number>deg`.
- Keep the CSS policy package-local.
  Do not add an `AGENTS.md` rule.
- Build repository-owned CSS tooling rather than selecting another external formatter or linter.
- Treat replacing dprint as one option family,
  not the assumed destination.

An unacceptable prose break is:

```text
I ate
a chicken.
```

A list-like break is acceptable:

```text
I ate a chicken,
a pig,
and a cow.
```

## Affected files and current contract gaps

The affected pages are:

- `package/learning/rust/lessons/index.html`
- `package/learning/rust/lessons/0001-whats-different-about-this-book.html`
- `package/learning/rust/reference/index.html`
- `package/learning/rust/reference/reading-loop.html`
- `package/learning/rust/reference/aquascope-decoder.html`

All 5 pages contained byte-identical style content at measured commit `68eb96063`.
Their current 7-nonblank-line styles satisfy the line budget,
but use `0` instead of the preferred `none` zero channels.

`package/learning/rust/NOTES.md` requires:

```html
<meta name="color-scheme" content="light dark">
```

Only `reference/aquascope-decoder.html` currently contains that element.
The other 4 pages have an independent package-content defect.

## Verified incumbent behavior

### Dprint and embedded CSS

The measured toolchain is:

- dprint 0.55.2;
- `markup_fmt` 0.23.1;
- Malva 0.14.1.

Pinned `markup_fmt` source always calls `format_style` for nonempty `<style>` content in
`markup_fmt/src/printer.rs:701-737` at
[commit `2d902a5a1`](https://github.com/g-plane/markup_fmt/commit/2d902a5a1af8d7f07e06f189ae1e3132ea344e74).
Its dprint adapter sends that content back to the host under a synthetic `index.html#.css` path in
`dprint_plugin/src/lib.rs:54-90`.

Dprint therefore already exposes the correct seam:
markup owns HTML structure,
while a CSS plugin owns style content.
The issue is the behavior of the adapters at that seam,
not an inability to separate the languages.

At width 90,
the incumbent markup formatter introduced breaks inside grammatical units.
Width probes produced these newly introduced text-node break contexts:

- width 160:
  94;
- width 240:
  30;
- width 320:
  11;
- width 500:
  3;
- width 600 on the current corpus:
  0.

Width 600 and 1000 produced the same current-corpus output,
with lines up to 593 characters.
Adversarial fixtures still split `original variable`,
`allocator deallocates`,
and `you reach`.
High width is therefore a bridge,
not grammatical ownership.

### Dprint scope

The measured dprint inventory contained 775 paths,
including 724 JSON,
TOML,
YAML,
XML,
or SVG paths unrelated to issue 401.
Replacing dprint across the repository is not justified by these 5 HTML files.

### Stylelint scope

The measured Stylelint stack is:

- Stylelint 17.14.1;
- `stylelint-config-standard` 40.0.0;
- `postcss-html` 1.8.1.

Stylelint discovered 51 CSS or HTML paths.
Root ignores removed 32 paths.
The active surface was 19 files containing 14 CSS regions.

The merged configuration enabled 82 rules:

- 34 advertised fixes;
- 48 did not.

The repository-wide run reported 185 diagnostics from 8 rule names:

- `at-rule-empty-line-before`:
  10;
- `declaration-block-single-line-max-declarations`:
  5;
- `declaration-empty-line-before`:
  5;
- `function-disallowed-list`:
  16;
- `hue-degree-notation`:
  10;
- `lightness-notation`:
  10;
- `property-disallowed-list`:
  62;
- `unit-disallowed-list`:
  67.

Current diagnostics do not define the retirement surface.
All 82 enabled rules need an explicit outcome.

### Stylelint configuration defect

`package/config/stylelint/index.mjs` intends to require `rem` in every media-feature dimension.
Its key `'/[\w-]+/'` is a JavaScript string,
so Stylelint receives `/[w-]+/`.

An executable probe accepted `@media (height: 10em)` while rejecting `@media (width: 10em)`.
The latter only matches accidentally because `width` contains `w`.
The repository-owned checker correctly rejects both and accepts `height: 10rem`.

### Dependency footprint

The Stylelint,
standard-config,
and HTML-syntax closure contains:

- 126 package identities;
- 2,507 files;
- 10,721,885 bytes in the installed tree.

A lockfile traversal starts from all 148 pnpm importers except the Stylelint configuration importer and
Stylelint-only root dependencies.
It reaches 604 snapshots with no unresolved non-workspace roots.
Peer-context variants count as shared,
which makes the result conservative.

Of the 126 Stylelint-closure identities:

- 16 are reachable elsewhere;
- 110 are exclusive;
- the exclusive installed footprint is 2,024 files and 7,845,733 bytes.

The existing `package/css/core-edit/` foundation uses one zero-dependency tokenizer package,
`@csstools/css-tokenizer`.

## Verified combined prototype

### Prose-preserving markup

A disposable `markup_fmt` patch added opt-in `preserveTextWrapping` behavior.
It:

- keeps authored nonempty text lines as formatter boundaries;
- normalizes repeated intra-line ASCII whitespace;
- continues formatting document structure,
  attributes,
  quotes,
  and embedded languages;
- leaves the default behavior unchanged when disabled.

The default-off upstream library tests passed.
A local dprint Wasm build succeeded.

Across all 5 pages,
the combined run produced:

- no dprint exclusion or ignore directive;
- no introduced text-node break context;
- original and formatted break counts of 0,
  16,
  0,
  0,
  and 259;
- 16 nonblank CSS lines per page;
- no change on a second run.

The grammar fixture kept `I ate a chicken`,
`original variable`,
`allocator deallocates`,
and `you reach` intact at width 90 when each phrase was authored on one line.

This is a feature direction,
not an established upstream defect.
No upstream issue has been filed.

### Repository-owned CSS formatter

The second disposable formatter uses `package/css/core-edit/` for strict CSS structure and preserves surrounding
HTML through region offsets.
Its external interface should remain deep:

```ts
formatCssSource({ source, profile }): string;
checkCssSource({ source, path, profile }): readonly CssDiagnostic[];
```

Callers do not need to know the CST,
tokenizer,
HTML extraction,
normalization passes,
or rule data.

The dprint-discovered tracked corpus contained:

- 36 standalone CSS files;
- 15 HTML files;
- 45 actual CSS regions.

Across all 45 regions,
the formatter reported:

- no parse failure;
- no second-pass difference;
- no semantic-token mismatch,
  ignoring optional trailing semicolons;
- no comment-token change;
- no host prefix or suffix change.

Fixtures exercised:

- compact nested learning CSS;
- block and directive comments;
- blank-line groups;
- strings containing CSS punctuation;
- selector lists and combinators;
- nested selectors;
- block and statement at-rules;
- declaration spacing;
- malformed strings,
  comments,
  braces,
  and rules.

The formatter establishes structural feasibility.
It does not yet implement all 27 formatting responsibilities in the retirement ledger.

### Dprint formatting adapter

A disposable `dprint-plugin-exec` 0.7.3 adapter reads CSS from stdin and writes formatted CSS to stdout.
The markup adapter must not pass Malva-only override keys to exec.
After removing those invalid keys,
dprint formatted standalone and embedded CSS and reached a second-pass fixed point.

A JSON-RPC probe launched `dprint lsp` 0.55.2 with unsaved editor buffers.
It returned formatting edits for both standalone CSS and HTML with embedded CSS,
with no stderr.
Open
[`dprint-plugin-exec#34`](https://github.com/dprint/dprint-plugin-exec/issues/34)
does not reproduce with the tested versions and configuration.

A 5-run benchmark used `--incremental=false` on the 5 pages:

- prose-preserving markup alone:
  344.2 ms mean;
- markup plus exec CSS delegation:
  451.2 ms mean;
- measured mean difference:
  107.0 ms per invocation.

Exec is a valid transition adapter.
A dedicated persistent dprint process adapter ranks above it for the final implementation because it can avoid a
child-command boundary per request and own cache invalidation directly.

### Failure behavior

Malformed standalone CSS and malformed embedded CSS both made dprint fail with the tokenizer diagnostic.
SHA-256 hashes before and after the failed run were identical for both files.
The production adapter should convert parser exceptions into concise file and range diagnostics while preserving
this no-write behavior.

### Repository-owned policy checker

The policy probe reproduced the incumbent semantic diagnostic totals:

- disallowed functions:
  16;
- disallowed properties:
  62;
- disallowed units:
  67.

It also reported:

- the missed `height: 10em` media case;
- 20 preferred `oklch()` channel changes on the current learning pages;
- exact source offsets for every diagnostic.

An adversarial HTML fixture contained false `<style>` text in a comment and an attribute.
`@lezer/html` found only the real `StyleText` region.
The checker mapped `width`,
`10px`,
and `rgb(` to exact host-file slices and line and column positions.
All diagnostics across the 14 active regions had valid nonempty ranges.

### Package-local learning contract

A package profile compares the CSS token structure and values against the one approved stylesheet and separately
checks the 5-to-20 nonblank-line budget.

The approved owned-formatter output has 16 nonblank lines.
Fixtures established independent checks:

- compact but semantically approved CSS fails only the line budget;
- an added `font-size` declaration passes the line budget but fails approved structure;
- a changed dark color passes the line budget but fails approved values;
- a 24-line semantic equivalent fails only the line budget.

After applying the preferred `none` channels in the disposable worktree,
all 5 pages passed both checks with one style region and 16 nonblank lines.
The current committed 7-line CSS passes the budget but fails the preferred-value check.

### Color semantics and browser baseline

[CSS Color 4 section 4.4](https://www.w3.org/TR/css-color-4/#missing)
says `none` behaves as zero outside interpolation.
During interpolation,
a missing component may borrow the corresponding component from the other color.
The preferred direct color forms must not be described as universally interchangeable with zero.

A Chromium 149 canvas probe rendered direct missing and zero lightness,
chroma,
and hue comparisons to equal RGBA bytes.
[Mozilla bug 1813481](https://bugzilla.mozilla.org/show_bug.cgi?id=1813481)
records missing color components as fixed in Firefox 113,
which predates the repository's Firefox ESR 140 baseline.

### Editor diagnostics

Dprint provides formatting edits,
not semantic diagnostics.
The repository currently recommends:

- `stylelint.vscode-stylelint` in `monochromatic.code-workspace`;
- the IntelliJ Stylelint plugin in `.idea/externalDependencies.xml`.

The repository also contains relevant owned precedents:

- a full-buffer stdio diagnostics server in `package/linter/rust/src/lsp.rs`;
- JetBrains LSP4IJ settings support under `package/file-enforcer/core/src/jetbrains/`.

The CSS checker should publish diagnostics from unsaved CSS and HTML buffers through a standard LSP adapter while
the CLI and editor share the same pure checking interface.
Stylelint remains necessary during shadow migration unless equivalent VS Code and IntelliJ clients land.

## Complete Stylelint responsibility ledger

The ledger assigns every active rule exactly once.
It has no missing,
unknown,
or duplicate names.
All 82 pinned upstream rule directories contain tests.
45 implementations import CSS reference data or a dedicated grammar or selector parser.
Those data obligations remain real even when the owned checker interface is narrow.

### Formatter responsibilities

These 27 rules select canonical,
idempotent source representation.
The formatter should leave no diagnostic behind after it runs.

- `alpha-value-notation`
- `at-rule-empty-line-before`
- `color-function-alias-notation`
- `color-function-notation`
- `color-hex-length`
- `comment-empty-line-before`
- `custom-property-empty-line-before`
- `declaration-block-single-line-max-declarations`
- `declaration-empty-line-before`
- `font-family-name-quotes`
- `font-weight-notation`
- `function-calc-no-unspaced-operator`
- `function-name-case`
- `function-url-quotes`
- `hue-degree-notation`
- `import-notation`
- `keyframe-selector-notation`
- `lightness-notation`
- `media-feature-range-notation`
- `no-irregular-whitespace`
- `rule-empty-line-before`
- `selector-attribute-quotes`
- `selector-not-notation`
- `selector-pseudo-element-colon-notation`
- `selector-type-case`
- `shorthand-property-no-redundant-values`
- `value-keyword-case`

The structural probe covers layout,
comments,
selectors,
declarations,
and whitespace.
Notation-specific normalization still needs repository fixtures before Stylelint retirement.

### Repository-policy responsibilities

These 16 rules express repository choices rather than CSS validity.
They belong in typed profiles and should report unsafe semantic changes rather than rewrite them.

- `at-rule-disallowed-list`
- `at-rule-no-vendor-prefix`
- `color-named`
- `container-name-pattern`
- `function-disallowed-list`
- `keyframe-declaration-no-important`
- `layer-name-pattern`
- `media-feature-name-disallowed-list`
- `media-feature-name-no-vendor-prefix`
- `media-feature-name-unit-allowed-list`
- `number-max-precision`
- `property-disallowed-list`
- `property-no-vendor-prefix`
- `selector-no-vendor-prefix`
- `unit-disallowed-list`
- `value-no-vendor-prefix`

The probe currently exercises function,
property,
unit,
media-unit,
lightness,
and hue policy behavior.
`number-max-precision` moved out of the formatter category because rounding can change values.

### Correctness-checker responsibilities

These 38 rules report malformed,
duplicate,
unknown,
deprecated,
or contradictory CSS without rewriting author intent.

- `annotation-no-unknown`
- `at-rule-descriptor-no-unknown`
- `at-rule-descriptor-value-no-unknown`
- `at-rule-no-deprecated`
- `at-rule-no-unknown`
- `at-rule-prelude-no-invalid`
- `block-no-empty`
- `block-no-redundant-nested-style-rules`
- `comment-no-empty`
- `custom-property-no-missing-var-function`
- `declaration-block-no-duplicate-custom-properties`
- `declaration-block-no-duplicate-properties`
- `declaration-block-no-shorthand-property-overrides`
- `declaration-property-value-keyword-no-deprecated`
- `font-family-no-duplicate-names`
- `font-family-no-missing-generic-family-keyword`
- `keyframe-block-no-duplicate-selectors`
- `media-feature-name-no-unknown`
- `media-feature-name-value-no-unknown`
- `media-query-no-invalid`
- `media-type-no-deprecated`
- `named-grid-areas-no-invalid`
- `nesting-selector-no-missing-scoping-root`
- `no-duplicate-at-import-rules`
- `no-duplicate-selectors`
- `no-empty-source`
- `no-invalid-double-slash-comments`
- `no-invalid-position-at-import-rule`
- `no-invalid-position-declaration`
- `property-no-deprecated`
- `property-no-unknown`
- `selector-anb-no-unmatchable`
- `selector-pseudo-class-no-unknown`
- `selector-pseudo-element-no-unknown`
- `selector-type-no-unknown`
- `string-no-newline`
- `syntax-string-no-invalid`
- `unit-no-unknown`

Unknown and deprecated vocabulary rules need an explicit maintained reference source.
Parser strictness alone does not replace them.

### Deliberate drop

- `no-descending-specificity`

Do not reproduce this selector-order heuristic.
The active tree already suppresses it for context-dependent selectors,
and the paused tree contains further suppressions.
Its false-positive and maintenance surface does not justify repository ownership.
This is a deliberate policy change,
not accidental missing parity.

## Suppression migration

The repository contains 18 actual `stylelint-disable` directives:

- 3 in the active tree;
- 15 under `package-paused/`;
- no matching `stylelint-enable` directives.

Do not create a parallel `css-check-disable` comment language.
Use typed exemption data with:

- file path;
- owned rule identifier;
- file or stable subject scope;
- mandatory rationale;
- unused-exemption reporting.

The 3 active migrations are:

- `doc/secret-management-caveman.html`:
  file-scoped function,
  property,
  and unit policy exemptions;
- `package/dependency-catalog/cli-cube/src/styles.css`:
  the same 3 file-scoped policy exemptions;
  discard its specificity exemption because that rule is dropped;
- `package/done/variant-postcss/src/client/styles.css`:
  a subject-scoped duplicate-selector exemption for the intentionally repeated `:root` grouping.

Paused packages stay outside discovery as they do today.
When one resumes,
remove each stale Stylelint directive and adjudicate the diagnostics.
Do not carry `copypaste` or `Cannot fix` forward as accepted rationale.

## Migration phases

### Phase 1: learning package seam

After owner acceptance:

1. Land opt-in prose-preserving markup behavior with its default disabled.
2. Route CSS host requests to the repository-owned formatter.
3. Add the package-local exact stylesheet and line-budget checks.
4. Change the affected colors and `NOTES.md` to preferred `none` channels.
5. Add the missing color-scheme meta element to the other 4 pages.
6. Add a transitional Stylelint override that converges with the owned output.
7. Keep Stylelint's remaining semantic checks and editor plugins active.

A disposable transition config already reached a fixed point:
Stylelint check passed,
Stylelint fix made no change,
and the next owned dprint check passed.

### Phase 2: repository shadow parity

Implement and fixture the remaining formatter,
policy,
and checker responsibilities.
Run owned diagnostics beside Stylelint and compare:

- file discovery;
- diagnostic identity;
- host ranges;
- configuration overrides;
- malformed input;
- unused exemptions;
- check and fix idempotence.

Every mismatch needs either an owned fix or a documented deliberate drop.
Current diagnostics alone are not a parity oracle.

### Phase 3: editor and dependency cutover

Before removing Stylelint:

1. Publish owned diagnostics for unsaved CSS and HTML buffers.
2. Wire the server into the supported VS Code and IntelliJ workflows.
3. Remove or replace all active Stylelint directives.
4. Remove Stylelint formatting from the root format task.
5. Remove Stylelint linting,
   configuration,
   dependencies,
   and editor recommendations.
6. Re-measure discovery and installed dependency footprint.

## Coverage by decision layer

### Symptom layer

The immediate symptoms are prose reflow,
CSS layout conflict,
40 learning-page diagnostics,
and missing meta elements.
High width and scoped incumbent configuration can reduce symptoms,
but do not own grammar or retire Stylelint.

### File layer

The formatter owns each HTML file and embedded style region without ignore directives.
The package checker independently validates style-region count,
approved tokens,
and the line budget.

### Package layer

A learning profile owns the exact stylesheet contract and color notation.
It remains separate from repository-wide CSS policy and adds no `AGENTS.md` rule.

### Repository layer

Dprint keeps discovery and formatting ownership.
Typed profiles and exemptions replace regex-shaped string configuration and tool-specific disable comments.
Shadow comparison prevents silent rule loss.

### Architecture layer

Pure formatter and checker interfaces sit behind CSS,
HTML,
dprint,
CLI,
and LSP adapters.
The dprint process adapter and diagnostic language server are separate because formatting and live diagnostics
have different host protocols.

## Option analysis and ranking

### Phased owned CSS with prose-preserving markup

Pros:

- preserves dprint ownership of every affected HTML file;
- passes current and adversarial prose fixtures;
- uses the existing embedded-language seam;
- gives the learning package exact policy now;
- retains Stylelint coverage while owned parity grows;
- provides a controlled path to dependency removal.

Cons:

- carries 2 lint systems during shadow migration;
- needs a dprint adapter and diagnostics clients;
- does not remove the Stylelint footprint in phase 1.

### Incumbent CSS bridge with prose-preserving markup

Keep Malva and Stylelint temporarily,
using the verified scoped values that meet the line budget.

Pros:

- preserves trusted HTML formatting;
- retains incumbent semantic and editor coverage;
- has a verified 7-line or 20-line CSS fixed point.

Cons:

- leaves the measured Stylelint dependency closure;
- generic Stylelint rules cannot require `none` for zero channels;
- retains the regex-string configuration hazard;
- does not satisfy the intended owned-CSS destination.

### Immediate full owned replacement

Remove Malva and Stylelint in one cutover.

Pros:

- reaches the desired ownership and dependency result immediately;
- has one formatter and one checker interface.

Cons:

- 75 ledger responsibilities still lack repository-owned implementation fixtures;
- 45 rules expose reference-data or grammar-parser ownership;
- equivalent editor diagnostics are not wired;
- removing the incumbents now would discard verified coverage.

### Owned CSS with high-width markup

Pros:

- uses the validated CSS direction;
- avoids changing `markup_fmt` source.

Cons:

- width 600 produces lines up to 593 characters;
- adversarial prose still splits grammatical units;
- it passes only the current corpus.

### Generated learning HTML or synchronized style regions

Pros:

- can keep one authoritative stylesheet;
- can preserve reader-time `file:///` use;
- confines generation to the learning package.

Cons:

- creates authored and generated source layers;
- conflicts with the package's repeated-inline authoring contract;
- adds author-time generation to a workspace documented as build-free;
- does not itself provide repository-wide CSS lint retirement.

### Full dprint replacement

Pros:

- could own prose and embedded CSS under one repository interface.

Cons:

- issue 401 supplies evidence for only 5 HTML files;
- 724 measured unrelated data or XML-family paths would need replacement coverage;
- duplicates working formatter ownership outside the affected seam.

### Status quo

Pros:

- requires no migration.

Cons:

- retains destructive prose formatting;
- retains 40 learning-page Stylelint diagnostics;
- leaves preferred channel syntax unenforced;
- leaves 4 required meta elements absent.

### Sorted ranking

Ranking:
phased owned CSS with prose-preserving markup
> incumbent CSS bridge with prose-preserving markup
> immediate full owned replacement
> owned CSS with high-width markup
> generated learning HTML
> full dprint replacement
> status quo.

- The phased direction ranks above the incumbent bridge because it preserves incumbent coverage while making
  measurable progress toward the accepted owned-CSS destination.
- The bridge ranks above immediate replacement because 75 owned responsibility fixtures and editor clients are
  still pending.
- Immediate replacement ranks above high width once its gates pass because it owns grammatical prose rather than
  relying on corpus-specific width.
- High width ranks above generation because it retains one authored HTML source layer.
- Generation ranks above full dprint replacement because its scope is confined to the learning package.
- Full dprint replacement ranks above status quo because it could solve the conflict,
  while status quo cannot.

Dprint exclusions,
file ignores,
and node ignores are disqualified rather than ranked.
They surrender formatter ownership and violate the owner's trust requirement.

## Verification gates after acceptance

### Formatter

- Run default-off `markup_fmt` upstream tests.
- Test preserved authored lines,
  repeated spaces,
  entities,
  inline elements,
  preformatted elements,
  malformed markup,
  and every supported markup language.
- Run all 5 learning pages twice through dprint.
- Confirm no introduced grammatical break context.
- Confirm 5-to-20 nonblank CSS lines.
- Confirm comments,
  strings,
  nested rules,
  selectors,
  declarations,
  and at-rules reach a second-pass fixed point.
- Confirm malformed standalone and embedded CSS never writes partial output.
- Verify formatting through the actual editor integration.

### Checker

- Test every exported branch and every enabled responsibility.
- Verify CSS and HTML host offsets with non-ASCII text before the style region.
- Verify parser failures become diagnostics at valid ranges.
- Verify check mode never writes.
- Verify typed exemptions are scoped,
  justified,
  and reported when unused.
- Compare shadow results against Stylelint until each mismatch is resolved or documented.

### Learning package

- Confirm exactly one real style region per page.
- Confirm approved selectors,
  declarations,
  values,
  and media conditions.
- Confirm the 5-to-20 nonblank-line budget independently.
- Confirm all pages contain the color-scheme meta element.
- Load the pages through `file:///` in light and dark modes.
- Confirm computed foreground and background colors.
- Confirm every relative link still works.

### Dependency and editor cutover

- Verify owned formatting in dprint CLI and LSP.
- Verify owned diagnostics in unsaved CSS and HTML buffers.
- Verify VS Code and IntelliJ launch the repository-owned checker.
- Remove Stylelint only after the owned ledger gate passes.
- Re-run lockfile reachability and installed-file measurements after removal.

## Open decisions

The owner still needs to accept or delegate:

- the phased combined direction;
- a local plugin patch versus an upstream `preserveTextWrapping` proposal;
- whether editor diagnostic parity is required in phase 1 or only before Stylelint removal.

No production implementation should begin until that decision is explicit.

# AUDIT: regex necessity

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/ide` is now `package/ide/app`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/deepmerge-ts.fuzz` is now `package/upstream-fork/deepmerge-ts.fuzz`;
> `package/module/conf-fork.fuzz` is now `package/upstream-fork/conf.fuzz`;
> `package/module/test-expect` is now `package/test-harness/core-expect`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/wg-quicker` is now `package/wg/cli-quicker`.

Repo-wide audit of regex use against one criterion,
 with one GitHub issue filed per unnecessary site.

Tracker entry points:
 issue #680 is the index of every site issue,
 and issue #679 tracks the gate coverage that let most sites through unreviewed.

## Criterion

A regex site is unnecessary when the feature it implements can be written without regex
 at similar or better time complexity.
Code length,
 familiarity,
 and "the pattern is safe" do not enter the test:
 a bounded,
 backtracking-free pattern that a string API answers directly is still unnecessary.

Two classes survive the criterion,
 and neither is filed:

- The feature is regex semantics over an arbitrary pattern somebody else supplies,
  such as a user-typed `/.../` filter in `deps-cube`,
  a user CLI pattern in `watch-restart`,
  or the `RegExp` branch of a Jest-compatible `toMatch`.
  This is a contract exception rather than a computational impossibility:
  a general regex engine could be written,
  but writing one is not "the same feature at similar complexity".
- The value must be a `RegExp` because an API,
  a type,
  or a test subject requires that type:
  `expect.stringMatching(pattern: RegExp)`,
  upstream pify's `Array<string | RegExp>` include list,
  `regExpOrThrow` inputs,
  deepmerge leaf values,
  the `no-regex` rule's own fixtures,
  and the `module/es` regexp tooling constructors.

A repo-owned annotation is not an external contract.
`readonly RegExp[]` in this repo's own source records a choice,
 so it does not by itself make a site necessary;
 the audit asks what the consumer requires,
 which is why `pi-plugin/auto-mode`'s secret patterns are filed as replaceable predicates.

## Method

Enumeration had to see justified sites too,
 so suppression directives could not simply be honoured.

1. Mirror every `.ts`,
   `.tsx`,
   `.mts`,
   `.cts`,
   `.js`,
   `.jsx`,
   `.mjs`,
   and `.cjs` file in the working tree into a scratch tree
   (`build-mirror.mjs`,
   6996 files),
   rewriting `oxlint-disable` to `oxlint-xisable` and `oxlint-enable` to `oxlint-xnable`.
   Both replacements are the same byte length,
   so line and column positions are unchanged.
2. Run the repository's own `no-restricted-syntax/no-regex` rule over the mirror with a
   scratch config whose `ignorePatterns` is empty and whose only enabled rule is `no-regex`.
   The detector is therefore the parser-backed rule the gate uses,
   not a text search.
3. Map each diagnostic back to the original path and slice the exact source text on UTF-8
   byte offsets.
   oxlint spans are byte offsets,
   so slicing code units drifts on any file containing non-ASCII;
   an early version of this harness had that bug and mis-sliced every pattern in
   `package/music-player/design/settings-pane.mjs`.
4. Dedupe on `(offset, length)`.
   Chained calls such as `md.replaceAll(/a/, '').replaceAll(/b/, '')` report one diagnostic
   per call with the same start offset and different lengths,
   and the regex belonging to a given call is the last literal inside its span.
   Keying on line and column instead collapsed 14 distinct sites into 4 and would have
   published duplicate issue bodies.
5. Recompute directive ranges from the original text
   (block `disable` to matching `enable` or end of file,
   plus `disable-next-line` and `disable-line`),
   so each site is attributed to the justification that actually covers it.
6. Classify necessity,
   derive the replacement shape from a tokenizer over the pattern body
   (literals,
   character classes,
   quantifiers,
   groups,
   anchors,
   boundaries,
   lookbehind),
   and resolve the owning package from the nearest `mise.toml` and `package.json`.
7. Cover the surfaces a JavaScript rule cannot see:
   extract every `<script>` block from tracked HTML,
   Vue,
   and Astro files into mirror `.js` files and scan them with the same rule
   (`extract-html.mjs`,
   102 blocks from 162 files),
   and enumerate Kotlin and inline YAML script strings by text search with each hit read
   individually.

## Results

1631 unique JS and TS regex sites,
 plus 28 on surfaces the rule cannot reach,
 of which 10 are excluded on provenance and 18 are filed.

- **298 unnecessary sites in tracked first-party code**,
  one issue each:
  239 in active trees and 59 in `package-paused/`.
  By surface:
  280 JS and TS,
  12 inside HTML `<script>` blocks,
  5 Kotlin,
  1 inline YAML script.
- **105 of the 298 carry a disable justification**,
  drawn from 40 distinct justification texts.
  Each of those issues quotes the justification and answers it.
  The other 193 carry none,
  because no gate could ask for one.
- **116 necessary sites** under the criterion,
  each with a recorded reason and no issue.
  Two sites were moved out of the necessary set on individual review,
  which is why the count is not the 118 a directory-prefix exemption produced:
  the metacharacter escape pass inside `module/es`'s `toRegExp`
  (the constructor beside it stays necessary),
  and the string branch of `toMatch` in
  `package/module/test-expect/src/expect-matchers-core.ts:294`.
- **1069 sites in untracked files**:
  gitignored local artifacts,
  dominated by `src/canary-lint/<model>/<probe>/canary.ts`,
  which `package-paused/dev-script/inference-canary/README.md` documents as generated model
  outputs,
  gitignored but preserved locally.
  Editing them would falsify a measurement record.
- **165 sites in `package/module/deepmerge-ts.fuzz/src/declared-type-soundness.generated.ts`**:
  generated corpus data marked "Do not edit",
  where each `new RegExp("x", "gu")` is a leaf value the merge must handle.
  Excluded as generated and classified necessary.
- **1 site in `package/module/conf-fork.fuzz/upstream-conf/index.ts`**:
  a vendored upstream snapshot under `@ts-nocheck`,
  whose value is that it matches upstream.
- **The directories the mirror skipped** (`bundle`,
  `dist`,
  `.pi`,
  plus tool caches) hold 9 tracked JS and TS files,
  all committed build output under `package/claude-code-plugin/*/bundle/node/`,
  and 2 tracked files under `.pi/` that are neither JS nor TS.
  Each of the 9 was checked directly for `RegExp(` and `new RegExp(` and contains
  neither,
  which is consistent with their sources:
  the audit found zero regex sites anywhere in `package/claude-code-plugin`,
  because the earlier sweep replaced them with text-scan helpers.
  Recorded as a verified decision rather than an oversight.

- **10 sites excluded on provenance**,
  found by tracing generators and directory ownership rather than by a filename marker:
  9 in `package/music-player/design/questions/template-editor.html`,
  an 18.6 MB file that `package/music-player/design/template-editor.mjs:121` writes from
  `template-editor.template.html`
  (whose own 9 sites are filed),
  and 1 in `package/music-player/design/uploads/music-player-mockup.html`,
  which `package/music-player/design/HANDOFF.md:3208` describes as part of
  "the user's own mockup,
  screenshots and reference images".
  Both were filed in the first round and closed afterwards:
  the generated ones as duplicates of the template issues,
  the upload as not planned.
  No other file in the filed population carries a generated marker in its first 1200
  bytes,
  and `template-editor.html` carries none at all,
  which is why provenance tracing rather than marker matching decided it.

Replacement shapes across the 298,
 derived from the pattern tokenizer:
 field extractions between literal delimiters,
 literal-plus-character-run scans,
 ordered literal and class sequences,
 alternations over a fixed candidate set,
 literal prefix tests,
 word-boundary substring tests,
 literal substring tests,
 runtime-compiled patterns,
 fixed-count character runs,
 small token grammars,
 literal global replacements,
 anchored literals,
 and lookbehind escape-parity scans.
Each issue states the shape it derived,
 and states that the shape is a proposal from the pattern syntax rather than a verified
 equivalent,
 with differential fixtures as an acceptance criterion.

## Measured refutations of recorded justifications

The audit did not accept a disable justification as evidence of necessity.
Where a justification made a checkable claim,
 the claim was checked,
 and two early probes were rebuilt after they turned out not to exercise the case they
 claimed to test.
All timings are Node v26.11.1 on this machine,
 three runs per input size,
 reported as doubling series rather than absolute claims.

- `package/typeface/aquaticat/src/parse-svg.ts:150` claims the element tokenizer is
  "Linear:
  every char is visited at most twice across the alternation".
  On a synthetic adversarial input,
  `"<rect "` repeated with no `/>` terminator anywhere,
  one `exec` call takes 9.3 ms at 15996 characters,
  37 ms at 31998,
  148 ms at 63996,
  and 589 ms at 127998:
  about four times per doubling,
  so quadratic,
  with roughly 3% spread.
  The proposed cursor replacement on the same input takes 0.1 ms,
  0 ms,
  0.1 ms,
  and 0.2 ms.
  This is a constructed worst case,
  not an observed Figma truncation.
- `package/oxlint-plugin/no-restricted-syntax/src/rule/prefer-describe-function-ref-name.ts`
  has two measured findings.
  Its re-export matcher `export\s*\{[^}]*\bNAME\b[^}]*\}\s*from\s*['"]` takes 10.4 ms over
  15 KB,
  28.7 ms over 30 KB,
  118 ms over 60 KB,
  and 460 ms over 120 KB of many unterminated `export {` openers,
  about four times per doubling,
  because each opener rescans the brace-free remainder.
  An earlier probe put the filler before a single opener and measured 0.16 ms,
  which is linear by construction and proved nothing;
  the corrected input is the one quoted here.
  Its `u`-flag claim was probed against all four production patterns and six identifiers
  with a positive control confirming six matching combinations:
  no result differed between the flagged and unflagged forms,
  so the honest statement is that no difference was observed for parser-produced
  identifiers,
  not that the claim is disproved for every possible input.
  The same probe surfaced a separate defect:
  an identifier containing `$`,
  which is a legal identifier character and an interpolated anchor,
  never matches in either mode.
- `package/cli/wg-quicker/src/config-parse-values.ts:128` calls `/^[0-9.]+$/u` an
  "Anchored IPv4 token classification".
  The function documents its own contract as "True when token is digits/dots or contains
  IPv6 colon",
  and the code matches that contract,
  so the justification mislabels a lexical classifier as an IP validator.
  Measured,
  the pattern accepts `...`,
  `.`,
  `1..2`,
  `1.2.3.4.5`,
  and `999.999.999.999`,
  and rejects `dns.google`.
  No octet-validation change is proposed;
  that would be a behaviour change,
  and this audit only asks for the regex to go.
- `package/ssg/aquati.cat/src/lib/git-dates.ts:199` parses `owner/repo` with
  `[^/]+\/[^/.]+`.
  The `[^/.]` class excludes dots from repository names,
  so measured results are `null` for `git@github.com:owner/repo.js.git`,
  `https://github.com/owner/dot.repo`,
  and `git@github.com:owner/repo.name.git`,
  while `gh api repos/socketio/socket.io` confirms dotted repository names are real.
  The traced consequence:
  `getGithubSlug` feeds `resolveGitDatesContext().githubSlug`,
  which `git-dates.ts:446` documents as required when shallow and `git-dates.ts:358`
  interpolates into `repos/<slug>/commits?path=...`,
  so a dotted-name remote leaves the shallow-clone date path without a slug.
  Fixing that is a behaviour change and is kept separate from the regex removal.
- `package-paused/webapp-forge/server/src/storage/write-buffer.unit.test.ts:107` states
  "expect API requires a RegExp".
  The site uses `.toThrow`,
  whose signature is
  `toThrow: (expected?: string | RegExp | (abstract new(...args: never) => unknown),) => void`
  (`package/module/test-expect/src/expect-matchers.ts:91-93`),
  and whose implementation passes the value straight to chai's `.throw`
  (`expect-matchers-core.ts:131-146`),
  where a string is a message-inclusion check.
  So the string form removes the regex from the feature entirely.
  `.toMatch` is different:
  its string form compiles `new RegExp(expected)` at `expect-matchers-core.ts:294`,
  which is why that site is filed as a separate finding about the string branch.
- Seven rolldown configs justify `/^node:/u` and `/^electron(\/|$)/u` as anchored tests
  the bundler needs.
  Rolldown's `external` accepts a string or a predicate:
  `type ExternalOption = StringOrRegExp | StringOrRegExp[] | ExternalOptionFunction` and
  `type ExternalOptionFunction = (id, parentId, isResolved) => NullValue<boolean>`
  (`node_modules/rolldown/dist/shared/define-config-CGhm2DU-.d.mts:3415-3417`).
  The predicate replaces the whole option value rather than one array entry,
  so the seven sites plus `anchoredExternal` and `escapePackageNameForRegExp` in
  `package/config/rolldown/src/package-externals.ts` must migrate as one change,
  and each issue in that group says so.
  **Correction (2026-10-17):**
  this bullet read the TypeScript declaration surface and stopped there.
  Tracing rolldown's Rust source at tag `v1.2.13` shows the `RegExp` entries never
  execute as JavaScript:
  the napi boundary extracts `source` and `flags` once at config time and compiles them
  into Rust's `regex` crate engine
  (a finite automaton,
  linear time by construction,
  backtracking impossible),
  matched natively per specifier inside the parallel module scan,
  while the predicate form costs one napi `ThreadsafeFunction` round trip to the single
  JS main thread per specifier check.
  Measured on the installed rolldown 1.2.13:
  byte-identical output from both forms,
  about 8 microseconds per predicate check,
  and a factor of about 1.4 on total build time at 2000 modules.
  Plain strings are not an alternative either:
  the string arm of the matcher is exact equality,
  so it cannot express the `(/|$)` prefix.
  The criterion's "similar or better time complexity" compared pattern shapes without
  modelling where each form executes.
  The group's issues
  (#726,
  #817 through #822,
  #834 through #837,
  #846 through #848)
  were closed as wontfix;
  evidence,
  citations,
  and both harnesses:
  `doc/troubleshooting/rolldown-external-regex-native-rust-matching.md`.
- `package-paused/module/es/.../t typeof/.../index.ts:135` and `:138` claim
  "no string API exposes the Unicode upper-letter set".
  Case mapping is a string API carrying the same Unicode data.
  Measured over 15 samples,
  the uppercase comparison showed no divergence from case mapping,
  and the lowercase comparison diverged on U+1D2C and U+1D43 (category Lm).
  A 15-sample probe cannot establish that those are the only divergences,
  so both sites are filed as judgement calls:
  an exact property-table lookup is a regex-free implementation at the same complexity,
  and which semantics the classifier wants is an owner decision the justification does not
  state.
- Two cost claims were tested and **rejected**,
  and no issue rests on them.
  Runtime compilation is not a win:
  1000 iterations of `new RegExp(source)` plus a test took 32.6 ms against 32.3 ms for a
  literal,
  and 5000 took 168.1 ms against 160.4 ms,
  inside run-to-run spread.
  The markdown-stream fence pattern is not quadratic either:
  a lazy `A[\s\S]*?A` with the global flag consumes through each terminator,
  so a single trailing unmatched opener costs one extra full scan
  (0.1 ms over 128 KB) and many matched pairs cost 0.4 ms over 128 KB.
  An earlier probe described its input as unmatched openers when it was adjacent pairs,
  and that claim is withdrawn.
  Both sites are filed on necessity alone.

## Gate coverage

192 of the 298 unnecessary sites are invisible to every gate,
 and a further 193rd carries no justification but is one the gate does report.
Issue #679 carries the full breakdown and the deciding evidence;
 in summary:
 163 JS and TS sites in packages with no `lint:oxlint` task,
 which the `fanout_packages` variable in `mise.toml` cannot reach because it builds its
 target list from the tasks that exist;
 10 in `doc/` scripts no package task covers;
 1 in a `.cjs` file that `package/config/oxlint/src/config-base.ts:63` excludes through
 `ignorePatterns`;
 and 18 on surfaces a JavaScript rule cannot express
 (12 HTML script blocks,
 5 Kotlin,
 1 inline YAML script).
No CI workflow runs a workspace-wide lint,
 so there is no second net.

One further site is reached and is red:
 `package/git-policy/cli/bin/build-git-test-image.mjs:61`,
 where `mise run //package/git-policy/cli:lint:oxlint` reports 75 errors and 65 warnings
 (measured 2026-10-12),
 one of them that `no-regex` error,
 before aborting on an unrelated tsgo panic inside
 `plugin-prefer-readonly-parameter-type`.

`doc/handover/no-regex.md` records the sweep as complete and green for every non-paused
package.
That was written at its own revision and is not called false here.
What is measurable now is drift plus a scope gap:
 the cross-check could only have covered packages that define an oxlint task,
 which is narrower than "every non-paused package".

## Unresolved classification

One site is recorded as unresolved rather than necessary or unnecessary:
 `package/desktop-app/ide/src/find.rs:170` escapes a user's find text and builds a
 case-insensitive Unicode matcher with the `regex` crate.
The feature is case-insensitive literal search with Unicode case folding.
A folded literal search with source-offset accounting is a plausible regex-free
 implementation at comparable complexity,
 and the code already bounds query length,
 so the asymptotic argument for keeping the crate is not settled.
Resolving it needs the crate's exact folding semantics,
 the matcher's non-overlap and truncation rules,
 and a real literal-search algorithm rather than a naive comparison at every position.
No issue is filed for it until that work is done.

## Surfaces checked and found empty or out of scope

Completeness needs the negative results too,
 each with the search that produced it.

- **Dot-directories the mirror skipped**:
  `.agents`,
  `.cargo`,
  `.changeset`,
  `.claude-plugin`,
  `.github`,
  `.idea`,
  `.opencodereview`,
  `.out-of-scope`,
  `.pi`,
  and `.serena` hold exactly one tracked JS or TS file between them,
  `.changeset/changelog.ts`,
  which contains no regex.
- **Shell-tool regex**:
  every tracked hit for `grep -E`,
  `sed -E`,
  `sed -r`,
  `path_regexp`,
  or `header_regexp` is inside a `doc/runbook/`,
  `doc/troubleshooting/`,
  `doc/handover/`,
  or `doc/decision/` file,
  as a command a human runs.
  Those are an external tool's own interface in a documented manual step,
  not repo code,
  and replacing them would mean telling the reader to write a program instead of a
  command.
  Out of scope by decision.
- **`mise.toml` task bodies and `Caddyfile`**:
  no `grep -E`,
  `sed -E`,
  `=~`,
  or Caddy regexp matcher.
- **Other declarative surfaces**:
  Slint,
  CSS,
  and Dockerfile files carry no regex constructs.
- **`.pnpmfile.mjs` and the root `playwright.*.config.ts`,
  `file-enforcer.config.ts`,
  `oxlint.config.ts`**:
  inside the mirrored tree,
  so they were scanned with everything else.

## Relationship to the earlier sweep

`doc/handover/no-regex.md` records the sweep that cleared the gate's findings and paused
 three packages.
 `doc/audit/regex-replacement-perf.md` audited the performance of the parsers that sweep
 introduced,
 and listed "regex deliberately kept with a justified disable" as out of scope.
This audit covers exactly that excluded set plus every site no gate can see,
 so the two documents are complementary rather than overlapping.

## Reproducing

The harness is committed beside this document in
[`regex-necessity/`](regex-necessity/),
 so the classification is reproducible from the repository rather than from one machine's
 scratch directory:

- `build-mirror.mjs` mirrors the tree with directives neutralised at equal byte length.
- `scan.config.json` enables only `no-restricted-syntax/no-regex` with empty
  `ignorePatterns`.
- `sites3.mjs` dedupes spans,
  slices patterns on byte offsets,
  and attributes directives.
- `extract-html.mjs` extracts HTML script blocks and enumerates the YAML surface.
- `pipeline.mjs` classifies necessity,
  resolves scopes,
  and finds enclosing declarations.
- `pattern-analysis.mjs` tokenizes pattern bodies and derives replacement shapes.
- `rebuttals.mjs` holds the per-justification answers.
- `supplement.mjs` adds the Kotlin,
  HTML,
  and YAML sites and the two reclassifications.
- `fix-html-patterns.mjs` resolves each HTML script-block site to its own regex
  literal,
  slicing on byte offsets.
- `generate.mjs` writes one issue payload per site,
  disambiguating titles when one source line carries several distinct sites.
- `reconcile.mjs` corrects already-filed issues in place and files the rest.
- `repair-labels.mjs`,
  `repair-titles.mjs`,
  and `close-excluded.mjs` are the correction passes:
  labels after a call that removed and added in one request,
  titles and bodies after post-filing corrections,
  and closure of the sites excluded on provenance.
- `verify.mjs` reconciles the ledger against the tracker.
- `evidence.mjs` and `evidence2.mjs` hold the measurements,
  with `evidence2.mjs` superseding the two mis-constructed probes in `evidence.mjs`.

Run order:
 `build-mirror.mjs`,
 the oxlint scan into `scan.json`,
 `sites3.mjs`,
 `extract-html.mjs` and its scan,
 `pipeline.mjs`,
 `supplement.mjs`,
 `generate.mjs`.
Timings quoted here are single-machine and are reported as doubling series with per-size
 repeats.

# Trailing whitespace enforcement layer

## Status

Recommendation written 2026-10-22,
answering the question the issue [#585][] triage raised:
"Should we enforce this in our git wrapper instead?"
Decided by the owner on 2026-10-22:
none of the options.
Trailing whitespace is not enforced,
`.editorconfig`'s trim rule stays inspiration,
and issue [#585][] is closed as not planned.
The record is `doc/decision/trailing-whitespace.md`.

## Question

`.editorconfig` promises `trim_trailing_whitespace = true` for every file type but `*.html`,
and no configured check enforces that promise for TypeScript.
Issue [#585][] proposes the fix:
a `stylistic/no-trailing-spaces` rule in `@monochromatic-dev/oxlint-plugin-stylistic`
with an autofix that leaves template literal text alone,
enabled in `package/config/oxlint/src/rule/style.ts`,
then one `format:oxlint` sweep.
The question is whether that enforcement belongs in the git wrapper
(`package/git-policy/cli`)
instead of the oxlint rule.

## Answer

No,
not instead.
The check is a literal-sensitive TypeScript judgment,
and the seam that can make it is the lint layer sitting on the TypeScript parser.
The wrapper's content-policy seam is exact bytes and path names,
so the wrapper can only delegate to the rule as a backstop,
never implement it.

## Measured state

Commands run on this worktree on 2026-10-22:

```sh
# Whitespace-only lines in tracked TypeScript.
git grep --count --extended-regexp '^[[:blank:]]+$' -- '*.ts' '*.tsx' '*.mts'
# Whitespace-only lines in every other tracked file kind.
git grep --count --extended-regexp '^[[:blank:]]+$'
```

- TypeScript holds 31,023 whitespace-only lines in 2,843 files.
  The same command with `[[:blank:]]+$`
  (any line ending in blanks)
  sums to the same 31,023,
  so every hit is a whitespace-only line.
  Issue [#585][] measured 30,859 lines in 2,800 files,
  and the debt grew slightly.
- `package/translation-repair/**` is clean,
  and its package-local scan keeps it clean.
- Markdown holds 26 such lines in 7 files,
  every one a blank context line inside a fenced unified diff
  (for example `doc/troubleshooting/xml-attribute-whitespace-serialization.md`).
- Patch attachments hold 248 such lines in 50 `*.patch` files,
  and unified diff format requires them,
  because a blank context line is one space.
- The vendored `package/pnpm-override/shim-katex/index.cjs` holds 2,
  which are upstream bytes.
- HTML holds 16 in 8 files,
  and `.editorconfig` unsets trimming for `*.html`,
  so those are out of scope by policy.
- One fuzz seed line under `package/`.
  Binary files matched by `git grep` are byte coincidences,
  not line findings.

So the enforceable debt is TypeScript,
and every non-TypeScript hit is content a trimmer must not touch.

## What the wrapper already enforces

The near-miss answer to "does the git wrapper already enforce trailing whitespace?"
is that it enforces final-newline hygiene,
which is about a file's last byte,
not about line-end whitespace.

- `final-newline` requires a non-empty UTF-8 text file without NUL bytes
  to end with exactly one LF byte,
  message "Non-empty text file must end with exactly one LF byte."
  (`package/git-policy/cli/src/native/policy_final_newline.rs`).
- Its normalizer rewrites only the run of trailing LF bytes at end of file,
  adds a missing one,
  and leaves interior bytes untouched
  (`normalized_final_newline` in `package/git-policy/cli/src/native/policy_final_newline.rs`).
  A file whose lines end in spaces before their LF passes this policy unchanged.
- Its preserved families are the two fuzz seed directories,
  `package/test-fixture/toml-edit/src/`,
  and files below `dist/final/node/` or `bundle/node/`
  (`is_final_newline_excluded` in the same module).
- The four optional content policies
  (`markdown/autofix`,
  `mono/forbidden-root-context`,
  `mono/dependent-version-bump`,
  `security/forbidden-strings`)
  never read line-end whitespace.
  `cli-git.config.jsonc` enables `markdown/autofix` for the `lfs-image-url` rule only.
- The wrapper's JavaScript import scanner drops trailing whitespace in memory
  while locating specifiers
  (`package/git-policy/cli/src/native/dependent_version_imports.rs`),
  which is parsing,
  not file enforcement.
- The active `.git/hooks` are Git LFS hooks only.
- `package/claude-code-plugin/bash-output-filter` removes trailing whitespace from
  terminal output lines,
  which is display hygiene,
  not file content.

So no layer trims or blocks line-end whitespace today,
and the policy whose name sounds closest
(`final-newline`)
is verified to leave it alone.

## Research coverage

Surfaces inspected for this recommendation on 2026-10-22:

- The issue [#585][] text and its comment thread.
- `.editorconfig`,
  `package/config/dprint/index.json`,
  and `monochromatic-lint.config.jsonc`.
- Every rule in `package/oxlint-plugin/stylistic/src/rule/`,
  the fixer API in `@oxlint/plugins`,
  and a probe of the oxlint 1.89.0 rules table
  (no trailing-whitespace rule exists upstream).
- The translation-repair scan and its fixtures
  (`package/translation-repair/source-scan/src/trailing-whitespace.unit.test.ts`).
- The wrapper's policy registry,
  dispatch,
  correction channel,
  management commands,
  config schema,
  `cli-git.config.jsonc`,
  `.git/hooks`,
  and the `normalized_final_newline` source.
- The CI workflow task inventory
  (`.github/workflows/`,
  `.depot/workflows/`),
  which runs no `lint:oxlint`.
- The monochromatic-lint rule inventory and its semantic-line-break source,
  used as the probe target for this doc's own rendered check.

Deeper digs available on request:
the removed incumbent TypeScript wrapper's history,
the exact fixable set of `monochromatic-lint --fix`,
and per-formatter trailing-whitespace behaviour of the active dprint plugins.

## Enforcement surfaces today

- `.editorconfig` sets `trim_trailing_whitespace = true` under `[*]`
  and unsets it only for `*.html`.
  An editor honouring it strips per line with no language awareness.
  That stripping is the diff noise the issue reports,
  and it also rewrites template literal text,
  where the whitespace is the value.
- `package/config/dprint/index.json` comments out the TypeScript plugin
  "to be replaced by our oxlint-stylistic",
  and comments out the Markdown plugin too.
  dprint still formats toml,
  html,
  css,
  yaml and
  json,
  so those formats lose trailing whitespace as a formatter side effect.
- `package/oxlint-plugin/stylistic` is
  "the active layout authority for TypeScript expressions"
  (`package/oxlint-plugin/stylistic/README.md`).
  Its 18 rules cover per-line layout and none reads end-of-line whitespace.
  Fixers see AST nodes and ranges
  (`context.report` with `fix(fixer)` in `package/oxlint-plugin/stylistic/src/rule/block-body-newline.ts`),
  and the plugin already skips protected regions in `semi.ts` and `utility/item-per-line-fix.ts`.
- oxlint 1.89.0 ships 814 core rules and none of them is a trailing-whitespace rule,
  so the project rule is necessary,
  not a configuration toggle.
- `package/linter/monochromatic-lint` owns the Rust and Markdown/MDX rules,
  and none covers trailing whitespace.
  Nothing covers Rust either,
  since the repository has no rustfmt configuration.
- `package/translation-repair/source-scan/src/trailing-whitespace.unit.test.ts`
  is the only guard that exists.
  It parses each file and exempts a trailing run that overlaps a `TemplateElement` text span,
  because a template's text is the only literal that can hold a line-end run.
  Its module doc states the same gap issue [#585][] reports.
- `package/git-policy/cli` runs content policies over exact would-be-committed
  and newly pushed bytes:
  `final-newline`,
  `markdown/autofix`,
  `mono/forbidden-root-context`,
  `mono/dependent-version-bump`,
  and `security/forbidden-strings`.
  `final-newline` is the byte-hygiene precedent,
  in `package/git-policy/cli/src/native/policy_final_newline.rs`.
  No shipped policy parses TypeScript or JavaScript.
- CI runs `mise run lint:monochromatic-lint` on changed `*.md`,
  `*.mdx` and
  `*.rs` files,
  and `mise run build` plus `mise run test:unit` in the baseline workflows.
  Nothing in CI runs `lint:oxlint`,
  `format:oxlint`,
  or dprint.

## Options

### A. Trailing-whitespace rule in oxlint-stylistic

The issue [#585][] proposal.
Implement `stylistic/no-trailing-spaces` with an autofix that leaves template literal text
alone,
enable it in `package/config/oxlint/src/rule/style.ts`,
run one `format:oxlint` sweep,
then retire `package/translation-repair/source-scan/src/trailing-whitespace.unit.test.ts`.

Pros:

- The judge and the fixer need template ranges,
  and the rule runs on the AST the stylistic plugin already visits.
  The `@oxlint/plugins` node union carries `TemplateElement` spans,
  and the translation-repair scan proves the range-masking approach on this repository's
  own fixtures.
- One fix path.
  `format:oxlint` already rewrites a package to a fixpoint,
  and the sweep is that task.
- The rule sits beside `require-asterisk-prefix`,
  the rule family whose TSDoc blank lines hold nearly all of the measured debt.
- Implementation is one rule file,
  one registration entry,
  and one severity line in `package/config/oxlint/src/rule/style.ts`.
- The duplicated package-local scan retires into the shared rule,
  which is the issue's own final step.

Cons:

- Enforced wherever `lint:oxlint` runs,
  not at the commit boundary,
  and nothing in CI runs `lint:oxlint` today.
- Covers TypeScript and TypeScript-adjacent files in the 204 packages that declare
  `lint:oxlint`,
  minus the fixtures and generated files `package/config/oxlint/src/config-base.ts` ignores.
  The `.editorconfig` promise is repo-wide.
- The sweep rewrites 2,843 files in one commit.

### B. Git wrapper content policy instead

Add a shipped policy to `package/git-policy/cli`,
beside `final-newline`,
that checks or rewrites would-be-committed bytes.

Pros:

- Runs at every commit and manual push without a lint invocation,
  so commits from any editor are covered.
  It is the only automatic gate that sees every staged byte today.
- Covers every file type at once,
  matching the `[*]` scope of `.editorconfig`.
- Precedent:
  `final-newline` is a byte-hygiene content policy with fix support,
  and the wrapper's `Correction` channel already carries full-content rewrites
  through the commit transaction into a private index.

Cons:

- Wrong seam for a literal-sensitive judgment.
  The policy interface is exact bytes and path names,
  and the requirement is precisely to leave template literal text alone.
  Meeting it needs a TypeScript parser behind that seam:
  either a second implementation in Rust,
  or a shelled-out fixer,
  which is the lint rule anyway.
  The wrapper's strongest existing JavaScript scan
  (`mono/dependent-version-bump`)
  is lexical quote scanning that treats a backtick as a plain quote
  (`package/git-policy/cli/src/native/dependent_version_imports.rs`),
  so no template-aware machinery exists there to reuse.
- Byte-level detection cannot separate noise from content.
  Template text,
  `package/pnpm-override/shim-katex/index.cjs`,
  `*.patch` attachments,
  fenced unified diffs in `doc/`,
  and fuzz seeds all hold legitimate line-end whitespace today,
  measured above.
- Recorded layering says local wrapper enforcement is best-effort fast feedback
  and CI is authoritative,
  and "Replacing CI as the authoritative enforcement layer" is a non-goal
  (`doc/decision/cli-git-policies-platform.md`).
  The wrapper gate also declines to absorb repository linters
  (`doc/handover/cli-git-native-policy-engine.md`).
- The 2026-10-04 Rust rewrite decision restricts the native wrapper to its shipped
  policy catalog,
  and names an expanded catalog out of scope
  (`doc/decision/cli-git-rust-rewrite.md`),
  so a new wrapper policy is itself a design decision,
  not a routine add.
- Cost:
  registry row and trigger entry,
  dispatch arm,
  config schema,
  JSONL event hint,
  unit,
  binary and
  config tests,
  a `cli.fuzz` target,
  per-format exemption families,
  and a crate version bump with release archives.
- Blast-radius precedent:
  `final-newline` needed ownership-grouped exclusions for 1,547 of 7,074 paths
  (`doc/planning/final-newline-normalization.md`),
  and this policy would meet a far larger legacy population.
- Two fix paths
  (`git cli-git fix` and `format:oxlint`)
  and two implementations of one rule to keep in sync.
- Before the sweep lands,
  every commit touching legacy files reports pre-existing lines.

### C. Option A plus a CI gate

Same rule and sweep,
plus a CI step running `lint:oxlint` on changed paths,
the shape `.github/workflows/advisory-core.yml` already uses for
`lint:monochromatic-lint`.

Pros:

- Makes "enforced" a hard gate in the layer the platform decision calls authoritative.
- One implementation of the rule,
  run in one more place.

Cons:

- Wiring `lint:oxlint` into CI gates every oxlint rule on every pull request,
  which is a repo-wide decision separable from issue [#585][].
- The plugin bundles must build in CI,
  and stale bundles already have history
  (`doc/handover/config-oxlint-stale-plugin-bundle-issue-570.md`).
- Any pre-existing lint finding in the tree turns CI red,
  so it needs a repo-wide clean-lint measurement first.

### D. Option A plus a delegating wrapper backstop

Same rule and sweep,
plus a thin wrapper policy that runs the same checker over candidate bytes,
the delegation shape `markdown/autofix` already uses for `monochromatic-lint --fix`.

Pros:

- Commit-boundary feedback without a second rule implementation.
- Add it later only if commit-time feedback proves missing.

Cons:

- Adds lint toolchain startup to every commit
  and reports findings lint already reports.
- Still needs exemption families for the content holders in the measured state.
- More surface than option C for less authority.

## Ranking

Ranking:
A > C > D > B.

- A over C:
  the question asks where the check lives,
  and option A answers that without folding in a repo-wide CI gate change
  with its own precondition
  (a clean `lint:oxlint` baseline)
  and blast radius.
  Record the gate as the follow-up decision,
  not as hidden scope.
- C over D:
  a CI gate runs in the recorded authoritative layer
  and reuses the rule unchanged,
  while a wrapper policy adds per-commit toolchain startup,
  duplicate findings
  and exemption families,
  and still sits in the best-effort layer.
- D over B:
  delegation keeps one implementation of the literal-safety logic,
  while a byte-level wrapper policy reimplements it
  and cannot tell the measured content holders from noise.

## Risks and limits

- `.editorconfig` trimming is not literal-aware.
  An editor honouring it strips template literal text on save,
  changing string values.
  The rule's autofix deliberately preserves that text,
  so the editor setting and the rule disagree.
  A follow-up option is to stop trimming `*.ts` in `.editorconfig`
  and let the literal-safe autofix own the cleanup,
  at the cost of no editor-side cleanup until the sweep lands.
- The sweep commit touches 2,843 files.
  Land it alone,
  after the rule is enabled and green on the packages already normalized,
  so the diff is reviewable as pure whitespace.
- The rule must keep the translation-repair exemption semantics:
  a trailing run is a finding unless a template element's text covers part of it.
  Ship adversarial fixtures for delimiters,
  escapes,
  substitutions,
  line continuations
  and an unterminated last line.
- "Enforced" means "wherever `lint:oxlint` runs" until a gate exists.
  If the requirement is a hard gate on every commit,
  option C is the piece to add,
  and it needs the clean-lint measurement named in its cons.

## Rejected

- A byte-level wrapper policy as the only enforcement
  (option B).
  It cannot honour the template-literal exemption,
  it flags the measured content holders,
  and it moves a style check out of the styling stack
  into the best-effort commit layer.
- Waiting for editors alone.
  `.editorconfig` strips only the files a save touches,
  which is the diff-noise cause the issue reports.
- A rule from upstream oxlint.
  The 1.89.0 rules table has no trailing-whitespace rule to enable.

## Outcome

The owner chose not to enforce trailing whitespace on 2026-10-22,
so nothing here is implemented.
The decision,
its rejected options and its accepted limits live in
`doc/decision/trailing-whitespace.md`,
and issue [#585][] is closed as not planned.

[#585]: https://github.com/Aquaticats/Monochromatic/issues/585

# Audit: fallow.tools against this monorepo

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

Evaluation of [fallow.tools](https://docs.fallow.tools),
 an MIT-licensed
codebase-intelligence CLI for TypeScript and JavaScript that surfaces unused code,
duplication,
 complexity,
 and architecture issues.
The fallow CLI ships under the `fallow` npm package.
Run as a multi-pass experiment on `main` (commit `c3e12b9d`) on 2026-04-28
across four invocation modes (default,
 with `fallow init` config,
with a hand-tuned `.fallowrc.json`,
 and with a file-enforcer-generated
`.fallowrc.json` from the prototype on branch `fallow-wrap-prototype`).

## TL;DR

Fallow is worth adopting locally as a periodic audit tool,
 **not** as an
auto-cleanup or always-on linter.

- Bare `bunx fallow` against this monorepo reports **2344 dead-code issues** and
  **235 clone groups** (7.8% duplication).
   First impression is overwhelmingly noisy.
- The default config emitted by `fallow init` (workspace pattern + `unused-dependencies: warn`)
  does not change those numbers.
- A hand-tuned `.fallowrc.json` drops the count to **869** (-63%).
- A file-enforcer-generated config that dynamically discovers tsdown
  config entries drops the count further to **744** (-68% from default).
- The remaining 744 findings include real signal (cross-package clones,
  dead deps,
   unlisted deps,
   unused class members) plus a known false-positive
  tail driven by per-package `mise.toml` task invocations and bare
  `tsdown.config.ts` files where fallow's built-in plugin does not activate.
  The bare-config category was eliminated on 2026-05-09 (every config now
  uses the `tsdown.<platform>.config.ts` form),
   so a re-run would surface
  fewer false positives in that band.
- The unique signal is genuinely out-of-scope for oxlint by design:
  oxc maintainers explicitly declined `import/no-unresolved` as inherently
  noisy and have `import/no-unused-modules` and `import/no-extraneous-dependencies`
  as low-priority TODOs (umbrella issue oxc-project/oxc#1117).
- Without CI or PR review in this repo,
   integration is purely local:
  a `mise` task that devs run before committing,
   with the current 744 findings
  baselined as accepted so future runs only surface new issues.

## What was tested

Four invocation modes:

1. **Bare default**:
    `bunx fallow` with no config
2. **`fallow init` default**:
    ran `bunx fallow init`,
    used the auto-generated config
3. **Hand-tuned `.fallowrc.json`**:
    a manual config that adds entry-point patterns
   for this repo's conventions (`mise.*.ts`,
    `tsdown.*.config.ts`,
    editord client
   modules,
    test-fixture overrides) and disables rules that overlap with oxlint
4. **file-enforcer-generated config**:
    the prototype on branch `fallow-wrap-prototype`,
   where `file-enforcer.config.ts` adds a `generateFallowConfig()` step that
   globs `package/*/*/tsdown.*.config.ts`,
    extracts each config's `entry` array
   (literal arrays via regex,
    plus base-config re-exports inferred from the
   imported filename),
    and merges discovered entries into the static base

Each pass was captured under `/tmp/fallow-out/`.
 The fallow source was
cloned at `/tmp/fallow-clone/fallow` and inspected for plugin behavior:
specifically `crates/core/src/plugins/tsdown.rs`,
 which constrains the
built-in tsdown plugin to match the bare `tsdown.config.{ts,js,cjs,mjs}`
filename and not the variant filenames this repo uses.

The tool installed cleanly via `bunx` after setting `BUN_TMPDIR` and `BUN_INSTALL`.
A persistent warning across all runs flagged a glob containing `${{`
(an unparsed GitHub Actions expression in some workflow file the entry-point
detector reads as TypeScript).
 Harmless but noted.

## Headline numbers

Across the four invocation modes:

- **Bare default**:
   2344 dead-code issues,
   235 clone groups (7.8% duplication),
  5347 functions analyzed,
   383 above the default complexity thresholds
  (cyclomatic > 20,
   cognitive > 15,
   CRAP >= 30.0),
   MI 85.9,
   241 entry points
  detected (145 from `package.json`,
   96 from plugins).
- **`fallow init` default config**:
   identical to bare.
   The default config is
  too minimal to affect anything;
   it sets `unused-dependencies` to `warn`
  and declares `package/*/*` as the workspace pattern,
   both of which fallow
  already infers.
- **Hand-tuned `.fallowrc.json`**:
   869 dead-code issues (-63%),
   232 clone groups
  (8.9% duplication),
   5313 functions,
   376 above threshold,
   MI 91.4,
  564 entry points (279 dynamically loaded,
   145 `package.json`,
   96 plugin,
  44 manual entry).
- **file-enforcer-generated config (prototype)**:
   744 dead-code issues
  (-68% from default;
   -14% from hand-tuned),
   232 clone groups,
   5315 functions,
  377 above threshold,
   MI 91.8,
   569 entry points (264 dynamically loaded,
  123 `package.json`,
   96 plugin,
   86 manual entry).

The prototype's improvement over the hand-tuned config came from three sources:

- Disabling `unresolved-imports` at the rule level (matches oxc's stance
  that the rule is inherently noisy):
   dropped 47 false positives
- Adding `unused-files: 'off'` to the override for `package/module/es/src/types/**`:
  dropped 56 false positives in the deeply nested type-system barrel files
- Dynamic discovery of tsdown config entries;
   caught secondary entries like
  `package/claude-code-plugin/bash-output-filter/src/filter.ts` that the
  built-in tsdown plugin missed,
   plus `**/src/cli.ts` as a static entry pattern
  for CLI scripts

## Real findings worth acting on

### Cross-package code clones

- `package/cli/mvm/src/index.ts:22-70` and `package/cli/vmsync/src/index.ts:20-67`
  share 49 lines of bootstrap logic
- `package/cli/mvm/src/spawn.ts:1-45` and `package/cli/vmsync/src/spawn.ts:7-51`
  share a 45-line spawn helper
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-hasownproperty.ts`,
  `no-promise-catch.ts`,
   and `no-promise-finally.ts` share 23 lines of rule scaffolding
- `package/desktop-daemon/editord/src/client/highlight/tags.ts:56-147` and
  `package/ssg/aquati.cat/src/client/tags.ts:35-126`
  duplicate 92 lines of highlight tag wiring

### Dead dependencies

- `the-new-css-reset` declared in `package/stylesheet/monochromatic/package.json` but unused
- `rehype-parse`,
   `rehype-stringify`,
   `unified`,
   `p-limit` declared in packages
  that do not import them but are used in **other** packages,
   suggesting
  the declarations belong with those consumers
- `@mitata/counters` unused in `package/test-fixture/file-enforcer-perf`

### Unlisted dependencies

A handful of packages imported but not declared in any `package.json`,
including `tailwindcss`.
These resolve today via root hoisting or transitive dependencies.

(The original 2026-04-28 audit listed eighteen packages including the now-removed
ESLint and Vite toolchains.
 Those have been deleted from the workspace and are
no longer findings.
)

### Unused class members

Real findings (9) in editord LSP code:
`DiagnosticStore.update`,
 `DiagnosticStore.delete`,
`LspClient.{initialized,shutdown,dead}`,
 `LspPool.resolveAll`,
`DirWatcher.{watchDir,suppressPath,close}`.
These are public methods,
 not `#private` syntax,
 so oxlint's
`no-unused-private-class-members` does not catch them.

### Circular dependency

`package/module/es/src/path/find-monorepo-root.ts` imports `dirname` from
`package/module/es/src/path/index.ts`,
 which re-exports `find-monorepo-root.ts`.
The cycle is **already documented and accepted** in source via
`// oxlint-disable-next-line import/no-cycle -- barrel re-export cycle;
dirname is fully initialized before findMiseMonorepoRoot runs`.
The prototype config disables fallow's `circular-dependencies` rule
because oxlint already covers this with explicit suppression.

## Where fallow's defaults break for this repo

### The tsdown plugin matches only the bare config name

Fallow's tsdown plugin (`crates/core/src/plugins/tsdown.rs`) hardcodes:

```rust
const CONFIG_PATTERNS: &[&str] = &["tsdown.config.{ts,js,cjs,mjs}"];
```

This monorepo's convention;
 documented in `CLAUDE.md`:
 splits bundle
targets into `tsdown.client.config.ts` and `tsdown.node.config.ts` per package.
The plugin does not match those filenames,
 so it does not parse them
to discover declared `entry` arrays.

The prototype closes this gap with a generator in `file-enforcer.config.ts`
that globs all `package/*/*/tsdown.*.config.ts`,
 extracts each config's
`entry` array,
 and merges discovered paths into the fallow config.
For configs that re-export the shared base
(`export { default } from '@monochromatic-dev/config-tsdown/.node.ts';`),
the generator infers the entry from which base is re-exported
(`.client.ts` -> `src/client.ts`,
 others -> `src/index.ts`,
matching `package/config/tsdown/src/index*.ts`).

20 of the 29 tsdown configs in this repo are re-export style;
 the other 9
declare literal `entry` arrays.

### Test fixtures are flagged as unused exports by default

`package/test-fixture/oxlint-stylistic/src/{valid,invalid}/*.ts` and
`package/test-fixture/oxlint-tsdoc/src/{valid,invalid}/*.ts` exist
specifically to be linted by tests.
The prototype config disables `unused-exports`,
 `unused-files`,
`unused-types`,
 and `duplicate-exports` for `package/test-fixture/**`.

### Barrel re-exports across the type-system folder structure

`package/module/es/src/types/...` has hundreds of `index.ts` files in a
deeply nested signature-encoding folder structure.
Names like `from`,
 `is`,
 `type`,
 `string`,
 `negative` appear in many barrel
files because the structure is the point.
The prototype config disables `unused-exports`,
 `unused-files`,
and `duplicate-exports` for that subtree.

### TS-extension imports are unresolvable to fallow

All "unresolved imports" findings in this repo are imports of the form

```ts
import type { ... } from '../../../protocol.ts';
```

The target file exists,
 but fallow's resolver treats the explicit `.ts`
extension as a miss.
 TypeScript 5+,
 Bun,
 and Node ESM all accept this form;
it is the project's preferred import style per `CLAUDE.md`.
The prototype config sets `"unresolved-imports": "off"` rule-wide,
matching oxc's documented decision to decline `import/no-unresolved`
as inherently noisy.

### Per-package mise.toml task invocations are not followed

Per-package `mise.toml` files declare task entries like
`run = "bun src/abort.ts"`.
 The 8 files in `package/runtime-error/bun/src/*.ts`
are flagged as unused because nothing imports them as modules and no
tsdown config or `package.json` field references them;
 the only references
are in the package's `mise.toml`.
The prototype handles the root `mise.toml` by adding `**/src/cli.ts` as a
catchall entry,
 but does not parse package-level `mise.toml` files.
Closing this gap would extend the generator to scan every
`package/*/*/mise.toml` for `bun <path>` invocations,
 a future iteration.

### Bare `tsdown.config.ts` files

**Resolved 2026-05-09**:
 every per-package config now uses
`tsdown.<platform>.config.ts`,
 so the false-positive class described below
no longer applies.
 The historical finding is preserved for context.

A few packages have `tsdown.config.ts` (no `.client` / `.node` infix).
Fallow's built-in tsdown plugin should handle these natively per its
source code,
 but they appear in unused-files findings,
 indicating the
plugin does not activate per-package in this monorepo.
 Cause not yet
investigated;
 treated as residual noise.

## Generator integration via file-enforcer

The prototype lives on branch `fallow-wrap-prototype`.
 It extends the
existing `file-enforcer.config.ts` with two functions:

- `extractTsdownEntries(source)`:
   regex-extracts a literal `entry: [...]`
  array,
   or infers the entry from a base-config re-export
- `generateFallowConfig()`:
   globs all `package/*/*/tsdown.*.config.ts`,
  resolves each declared entry relative to its package directory,
  merges the results into a static base config,
   writes `.fallowrc.json`

`generateFallowConfig()` runs alongside the existing `CLAUDE.md` / `mise.toml`
generators when `bun file-enforcer.config.ts` or `mise run sync:files` is
invoked.
 Adding new packages or new `tsdown.*.config.ts` files automatically
updates `.fallowrc.json` on the next sync run.

The static base in `generateFallowConfig()` includes:

- Entry patterns for `mise.*.ts`,
   `tsdown.*.config.{ts,js,cjs,mjs}`,
  `**/src/cli.ts`,
   `oxlint.config.ts`,
   `file-enforcer.config.ts`,
  `playwright.*.config.ts`
- `dynamicallyLoaded` patterns for `client/**/*.ts` paths in editord,
  ssg-test,
   doodle-widget,
   done,
   done-h-css-test,
   exa-search,
   plus
  generic `**/*.css` and `**/*.html`
- `ignorePatterns` for `**/*.generated.ts`,
   `**/dist/**`,
   `**/node_modules/**`,
  and known generated subtrees
- Workspace pattern `package/*/*` (matches `pnpm-workspace.yaml`)
- Overrides disabling unused-* and duplicate-* rules for
  `package/test-fixture/**` and `package/module/es/src/types/**`
- Rule-level toggles:
   `circular-dependencies: off` (oxlint covers it),
  `unresolved-imports: off` (matching oxc's stance),
  `unused-dependencies: warn`

Limitations of the regex-based extractor:

- Computed entries (e.g. dynamically built from a glob) are not handled
- tsdown configs that spread the base and add extra entries via JS logic
  are partially handled (the spread base entry is inferred,
   but additional
  programmatic entries are not)
- None of the current tsdown configs use those patterns;
   if one starts to,
  the extractor would need to grow or be replaced with a Bun-import evaluation

## Other commands worth knowing about

- `bunx fallow --summary`:
   compact category-only output,
   useful for terminal use
- `bunx fallow audit`:
   changed-files-only mode for a per-commit gate
- `bunx fallow fix --dry-run`:
   preview an automatic cleanup pass.
  Tested on this repo:
   aggressive on `module/es/src/types/...`.
   Not safe to run
  unattended;
   useful for cherry-picking obvious fixes.
- `bunx fallow watch`:
   re-runs analysis on file changes
- `bunx fallow setup-hooks`:
   installs a Claude Code PreToolUse hook
  gating `git commit` / `git push` on `fallow audit`
- `bunx fallow --save-baseline foo.json` and `--baseline foo.json`:
  fail only on **new** issues;
   the baseline workflow that lets a noisy
  initial state be accepted while still catching regressions
- `bunx fallow coverage`:
   runtime coverage workflow (paid feature;
   not tested)

## Recommendation

Adopt locally as a periodic audit tool,
 with the current 744 findings
baselined as accepted.

This repo does not run CI or PR review,
 so the integration is local-only.
Recommended shape:

- Land the prototype on `fallow-wrap-prototype`:
   commits the modified
  `file-enforcer.config.ts`,
   the generated `.fallowrc.json`,
   and this audit
- Save three per-analysis baselines (fallow audit rejects the global
  `--baseline` flag and requires per-analysis baselines because each
  sub-analysis uses a different baseline format):

  ```bash
  bunx fallow dead-code --save-baseline fallow-baselines/dead-code.json
  bunx fallow health    --save-baseline fallow-baselines/health.json
  bunx fallow dupes     --save-baseline fallow-baselines/dupes.json
  ```

- Either pass these on every audit invocation,
   or declare them once in
  `.fallowrc.json` under
  `audit.deadCodeBaseline` / `audit.healthBaseline` / `audit.dupesBaseline`
  so `bunx fallow audit` picks them up automatically.
   The
  `generateFallowConfig()` helper can be extended to emit the audit
  block once the baselines exist.
- Add task definitions to `mise.no-env.toml` (not `mise.toml`:
   the latter
  is generated by file-enforcer from the former and gets overwritten on
  every sync).
   Suggested tasks:

  ```toml
  [tasks.fallow]
  description = "Full fallow snapshot against the current .fallowrc.json"
  run = "bunx fallow --summary"
  ```

- The `fallow audit` (delta-since-base) workflow does **not** work in this
  repo as of 2026-04-28 because the repo root contains empty files named
  `HEAD`,
   `config`,
   `hooks`,
   `objects`,
   and `refs`
  (gitignored but present on disk).
  These conflict with git's ref resolution:
   `git rev-parse HEAD~1`
  errors with `ambiguous argument 'HEAD': both revision and filename`,
  which fallow surfaces as
  `Error: could not determine changed files for base ref 'main'`.
  Removing those files (none are tracked,
   all are empty) would unblock
  `fallow audit` and a corresponding `mise run fallow:audit` task that
  runs `bunx fallow audit --base HEAD~1`.
   Until then,
   the periodic
  full-snapshot task is the only available shape.
- Optional:
   invoke `bunx fallow setup-hooks` to install the bundled Claude
  Code PreToolUse hook that gates `git commit` on `fallow audit`.
   This
  hook will inherit the same blocker until the root git-internals
  files are removed.

Reasoning:

- The unique signal (cross-package clones,
   dead deps,
   unlisted deps,
  unused class members) is real and out-of-scope for oxlint by design:
  the oxc maintainers have declined `no-unresolved` and have
  `no-unused-modules` and `no-extraneous-dependencies` as low-priority TODOs.
- The one-time cleanup is concrete:
   4 truly dead deps,
   18 unlisted
  dependencies (the 18 packages imported but not declared),
   9 unused class
  members in editord LSP code,
   4 cross-package clone clusters.
   That's
  enough actionable cleanup to pay for the integration.
- The 744 baselined noise is bounded;
   the diminishing-returns curve
  from each fix iteration (-56,
   -1,
   -1) is direct evidence the residual
  is stable;
   future runs surface only the deltas,
   not the full backlog.
- file-enforcer integration keeps `.fallowrc.json` current as packages
  and conventions evolve,
   instead of being a hand-maintained config.

Don't adopt as:

- A blocking gate on every save;
   the value is in periodic review,
   not
  live editor feedback
- The `fallow fix` auto-cleanup tool;
   it's too aggressive on the
  deeply-nested type system (verified during the prototype)

Suggested cleanup sequence after adoption:

1. Triage the 18 unlisted dependencies into the right `package.json` files
2. Remove the 4 truly dead dependencies
3. Move the 6 misplaced devDep declarations to the packages that import them
4. Delete the 9 unused class members in editord LSP code
5. Decide whether to extract `package/cli/_shared/` (or similar) for the
   mvm/vmsync clones,
    or accept them
6. After each cleanup batch,
    re-save the three baselines:

   ```bash
   bunx fallow dead-code --save-baseline fallow-baselines/dead-code.json
   bunx fallow health    --save-baseline fallow-baselines/health.json
   bunx fallow dupes     --save-baseline fallow-baselines/dupes.json
   ```

After step 6,
 the baselines shrink and future audit runs surface a smaller
noise floor,
 which makes new findings stand out faster.

## Notes for re-running

To reproduce the bare run:

```bash
mkdir -p /tmp/fallow-out
BUN_TMPDIR=/tmp BUN_INSTALL="$HOME/.bun" \
  bunx fallow --quiet --format markdown \
  > /tmp/fallow-out/all.md 2> /tmp/fallow-out/all.err
```

To reproduce the file-enforcer-generated config:

```bash
git checkout fallow-wrap-prototype
bun file-enforcer.config.ts          # regenerates .fallowrc.json
BUN_TMPDIR=/tmp BUN_INSTALL="$HOME/.bun" bunx fallow --summary
```

The fallow source is at `/tmp/fallow-clone/fallow` for inspecting plugin
patterns.
 The schema for `.fallowrc.json` is at
`https://raw.githubusercontent.com/fallow-rs/fallow/main/schema.json`,
referenced as `$schema` in the generated config.

Useful diagnostic commands during config tuning:

- `bunx fallow --summary`:
   compact category-only counts
- `bunx fallow --quiet --format markdown > out.md`:
   full markdown report
- `awk '/^### Unused files/,/^### Unused exports/' out.md` -- isolate one section
- `grep -nE '^### ' out.md` -- list section headers and counts

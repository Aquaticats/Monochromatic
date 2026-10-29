# Consistency audit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/fy` is now `package/fy/cli`;
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/rgffplay` is now `package/rgffplay/cli`;
> `package/cli/terminal-exec` is now `package/terminal-exec/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

Generated 2026-04-05 from automated analysis of all 69 packages (across 19 categories)
plus root-level documentation files.

## Missing required files

Per `CLAUDE.md`,
 every package needs `package.json`,
 `mise.toml`,
 and `README.md`.

- `claude-code-plugin/verbose-tool-output`:
   missing **all three** (`package.json`,
   `mise.toml`,
   `README.md`)
- `claude-code-plugin/statusline`:
   missing `package.json`,
   `mise.toml`
- `claude-code-plugin/research-agent`:
   missing `package.json`,
   `mise.toml`
- `intellij-plugin/islands-black`:
   missing `package.json`,
   `mise.toml`
- `config/tofu`:
   missing `mise.toml`
- `config/cosign`:
   missing `mise.toml`
- `config/dotfiles`:
   missing `mise.toml`
- `test-fixture/oxlint-stylistic`:
   missing `README.md`

## Package naming

### Anomalous name prefix

Every package under `test-fixture/` uses the `@monochromatic-dev/test-fixture-*` name pattern
except one:

- `test-fixture/file-enforcer-perf` is named `@monochromatic-dev/fixture-file-enforcer-perf`
  (uses `fixture-` instead of `test-fixture-`)

### Singular vs plural category in rolldown-plugin

The directory is `rolldown-plugin/import-attributes` (plural)
but the package name is `@monochromatic-dev/rolldown-plugin-import-attributes` (singular).
No other multi-word category has this singular/plural mismatch.

## README heading inconsistency

README `# heading` lines use at least four different conventions for the same kind of thing:

**Full scoped package name:
**
`# @monochromatic-dev/module-es`,
 `# @monochromatic-dev/config-oxlint`,
 etc.

**Bare package slug (no scope):
**
`# config-dprint`,
 `# module-dom`,
 `# file-enforcer`,
 etc.

**Human-readable title:
**
`# Done`,
 `# RSS Reader`,
 `# Hall Monitor`,
 `# Exa Search Interface`,
 `# Islands Black`

**Mixed / unique:
**
`# Tsconfig-monochromatic` (title-case with slug),
 `# @monochromatic-dev/build-css` (scope + different slug than directory `build-tool/css`),
`# cli-vm (mvm)` (parenthetical alias),
 `# clauce-code-plugins-terminal-title` (typo:
 "clauce")

The typo `clauce` in `claude-code-plugin/terminal-title/README.md` heading is worth fixing regardless of
which convention is chosen.

Packages with **blank/empty** README first lines:
 `config/stylelint`,
 `dev-script/backup-path`.

## package.json field presence

### `version`

15 packages have no `version` field at all.
 The rest are split across five different versions:

- `0.0.1` (46 packages):
   the majority
- `0.0.125` (1):
   module/es
- `0.0.5` (1):
   config/typescript
- `0.1.0` (4):
   mcp/nvim,
   mcp/stdio,
   mcp/mvm,
   stylesheet/monochromatic
- `1.0.0` (2):
   webapp-productivity/rss,
   webapp-search/ai-tree
- missing (13):
   config/dprint,
   config/stylelint,
   config/tsdown,
   oxlint-plugin/tsdoc,
  config/tofu,
   oxlint-plugin/no-restricted-syntax,
  oxlint-plugin/stylistic,
   config/oxlint,
  ssg/aquati.
  cat,
   webapp-productivity/done,
   webapp-productivity/done-h-css-test,
  desktop-daemon/editord,
   audit/oph-common-look-and-feel

### `private`

30 packages have no `private` field (neither `true` nor `false`).
39 packages have `private: true`.
No package explicitly sets `private: false`.

Packages without `private` that look like they should be private
(dev-scripts,
 claude-code-plugin,
 single-use configs):
`dev-script/file-enforcer`,
 `dev-script/backup-path`,
 `dev-script/inference-canary`,
`dev-script/inference-canary-viewer`,
 `dev-script/catalog-tighten`,
`claude-code-plugin/terminal-title`,
 `claude-code-plugin/hook-type`,
`claude-code-plugin/session-start-housekeeping`,
 `claude-code-plugin/stop-reminder`,
`claude-code-plugin/hook-utils`,
 `claude-code-plugin/bash-output-filter`,
`claude-code-plugin/claude-spawn`,
 `claude-code-plugin/guardrail`.

Packages without `private` that are likely intended for publishing (have `files` field):
`config/dprint`,
 `config/stylelint`,
 `config/tsdown`,
 `config/typescript`,
`oxlint-plugin/tsdoc`,
 `oxlint-plugin/no-restricted-syntax`,
 `oxlint-plugin/stylistic`,
`config/oxlint`,
 `module/es`,
 `module/hyperscript`,
 `build-tool/css`,
`rolldown-plugin/import-attributes`,
 `stylesheet/monochromatic`.

The ambiguity means `pnpm publish` behavior for these packages is undefined.

### `author` and `repository`

Only a handful of packages include `author` and `repository` fields
(config/dprint,
 config/stylelint,
 config/typescript,
 module/es,
 module/hyperscript,
and a few others).
 The vast majority omit both.

### `sideEffects`

Three different states with no clear pattern by package type:

- `false` (4 packages):
   dev-script/file-enforcer,
   dev-script/backup-path,
   claude-code-plugin/hook-type,
   claude-code-plugin/hook-utils
- `true` (19 packages):
   all CLIs,
   all claude-code-plugin (except hook-types/hook-utils),
   dev-scripts,
   desktop-daemon/hall-monitor
- missing (45 packages):
   everything else

Libraries that likely should declare `sideEffects: false` (module/es,
 module/hyperscript,
 etc.) do not.

### `files`

- 43 packages have no `files` field,
   meaning they would publish their entire directory
- Among packages with `files`,
   the value patterns vary:
   `["src"]`,
   `["src/"]`,
   `["dist/final"]`,
   `["dist/final","src"]`,
   `["src/","dist/"]`
- The trailing-slash inconsistency (`"src"` vs `"src/"`) is cosmetically different but functionally identical in npm

## Entry point patterns

Packages use a mix of `main`,
 `module`,
 `exports`,
 and `bin` with no consistent pattern
for similar package types:

**Config packages**:
 mixed approaches:

- `config/dprint`:
   `main` + `module` (no `exports`)
- `config/stylelint`:
   `exports` + `main` + `module`
- `config/tsdown`:
   `exports` only
- `config/oxlint*`:
   `exports` only
- `config/tofu`,
   `config/cosign`,
   `config/dotfiles`:
   none

**Module packages**:
 mixed approaches:

- `module/es`:
   `exports` + `module` (no `main`)
- `module/dom`:
   `exports` only
- `module/test`:
   `exports` + `module`

**CLI packages**:
 reasonably consistent (`main` + `bin`),
 except:

- `cli/mvm`,
   `cli/terminal-exec`,
   `cli/vmsync`:
   `exports` + `main` + `bin`
- `cli/fy`,
   `cli/rgffplay`,
   `cli/git`:
   `main` + `bin` (no `exports`)

**Dev-script packages**:
 inconsistent:

- `dev-script/file-enforcer`,
   `dev-script/backup-path`:
   `exports` + `main` + `module`
- `dev-script/inference-canary`,
   `dev-script/catalog-tighten`:
   `main` only
- `dev-script/vm-builder`:
   `bin` only

## Exports shape inconsistency

Among packages that have `exports`,
 the value structure differs:

- Some use **string** values:
   `{ ".": "./index.mjs" }`
- Some use **condition objects**:
   `{ ".": { "types": "...", "node": "...", "default": "..." } }`
- Some mix both in the same package

There is no consistent convention for when to use conditions vs direct paths.

## mise.toml task naming

Task names use mixed quoting:
 some use `[tasks."lint:types"]` (double-quoted)
and others use `[tasks.'lint:types']` (single-quoted) within the same codebase.

4 packages with `mise.toml` define **zero tasks** (empty `mise.toml`):
test-fixture/css-imported,
 test-fixture/css-imported-no-exports,
test-fixture/css-importing,
 test-fixture/css-importing-filepath.

## Root-level documentation file naming

### Bug report prefix

Two different naming conventions exist:

**Fixed:
** renamed `BUGREPORT.*` files to `BUG-REPORT.*` for consistency.

### TROUBLESHOOTING subtopic separator

Uses **dots** for hierarchical subtopics (`TROUBLESHOOTING.performance.build.md`)
but **hyphens** for compound tool names (`TROUBLESHOOTING.bun-fetch-streaming.md`,
`TROUBLESHOOTING.css-tooling.md`).
This is probably intentional (dots = hierarchy,
 hyphens = slug),
but one file breaks the pattern:
`TROUBLESHOOTING.cLikeComments.md` uses **camelCase** instead of kebab-case.

### TODO subtopic separator

Uses dots for hierarchy (`TODO.performance.build.md`)
and hyphens within slugs (`TODO.ai-auto-commit.md`),
 consistent with TROUBLESHOOTING,
except `TODO.package-structure.md` vs `TODO.packages.md` (inconsistent plural/singular).

### MIGRATION / LESSONS-LEARNED separators

`MIGRATION-test-harness.md` and `MIGRATION-vlt.md` use **hyphens** as the topic separator,
while every other file family (TROUBLESHOOTING,
 TODO,
 PHILOSOPHY,
 BUGREPORT) uses **dots**.

## Dependency version management

### Non-catalog dependencies

Two packages pin versions outside the catalog system:

- `dev-script/inference-canary`:
   `why-is-node-running@^3.2.2`
- `dev-script/inference-canary-viewer`:
   `@lobehub/icons-static-svg@^1.82.0`

### ~~Duplicate catalog definitions~~ (fixed)

The stale `workspaces.catalog` in `package.json` has been removed.
The single source of truth is `pnpm-workspace.yaml`.

## License inconsistency

Two different license values across 67 packages:

- `LGPL-3.0-or-later` (65 packages)
- `LGPL-3.0-or-later AND CC-BY-SA-4.0` (2):
   ssg/aquati.
  cat,
   typeface/aquaticat

The mixed licenses are likely intentional (content vs code),
but there is no documentation explaining the licensing model.

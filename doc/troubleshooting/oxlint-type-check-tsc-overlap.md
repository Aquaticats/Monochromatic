# Oxlint 1.74.0 and 1.81.0 package lint reports TypeScript diagnostics but does not cover every `tsc` input

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

The package lint command:

```bash
mise run //package/<category>/<package>:lint:oxlint
```

can report compiler-style diagnostics such as:

```text
typescript(TS1109): Expression expected.
typescript(TS2322): Type 'number' is not assignable to type 'string'.
```

The `typescript(TS...)` label makes the command look like ordinary type-aware linting has started replacing
`tsc`.
That reading is incomplete.
The repository has separately enabled Oxlint's experimental type-check mode.
Its per-file syntactic and semantic diagnostics are restricted to files Oxlint selected for linting,
while configuration and program-creation diagnostics follow a separate path.

## Root cause

The behavior has four steps.

First,
 the shared repository config enables type-aware lint rules and compiler diagnostics.
`package/config/oxlint/src/config-base.ts:28-35` contains:

```ts
options: {
  denyWarnings: true,
  reportUnusedDisableDirectives: 'warn',
  typeAware: true,
  typeCheck: true,
},
```

Second,
 the package task delegates to Oxlint without repeating those config options on the CLI.
The source task template at `mise.no-env.toml:589-596`,
`task_templates.lint:oxlint`,
contains:

```toml
[task_templates."lint:oxlint"]
description = "Lint with Oxlint"
shell = "node --input-type=module-typescript -e"
run = """
{{vars.dispatch_workspace_node}}
ensureOxlintConfig()
runWorkspaceNode('package/monorepo-task/core-exec', 'oxlint-wrapper', [])
"""
```

`typeAware: true` and `typeCheck: true` are different controls.
The former enables rules that need type information.
The latter asks the TypeScript Go backend to return compiler diagnostics.
Git history shows that commit `df8948e6b0eb96a5c540ddca17dd25e36490df0a` enabled these stricter options on
2026-05-18.
Commit `fce6c5b81` removed the redundant CLI flag on 2026-09-06.

The capability is newer than Oxlint's original syntax linting.
Oxlint's [type-aware alpha announcement][] introduced compiler diagnostics on 2025-12-08.
The `apps/oxlint/CHANGELOG.md:685-690` entry for Oxlint `1.51.0` records that `typeCheck` became a config option on
2026-03-02.

Third,
 Oxlint combines the CLI and config settings before constructing the type-aware runner.
The Oxlint `1.74.0` source at commit `2d4e8d20644e0e7446f0a381894b45ea339a0625`,
`apps/oxlint/src/lint.rs:377-381`,
 contains:

```rust
let type_aware =
    type_check_only || self.options.type_aware || config_store.type_aware_enabled();
let type_check =
    type_check_only || self.options.type_check || config_store.type_check_enabled();
```

The same file at `apps/oxlint/src/lint.rs:487-493` passes both results into the runner:

```rust
let lint_runner = match LintRunner::builder(options, linter)
    .with_type_aware(type_aware)
    .with_type_check(type_check)
    .with_silent(misc_options.silent)
    .with_fix_kind(fix_options.fix_kind())
    .with_type_check_only(type_check_only)
```

Fourth,
 Oxlint sends the type-check choice and the selected lint paths to `tsgolint`.
`crates/oxc_linter/src/tsgolint.rs:605-614` contains:

```rust
configs: config_groups
    .into_iter()
    .map(|(rules, file_paths)| Config {
        file_paths,
        rules: rules.into_iter().collect(),
    })
    .collect(),
source_overrides,
report_syntactic: self.type_check,
report_semantic: self.type_check,
```

The `tsgolint` `0.24.0` source at commit `5a37e8902f65440900be1436b814919fcdb4e3d4`,
`internal/linter/linter.go:320` and `internal/linter/linter.go:335`,
then requests TypeScript syntactic and semantic diagnostics:

```go
syntacticDiagnostics := program.GetSyntacticDiagnostics(ctx, file)
```

```go
semanticDiagnosticsByFile := program.GetSemanticDiagnosticsWithoutNoEmitFiltering(ctx, files)
```

This path produces the `typescript(TS1109)` label.
It is a TypeScript compiler diagnostic returned through Oxlint,
 not an Oxlint rule named `TS1109`.

The important scope boundary is the `files` collection.
`internal/linter/linter.go:121-132` intersects the TypeScript program with the paths Oxlint supplied:

```go
fileSet := make(map[string]struct{}, len(filePaths))
for _, f := range filePaths {
    fileSet[f] = struct{}{}
}

sourceFiles := make([]*ast.SourceFile, 0, len(filePaths))
for _, sf := range program.SourceFiles() {
    if _, ok := fileSet[sf.FileName()]; ok {
        sourceFiles = append(sourceFiles, sf)
        delete(fileSet, sf.FileName())
    }
}
```

Oxlint therefore uses the full TypeScript program for resolution,
but reports per-file syntactic and semantic diagnostics for the lint-selected files.
It does not automatically turn every file matched by `tsconfig.json` into a lint target.

This selected-file boundary does not cover every diagnostic category.
`internal/utils/create_program.go:57-116` separately returns configuration and program-creation diagnostics:

```go
if len(configParseResult.Errors) > 0 && !suppressProgramDiagnostics {
```

```go
program_diagnostics := program.GetProgramDiagnostics()
if len(program_diagnostics) > 0 && !suppressProgramDiagnostics {
```

That distinction matters in this repository.
The TypeScript config includes `src/**/*.ts` at
`package/config/typescript/tsconfig.options.json:2-6`,
 while Oxlint ignores generated TypeScript files at
`package/config/oxlint/src/config-base.ts:84-89`:

```ts
'**/*.astro',
'**/i18n/i18n-types.ts',
'**/i18n/i18n-util.ts',
'**/i18n/i18n-util.sync.ts',
'**/i18n/i18n-util.async.ts',
'**/*.generated.ts',
```

A compiler error in one of those ignored `.ts` files can still be found by `lint:types` and missed by
`lint:oxlint`.

The existing TypeScript task also has a different execution contract from upstream's `tsc --noEmit` replacement
example.
`mise.toml:597-603` invokes the repository wrapper in build mode:

```toml
[task_templates."lint:types"]
description = "TypeScript"
shell = "node --input-type=module-typescript -e"
run = """
{{vars.dispatch_workspace_node}}
runWorkspaceNode('package/monorepo-task/core-exec', 'tsc-filter', ['--build'])
"""
```

Replacing that task requires checking build-mode and wrapper-policy parity,
not only ordinary semantic diagnostic parity.

## Verification

Versions under test on 2026-07-16:

- `oxlint@1.74.0`,
   release tag `apps_v1.74.0`,
   commit
  `2d4e8d20644e0e7446f0a381894b45ea339a0625`.
- `oxlint-tsgolint@0.24.0`,
   release tag `v0.24.0`,
   commit
  `5a37e8902f65440900be1436b814919fcdb4e3d4`.
- `typescript@7.0.2`.

A follow-up positive-control probe on 2026-09-06 used `oxlint@1.81.0`.
It imported the shared config options,
disabled `typeCheck` to isolate type-aware lint rules,
and enabled `typescript/no-floating-promises` over this source:

```typescript
// probe.ts
Promise.resolve(1);
```

The config-only case retained the shared `typeAware: true` value:

```typescript
// oxlint.config.ts
import shared from '@monochromatic-dev/config-oxlint';

export default {
  options: {
    ...shared.options,
    typeCheck: false,
  },
  plugins: ['typescript'],
  rules: {
    'typescript/no-floating-promises': 'error',
  },
};
```

The negative control changed only `typeAware`:

```typescript
// oxlint.disabled.config.ts
import shared from '@monochromatic-dev/config-oxlint';

export default {
  options: {
    ...shared.options,
    typeAware: false,
    typeCheck: false,
  },
  plugins: ['typescript'],
  rules: {
    'typescript/no-floating-promises': 'error',
  },
};
```

A disposable directory linked the repository `node_modules` directory and used these tasks.
The executed harness used the package-name import and relative binary paths shown in these snippets:

```toml
# mise.toml
[tasks."probe:config"]
run = "./node_modules/.bin/oxlint --config=oxlint.config.ts probe.ts"

[tasks."probe:disabled"]
run = "./node_modules/.bin/oxlint --config=oxlint.disabled.config.ts probe.ts"

[tasks."probe:cli"]
run = "./node_modules/.bin/oxlint --config=oxlint.disabled.config.ts --type-aware probe.ts"
```

`mise run probe:config` exited `1` without a CLI flag:

```text
probe.ts:1:1: error typescript(no-floating-promises): Promises must be awaited, add void operator to ignore.
```

`mise run probe:disabled` exited `0` without diagnostics.
`mise run probe:cli` exited `1` with the same diagnostic as the config-only case.
The negative control proves the fixture distinguishes disabled type-aware analysis.
The matching config-only and CLI cases prove the shared config option replaces the manual flag.

A nested task then exercised config discovery through an upward directory walk,
matching package-scoped production invocation:

```toml
# nested/mise.toml
[tasks.probe]
run = "../node_modules/.bin/oxlint probe.ts"
```

`nested/probe.ts` contained the same floating promise.
Running `mise run probe` from `nested/` without `--config` or `--type-aware` exited `1`
with the same `typescript(no-floating-promises)` diagnostic.
This verifies that upward config discovery honors the shared `options.typeAware` value.

The original disposable harness reproduced the distinction between type-aware rules and type checking:

```bash
repo=/var/home/user/Monochromatic
oxlint="$repo/node_modules/.bin/oxlint"
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
fixture="$(mktemp --directory /tmp/agent/oxlint-typecheck-doc.XXXXXX)"
cd -- "$fixture"
printf '%s\n' \
  '{"compilerOptions":{"strict":true,"noEmit":true},"include":["valid.ts","semantic.ts","syntax.ts"]}' \
  > tsconfig.json
printf '%s\n' '{"options":{"typeAware":true,"typeCheck":true}}' > with-type-check.json
printf '%s\n' '{"options":{"typeAware":true,"typeCheck":false}}' > without-type-check.json
printf '%s\n' 'export const valid: string = "ok";' > valid.ts
printf '%s\n' 'export const semantic: string = 1;' > semantic.ts
printf '%s\n' 'export const syntax = ;' > syntax.ts

"$oxlint" --config=with-type-check.json --type-aware --allow=all valid.ts
"$oxlint" --config=without-type-check.json --type-aware --allow=all semantic.ts
"$oxlint" --config=with-type-check.json --type-aware --allow=all semantic.ts
"$oxlint" --config=with-type-check.json --type-aware --allow=all syntax.ts
```

Working catalog:

- Valid source with type checking exits `0`.
- Semantic mismatch with `typeCheck: false` exits `0`.
  This proves `--type-aware` alone is not a compiler-check replacement.

Failing catalog,
 semantic diagnostics:

```text
semantic.ts:1:14: error typescript(TS2322): Type 'number' is not assignable to type 'string'.
```

Failing catalog,
 syntactic diagnostics:

```text
syntax.ts:1:23: error: Unexpected token
syntax.ts:1:23: error typescript(TS1109): Expression expected.
```

The duplicate syntax messages come from the regular Oxc parser and the enabled TypeScript checker.

A second disposable harness reproduced the file-selection difference with the same `tsconfig.json`:

```bash
repo=/var/home/user/Monochromatic
oxlint="$repo/node_modules/.bin/oxlint"
tsc="$repo/node_modules/.bin/tsc"
mkdir --parents /tmp/agent
chmod 700 /tmp/agent
fixture="$(mktemp --directory /tmp/agent/oxlint-typecheck-ignore.XXXXXX)"
mkdir --parents "$fixture/src"
cd -- "$fixture"
printf '%s\n' \
  '{"compilerOptions":{"strict":true,"noEmit":true},"include":["src/**/*.ts"]}' \
  > tsconfig.json
printf '%s\n' \
  '{"options":{"typeAware":true,"typeCheck":true},"ignorePatterns":["**/*.generated.ts"]}' \
  > .oxlintrc.json
printf '%s\n' 'export const selected: string = "ok";' > src/index.ts
printf '%s\n' 'export const ignored: string = 1;' > src/schema.generated.ts

"$oxlint" --type-aware --allow=all .
# exit 0

"$tsc" --project tsconfig.json
# src/schema.generated.ts(1,14): error TS2322: Type 'number' is not assignable to type 'string'.
# exit 1
```

The real package boundary was also exercised with both commands:

```bash
mise run //package/caught-value/core:lint:oxlint
# Found 0 warnings and 0 errors.

mise run //package/caught-value/core:lint:types
# exit 0
```

These passing commands show that both integrations currently run.
They do not prove that their input sets are identical.

## Verified operating choices

### Keep `lint:types` as the completion check

Continue running:

```bash
mise run //package/<category>/<package>:lint:types
```

after TypeScript edits,
even when `lint:oxlint` already reported TypeScript diagnostics.
This preserves checks over every input selected by `tsconfig.json`,
including inputs excluded by Oxlint's ignore patterns.
It also retains the repository's established `--build` wrapper behavior while Oxlint type checking remains
experimental.

Tradeoff:
 ordinary lint-selected `.ts` files are checked twice,
 once through `tsgolint` and once through `tsc`.

### Use `lint:oxlint` compiler diagnostics for early feedback

Treat `typescript(TS...)` findings from `lint:oxlint` as real compiler errors and fix them immediately.
For an ordinary lint-selected `.ts` file,
Oxlint's type-check mode provides compiler-style syntactic and semantic diagnostics using its bundled
TypeScript Go backend.
This is useful early feedback,
not proof of complete parity with the repository's installed `tsc` executable.

Tradeoff:
a clean result is only evidence for the files Oxlint selected and the diagnostics that backend emitted.
It is not evidence that every `tsconfig.json` input was selected or that `tsc --build` would pass.

### Retire `lint:types` only after proving scope parity

A future consolidation can remove the separate task if all package configurations satisfy these conditions:

- Oxlint selects every TypeScript input that `tsc` is expected to check.
- Differential fixtures show diagnostic parity for the repository's compiler options and project layouts.
- The migration preserves or deliberately retires the current wrapper's build-mode behavior.
- No workflow relies on TypeScript emit or build-mode artifacts.

The [Oxlint type-aware guide][oxlint type-aware guide]
explicitly says `oxlint --type-aware --type-check` can replace `tsc --noEmit`.
An Oxlint collaborator gave the same answer in [discussion 19571][]:
a separate `tsc` run is unnecessary when Oxlint performs the same checks,
but remains necessary when a monorepo relies on emitted output.

Tradeoff:
proving parity requires a repository-wide configuration audit and differential regression fixtures for ignored
inputs,
nonstandard inputs,
compiler options,
project references,
and build-mode behavior.
The present ignore mismatch already fails that gate.

## What does not work

- Inferring full type-check coverage from the `typescript(TS1109)` prefix.
  The prefix proves type-check mode ran for that file,
   not that all project files were selected.
- Treating `--type-aware` as equivalent to `--type-check`.
  The harness shows a semantic mismatch exits cleanly with type-aware rules alone.
- Removing `lint:types` because one representative package passes both commands.
  Passing results do not compare the commands' file sets.
- Assuming the package task must pass `--type-aware` to enable type-aware rules.
  The Oxlint 1.81.0 positive control reports `typescript/no-floating-promises` from the config-only case,
   while the disabled control exits cleanly and the CLI case reproduces the config-only diagnostic.

## Upstream filing decision

`.out-of-scope/` contains no Oxlint or `tsgolint` exemption matching this behavior.
The exact replacement question already exists as [discussion 19571][],
including the monorepo emit caveat.

1. **Is it really upstream's fault?
   ** No.
   Oxlint intentionally type-checks the files selected for linting,
    and its documented replacement example is
   `tsc --noEmit` over the same intended source surface.
   This repository has a broader TypeScript include surface than its Oxlint surface.
2. **Can upstream fix it?
   ** There is no demonstrated upstream defect to fix.
   Oxlint could add a whole-project diagnostic mode,
    but that would be a separate feature rather than a correction
   to the observed behavior.
3. **Are they supporting this use case?
   ** Yes,
    with boundaries.
   The type-aware guide supports replacing a separate `tsc --noEmit` step,
    and the collaborator answer says emit
   users must retain `tsc`.
4. **Would the repository welcome a contribution?
   ** Yes,
    conditionally.
   Oxc's `CONTRIBUTING.md:12-21` welcomes reviewed and tested AI-assisted work when AI use is disclosed,
    and its
   [PR policy][oxc pr policy] asks contributors to open a discussion before architectural changes.
   No contribution is justified without an upstream defect or missing documented case.
5. **Will they likely fix it?
   ** Not applicable.
   The current behavior matches the selected-file architecture,
    and the existing discussion treats replacement as
   configuration-dependent.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** No upstream patch was attempted because
   constraint 1 fails.
   The disposable harness instead proves the downstream scope mismatch.

### Upstream filing artifact

Nothing to add.
The existing discussion already answers the general question,
and this repository-specific ignore mismatch does
not belong in an upstream issue or comment.

[discussion 19571]: https://github.com/oxc-project/oxc/discussions/19571
[oxlint type-aware guide]: https://oxc.rs/docs/guide/usage/linter/type-aware.html
[type-aware alpha announcement]: https://oxc.rs/blog/2025-12-08-type-aware-alpha.html
[oxc pr policy]: https://oxc.rs/docs/contribute/rules.html

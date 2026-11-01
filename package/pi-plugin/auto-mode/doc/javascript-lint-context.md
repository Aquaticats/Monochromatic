# JavaScript lint context

## Historical configuration, retired 2026-10-19

The user removed the unapproved native Pi implementation during the ground-up rebuild.
The mixed-language lint project and separate `tsconfig.types.json` are retired.
One `tsconfig.json` now extends the standard repository configuration for the future TypeScript plugin.
Two empty, untracked probe configs were also removed.
The findings that follow describe the removed implementation, not a required plugin architecture.

## Original reason for the split

The default TypeScript project includes maintained MJS files so semantic lint can resolve their imports.
The previous TS-only project leaves ordinary Node assertion imports error-typed in MJS consumers.
This is an input-project problem,
not evidence that Node assertions or every affected producer value are unsafe.

The discriminating private probe uses both default and named Node assertion imports,
plus a deliberately untyped member access and return.
The TS-only project reports unsafe assertion receivers.
The JavaScript-inclusive default project removes those findings while retaining both intentional unsafe findings.
The equivalent TypeScript file also resolves Node assertions.
Changing only Oxlint's import-resolution `--tsconfig` option does not change this result.
The backend discovers an owning project through file inclusion;
its current resolver is `internal/utils/find_tsconfig.go` in `oxc-project/tsgolint`.

`tsconfig.types.json` retains the original TypeScript project options and input patterns.
Both `lint:types` and `watch:lint:types` select it explicitly.
The default lint/editor project adds JavaScript and disables declaration isolation and composite input membership
for that mixed-language analysis.
TypeScript disallows JavaScript input with isolated declaration checking.
The original TypeScript check keeps its existing declaration options;
no lint rule changes or suppressions are involved.

The preserved type project rejects an intentionally wrong string assignment with `TS2322`.
Removing the fixture restores a successful complete check against the qualified phase artifact.
An initial isolated-declaration experiment produced no isolated diagnostic under the existing no-emit check;
it does not establish declaration-emission coverage.
Full-package lint and actual newly typed producer consumers still require fresh combined verification.

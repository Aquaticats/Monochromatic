# Why `bin-paths.ts` formats its caught value inline

`bin-paths.ts` carries one scoped disable of
`no-restricted-syntax/prefer-caught-value-text`
around the `Error.isError(error) ? error.message : String(error)` expression in its `catch` block.
This file records why the shared helper cannot be used there
and why lint configuration cannot express the exception.

## The constraint: Node built-ins only

The IDE's toolchain lookup runs `bin-paths.ts` with node
inside the language-server sandbox,
with the project mounted read-only and a cleared environment
(`package/ide/app/src/language/toolchain/command.rs`, module comment).
The script's own contract says it exits 0 with empty `pathDirs` and `bins` on an unprovisioned toolchain,
and its header states "Node builtins only".
An unprovisioned worktree may have no `node_modules`,
so any workspace import (such as `@monochromatic-dev/caught-value-core/ts`)
would make the lookup fail with a module-resolution error instead of reporting an empty toolchain.

The import closure of `bin-paths.ts` was measured on 2026-10-29:
7 files, none importing anything outside `node:` built-ins.
The same walk over `providers.bin.ts` (positive control) found
`@monochromatic-dev/caught-value-core/ts`, `@monochromatic-dev/logger-core/ts` and `nano-spawn`,
so the walk does detect workspace imports.

## Why configuration cannot carry the exception

The rule lives in `package/oxlint-plugin/no-restricted-syntax/src/rule/prefer-caught-value-text.ts`.
Its `meta` declares no options schema,
and its only exemption is the hard-coded `CANONICAL_FORMATTER_PATH`
(`package/caught-value/core/src/index.ts`) checked by `isCanonicalFormatterFile`.
The repository enables it as `'no-restricted-syntax/prefer-caught-value-text': 'error'`
in `package/config/oxlint/src/rule/restriction.ts`.
So the only configuration route is a file override that turns the rule off for `bin-paths.ts` as a whole,
which loosens the rule for every future line of the file.
The scoped disable covers only the one expression.

## When to remove it

Remove the disable if `bin-paths.ts` ever gains a workspace import for another reason,
or if `prefer-caught-value-text` grows an option for files restricted to Node built-ins.

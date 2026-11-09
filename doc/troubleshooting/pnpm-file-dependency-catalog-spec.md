# pnpm 12.18.0: `catalog:` inside a `file:` dependency fails with ERR_PNPM_SPEC_NOT_SUPPORTED_BY_ANY_RESOLVER

Investigated on 2026-11-09 while proving
`package/monorepo-task/cli`'s new exports map from a consumer outside the repository
(issue #5846).
Any scratch consumer of a workspace package whose manifest uses `catalog:` hits this.

## Short answer

`catalog:` resolves only for projects pnpm treats as part of a workspace.
A `file:` dependency is an external project,
so pnpm resolves its manifest's dependency specs without a catalog and refuses the specifier.
Declaring the same catalog entries in the consumer's own `pnpm-workspace.yaml` does not help,
with or without `packages: ['.']`.
No workaround through pnpm was found;
the layout pnpm would have written was written by hand (see "Workaround").

## Symptom

Installing a scratch consumer whose only dependency is a `file:` path to a workspace package:

```text
Error: ERR_PNPM_SPEC_NOT_SUPPORTED_BY_ANY_RESOLVER

  × installing dependencies
  ├─▶ Failed to resolve dependency tree: "string-dedent@catalog:" isn't
  │   supported by any available resolver.
  │
  │   Failed to resolve string-dedent@catalog:
  │   This error happened while installing the dependencies of @
  ╰─▶ "string-dedent@catalog:" isn't supported by any available resolver.
```

The last line of the inner block is truncated at the `@` of the dependency's scope,
so grepping for a package name from the message finds only the bare specifier text.

## Minimal reproduction

Three files,
no repository involved:

```json
// ${HOME}/temp/agent/catalog-probe/package.json
{ "name": "probe-consumer", "version": "0.0.0", "private": true, "type": "module",
  "dependencies": { "@probe/dep": "file:./dep" } }
```

```json
// ${HOME}/temp/agent/catalog-probe/dep/package.json
{ "name": "@probe/dep", "version": "0.0.1", "type": "module", "main": "index.mjs",
  "dependencies": { "string-dedent": "catalog:" } }
```

```yaml
# ${HOME}/temp/agent/catalog-probe/pnpm-workspace.yaml
packages:
  - '.'
catalog:
  'string-dedent': '>=3.0.2'
```

`pnpm install --offline` in that directory fails with the error above.

Positive control,
the same three files with `"string-dedent": ">=3.0.2"` in place of `"string-dedent": "catalog:"`:
`pnpm install --offline` succeeds in 1.2 s (`Packages: +2`,
`+ @probe/dep file:dep`).
The control shows the failure is the specifier,
not offline installs,
`file:` links,
or the store.
A second control whose `file:` package declares no dependencies also installed (965 ms).

## What does not help

- Adding the catalog entries to the consumer's `pnpm-workspace.yaml`:
  same error.
- Adding `packages: ['.']` so the consumer is a workspace project itself:
  same error.
- `--offline`:
  the failure happens during dependency-tree resolution,
  before any fetch.

## Workaround: write the layout pnpm would have written

Node only needs the link,
and it resolves the package's exports map through the link's real path.
Run from the consumer directory,
with `PACKAGE_DIR` naming the package's absolute path
and `BIN_TARGET` the value of its `bin` entry:

```sh
# ${HOME}/temp/agent/consumer
mkdir --parents node_modules/@scope node_modules/.bin
ln --symbolic "$PACKAGE_DIR" node_modules/@scope/name
ln --symbolic "../@scope/name/$BIN_TARGET" node_modules/.bin/command-name
```

Then `node probe.ts` importing `@scope/name` resolves the built entry the exports map names,
and `node_modules/.bin/command-name` runs the command through its shebang.

Limits:

- The consumer installs nothing.
  The package's own dependencies resolve from its real path inside the repository's install,
  so this proves the manifest,
  the exports map,
  and the code,
  not a fresh install of a published tarball.
- The symbolic links are not recorded in any lockfile,
  so the layout is not evidence for reproducibility of an install.

The alternative,
declaring the workspace package as a `workspace:` dependency of a real repository
project and installing there,
needs no workaround but changes the repository.

## Not established

- Why pnpm does not consult the consumer's catalog for a `file:` project's specs.
  The pnpm source was not read;
  the observation is that it does not,
  and that the consumer's own catalog block changes nothing.
- Whether `workspace:` or `link:` dependencies behave the same way.
  Only `file:` was measured.

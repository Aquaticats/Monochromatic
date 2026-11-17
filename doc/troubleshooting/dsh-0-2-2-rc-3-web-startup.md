# DSH 0.2.2-rc.3 web startup skips bundles and leaves workspace services unavailable

Diagnosis verified against the installed packages and release artifacts. Live profile repair remains open.

## Symptom

`dsh --profile web` reports `dsh: skipping profile bundle` for Advisor, NeuralWatt, and Qwen Token Plan Individual.
It then prints the web URL and `dsh: warning: 5 entries did not activate`.
The warning includes `ERR_MODULE_NOT_FOUND: Cannot find package 'zod'` under `typert-loader` and
`workspace domain is inconsistent` under `workspace`. The session and workspace controllers wait for
`workspaceRegistry`. The deliverables UI waits for `sessionController`.

The `[mcp-stdio] <-` and `[mcp-stdio] ->` lines show a successful `tools/list` exchange with Search and Fetch.

## Root cause

### Linked bundles require rc.1

The web profile links the three local plugin packages. Each manifest sets `"dsh": "0.2.2-rc.1"` in `engines`.
DSH 0.2.2-rc.3 checks that field before considering peer dependencies.
The exact requirement fails for rc.3 before DSH loads the bundle. [Compatibility evaluator][engines]

```ts
// packages/boot/app-boot/src/plugin-compatibility.ts:60
return satisfies(runtimeVersion, range) ? {} : { 'engines.dsh': range }
```

### The published web Typert artifact has an undeclared dependency

`@deepseek-ai/dsh-web` exports `./typert` as `lib/typert.host.js`. Its generated host artifact contains a bare
runtime import of `zod`, which the package's published dependency fields omit. Node resolves the import from
the `dsh-web` package and fails. The Typert loader catches the import failure and reports the contribution
that failed. [Generator][generator] · [Web manifest][manifest] · [Typert loader][loader]

```js
// @deepseek-ai/dsh-web@0.2.2-rc.3/lib/typert.host.js:2
import { z } from 'zod'
```

### Two workspace IDs claim one path

During initialization `WorkspaceRegistry` reads the durable workspace domain and validates it. For the JSON
storage used here, that domain is `~/.dsh/storages/workspace.json`. Its `tables.workspaces` had two distinct IDs
with the same `path` and separate session memberships. The validator throws and the registry service stays
unavailable. The activation summary counts both failed entries and all three entries waiting on their
dependencies. [Registry initialization][workspace-init] · [Path check][workspace] · [Activation summary][activation]

```ts
// packages/workspace/workspace/src/index.ts:785
const paths = new Map<string, WorkspaceId>()
const accounted = new Map<SessionId, WorkspaceId>()
for (const [id, record] of table.entries()) {
  const pathHolder = paths.get(record.path)
  if (pathHolder !== undefined) {
    throw new Error(
      `workspace domain is inconsistent: path '${record.path}' is claimed `
      + `by both workspace '${pathHolder}' and workspace '${id}'`,
    )
  }
  paths.set(record.path, id)
```

Both records' stored creation timestamps predate the rc.3 release. The same duplicate-path check exists in
rc.1. The writer that produced the duplicate remains unknown. [rc.1 path check][old-workspace]

### The built Search and Fetch server echoes frames

`package/search-fetch/dsh-plugin/cordis.patch.yml:31` launches the built MCP server from
`package/search-fetch/mcp/dist/final/node/index.mjs`. The local build still contains the older unconditional
`console.error` frame echoes. The checked-in transport source uses debug logging at
`package/mcp/stdio/src/transport.ts:360` and `package/mcp/stdio/src/transport.ts:466`.

```ts
// package/mcp/stdio/src/transport.ts:360
.debug(`writing frame: ${frame}`,);
```

## Verification

`dsh --version` returned `0.2.2-rc.3`. The tagged source commit is
`38a6a5c965aa99ab525161efb7cc5cf1fe62fced`. On Node `26.13.1`, the installed compatibility evaluator accepted
each manifest for runtime `0.2.2-rc.1` and returned `{"engines.dsh":"0.2.2-rc.1"}` for rc.3.
The installed workspace validator rejected the saved records and a synthetic duplicate-path fixture. It accepted
that fixture with distinct paths. A `tools/list` request to the built Search and Fetch server returned `web_search`
and `web_fetch` on stdout with matching request and response traces on stderr.

This import probe reproduces the Typert failure without starting a server or opening the profile:

```sh
# Run with the pnpm installation from the incident available in the user's home.
node --input-type=module <<'NODE'
import assert from 'node:assert/strict';
import { readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const versionDir = join(homedir(), '.local/share/pnpm/store/v11/links/@deepseek-ai/dsh-web/0.2.2-rc.3');
const host = join(versionDir, readdirSync(versionDir)[0], 'node_modules/@deepseek-ai/dsh-web/lib/typert.host.js');
await assert.rejects(import(pathToFileURL(host)), { code: 'ERR_MODULE_NOT_FOUND', message: /Cannot find package 'zod'/ });
console.log('rc.3 Typert import cannot resolve zod');
NODE
```

## Verified diagnostic controls and repair limits

A process-local Node resolver hook supplied the `zod` already installed for `dsh-workspace` to the `dsh-web`
artifact. The same artifact then imported successfully and exported `TYPERT`. This verifies the missing
dependency edge at the import boundary. An installation using corrected dependency metadata remains untested.

The two local workspace records both own sessions. A live repair needs to preserve both session memberships
and keep one record for the path. The distinct-path control verifies the uniqueness rule on synthetic data.

## What does not work

`dsh plugin allow-version` addresses the bundle version check only. The Typert dependency and workspace registry
failures occur separately. `@deepseek-ai/dsh-web@0.2.2-rc.4` still ships the bare `zod` import and omits `zod`
from its manifest, verified against the published tarball and registry metadata.

## Upstream filing decision

- Responsibility. The missing runtime dependency is in the published DSH web package. The bundle pins and
  stale Search and Fetch build belong to this integration. The duplicate workspace writer is unknown.
- Fixability. Adding a runtime dependency to the package that owns the generated import is the candidate
  correction. The process-local import probe confirms the dependency required by that artifact.
- Supported use. The web package explicitly publishes the generated `./typert` export consumed by the loader.
- Intake. GitHub Issues are disabled and the contribution guide directs bug reports to GitHub Discussions.
  The guide declines external pull requests. [Contribution guide][contributing]
- Existing work. GitHub pull request searches for `zod typert` and `dsh-web zod` returned no entries.
  The rc.4 package still contains the defect. The discussion history remains unchecked.
- Prototype. The import control exercised the published artifact. An upstream packaging patch remains untested.

The filing artifact is this diagnostic record. Upstream posting was outside the diagnosis request.

[engines]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/boot/app-boot/src/plugin-compatibility.ts#L55-L61
[generator]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/typert/generator/src/emitter.ts#L187-L199
[manifest]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/web/web/package.json#L52-L64
[loader]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/typert/loader/src/index.ts#L408-L417
[workspace-init]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/workspace/workspace/src/index.ts#L214-L229
[workspace]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/workspace/workspace/src/index.ts#L785-L795
[activation]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/boot/app-boot/src/index.ts#L1092-L1118
[old-workspace]: https://github.com/deepseek-ai/deepseek-harness/blob/66491b05e3f05ad35bcdd4f6cd7ed26041d9e3db/packages/workspace/workspace/src/index.ts#L763-L773
[contributing]: https://github.com/deepseek-ai/deepseek-harness/blob/3f321d32bfe382212a8ed8785c1d47a5a300aa23/CONTRIBUTING.md#L9-L13

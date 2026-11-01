# module-fs-path

POSIX path operations and repository-root discovery that run under Node,
 Bun,
 and browsers,
 plus Node-only directory and file helpers.
The package was extracted from `@monochromatic-dev/module-es`'s `path/` submodule so consumers that need only
filesystem and path utilities can depend on it directly.

## Entries

### `.` (platform-neutral)

Path operations:

- `dirname`,
   `join`,
   `resolve`,
   `normalize`,
   `isAbsolute`,
   `sep`:
   POSIX path operations.
   `join` and `resolve` take one array of segments.
- `trimLeadingSlash`,
   `trimTrailingSlash`:
   strip one leading or trailing `/` unless the path is `/`.

Root discovery,
 one walker driven by marker values:

- `findRoot({ marker, cwd?, fs? })`:
   nearest ancestor of `cwd` (itself included) that `marker` accepts,
   spelled as the caller spelled `cwd`.
   `cwd` defaults to the process working directory,
   or `/` where no process exists;
   `fs` defaults to the runtime adapter.
   Rejects with `RootNotFoundError`,
   which carries `marker` (the marker name) and `startDir`.
- `findRootCached({ marker, cwd? })`:
   memoised variant over the runtime adapter,
   keyed by marker name and start directory resolved at call time.
   Concurrent first callers share one walk;
   a rejection stays memoised for its key.
- `MISE_MONOREPO`:
   nearest ancestor whose `mise.toml` has a `[monorepo]` table header on its own line
   (CRLF endings,
   trailing spaces,
   and a trailing comment tolerated;
   a header quoted inside a value refused).
- `GIT_REPOSITORY`:
   nearest ancestor with a structurally usable `.git` directory or gitfile
   (HEAD,
   objects,
   refs,
   relative targets,
   and linked-worktree `commondir` pointers are validated;
   invalid nearer markers are skipped).
- `PNPM_WORKSPACE`:
   nearest ancestor holding a `pnpm-workspace.yaml` file.
- `packageNamed(name)`:
   nearest ancestor whose `package.json` declares that `name`,
   anchoring a package on its own root in source and built modes;
   an unparsable or differently named manifest is walked past.
- `fileNamed(name)`,
   `directoryNamed(name)`:
   nearest ancestor holding a regular file,
   or a directory,
   with that name.
- `RootMarker`:
   `{ name, matches({ dir, fs }) }`.
   A custom marker is one object literal;
   its `name` must differ from the shipped ones because it keys the memo.
- `createMemoryRootFilesystem({ files, directories, links })`:
   a `RootFilesystem` over a tree held in memory.
   Every ancestor of a declared path is a directory;
   `exists` and `readSymbolicLink` look at the entry itself,
   the other probes look through links.
- `RootFilesystem`,
   `ABSENT`:
   the seam the adapters fill
   (`readTextFile`,
   `readSymbolicLink`,
   `exists`,
   `isDirectory`,
   `isFile`)
   and the sentinel `readTextFile` and `readSymbolicLink` return for an absent path.

### `./node` (Node and Bun only)

- `ensureDir`,
   `ensureFile`,
   `ensurePath`:
   create the path when missing (parents included) and verify owner read and write access when it exists,
   granting the owner bits when the mode denies them.
   `ensurePath` picks file or directory by whether the path has an extension.
- `emptyDir`,
   `emptyFile`,
   `emptyPath`,
   `removeEmptyFilesInDir`:
   clear a directory's entries,
   truncate a file (a `?query` suffix is stripped first),
   dispatch by extension,
   or delete files that are empty after trimming.

## Usage

```ts
import {
  findRoot,
  findRootCached,
  GIT_REPOSITORY,
  join,
  MISE_MONOREPO,
  packageNamed,
} from '@monochromatic-dev/fs-path-core';

const repoRoot = await findRootCached({ marker: MISE_MONOREPO, },);
const gitRoot = await findRoot({ cwd: import.meta.dirname, marker: GIT_REPOSITORY, },);
const packageRoot = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@scope/pkg',),
},);
const manifest = join([repoRoot, 'package.json',],);
```

A custom marker,
 and a test that walks a tree without touching a disk:

```ts
import {
  createMemoryRootFilesystem,
  findRoot,
  type RootMarker,
} from '@monochromatic-dev/fs-path-core';

const QUARANTINE_WARD: RootMarker = {
  name: 'quarantine ward',
  matches: async ({ dir, fs, },) => await fs.isDirectory(`${dir}/quarantine`,),
};

const fs = createMemoryRootFilesystem({
  directories: ['/registry/quarantine', '/registry/north/3',],
},);
await findRoot({ cwd: '/registry/north/3', fs, marker: QUARANTINE_WARD, },); // '/registry'
```

```ts
import {
  emptyDir,
  ensureDir,
} from '@monochromatic-dev/fs-path-core/node';

await ensureDir('dist',);
await emptyDir('dist',);
```

## Runtime support

The backends are chosen when the package is resolved,
 not at runtime.
`package.json` `imports` maps two private aliases by export condition:

- `#posix-path`:
   `node:path/posix` under the `node` condition,
   a pure-JS implementation with the same semantics under `default`.
- `#root-filesystem`:
   `node:fs/promises` under the `node` condition;
   under `default`,
   the origin private file system through `navigator.storage.getDirectory()` where the platform grants it,
   and otherwise an empty backend that finds no marker,
   so the finder rejects.

Node and Bun resolve the `node` condition (Bun's order is `bun`,
 `node-addons`,
 `node`,
 `require`,
 `import`,
 `default`),
 so they get `dist/final/node/index.mjs`.
Browsers and bundlers targeting them resolve `default` and get `dist/final/neutral/index.mjs`,
 which names no `node:` module.
Neither build contains a dynamic `import()`;
 a unit test reads every built chunk and rejects leaks in either direction,
 and a Playwright test drives the neutral build over OPFS in Chromium and Firefox
 (headless WebKit refuses OPFS writes and reports itself skipped).

In browsers,
 paths are absolute POSIX strings rooted at the OPFS root:
 `/repo/mise.toml` is the file `mise.toml` in the directory `repo` under `navigator.storage.getDirectory()`.
Marker files must exist there to be found.

A Node consumer whose bundler resolves the `default` condition gets the neutral build:
 path operations work,
 root discovery finds no filesystem and rejects.
Point the bundler at the `node` condition to keep `node:fs`.

## Logging

Diagnostics go through `@monochromatic-dev/logger-core`,
 inlined into both builds under the tags `rootDiscovery` (one line per walk),
 `rootFilesystem`,
 `rootMarker`,
 `path/ensure`,
 and `path/empty`.
The inlined logger builds its default sinks on first use;
 a consumer that also installs `@monochromatic-dev/logger-core` runs a second,
 independent logger instance.

## Source files

- `src/index.ts`:
   platform-neutral entry.
- `src/node.ts`:
   Node-only entry.
- `src/posix-path.node.ts`,
   `src/posix-path.neutral.ts`:
   the `#posix-path` backends.
- `src/root-filesystem-contract.ts`,
   `src/root-filesystem.node.ts`,
   `src/root-filesystem.neutral.ts`,
   `src/memory-root-filesystem.ts`:
   the `RootFilesystem` seam and its three adapters.
- `src/root-discovery.ts`:
   `findRoot`,
   `findRootCached`,
   `RootNotFoundError`.
- `src/root-marker-contract.ts`,
   `src/root-marker.ts`,
   `src/git-marker.ts`:
   the `RootMarker` type,
   the shipped markers,
   and Git administrative marker validation.
- `src/ensure.ts`,
   `src/empty.ts`,
   `src/trim.ts`:
   the remaining helpers.

Decisions:
 `DECISIONS.md`.

/**
 Platform-neutral entry (`\@monochromatic-dev/fs-path-core`).

 POSIX path operations and upward root discovery that run under Node, Bun,
 and browsers. The backends are chosen when the package is resolved, not at
 runtime: `package.json` maps `#posix-path` and `#root-filesystem` to
 `node:path/posix` and `node:fs/promises` under the `node` condition, and
 to the pure-JS path code and the origin private file system under
 `default`. Neither built artifact contains a dynamic import, and the
 neutral one names no `node:` module.

 Root discovery is one walker, `findRoot`, driven by a `RootMarker` value:
 the shipped presets and factories cover mise monorepos, Git repositories,
 pnpm workspaces, named files and directories, and named packages, and a
 custom marker is one object literal. `createMemoryRootFilesystem` walks a
 tree held in memory through the same seam the runtime adapters fill.

 Helpers that only make sense over a real filesystem (`ensureDir`,
 `emptyDir`, and their siblings) ship from
 `\@monochromatic-dev/fs-path-core/node`.

 @module
 */

export {
  dirname,
  isAbsolute,
  join,
  normalize,
  resolve,
  sep,
} from '#posix-path';
export {
  createMemoryRootFilesystem,
  type MemoryTree,
} from './memory-root-filesystem.ts';
export {
  type FindRootCachedOptions,
  type FindRootOptions,
  findRoot,
  findRootCached,
  RootNotFoundError,
} from './root-discovery.ts';
export {
  ABSENT,
  type RootFilesystem,
} from './root-filesystem-contract.ts';
export {
  directoryNamed,
  fileNamed,
  GIT_REPOSITORY,
  MISE_MONOREPO,
  packageNamed,
  PNPM_WORKSPACE,
} from './root-marker.ts';
export type {
  RootMarker,
  RootMatcher,
  RootMatcherArgs,
} from './root-marker-contract.ts';
export {
  trimLeadingSlash,
  trimTrailingSlash,
} from './trim.ts';

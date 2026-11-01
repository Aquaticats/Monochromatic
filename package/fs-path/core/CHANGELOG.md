# @monochromatic-dev/module-fs-path

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-fs-path` is now `@monochromatic-dev/fs-path-core`.

## 0.2.2

### Patch changes

- First release of `@monochromatic-dev/fs-path-core` through `npm-publish.yml`,
  so it carries npm provenance and validates the package's new trusted publisher.
  0.2.1 under this name was the first, manual publish after the rename from `@monochromatic-dev/module-fs-path`,
  which is now deprecated.
  The public API is unchanged.

## 0.2.1

### Patch changes

- Update repository metadata after the transfer to `Aquaticats/Monochromatic`.
  Publish through `npm-publish.yml` to validate the new npm trusted publisher.
  The public API is unchanged.

## 0.2.0

### Minor Changes

- Root discovery is one walker driven by marker values.
  `findRoot({ marker, cwd?, fs? })` and `findRootCached({ marker, cwd? })` replace `findMiseMonorepoRoot`,
   `findGitRepoRoot`,
   `findPnpmWorkspaceRoot`,
   `findPackageRoot`,
   and their cached variants.
  Shipped markers:
   `MISE_MONOREPO`,
   `GIT_REPOSITORY`,
   `PNPM_WORKSPACE`,
   and the factories `packageNamed`,
   `fileNamed`,
   `directoryNamed`;
   a custom marker is one `{ name, matches }` object.
  Every failed walk rejects with `RootNotFoundError`,
   which carries `marker` and `startDir`;
   `GitRepositoryRootNotFoundError` is gone.
  `createMemoryRootFilesystem({ files, directories, links })` builds a `RootFilesystem` over a tree held in memory,
   for tests and callers without a disk;
   the `RootFilesystem` and `RootMarker` types and the `ABSENT` sentinel are exported.
  `findRootCached` memoises per marker name and start directory,
   resolved at call time,
   so a caller that names its directory never receives another directory's answer.
  Package-root discovery moved to the root entry as `packageNamed`;
   `./node` now ships only the ensure and empty helpers.
  The `[monorepo]` header in `mise.toml` now matches through CRLF line endings,
   trailing spaces,
   and a trailing comment.
  `PNPM_WORKSPACE` accepts only a regular file named `pnpm-workspace.yaml`,
   where the old finder accepted any entry.
  Commit `80302aa`.

## 0.1.0

### Minor Changes

- First release.
  The root entry is platform-neutral (POSIX path operations and the mise,
   git,
   and pnpm root finders) and runs under Node,
   Bun,
   and browsers,
   where root discovery reads the origin private file system.
  `ensureDir`,
   `ensureFile`,
   `ensurePath`,
   `emptyDir`,
   `emptyFile`,
   `emptyPath`,
   `removeEmptyFilesInDir`,
   `findPackageRoot`,
   and `findPackageRootCached` ship from `@monochromatic-dev/module-fs-path/node`.
  Backends are chosen by export condition through `package.json` `imports`;
   neither built artifact contains a dynamic `import()`,
   and `happy-opfs` is no longer a dependency.
  `normalize` is public and delegates to `node:path/posix` under the `node` condition;
   `dirnameFallback`,
   `joinFallback`,
   and `resolveFallback` are no longer exported.
  `ensureDir` and `ensureFile` now grant the owner read and write bits
   when repairing an inaccessible path instead of setting mode `0o006`.
  `findMiseMonorepoRoot` now matches a `[monorepo]` header
   on the first line of `mise.toml`
   and on a last line without a trailing newline.
  Commit `bbb3954`.

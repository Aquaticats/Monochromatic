# @monochromatic-dev/dependency-catalog-cli-tighten.matrix

Containerised end-to-end matrix for `catalog-tighten`.
 A sidecar of
`package/dependency-catalog/cli-tighten`,
 it proves the tool resolves the active
installed version and tightens the catalog floor correctly across every pnpm
install layout,
 in isolated containers.

## What it covers

Each scenario installs a tiny fixture workspace (two consumer packages,
 both
depending on a pinned `picomatch`) under one pnpm layout,
 applies an optional
post-install mutation,
 then asserts the tool tightens `picomatch` from `>=4.0.0`
to `>=4.0.2`,
 reports a MISS or an UNDCL,
 or fails cleanly.

Layout and settings (expect tighten):

- `nodeLinker: isolated`,
   hoist on and off.
- `nodeLinker: hoisted`,
   hoist on and off.
- `nodeLinker: pnp` (pnpm's pnp is a hybrid that keeps per-importer `node_modules` symlinks).
- `nodeLinker: pnp` with `symlink: false` (pnpm's recommended pnp config):
   a true no-`node_modules`
  layout,
   resolved through `.pnp.cjs`;
   this is the case that exercises the PnP reader.
- `modulesDir` renamed,
   `virtualStoreDir` relocated,
   `enableGlobalVirtualStore`,
   `storeDir` relocated.
- Stale orphan:
   a higher `picomatch@4.0.4` is seeded into the virtual store with
  no symlink;
   the tool must tighten to the active `4.0.2`,
   never the orphan.
   This
  is the regression proving the old store-scan removal was correct
  (see `doc/troubleshooting/pnpm-modules-cache.md`).

Missing-X robustness:

- Missing lockfile,
   missing store,
   missing some `node_modules` (one consumer),
  missing pnpm (`pnpm config get` unavailable),
   and missing `.pnp.cjs` under the default pnp linker
  (its symlinks survive):
   still tighten,
   because resolution reads on-disk state.
- Missing virtual store (`node_modules/.pnpm` deleted,
   symlinks left dangling):
   MISS.
   Missing
  `.pnp.cjs` under `symlink: false` (no symlinks left to fall back on):
   MISS.
- Store-only orphan (package undeclared in both consumers and their `node_modules` removed,
   root `.pnpm`
  kept):
   UNDCL,
   because the package is present in the store but no importer declares it directly.
   A package
  a live importer still declares is never labelled undeclared,
   even when its resolution path is broken (the
  `symlink: false` plus missing `.pnp.cjs` case stays a MISS).
- Missing all `node_modules`,
   missing `pnpm-workspace.yaml`:
   fail cleanly with a clear error.

## How it runs

`test:matrix` runs the orchestrator,
 which for each combination launches a
podman container:

- The monorepo is bind-mounted read-only at `/repo`,
   so catalog-tighten and its
  `yaml` dependency resolve through the mounted symlinks without a build.
- The fixture is written and installed in a writable tmpfs at `/work`.
- pnpm is provisioned by corepack at the pinned version;
   the rootfs is
  read-only and caches go to tmpfs.
- Per-container memory,
   cpu,
   and pid caps bound resource use.

Network is required (the fixture install pulls from the registry),
 so this is
not part of the standard `test:unit` run.

## Usage

```sh
# Run the full container matrix (needs podman + network)
mise run //package/dependency-catalog/cli-tighten.matrix:test:matrix

# Fast host-only unit tests for the fixture builder
mise run //package/dependency-catalog/cli-tighten.matrix:test:unit
```

## Layout

- `src/combos.ts`:
   the combinations and the fixture file content (pure,
   unit-tested).
- `src/in-container.ts`:
   the per-combination entrypoint that runs inside a container.
- `src/matrix.unit.matrix.test.ts`:
   the orchestrator that launches one container per combination.

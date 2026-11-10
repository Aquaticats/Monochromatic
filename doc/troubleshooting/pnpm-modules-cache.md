# pnpm 11.9.0 retains stale node_modules/.pnpm entries under modulesCacheMaxAge

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

A project can have one active dependency version while `node_modules/.pnpm` still shows older package
versions.

Observed in this workspace:

```txt
# /var/home/user/Monochromatic
pnpm why --recursive @oxlint/plugins
@oxlint/plugins@1.71.0
├── @monochromatic-dev/oxlint-plugin-no-restricted-syntax@0.0.1 (dependencies)
├── @monochromatic-dev/oxlint-plugin-stylistic@0.0.1 (dependencies)
├── @monochromatic-dev/oxlint-plugin-tsdoc@0.0.1 (dependencies)
└── monochromatic (devDependencies)

Found 1 version of @oxlint/plugins
```

But the virtual store contained both versions:

```txt
# /var/home/user/Monochromatic
node_modules/.pnpm/@oxlint+plugins@1.70.0
node_modules/.pnpm/@oxlint+plugins@1.71.0
```

The direct package symlink still pointed at the active version:

```txt
# /var/home/user/Monochromatic
node_modules/@oxlint/plugins -> ../.pnpm/@oxlint+plugins@1.71.0/node_modules/@oxlint/plugins
```

`du` made the stale entries look like duplicate disk use:

```txt
# /var/home/user/Monochromatic
208K node_modules/.pnpm/@oxlint+plugins@1.70.0
208K node_modules/.pnpm/@oxlint+plugins@1.71.0
```

On this Btrfs filesystem,
 identical file data was actually shared by copy-on-write extents:

```txt
# /var/home/user/Monochromatic
filefrag -v node_modules/.pnpm/@oxlint+plugins@1.70.0/node_modules/@oxlint/plugins/index.d.ts \
  node_modules/.pnpm/@oxlint+plugins@1.71.0/node_modules/@oxlint/plugins/index.d.ts

0: 0..40: 464494..464534: 41: last,shared,eof
0: 0..40: 464494..464534: 41: last,shared,eof
```

The confusing part is therefore not only disk use.
It is that stale package versions remain visible in `node_modules/.pnpm`,
which can mislead manual inspection and can interact badly with tools that cache resolved paths.

## Root cause

pnpm intentionally keeps project-local virtual-store orphans for a configured age.
The default is seven days.

Source checked:

- pnpm repository:
   `https://github.com/pnpm/pnpm.git`
- clone path:
   `/tmp/agent/pnpm-20260629`
- commit:
   `a6f04d4ae50a133358371fb77d018d55837e6b19`

`pnpm11/installing/deps-installer/src/install/extendInstallOptions.ts:368-376`
defines the default:

```ts
// /tmp/agent/pnpm-20260629/pnpm11/installing/deps-installer/src/install/extendInstallOptions.ts
    verifyStoreIntegrity: true,
    enableModulesDir: true,
    virtualStoreOnly: false,
    modulesCacheMaxAge: 7 * 24 * 60,
    resolveSymlinksInInjectedDirs: false,
    dedupeDirectDeps: true,
```

`pnpm11/installing/deps-installer/src/install/index.ts:484-487` decides whether to prune the
virtual store.
With a positive `modulesCacheMaxAge`,
 pnpm compares the previous prune timestamp to the configured
age.
With `modulesCacheMaxAge: 0`,
 the `opts.modulesCacheMaxAge > 0` guard is false,
 so
`pruneVirtualStore` becomes `true` on every install:

```ts
// /tmp/agent/pnpm-20260629/pnpm11/installing/deps-installer/src/install/index.ts
  const pruneVirtualStore = !opts.enableGlobalVirtualStore && (ctx.modulesFile?.prunedAt && opts.modulesCacheMaxAge > 0
    ? cacheExpired(ctx.modulesFile.prunedAt, opts.modulesCacheMaxAge)
    : true
  )
```

`pnpm11/installing/deps-installer/src/install/index.ts:1221-1223` implements the age comparison in
minutes:

```ts
// /tmp/agent/pnpm-20260629/pnpm11/installing/deps-installer/src/install/index.ts
function cacheExpired (prunedAt: string, maxAgeInMinutes: number): boolean {
  return ((Date.now() - new Date(prunedAt).valueOf()) / (1000 * 60)) > maxAgeInMinutes
}
```

`pnpm11/installing/linking/modules-cleaner/src/prune.ts:132-194` computes virtual-store orphans by
comparing current lockfile package paths to wanted lockfile package paths.
If pruning is enabled,
 it removes each orphan path and then removes available virtual-store directories not in the wanted
package set:

```ts
// /tmp/agent/pnpm-20260629/pnpm11/installing/linking/modules-cleaner/src/prune.ts
  const currentPkgIdsByDepPaths = equals(selectedImporterIds, Object.keys(opts.wantedLockfile.importers))
    ? getPkgsDepPaths(opts.currentLockfile.packages ?? {}, opts.skipped)
    : getPkgsDepPathsOwnedOnlyByImporters(selectedImporterIds, opts.currentLockfile, opts.include, opts.skipped)
  const wantedPkgIdsByDepPaths = getPkgsDepPaths(wantedLockfile.packages ?? {}, opts.skipped)

  // Source line 137 wrapped here for markdown width.
  const orphanDepPaths = (Object.keys(currentPkgIdsByDepPaths) as DepPath[])
    .filter((path: DepPath) => !wantedPkgIdsByDepPaths[path])
  const orphanPkgIds = new Set(orphanDepPaths.map(path => currentPkgIdsByDepPaths[path]))
```

```ts
// /tmp/agent/pnpm-20260629/pnpm11/installing/linking/modules-cleaner/src/prune.ts
    if (opts.pruneVirtualStore !== false) {
      const _tryRemovePkg = tryRemovePkg.bind(null, opts.lockfileDir, opts.virtualStoreDir)
      await Promise.all(
        orphanDepPaths
          .map((orphanDepPath) => depPathToFilename(orphanDepPath, opts.virtualStoreDirMaxLength))
          .map(async (orphanDepPath) => _tryRemovePkg(orphanDepPath))
      )
      const neededPkgs = new Set<string>(['node_modules'])
      for (const depPath of Object.keys(opts.wantedLockfile.packages ?? {})) {
        if (opts.skipped.has(depPath as DepPath)) continue
        neededPkgs.add(depPathToFilename(depPath, opts.virtualStoreDirMaxLength))
      }
      const availablePkgs = await readVirtualStoreDir(opts.virtualStoreDir, opts.lockfileDir)
      await Promise.all(
        availablePkgs
          .filter((availablePkg) => !neededPkgs.has(availablePkg))
          .map(async (orphanDepPath) => _tryRemovePkg(orphanDepPath))
      )
    }
```

The behavior is intentional.
`pnpm/pnpm#3115` describes the feature goal as avoiding hardlink recreation when switching branches
and says an option should control how frequently the virtual store is pruned.
`pnpm/pnpm#3124` implemented that option.

## Verification

### Workspace configuration check

This workspace uses isolated pnpm linking and Btrfs clone-or-copy imports:

```txt
# /var/home/user/Monochromatic
pnpm config get node-linker
isolated

pnpm config get package-import-method
clone-or-copy

stat --file-system --format='fs=%T' node_modules/.pnpm/@oxlint+plugins@1.71.0
fs=btrfs
```

Before this fix,
 no project-level `modulesCacheMaxAge` was configured:

```txt
# /var/home/user/Monochromatic
pnpm config get modules-cache-max-age
undefined
```

### Benchmark harness

Benchmarks ran under `~/temp`,
 not `/tmp`,
 because `/tmp` is RAM-backed on this machine.
Every benchmark worktree had scripts disabled in config and at command invocation:

```yaml
# pnpm-workspace.yaml in each benchmark worktree
autoInstallPeers: false
ignoreScripts: true
modulesCacheMaxAge: 10080 # or 0
```

```sh
# benchmark invocation shape
pnpm install --ignore-scripts --no-frozen-lockfile --prefer-offline --reporter=append-only \
  --config.confirmModulesPurge=false
```

The true branch-switch benchmark used committed lockfile states and alternated:

```sh
# benchmark invocation shape
git checkout --detach <benchmark-commit>
pnpm install --ignore-scripts --prefer-offline --reporter=append-only \
  --config.confirmModulesPurge=false
```

Benchmark result files are recorded in the handover:
`../handover/pnpm-modules-cache-benchmark.md`.

### Benchmark results

Catalog edit benchmarks:

- `oxlint` toggle:
  default cache mean was 3826 ms;
  zero cache mean was 3837 ms.
- `stylelint` toggle:
  default cache mean was 3766 ms;
  zero cache mean was 3903 ms.
- `rolldown` toggle:
  default cache mean was 3794 ms;
  zero cache mean was 3802 ms.

True branch-switch `oxlint` benchmark:

- default cache total checkout plus install mean was 939 ms.
- zero cache total checkout plus install mean was 931 ms.

The branch-switch benchmark confirmed the visible-state difference:

```txt
# /home/user/temp/pnpm-branch-switch-bench-20260629-121207/branch-results.csv
"default-cache","1.71.0",55,888,942,718,"[\"@oxlint+plugins@1.70.0\",\"@oxlint+plugins@1.71.0\"]",12
"zero-cache","1.71.0",56,871,927,710,"[\"@oxlint+plugins@1.71.0\"]",6
```

Btrfs exclusive data stayed effectively unchanged even when stale entries were visible:

```txt
# /home/user/temp/pnpm-rolldown-bench-20260629-120805
btrfs filesystem du -s default-cache/node_modules/.pnpm zero-cache/node_modules/.pnpm
     Total   Exclusive  Set shared  Filename
   1.15GiB   372.00KiB     1.12GiB  default-cache/node_modules/.pnpm
   1.04GiB   372.00KiB     1.01GiB  zero-cache/node_modules/.pnpm
```

## Verified workarounds

### Set modulesCacheMaxAge to zero

Add this to `pnpm-workspace.yaml`:

```yaml
# pnpm-workspace.yaml
modulesCacheMaxAge: 0
```

This makes normal `pnpm install` prune virtual-store orphans every time.
Tradeoff:
branch switches and dependency downgrades lose pnpm's seven-day virtual-store orphan cache.
Measured cost in this workspace was negligible in three benchmark shapes and small in one
small-sample stylelint run.

### Run pnpm prune on demand

`pnpm11/installing/commands/src/prune.ts:50-56` shows `pnpm prune` forces
`modulesCacheMaxAge: 0` and also enables broader prune behavior:

```ts
// /tmp/agent/pnpm-20260629/pnpm11/installing/commands/src/prune.ts
export async function handler (
  opts: install.InstallCommandOptions
): Promise<void> {
  await install.handler({
    ...opts,
    modulesCacheMaxAge: 0,
    pruneDirectDependencies: true,
    pruneStore: true,
  })
}
```

Tradeoff:
this is manual or task-driven cleanup.
It also requests direct-dependency and store pruning,
so it is a larger operation than simply making normal installs prune virtual-store orphans.

### Remove node_modules and reinstall

A clean reinstall removes stale project-local virtual-store entries.
Tradeoff:
it is slower than pruning and throws away all project-local install layout,
not just stale entries.
Use a disposable worktree or a deliberate local cleanup command,
not a shared or production state target.

## What does not work

### pnpm why does not enumerate stale virtual-store entries

`pnpm why` reports the active dependency graph.
It correctly reported only `@oxlint/plugins@1.71.0` while `node_modules/.pnpm` still contained
`@oxlint+plugins@1.70.0`.
That means `pnpm why` is not a stale-entry detector.

### du does not prove unique disk usage on Btrfs

Plain `du` reported stale package directories as if each occupied full blocks.
`filefrag -v` and `btrfs filesystem du -s` showed shared extents and only 372 KiB exclusive data for
whole `.pnpm` trees in the benchmark worktrees.
Use Btrfs-aware accounting when the question is physical disk use.

### pnpm store prune is the wrong layer for this symptom

The symptom is in the project-local virtual store,
`node_modules/.pnpm`.
`pnpm store prune` targets the content-addressable store and,
when the global virtual store is enabled,
that global virtual store.
This repo has the default project-local virtual store,
so setting `modulesCacheMaxAge: 0` or running `pnpm prune` addresses the observed stale entries more
directly.

## Upstream filing decision

### Out-of-scope check

Checked `.out-of-scope/`.
No pnpm or modules-cache exemption exists.
The closest package-manager exemption is `.out-of-scope/bun-install.md`,
which explicitly says this workspace's package manager is pnpm,
not Bun.

### Duplicate search

Relevant upstream threads already exist:

- `pnpm/pnpm#3115`,
   closed:
  original feature request to stop pruning the modules directory on every install.
- `pnpm/pnpm#3124`,
   merged:
  implementation of the modules cache prune-frequency option.
- `pnpm/pnpm#11011`,
   open:
  large TypeScript monorepo reports stale `node_modules/.pnpm` paths causing type-check problems;
  maintainer suggests `modulesCacheMaxAge: 0`.
- `pnpm/pnpm#2694`,
   open:
  broader global-store automatic pruning discussion.
- `pnpm/pnpm#10132`,
   open:
  pruning and global virtual store inconsistency report.

Searches run:

```txt
# /var/home/user/Monochromatic
gh search issues --repo pnpm/pnpm '"modules cache"' --state open --limit 20
gh search issues --repo pnpm/pnpm '"modules cache"' --state closed --limit 20
gh search prs --repo pnpm/pnpm '"modules cache"' --state closed --limit 20
```

### Six-constraint check

1.  Is it really upstream's fault?
    No for the stale-entry default itself:
    retention is documented and implemented deliberately.
    Yes only for any downstream tool interaction that treats stale `.pnpm` paths as active.
2.  Can upstream fix it?
    Yes,
    but a fix would be a policy/default change or better stale-entry isolation,
    not a local correctness patch identified here.
3.  Are they supporting this use case?
    Yes.
    pnpm documents `modulesCacheMaxAge`,
    and `pnpm/pnpm#3115` says branch switching is the intended use case.
4.  Would the repo welcome our contribution?
    Likely yes for a well-reproduced bug:
    `CONTRIBUTING.md` describes setup,
    tests,
    and PR submission;
    `.github/ISSUE_TEMPLATE/bug-report.yaml` asks for latest-release verification and a reproduction;
    `.github/pull_request_template.md` requests tests and documentation.
    No policy banning AI-assisted reports was found in those files.
5.  Will they likely fix it?
    Not from this evidence alone.
    The default is an intentional performance tradeoff,
    and the open stale-path issue already has a maintainer-provided local setting workaround.
6.  Have we prototyped a minimal fix compatible with their architecture?
    No.
    Constraints one and five do not hold for the default behavior as a bug,
    so this doc does not trigger the auto-prototype step.

### Filing artifact

Do not file a new upstream issue for the default retention behavior.
It is by design and already has a configuration escape hatch.

Do not comment on `pnpm/pnpm#11011` from this work as-is.
The benchmark data supports this workspace's local choice,
but it does not reproduce that issue's TypeScript failure mode and would not add a root-cause trace
or failing fixture that upstream lacks.

If a future session reproduces the TypeScript stale-path failure with a minimal public fixture,
reopen the filing decision and draft an additive comment for `pnpm/pnpm#11011` rather than opening a
new issue.

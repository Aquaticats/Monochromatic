# pnpm modules cache benchmark handover

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Status

The user is questioning whether pnpm's project-local virtual-store orphan retention is worth its
machinery and bug surface.
They asked to continue investigating and benchmarking pnpm,
with benchmarks on filesystem-backed storage.
They explicitly noted `/tmp` is RAM-backed and `~/temp` is filesystem-backed.

This handover file is intentionally persisted in the main worktree at the user's request.
Keep updating it as the benchmark and interpretation change.

## Current benchmark setup

Benchmark root:

```txt
/home/user/temp/pnpm-cache-bench-20260629-120008
```

Worktrees created from `/var/home/user/Monochromatic` at `3fd737249`:

- `/home/user/temp/pnpm-cache-bench-20260629-120008/default-cache`
- `/home/user/temp/pnpm-cache-bench-20260629-120008/zero-cache`

Both are detached worktrees.
They are under `~/temp`,
 not `/tmp`,
 so benchmark IO lands on the filesystem-backed Btrfs volume.

Scripts are banned in both worktrees by adding `ignoreScripts: true` to `pnpm-workspace.yaml`.
The benchmark runner also invokes pnpm with `--ignore-scripts` and `npm_config_ignore_scripts=true`.

`modulesCacheMaxAge` differs intentionally:

- `default-cache`:
   `modulesCacheMaxAge: 10080`
- `zero-cache`:
   `modulesCacheMaxAge: 0`

The benchmark runner is:

```txt
/home/user/temp/pnpm-cache-bench-20260629-120008/bench.mjs
```

It changes the catalog entries for `oxlint` and `@oxlint/plugins` between exact `1.70.0` and
`1.71.0`,
 then runs:

```sh
pnpm install --ignore-scripts --no-frozen-lockfile --prefer-offline --reporter=append-only
```

Planned output paths:

```txt
/home/user/temp/pnpm-cache-bench-20260629-120008/results.json
/home/user/temp/pnpm-cache-bench-20260629-120008/results.csv
```

## Source evidence already collected

Official pnpm docs fetched from `https://pnpm.io/settings#modulescachemaxage` say:

- `nodeLinker: isolated` uses a virtual store at `node_modules/.pnpm`.
- `packageImportMethod: clone-or-copy` first tries CoW clones,
   then falls back to copying.
- `modulesCacheMaxAge` defaults to `10080`,
   which is seven days in minutes.
- `modulesCacheMaxAge` controls when orphan packages from the modules directory are removed.
- The stated reason is installation speed when switching branches or downgrading dependencies.

Official pnpm docs fetched from `https://pnpm.io/cli/store` say `pnpm store prune` removes
unreferenced packages from the global store and that pnpm intentionally does not remove them
automatically during normal installs.

Source clone:

```txt
/tmp/agent/pnpm-20260629
```

Relevant source paths in the clone:

- `pnpm11/installing/deps-installer/src/install/extendInstallOptions.ts`
  sets default `modulesCacheMaxAge: 7 * 24 * 60`.
- `pnpm11/installing/deps-installer/src/install/index.ts`
  computes `pruneVirtualStore` from `ctx.modulesFile.prunedAt` and `modulesCacheMaxAge`.
- `pnpm11/installing/linking/modules-cleaner/src/prune.ts`
  finds `orphanDepPaths` by comparing current lockfile packages to wanted lockfile packages.
  When `pruneVirtualStore !== false`,
   it removes orphan package directories from
  `node_modules/.pnpm` and removes available virtual-store directories not present in the wanted
  package set.
- `pnpm11/installing/deps-installer/test/install/modulesCache.ts`
  verifies that an uninstalled package remains in `.pnpm` until the configured modules cache age
  expires.
- `pnpm11/installing/commands/src/prune.ts`
  makes `pnpm prune` call install with `modulesCacheMaxAge: 0`,
   `pruneDirectDependencies: true`,
  and `pruneStore: true`.
  Update (2026-10-29):
   the handler passes a fourth option,
   `optimisticRepeatInstall: false`
   (`prune.ts:55-65`),
   and the `pnpm` 12 pacquet CLI that this repo's toolchain runs implements prune
   differently:
   a separate `run_prune` entry point with `prune_direct_dependencies: true`
   and no `modulesCacheMaxAge` or store override
   (`pnpm/crates/cli/src/cli_args/prune.rs:66-71`,
   `pnpm/crates/package-manager/src/install/entry_points.rs:71-84`).
   See `doc/troubleshooting/pnpm-prune-before-install.md`.

## Local observations already made

In the main worktree:

- `pnpm why --recursive @oxlint/plugins` reports only `@oxlint/plugins@1.71.0`.
- `node_modules/@oxlint/plugins` points to `../.pnpm/@oxlint+plugins@1.71.0/...`.
- `node_modules/.pnpm` contains both `@oxlint+plugins@1.70.0` and `@oxlint+plugins@1.71.0`.
- `pnpm-lock.yaml` contains `1.71.0`,
   not `1.70.0`.
- `@oxlint/plugins` package versions each contain only six files.
- `du` reports both directories as `208K`,
   but apparent size is `193K` for each.
- The filesystem is Btrfs.
- Matching files across the two package versions have identical SHA-256 hashes except
  `package.json`.
- `filefrag -v` on `index.d.ts` shows both versions share the same physical extent with
  the `shared` flag.

Interpretation so far:

- The duplicate `@oxlint/plugins` versions are not two active dependency versions.
- They are project-local virtual-store cache entries.
- On this repo's Btrfs setup with `packageImportMethod: clone-or-copy`,
   most identical file content
  is physically shared by CoW reflinks rather than hardlinks.
- `du` is not enough evidence of unique disk usage on this filesystem.
- The human confusion and stale-version noise remain real even when physical data blocks are shared.

## User preferences and constraints

- The user is skeptical of pnpm's cache benefit versus the machinery and bugs.
- Benchmark using new worktrees off this repo.
- Benchmarks must run under `~/temp`,
   not `/tmp`.
- Ban all scripts in benchmark worktrees after creating them.
- Changing dependency specs to exact versions inside benchmark worktrees is allowed.

## First benchmark result: oxlint 1.70.0 to 1.71.0 toggles

The first attempted runner under `/home/user/temp/pnpm-cache-bench-20260629-120008` failed because it
picked a pnpm executable that treated the lockfile as incompatible and then failed resolving a
`catalog:` dependency.
 A fresh run pinned the pnpm executable to
`/home/user/.local/share/mise/installs/pnpm/11.9.0/pnpm`.

Successful benchmark root:

```txt
/home/user/temp/pnpm-cache-bench-20260629-120304-fresh
```

Result files:

```txt
/home/user/temp/pnpm-cache-bench-20260629-120304-fresh/results.json
/home/user/temp/pnpm-cache-bench-20260629-120304-fresh/results.csv
```

Observed switch timings after the initial install:

- `default-cache`:
   3798 ms,
  3843 ms,
  3830 ms,
  3834 ms.
- `zero-cache`:
   3826 ms,
  3761 ms,
  3895 ms,
  3864 ms.

Interpretation:

- Mean switch time was effectively identical for this workload:
  about 3826 ms with default cache and about 3837 ms with zero cache.
- `default-cache` retained both oxlint versions after the first downgrade:
  virtual-store entry count rose from 710 to 718.
- `zero-cache` stayed at 710 entries and had only the active oxlint version each time.
- `du --block-size=1 node_modules/.pnpm` reported roughly 1.36 GB for `default-cache` after both
  versions were retained,
   versus roughly 1.25 GB or 1.246 GB for `zero-cache` depending on active
  oxlint version.
- `btrfs filesystem du -s` reported only 372 KiB exclusive data for each `.pnpm` directory,
   because
  most file extents are shared with the store or between reflinked copies.
- For this exact workload,
   `modulesCacheMaxAge: 0` removed stale-version noise with no measured
  install-time loss.

## Second benchmark result: stylelint 17.13.0 to 17.14.0 toggles

Benchmark root:

```txt
/home/user/temp/pnpm-stylelint-bench-20260629-120538
```

Result files:

```txt
/home/user/temp/pnpm-stylelint-bench-20260629-120538/results.json
/home/user/temp/pnpm-stylelint-bench-20260629-120538/results.csv
```

Observed switch timings after the initial install:

- `default-cache`:
   3813 ms,
  3733 ms,
  3742 ms,
  3776 ms.
- `zero-cache`:
   3941 ms,
  3836 ms,
  3981 ms,
  3854 ms.

Interpretation:

- Mean switch time:
   about 3766 ms with default cache and about 3903 ms with zero cache.
  The retained virtual-store entries saved about 137 ms per switch in this run,
  about 3.5% of the install step.
- `default-cache` retained both stylelint versions and both TypeScript peer variants:
  virtual-store entry count rose from 710 to 714.
- `zero-cache` stayed at 710 entries and had only the active stylelint version each time.
- `du --block-size=1 node_modules/.pnpm` showed default-cache about 4.16 MB above zero-cache
  when both stylelint versions were retained.
- `btrfs filesystem du -s` showed both worktree `.pnpm` directories with only 372 KiB exclusive
  data.
   The extra stylelint copies were shared extents.

## Third benchmark result: rolldown 1.1.2 to 1.1.3 toggles

Benchmark root:

```txt
/home/user/temp/pnpm-rolldown-bench-20260629-120805
```

Result files:

```txt
/home/user/temp/pnpm-rolldown-bench-20260629-120805/results.json
/home/user/temp/pnpm-rolldown-bench-20260629-120805/results.csv
```

Observed switch timings after the initial install:

- `default-cache`:
   3820 ms,
  3738 ms,
  3819 ms,
  3797 ms.
- `zero-cache`:
   3811 ms,
  3792 ms,
  3841 ms,
  3763 ms.

Interpretation:

- Mean switch time was effectively identical:
  about 3794 ms with default cache and about 3802 ms with zero cache.
- The `1.1.2` state contains both `rolldown@1.1.2` and `rolldown@1.1.3` even under
  `modulesCacheMaxAge: 0`,
   because `1.1.3` remains active through another dependency path.
  The `1.1.3` state is the valid prune comparison:
  `default-cache` retained both versions,
  while `zero-cache` pruned back to only `1.1.3`.
- `du --block-size=1 node_modules/.pnpm` showed about 1.36 GB for default-cache when both rolldown
  versions were retained versus about 1.246 GB for zero-cache after pruning back to `1.1.3`.
- `btrfs filesystem du -s` again showed both worktree `.pnpm` directories with only 372 KiB
  exclusive data,
  and retained rolldown package/binding directories with zero exclusive data.

## Fourth benchmark result: true branch-switch oxlint toggles

Benchmark root:

```txt
/home/user/temp/pnpm-branch-switch-bench-20260629-121207
```

Result files:

```txt
/home/user/temp/pnpm-branch-switch-bench-20260629-121207/branch-results.json
/home/user/temp/pnpm-branch-switch-bench-20260629-121207/branch-results.csv
```

Notes:

- The first attempt failed because the benchmark commits triggered the repo's pre-commit hook and
  `hk` was not on `PATH` in the scratch worktree environment.
- The rerun used `HK=0` for scratch benchmark commits only.
- Each benchmark worktree created two commits with exact oxlint catalog/lockfile states:
  `1.71.0` and `1.70.0`.
- The measured loop alternated `git checkout --detach <commit>` plus
  `pnpm install --ignore-scripts --prefer-offline --reporter=append-only`.

Branch-switch timing means across five switches:

- checkout time:
  `default-cache` 56.6 ms,
  `zero-cache` 56.8 ms.
- install time:
  `default-cache` 882.6 ms,
  `zero-cache` 873.4 ms.
- total checkout plus install time:
  `default-cache` 939.0 ms,
  `zero-cache` 930.6 ms.

Interpretation:

- In the realistic branch-switch shape,
  `modulesCacheMaxAge: 0` was about 8 ms faster overall,
  a 0.9% difference and well inside run noise.
- `default-cache` retained both oxlint versions throughout,
  with 718 virtual-store entries and 12 oxlint binding entries.
- `zero-cache` stayed at 710 virtual-store entries and six oxlint binding entries,
  matching only the active branch's oxlint version.

## Cross-benchmark summary

Switch-only mean timings:

- catalog-edit `oxlint`:
   default-cache 3826 ms,
  zero-cache 3837 ms.
- catalog-edit `stylelint`:
   default-cache 3766 ms,
  zero-cache 3903 ms.
- catalog-edit `rolldown`:
   default-cache 3794 ms,
  zero-cache 3802 ms.
- true branch-switch `oxlint`:
   default-cache 939 ms total checkout plus install,
  zero-cache 931 ms total checkout plus install.

Overall interpretation:

- In this repo on Btrfs with `packageImportMethod: clone-or-copy`,
   retaining stale virtual-store
  entries produced no meaningful speedup for oxlint,
  rolldown,
  or true branch-switch oxlint toggles,
  and a small stylelint speedup around 137 ms per install switch.
- The stylelint result is suggestive rather than statistically conclusive;
  the run count is small.
- Retention consistently increased visible `.pnpm` entry counts and left stale versions visible.
- `du` inflated the apparent cost of stale entries by about 4 MB to 115 MB depending on package
  family,
  while Btrfs exclusive-data accounting showed the extra retained entries consumed effectively no
  unique file data in these runs.
- The main cost is therefore not physical disk blocks on this machine;
  it is stale-version noise,
  confusing disk accounting,
  and bug surface with tools that cache paths into `.pnpm`.

## Relevant upstream issue evidence

- `pnpm/pnpm#3115` is the original closed feature issue for not pruning the modules directory on
  every install.
   It states the intended benefit:
  avoid recreating hardlinks when switching branches,
  and adds the automatic prune-frequency option.
- `pnpm/pnpm#11011` is an open issue reporting stale packages or links requiring recursive
  `node_modules` deletion in large TypeScript monorepos.
   A pnpm maintainer suggested setting
  `modulesCacheMaxAge` to `0`,
   explicitly saying stale packages inside `.pnpm` have no symlinks
  leading to them but TypeScript may have cached their locations.
- `pnpm/pnpm#2694` is an open issue about automatic global-store pruning.
   It shows the broader pnpm
  cleanup philosophy:
   avoid pruning constantly,
  but the tradeoff is extra state users may need to reason about.

## Applied follow-up

The user asked to write a troubleshooting doc and flip `modulesCacheMaxAge` to zero.
Completed changes:

- `doc/troubleshooting/pnpm-modules-cache.md` documents symptoms,
  pnpm source trace,
  benchmark verification,
  workarounds,
  and upstream filing decision.
- `pnpm-workspace.yaml` now sets `modulesCacheMaxAge: 0` near the top-level pnpm settings.

Verification after the config edit:

```txt
# /var/home/user/Monochromatic
pnpm config get modules-cache-max-age
0

pnpm install --lockfile-only --frozen-lockfile --ignore-scripts --reporter=append-only
Done in 231ms using pnpm v11.9.0

pnpm install --ignore-scripts --frozen-lockfile --prefer-offline --reporter=append-only \
  --config.confirmModulesPurge=false
Done in 727ms using pnpm v11.9.0

find node_modules/.pnpm -maxdepth 1 -type d \
  \( -name 'oxlint@1.70.0*' -o -name '@oxlint+plugins@1.70.0' -o \
  -name '@oxlint+binding-*1.70.0' \) -printf '%f\n' | sort | wc --lines
0
```

Relevant commits:

- `bf4bfee44 docs(pnpm): document modules cache retention`
- `7ae77227d build(pnpm): prune modules cache on install`

Main worktree still has unrelated dirty `mise.lock`.

## Current recommendation

For this repo,
 `modulesCacheMaxAge: 0` is now applied.
The measured performance cost was negligible in three benchmark shapes and showed only a suggestive
about-137 ms benefit in one small-sample stylelint run.
That small possible benefit did not look worth stale-version noise and TypeScript/path-cache risk
for this workspace.

## Next steps

- If continuing the investigation,
  test a true branch-switch benchmark rather than catalog edits,
  for example create two branches with committed `pnpm-workspace.yaml` changes and alternate
  `git checkout` plus `pnpm install` inside one worktree.
- If the user asks to apply the recommendation,
  edit `pnpm-workspace.yaml` in the main worktree to add `modulesCacheMaxAge: 0`,
  run the relevant install/check command,
  and commit the change separately from this handover.

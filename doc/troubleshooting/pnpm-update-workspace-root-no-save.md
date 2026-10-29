# pnpm 11.9.0 root `update --no-save` reports current while recursive importers are stale

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

From the repository root,
 these commands can look like they prove the workspace is fully current:

```shell
# /var/home/user/Monochromatic
pnpm update -i -w --no-save
pnpm update --no-save
```

The interactive command prints:

```text
All of your dependencies are already up to date inside the specified ranges. Use the --latest option to update the ranges in package.json
```

The non-interactive command prints:

```text
Already up to date
```

At the same time,
 a recursive inspection still reports stale workspace importers.
On 2026-06-26,
 `pnpm outdated --format json --recursive` reported these direct dependencies as stale:

```text
@types/node        26.0.0 -> 26.0.1
browserslist       4.28.2 -> 4.28.4
rolldown           1.1.2  -> 1.1.3
@oxlint/plugins    1.70.0 -> 1.71.0
expect-type        1.3.0  -> 1.4.0
oxlint             1.70.0 -> 1.71.0
stylelint          17.13.0 -> 17.14.0
@anthropic-ai/sdk  0.105.0 -> 0.106.0
```

## Root cause

There are two overlapping behaviors.

First,
 `-w` means `--workspace-root`,
 not `--workspace-wide`.
The parser rewrites the command directory to the workspace root when the flag is present.
From pnpm source,
 tag `v11.9.0`,
 commit `9671d9aeedb0039a114c2a2ff000170e51c61e3a`:

`pnpm11/cli/parse-cli-args/src/index.ts:228`

```ts
if (options['workspace-root']) {
  if (options['global']) {
    throw new PnpmError('OPTIONS_CONFLICT', '--workspace-root may not be used with --global')
  }
  if (!workspaceDir) {
    throw new PnpmError('NOT_IN_WORKSPACE', '--workspace-root may only be used inside a workspace')
  }
  options['dir'] = workspaceDir
}
```

The interactive update path uses `selectedProjectsGraph` only when recursive filtering already built one.
Otherwise it checks a single manifest at `opts.dir`.
With `-i -w` and no `--recursive`,
 that manifest is only the root package.

`pnpm11/installing/commands/src/update/index.ts:206`

```ts
const projects = (opts.selectedProjectsGraph != null)
  ? Object.values(opts.selectedProjectsGraph).map((wsPkg) => wsPkg.package)
  : [
    {
      rootDir: opts.dir as ProjectRootDir,
      manifest: await readProjectManifestOnly(opts.dir, opts),
    },
  ]
```

The non-interactive update path also becomes a selected-project update when it is run inside a workspace.
If no recursive selection exists,
 pnpm selects only the project containing `opts.dir`.
When invoked at the repository root,
 that is the root package.

`pnpm11/installing/commands/src/installDeps.ts:272`

```ts
if (opts.workspaceDir) {
  const selectedProjectsGraph = opts.selectedProjectsGraph ?? selectProjectByDir(allProjects, opts.dir)
  if (selectedProjectsGraph != null) {
    const sequencedGraph = sequenceGraph(selectedProjectsGraph)
```

The recursive path then operates on exactly the selected importers.

`pnpm11/installing/commands/src/recursive.ts:652`

```ts
function getImporters (opts: Pick<RecursiveOptions, 'selectedProjectsGraph' | 'ignoredPackages'>): Array<{ rootDir: ProjectRootDir, rootDirRealPath: ProjectRootDirRealPath }> {
  let rootDirs = Object.keys(opts.selectedProjectsGraph) as ProjectRootDir[]
  if (opts.ignoredPackages != null) {
    rootDirs = rootDirs.filter((rootDir) => !opts.ignoredPackages!.has(rootDir))
  }
  return rootDirs.map((rootDir) => ({ rootDir, rootDirRealPath: opts.selectedProjectsGraph[rootDir].package.rootDirRealPath }))
}
```

Second,
 the non-interactive `Already up to date` line is a package-count reporter summary.
It is printed when the current package has no added or removed packages,
 not when every workspace importer has been checked for latest versions.

`pnpm11/cli/default-reporter/src/reporterForClient/reportStats.ts:64`

```ts
return stats$.pipe(
  take((opts.cmd === 'install' || opts.cmd === 'install-test' || opts.cmd === 'add' || opts.cmd === 'update' || opts.cmd === 'dlx') ? 2 : 1),
  reduce((acc, log) => {
    if (typeof log['added'] === 'number') {
      acc['added'] = log['added']
    } else if (typeof log['removed'] === 'number') {
      acc['removed'] = log['removed']
    }
    return acc
  }, {} as { added?: number, removed?: number }),
  map((stats) => {
    if (!stats['removed'] && !stats['added']) {
      if (opts.cmd === 'link') {
        return Rx.NEVER
      }
      return Rx.of({ msg: 'Already up to date' })
```

`--no-save` is a separate behavior.
It prevents package and workspace manifest writes,
 including catalog range writes,
 but it still allows the lockfile to be rewritten.

`pnpm11/installing/commands/src/update/index.ts:333`

```ts
update: true,
updateToLatest: opts.latest,
updateMatching,
updatePackageManifest: opts.save !== false,
resolutionMode: opts.save === false ? 'highest' : opts.resolutionMode,
```

`pnpm11/installing/commands/src/installDeps.ts:449`

```ts
// `opts.save === false` (e.g. `--no-save`) means "don't persist anything
// from this install" — both package.json and the workspace manifest.
// Skip the pick so the info log doesn't claim entries were added that
// were never written; the next install will resurface them.
if (opts.save !== false && !opts.dryRun) {
```

The `recursiveInstall` setting is not the update switch.
The current settings page documents `recursiveInstall` under CLI settings,
 but pnpm source applies it only after a command has opted into recursive-by-default handling.
When that setting is `false`,
 pnpm still marks the command recursive and narrows the install with the `{.}...` filter.
That filter includes the current workspace project and its workspace dependency closure.

`pnpm11/pnpm/src/main.ts:248`

```ts
if (
  cmd != null && recursiveByDefaultCommands.has(cmd) &&
  typeof workspaceDir === 'string'
) {
  cliOptions['recursive'] = true
  config.recursive = true

  if (!config.recursiveInstall && !config.filter && !config.filterProd) {
    config.filter = ['{.}...']
  }
}
```

`pnpm update` is not a recursive-by-default command in pnpm 11.9.0.
The update module exports command names only;
 unlike the install module,
 it does not export `recursiveByDefault = true`.

`pnpm11/installing/commands/src/update/index.ts:94`

```ts
export const commandNames = ['update', 'up', 'upgrade']
```

`pnpm11/installing/commands/src/install.ts:109`

```ts
export const commandNames = ['install', 'i']

export const recursiveByDefault = true
```

## Verification

Version under test:

```shell
# /var/home/user/Monochromatic
mise exec -- pnpm --version
# 11.9.0
```

The current settings page was fetched on 2026-06-26.
It documents `recursiveInstall` as a pnpm version 11 `pnpm-workspace.yaml` setting with default `true`.
It also says `recursiveInstall: false` makes `pnpm install` exclusively build the package in the current directory.
A disposable workspace shows the implementation is broader than that wording:
 when `package/a` depends on workspace package `b`,
 `recursiveInstall: false` still selects both `a` and `b`.

```shell
# /tmp/pnpm-recursive-install-*/packages/a
pnpm install --ignore-scripts --reporter=append-only
# Scope: 2 of 3 workspace projects

find /tmp/pnpm-recursive-install-* -maxdepth 4 -type d -name node_modules -print | sort
# /tmp/pnpm-recursive-install-*/node_modules
# /tmp/pnpm-recursive-install-*/packages/a/node_modules
# /tmp/pnpm-recursive-install-*/packages/b/node_modules
```

Root-only check,
 clean:

```shell
# /var/home/user/Monochromatic
mise exec -- pnpm outdated --format json
# {}
```

Recursive check,
 stale:

```shell
# /var/home/user/Monochromatic
mise exec -- pnpm outdated --format json --recursive
# reports @types/node, browserslist, rolldown, @oxlint/plugins,
# expect-type, oxlint, stylelint, and @anthropic-ai/sdk
```

The root importer is already ahead for some cataloged packages.
For example,
 root `package.json` uses `catalog:` for `@types/node`,
 and the root importer already records `26.0.1`,
 while a workspace package still records `26.0.0`.

```yaml
# pnpm-lock.yaml
importers:
  .:
    devDependencies:
      '@types/node':
        specifier: 'catalog:'
        version: 26.0.1

  package/css/cli:
    devDependencies:
      '@types/node':
        specifier: 'catalog:'
        version: 26.0.0
```

A throwaway worktree verified that recursive update reaches the stale importers
and rewrites only `pnpm-lock.yaml` when `--no-save --lockfile-only` is used:

```shell
# /tmp/agent/monochromatic-pnpm-lockfile-only-20260626
/var/home/user/.local/share/mise/installs/pnpm/11.9.0/pnpm \
  update --recursive --no-save --lockfile-only --reporter append-only

git status --short
#  M pnpm-lock.yaml

git diff --stat -- pnpm-lock.yaml pnpm-workspace.yaml package.json
# pnpm-lock.yaml | 827 +++++++++++++++++++--------------------------------------
# 1 file changed, 274 insertions(+), 553 deletions(-)
```

The same throwaway run updated the lockfile catalog snapshot and package importers,
 for example:

```diff
# pnpm-lock.yaml
@@
     '@types/node':
       specifier: '>=25.9.2'
-      version: 26.0.0
+      version: 26.0.1
@@
   package/css/cli:
@@
       '@types/node':
         specifier: 'catalog:'
-        version: 26.0.0
+        version: 26.0.1
```

## Verified workarounds

Use recursive inspection when the question is workspace freshness:

```shell
# /var/home/user/Monochromatic
pnpm outdated --recursive
```

Tradeoff:
 `pnpm outdated` exits nonzero when it finds stale dependencies,
 so scripts must treat that exit code as expected when the command is used as a report.

Use recursive update when the goal is to update every workspace importer inside existing ranges:

```shell
# /var/home/user/Monochromatic
pnpm update --recursive --no-save
```

Tradeoff:
 without `--lockfile-only`,
 a fresh worktree also materializes `node_modules`
and may print bin-link warnings for workspace packages
whose build outputs do not exist yet.

Use the lockfile-only form when the goal is only the lockfile bump:

```shell
# /var/home/user/Monochromatic
pnpm update --recursive --no-save --lockfile-only
```

Tradeoff:
 this updates `pnpm-lock.yaml` without installing the new package contents into `node_modules`,
 so run a normal install later before relying on local runtime resolution.

Do not combine `--recursive` with `--workspace-root` unless the intended target is only the root importer.

## What does not work

`pnpm update -i -w --no-save` does not inspect workspace packages.
`-w` means the workspace root package.
It is useful when the root importer is the target,
 not when every package in `package/*/*` is the target.

`pnpm update --no-save` from the repository root also does not inspect every workspace package.
It selects the root package because the command is not recursive.

Setting `recursiveInstall` does not make `pnpm update` recursive.
That setting controls `pnpm install` behavior after install has opted into recursive-by-default handling.
It is not consulted as a workspace-wide update default.

The non-interactive `Already up to date` line is not proof that every workspace importer is current.
It is the default reporter message when package add/remove counters are both absent.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was checked on 2026-06-26.
No pnpm exemption file exists.
The checked files were:

```text
.out-of-scope/bun-install.md
.out-of-scope/cargo-workspace.md
.out-of-scope/claude-code-upstream-bugs.md
.out-of-scope/codex-harness.md
.out-of-scope/jsr.md
.out-of-scope/lightningcss.md
.out-of-scope/low-impact-typescript-formatting.md
.out-of-scope/module-es-monolith.md
.out-of-scope/pi-gpt55-long-context.md
.out-of-scope/terminal-title-fork-parity-tests.md
.out-of-scope/typescript-project-references.md
```

Duplicate search was run on 2026-06-26:

```shell
gh search issues --repo pnpm/pnpm "update workspace-root already up to date" --state open --limit 5
gh search issues --repo pnpm/pnpm "update workspace-root already up to date" --state closed --limit 5
gh search prs --repo pnpm/pnpm "update workspace-root already up to date" --state open --limit 5
gh search prs --repo pnpm/pnpm "update workspace-root already up to date" --state closed --limit 5
gh search issues --repo pnpm/pnpm "catalog update no-save workspace-root" --state open --limit 5
gh search issues --repo pnpm/pnpm "catalog update no-save workspace-root" --state closed --limit 5
gh search prs --repo pnpm/pnpm "catalog update no-save workspace-root" --state open --limit 5
gh search prs --repo pnpm/pnpm "catalog update no-save workspace-root" --state closed --limit 5
gh search issues --repo pnpm/pnpm "pnpm update Already up to date no-save recursive" --state open --limit 10
gh search issues --repo pnpm/pnpm "pnpm update Already up to date no-save recursive" --state closed --limit 10
gh search prs --repo pnpm/pnpm "pnpm update Already up to date no-save recursive" --state open --limit 10
gh search prs --repo pnpm/pnpm "pnpm update Already up to date no-save recursive" --state closed --limit 10
```

Those searches returned no matching issue or pull request.

Constraint check:

- Is it really upstream's fault?
   No for the main behavior.
   `--workspace-root`,
   non-recursive update scope,
   and `--no-save` lockfile behavior match pnpm's command model.
   The reporter wording can be surprising,
   but the checked commands were root scoped.
- Can upstream fix it?
   A wording change is possible,
   but the observed stale workspace importers are solved by invoking the recursive command.
- Are they supporting this use case?
   Yes.
   `pnpm update --recursive` is the supported workspace-wide update path.
- Would the repo welcome our contribution?
   The repository has `CONTRIBUTING.md` with setup,
   test,
   and PR instructions.
   `.github/ISSUE_TEMPLATE/bug-report.yaml` accepts bug reports
   and asks for latest-release verification plus reproduction steps.
   No AI-assistance ban was found in `CONTRIBUTING.md`,
   the issue template,
   or a pull request template search.
- Will they likely fix it?
   Not applicable because this diagnosis does not identify an upstream defect worth filing.
- Have we prototyped a minimal fix compatible with their architecture?
   No. The first constraint failed,
   so the auto-prototype gate does not fire.

No upstream issue or comment should be filed from this finding as-is.
The actionable local correction is to use `--recursive` for workspace freshness checks and updates.

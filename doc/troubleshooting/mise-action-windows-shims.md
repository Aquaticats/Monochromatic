# `mise run` cannot find installed tools on Windows with mise-action 4.0.1

> Toolchain status:
>  `doc/decision/mpm-toolchain-adoption.md` (2026-10-26) moved tool provisioning off
>  Mise `[tools]` and `mise.lock`,
>  so the `mise install node pnpm` exposure story in the `Symptom` and `Root cause`
>  sections is historical.
>  Live provisioning runs `mise run toolchain:install` against `toolchain.lock.json`,
>  which exposes binaries through `.cache/toolchain-bin`
>  (`doc/troubleshooting/mpm-toolchain-migration.md`);
>  Windows provisioning parity is unverified
>  (`doc/planning/mise-removal-coverage.md`,
>  section `Tool acquisition, versions, and bootstrap prerequisite`).

## Symptom

A Windows GitHub Actions job installed Node and pnpm successfully through `mise install node pnpm`,
but a later `mise run` task failed first on `tsdown` and then on a direct `node` invocation:

```text
mise WARN mise-shim.exe not found next to ...\mise.exe or on PATH, falling back to "file" shim mode
'node' is not recognized as an internal or external command
```

Linux and macOS jobs using the same workflow passed.

## Root cause

The workflow pinned `jdx/mise-action` commit `1648a7812b9aeae629881980618f079932869151`,
which is release `4.0.1` from 2026-03-22.
Its `src/index.ts` Windows extraction branch moved only `mise/bin/mise.exe`
from the release archive into the action's bin path.
The action then added mise's shims directory to `PATH`,
but `mise-shim.exe` was absent.

Upstream issue [jdx/mise-action#475](https://github.com/jdx/mise-action/issues/475)
reports the same warning and identifies the same single-file extraction line.
Upstream PR [jdx/mise-action#476](https://github.com/jdx/mise-action/pull/476)
added fresh-install and cache-repair logic for `mise-shim.exe`,
a Windows assertion,
and eighteen passing checks.
The fix was released in `mise-action` `4.1.0`.

## Resolution

Update the pinned action to `4.2.0` commit `e6a8b3978addb5a52f2b4cd9d91eafa7f0ab959d`.
This release contains the `mise-shim.exe` fix and remains pinned to an immutable commit.
On 2026-09-08 every workflow moved on to `4.3.0` commit `c2a87611a18de5b3828c5652fe268e992400cb5c`
 with `minimum_release_age: 1d`,
 for the reason recorded in `mise-action-version-ahead-of-release.md`;
 that release keeps the shim fix.

The plugin package provides host-acceptance tasks with separate Windows commands.
Unix uses direct Node invocations.
Windows invokes the action-installed mise executable through its stable `LOCALAPPDATA` location,
then uses `mise exec` to activate Node and,
for the external-consumer probe,
pnpm:

```toml
# package/oxlint-plugin/no-restricted-syntax/mise.toml
run = "node src/external-consumer.unit.test.ts"
run_windows = '"%LOCALAPPDATA%\mise\bin\mise.exe" exec node pnpm -- node src/external-consumer.unit.test.ts'
```

The build task uses the same Windows bootstrap and invokes the directly declared tsdown package entry through Node.
This avoids both pnpm's bare-command shim and mise's automatic child-shell tool activation.
The workflow runs these package tasks after its explicit build rather than `buildAndTest`,
which would launch a nested `mise` build from its Node task body.

## Verification

The failing Windows jobs showed all of these facts in one log:

- `mise install node pnpm` completed;
- the workspace install reported `tsdown 0.22.5`;
- the action warned that `mise-shim.exe` was absent;
- plain `mise run` could not resolve either `tsdown` or `node`.

The replacement workflow must pass these consumer-boundary steps on `windows-latest`:

- build the plugin artifact through its package `mise` task;
- run the TypeScript 7 lifecycle and path tests through package host-acceptance task;
- pack and exercise the artifact from an external consumer.

## What does not work

- Declaring `tsdown` directly fixes dependency ownership but not mise's missing Windows runtime shim.
- A full pnpm workspace install does not repair `mise-shim.exe`.
- Wrapping `mise run` in `mise exec node --` does not help when the nested task shell reconstructs a Windows environment
  without the runtime path.
- Invoking `mise` again inside a package task has the same nested-shell problem.
- `mise run --tool node` still left `node` undiscoverable in the observed Windows child task shell.
- Invoking `node` directly inside a package task still fails unless the Windows command bootstraps it through absolute
  mise executable.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?
   ** Yes.
   `mise-action` `4.0.1` discarded `mise-shim.exe` from the Windows archive.
2. **Can upstream fix it?
   ** Yes.
   The action can install and repair the shim beside `mise.exe`.
3. **Are they supporting this use case?
   ** Yes.
   The action documents Windows and defaults to adding the shims directory to `PATH`.
4. **Would the repo welcome our contribution?
   ** Already resolved upstream.
5. **Will they likely fix it?
   ** The fix merged on 2026-05-14 and shipped in `4.1.0`.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** Upstream PR 476 supplied and tested the
   architecture-compatible fix.

Nothing should be filed upstream because issue 475 and PR 476 already resolved the defect.

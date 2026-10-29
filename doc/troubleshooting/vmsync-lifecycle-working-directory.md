# Vmsync lifecycle task resolves its mvm command from the wrong directory

## Symptom

At immutable source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`,
Depot run `99tl7zjfgp` built the vmsync bundle successfully,
then ran `mise run //package/vmsync/cli:test:e2e`.
Node reported `MODULE_NOT_FOUND` for a path ending in:

```text
source/package/vmsync/cli/package/mvm/cli/src/cli.ts
```

The failure precedes VM creation and the lifecycle assertions.
KVM availability cannot repair that relative path.

## Root cause

`package/vmsync/cli/mise.toml:42` launches the source test from the package's task directory:

```toml
# package/vmsync/cli/mise.toml at ef01490f5e
[tasks.'test:e2e']
description = "Run vmsync VM lifecycle tests (creates VMs, requires KVM)"
depends = ["build:js:node"]
run = "node src/lifecycle.expensive.unit.test.ts"
```

`package/vmsync/cli/src/lifecycle.expensive.unit.test.ts:36` defines a root-relative command argument:

```typescript
// package/vmsync/cli/src/lifecycle.expensive.unit.test.ts:36 at ef01490f5e
const MVM_ARGS = ['package/mvm/cli/src/cli.ts',];
```

The `nanoSpawn` call at line 75 passes a timeout and no working directory override.
The resulting path in Node's error directly establishes the incompatible resolution boundary.
The main worktree still contained these same relative command arguments when inspected on 2026-10-16.

## Verification and proposed repair

The cloud log retains the exact package-task failure after a successful build.
The same run successfully invoked the actual `mvm` source entry point by absolute path with `--help`.
That is a command-resolution positive control,
not a passing lifecycle suite.

The trial VM adapter preserves the original task result,
then attempts the unchanged test source from the repository root after guest prerequisites are prepared.
It never allows that supplementary experiment to turn an original task failure into success.
The later recovery created and destroyed its private Ubuntu guest successfully.
Its Windows template verification then failed before the supplementary source-suite command was reached.
Those [environment results](depot-vm-runtime-paths.md#separate-windows-template-verification-timeout)
do not execute either original lifecycle test.

## Repository-root consumer boundary

The 2026-10-17 source audit preserves `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`.
The additional command uses the same file and no test arguments:

```sh
# Immutable source repository root, in an isolated disposable consumer environment.
node /absolute/source/package/vmsync/cli/src/lifecycle.expensive.unit.test.ts
```

The absolute outer file path and repository-root working directory resolve the helper command
without changing the original authored task.
The bundle path also remains bound to the original package:
`package/vmsync/cli/src/lifecycle.expensive.unit.test.ts:44` to line 51 derives it from the source file.

```typescript
// package/vmsync/cli/src/lifecycle.expensive.unit.test.ts:44 at ef01490f5e
const PKG_ROOT = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/vmsync-cli',),
},);
const BUNDLE_PATH = join(PKG_ROOT, 'dist', 'final', 'node', 'index.mjs',);
```

This file has no Linux-only setup selection.
Linux guest creation and artifact transfer occur at lines 150 to 158,
before its `describe` at line 185.
After the Linux block,
Windows creation is unconditional at lines 319 to 327,
before its own `describe` at line 379:

```typescript
// package/vmsync/cli/src/lifecycle.expensive.unit.test.ts:319 at ef01490f5e
await safeDestroy(VM,);
await mvm({
  args: ['create', VM, '--image', 'windows',],
  timeout: CREATE_TIMEOUT_MS,
},);
await mvm({ args: ['push', VM, BUNDLE_PATH, 'index.mjs',], },);
```

The outer path/cwd bridge therefore cannot establish a complete unchanged-file pass
while Windows fixtures remain excluded.
Selecting a test name cannot avoid setup that runs before that test is registered.
Any direct natural-bundle CLI checks are separate artifact consumption;
they do not mark `vmsync lifecycle (Linux) > full CLI lifecycle`
or `vmsync lifecycle (Windows) > full CLI lifecycle` as executed.

The owned bounded probe is
`~/temp/agent/depot-vm-artifact-bridge-20261017/mise.toml`.
It uses the private exact-source fixture from the cohort controls,
with a 2 GiB memory limit and two CPUs.
It does not write either shared historical checkout.
The natural frozen dependency install and `//package/vmsync/cli:build:js:node` remain source-owned tasks.
No emulator or provider run is dispatched by this probe.

### Actual natural-build and artifact controls

The bounded probe completed the frozen source install and natural `build:js:node` task.
Its 502093-byte bundle has SHA-256
`d40a77b4ce8f3387b1b1d921f1c592ab952cc2a396d4c6e523b85458f65455b6`.
The build/install container's measured peak was 1776865280 bytes against its 2147483648-byte limit,
a fraction of 0.8274.
These are functional controls;
the timings are not a provider performance comparison.

The actual authored `test:e2e` command rebuilt the bundle and reproduced `MODULE_NOT_FOUND`.
The unchanged absolute test file from the repository root passed that resolution boundary,
then its actual Linux `mvm create` command stopped at `mvm: fetch failed`
under the deliberate `--network=none` constraint.
No image was downloaded,
neither lifecycle body executed,
and no Windows fixture or guest was created.

The copied natural bundle passed standalone `--help` and empty `list` checks
without source,
`node_modules`,
or network access.
The final VM workflow registers this as a separate,
cohort-bound [artifact consumer](depot-vm-runtime-paths.md#standalone-natural-bundle-consumer).
It runs after the failed original command only when the actual build and source/cohort guards succeeded.
The source gate remains failed.
The [retained controls](../audit/depot-vm-artifact-consumer-2026-10-17.evidence.json)
keep the root-cwd download stop and the standalone bundle result separate.

Guest cleanup also requires an outer guard:
the Linux cleanup declaration at line 184 follows its setup commands,
and the Windows cleanup declaration at line 378 does the same.
A setup failure can precede both declarations.
The immutable `mvm` defaults are 8192 MiB and four vCPUs
in `package/mvm/cli/src/config.ts:38` and line 43;
the source test does not pass resource overrides to its `create` calls.

A production repair should derive the `mvm` entry point from an owned absolute source location
or set an explicit repository-root command directory.
That change is not part of the historical-source measurement.
Verification must exercise both Linux and Windows lifecycle assertions in disposable guests,
and cleanup must run even when guest setup fails.

## What does not work

- Installing libvirt or enabling KVM does not change Node's command path resolution.
- Treating a successful vmsync build as lifecycle-test coverage misses the failed invocation.
- Running the fixture against the user's existing libvirt session risks its fixed test domain names.

## Upstream filing decision

1.  This is a repository test-path defect,
    not a demonstrated Node or mise defect.
2.  The repository can derive an absolute entry point.
3.  The documented package task is the affected supported invocation.
4.  External contribution policy is inapplicable to a local-source finding.
5.  The observed impact is failure before VM assertions execute.
6.  The cloud failure and absolute-path help control are retained;
    full lifecycle recovery is not established.

The repository title search for `vmsync` found existing naming and refactoring issues,
but no duplicate of this working-directory failure.
No external upstream filing or draft is warranted.

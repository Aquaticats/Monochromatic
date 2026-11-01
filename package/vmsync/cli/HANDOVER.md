# Vmsync test implementation handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

## What's done

### Unit tests (45 tests, all passing)

- `src/config.unit.test.ts`:
   `validateName`,
   `stripJsoncComments`,
   `vmDir`,
   `vmConfigPath`,
   `detectHypervisor`
- `src/boot.unit.test.ts`:
   `parseMemoryToBytes` (exported for testing)
- `src/import.unit.test.ts`:
   `nameFromPath` (exported for testing)

Run unit tests with `mise run //package/cli/vmsync:test:unit`.
Run lifecycle tests with `mise run //package/cli/vmsync:test:e2e`.

### Linux lifecycle e2e test (passing)

`src/lifecycle.expensive.unit.test.ts` contains the Linux lifecycle test.
It is named `.unit.test.ts` because `.e2e.test.ts` is reserved for Playwright in this monorepo.
It uses sequential test structure because VM lifecycle tests share expensive external state.

Flow:
 creates Ubuntu VM via mvm,
 pushes tsdown bundle via virtiofs,
installs mise+Node inside guest,
 runs all vmsync subcommands.

Run:
 `mise run //package/cli/vmsync:test:e2e`

### Windows lifecycle e2e test

Implemented in `src/lifecycle.expensive.unit.test.ts`.

Current flow:

1. Creates `vmsync-e2e-win` via `mvm create --image windows`.
2. Pushes the bundled vmsync entry point via `mvm push`.
3. Installs the Visual C++ runtime because `mise.exe` requires `vcruntime140.dll`.
4. Downloads the latest mise Windows zip via the GitHub release redirect.
5. Installs Node through mise.
6. Exercises help,
    list,
    status,
    config,
    and already-synced sync behavior.

The test assumes VirtioFsSvc maps the `mvm-shared` virtiofs tag to `Z:\`.

## What changed in mvm

### virtiofs shared directory

- `domain-xml.ts`:
   added `sharedDir` param,
   `<filesystem>` + `<memoryBacking>` elements
- `cloud-init.ts`:
   auto-mounts `mvm-shared` at `/mnt/shared` via cloud-init `mounts` directive
- `create.ts` + `clone.ts`:
   create `shared/` directory per VM,
   pass to `domainXml`
- `config.ts`:
   added `SHARED_DIR_NAME`,
   `GUEST_MOUNT_POINT`,
   `WINDOWS_GUEST_MOUNT_POINT` constants

### push/pull CLI commands

- `file-transfer.ts`:
   `pushFile` (host copy to shared dir),
   `pullFile` (read from shared dir)
- `index-parsers-cmds.ts` + `index-parsers.ts`:
   added `pushCmd`,
   `pullCmd` parsers + union members
- `index.ts`:
   dispatch handlers for push/pull
- `package.json`:
   added `./file-transfer` export

Performance:
 95 MB in ~90 ms via virtiofs (vs minutes via guest agent protocol).

### Windows template changes

- `autounattend-virtio.ts`:
   includes `viofs` in `VIRTIO_DRIVER_DIRS`
- `template-windows.ts`:
   downloads the Windows ISO,
   virtio-win ISO,
   and WinFsp MSI
- `template-windows.ts`:
   installs WinFsp and VirtioFsSvc dependencies via guest agent after QEMU-GA is reachable
- `autounattend.ts`:
   does not embed mise.
   Windows lifecycle tests install mise at runtime.

## Bug fix found

`import.ts` was missing `mkdir` for the VM directory before calling `qemu-img convert`.
Added `import { mkdir } from 'node:fs/promises'` and `await mkdir(dir, { recursive: true })`
before `convertSourceImage`.
 This is a real bug,
 not test-specific.

## Build setup

- `tsdown.node.config.ts`:
   bundles `@monochromatic-dev/*`,
   `@optique/*`,
   `nano-spawn`
- `mise.toml`:
   `build:js:node` task,
   `test:e2e` depends on it
- `package.json`:
   added `@monochromatic-dev/config-tsdown`,
   `find-up` as devDependencies

## Key files

<table>
<thead>
<tr>
<th>File</th>
<th>Purpose</th>
</tr>
</thead>
<tbody>
<tr>
<td>`package/cli/vmsync/src/config.unit.test.ts`</td>
<td>Pure function unit tests</td>
</tr>
<tr>
<td>`package/cli/vmsync/src/boot.unit.test.ts`</td>
<td>parseMemoryToBytes tests</td>
</tr>
<tr>
<td>`package/cli/vmsync/src/import.unit.test.ts`</td>
<td>nameFromPath tests</td>
</tr>
<tr>
<td>`package/cli/vmsync/src/lifecycle.expensive.unit.test.ts`</td>
<td>Linux + Windows expensive lifecycle</td>
</tr>
<tr>
<td>`package/cli/vmsync/tsdown.node.config.ts`</td>
<td>Bundle config</td>
</tr>
<tr>
<td>`package/cli/vmsync/mise.toml`</td>
<td>Task definitions</td>
</tr>
<tr>
<td>`package/cli/mvm/src/file-transfer.ts`</td>
<td>virtiofs push/pull</td>
</tr>
<tr>
<td>`package/cli/mvm/src/domain-xml.ts`</td>
<td>virtiofs XML generation</td>
</tr>
<tr>
<td>`package/cli/mvm/src/autounattend-virtio.ts`</td>
<td>viofs driver addition</td>
</tr>
</tbody>
</table>

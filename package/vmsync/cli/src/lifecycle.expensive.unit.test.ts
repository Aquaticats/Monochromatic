/**
 End-to-end lifecycle tests for the vmsync CLI on Linux and Windows guests.
 
 Named `*.unit.test.ts` (not `*.e2e.test.ts`) because `*.e2e.test.ts`
 is reserved for Playwright browser tests in this monorepo, and expensive
 unit tests are included only when the package task opts into them.
 
 Uses mvm to create ephemeral VMs, pushes the tsdown-bundled vmsync
 entry point via virtiofs, installs mise and Node inside each guest,
 and exercises subcommands against real disk images.
 
 Run via: `mise run //package/vmsync/cli:test:e2e`
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { join, } from 'node:path';

import {
  findRootCached,
  packageNamed,
} from '@monochromatic-dev/module-fs-path/ts';
import nanoSpawn from 'nano-spawn';

//region Constants

/** Path to the mvm CLI runtime. */
const MVM = 'node';

/** Timeout for VM creation (includes cloud-init / guest agent wait). */
const CREATE_TIMEOUT_MS = 180_000;

/** Timeout for individual mvm exec commands. */
const EXEC_TIMEOUT_MS = 120_000;

/** Absolute path to the vmsync package root: the nearest ancestor of this file whose manifest names the package. */
const PKG_ROOT = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/vmsync-cli',),
},);

/** Args prefix for running mvm from any working directory. */
const MVM_ARGS = [join(PKG_ROOT, '..', 'mvm', 'src', 'cli.ts'),];

/** Absolute path to the tsdown bundle. */
const BUNDLE_PATH = join(PKG_ROOT, 'dist', 'final', 'node', 'index.mjs',);

//endregion Constants

//region Helpers

/**
 Runs an mvm CLI command and returns stdout.
 
 @param args - Arguments after `mvm`
 
 @param timeout - Optional timeout in ms
 
 @returns Trimmed stdout
 */
async function mvm(
  {
    args,
    timeout = EXEC_TIMEOUT_MS,
  }: {
    args: readonly string[];
    timeout?: number;
  },
): Promise<string> {
  const { stdout, } = await nanoSpawn(
    MVM,
    [...MVM_ARGS, ...args,],
    { timeout, },
  );
  return stdout.trim();
}

/**
 Runs a command inside a VM and returns stdout.
 
 @param vmName - VM name
 
 @param command - Shell command to execute inside the VM
 
 @param timeout - Optional timeout in ms
 
 @returns Trimmed stdout
 */
async function execInVm(
  {
    vmName,
    command,
    timeout = EXEC_TIMEOUT_MS,
  }: {
    vmName: string;
    command: string;
    timeout?: number;
  },
): Promise<string> {
  return mvm({
    args: ['exec', vmName, command,],
    timeout,
  },);
}

/**
 Destroys a VM, ignoring errors if it does not exist.
 
 @param vmName - VM name to destroy
 */
async function safeDestroy(vmName: string,): Promise<void> {
  try {
    await mvm({ args: ['destroy', vmName,], },);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // VM may not exist
  }
}

//endregion Helpers

//region Linux lifecycle tests

if (process.env.VMSYNC_E2E_GUEST !== 'windows') {
  /** VM name for the Linux lifecycle test. */
  const VM = 'vmsync-e2e-linux';

  /** Shell preamble to set HOME and activate mise-managed Node. */
  const MISE =
    'export HOME=/home/ubuntu && eval "$(/home/ubuntu/.local/bin/mise activate bash)"';

  // beforeAll setup
  await safeDestroy(VM,);
  await mvm({
    args: ['create', VM,],
    timeout: CREATE_TIMEOUT_MS,
  },);

  // Push the tsdown bundle. Without virtiofsd the guest-agent route needs an
  // absolute guest path; with virtiofs the command returns the mounted path.
  const pushOutput = await mvm({
    args: ['push', VM, BUNDLE_PATH, '/home/ubuntu/index.mjs',],
  },);
  /** Guest path to the vmsync bundle, as reported by mvm push. */
  const BUNDLE = (/ -> (\S+) in VM /u.exec(pushOutput))?.[1] ?? '/mnt/shared/index.mjs';

  /**
   Guest command to run vmsync with sudo.
   All commands use sudo for consistent HOME since import requires
   root for qemu-nbd, and non-root commands must read data from the same location.
   */
  const VMSYNC = `${MISE} && sudo -E "$(which node)" ${BUNDLE}`;

  // Install the runtime library the mise-managed Node needs before Node starts.
  await execInVm({
    vmName: VM,
    command:
      'sudo apt-get update -qq && sudo apt-get install -y -qq libatomic1 > /dev/null 2>&1',
    timeout: EXEC_TIMEOUT_MS,
  },);

  // Install mise, then use mise to install Node (handles arch detection).
  // guest-exec runs without login shell so HOME is unset; export it explicitly.
  await execInVm({
    vmName: VM,
    command:
      'export HOME=/home/ubuntu && curl -fsSL https://mise.jdx.dev/install.sh | sh',
    timeout: EXEC_TIMEOUT_MS,
  },);
  await execInVm({
    vmName: VM,
    command: 'export HOME=/home/ubuntu && /home/ubuntu/.local/bin/mise use -g node@latest',
    timeout: EXEC_TIMEOUT_MS,
  },);

  // Install qemu-img, fdisk, and nbd module for disk image operations
  await execInVm({
    vmName: VM,
    command:
      'sudo apt-get update -qq && sudo apt-get install -y -qq qemu-utils fdisk nbd-client libatomic1 > /dev/null 2>&1',
    timeout: EXEC_TIMEOUT_MS,
  },);
  // Pre-load nbd kernel module (needed for vmsync import UEFI validation)
  await execInVm({ vmName: VM, command: 'sudo modprobe nbd max_part=0', },);

  await using _cleanup = { [Symbol.asyncDispose]: () => safeDestroy(VM,), };
  await describe({
    name: 'vmsync lifecycle (Linux)',
    children: [
      // All assertions in a single test to guarantee sequential execution.
      // The test task may run files independently;
      // these commands have ordering dependencies (import before status, etc.).
      it({
        name: 'full CLI lifecycle',
        fn: async () => {
          //region --help
          const help = await execInVm({ vmName: VM, command: `${VMSYNC} --help`, },);
          expect(help,).toContain('vmsync',);
          expect(help,).toContain('import',);
          expect(help,).toContain('boot',);
          expect(help,).toContain('sync',);
          expect(help,).toContain('status',);
          expect(help,).toContain('list',);
          expect(help,).toContain('config',);
          //endregion --help

          //region list (empty)
          const emptyList = await execInVm({ vmName: VM, command: `${VMSYNC} list`, },);
          expect(emptyList,).toContain('no managed VMs',);
          //endregion list (empty)

          //region Create test UEFI image
          await execInVm({
            vmName: VM,
            command: [
              'qemu-img create -f raw /tmp/test-uefi.raw 512M',
              String
                .raw`printf "g\nn\n1\n2048\n+100M\nt\n1\nw\n" | fdisk /tmp/test-uefi.raw`,
              'qemu-img convert -f raw -O qcow2 /tmp/test-uefi.raw /tmp/test-uefi.qcow2',
            ]
              .join(' && ',),
          },);
          //endregion Create test UEFI image

          //region import --name
          const importOutput = await execInVm({
            vmName: VM,
            command: `${VMSYNC} import /tmp/test-uefi.qcow2 --name test-vm`,
          },);
          expect(importOutput,).toContain('imported "test-vm"',);
          //endregion import --name

          //region list (populated)
          const populatedList = await execInVm({ vmName: VM,
            command: `${VMSYNC} list`, },);
          expect(populatedList,).toContain('test-vm',);
          expect(populatedList,).toContain('synced',);
          //endregion list (populated)

          //region status
          const status = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          expect(status,).toContain('name:       test-vm',);
          expect(status,).toContain('synced:     true',);
          expect(status,).toContain('last boot:  never',);
          expect(status,).toContain('qcow2 hash: sha256:',);
          expect(status,).toContain('vhdx hash:  sha256:',);
          //endregion status

          //region config update
          const configOutput = await execInVm({
            vmName: VM,
            command: `${VMSYNC} config test-vm --memory 8G --cpus 8`,
          },);
          expect(configOutput,).toContain('memory=8G',);
          expect(configOutput,).toContain('cpus=8',);

          const statusAfterConfig = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          expect(statusAfterConfig,).toContain('memory:     8G',);
          expect(statusAfterConfig,).toContain('cpus:       8',);
          //endregion config update

          //region config partial update (memory only, cpus preserved)
          await execInVm({ vmName: VM,
            command: `${VMSYNC} config test-vm --memory 2G`, },);
          const statusAfterPartial = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          expect(statusAfterPartial,).toContain('memory:     2G',);
          expect(statusAfterPartial,).toContain('cpus:       8',);
          //endregion config partial update

          //region import with auto-derived name
          const autoNameOutput = await execInVm({
            vmName: VM,
            command: `${VMSYNC} import /tmp/test-uefi.qcow2`,
          },);
          expect(autoNameOutput,).toContain('imported "test-uefi"',);
          //endregion import with auto-derived name

          //region sync (already synced)
          const syncOutput = await execInVm({ vmName: VM,
            command: `${VMSYNC} sync test-vm`, },);
          expect(syncOutput,).toContain('already synced',);
          //endregion sync (already synced)
        },
      },),
    ],
  },);
}

//endregion Linux lifecycle tests

//region Windows lifecycle tests

if (process.env.VMSYNC_E2E_GUEST !== 'linux') {
  /** VM name for the Windows lifecycle test. */
  const VM = 'vmsync-e2e-win';

  /**
   Guest path to vmsync bundle on the virtiofs mount.
   VirtioFsSvc maps the `mvm-shared` virtiofs tag to `Z:` by default.
   */
  const BUNDLE = String.raw`Z:\index.mjs`;

  /** Full path to mise binary installed at test runtime. */
  const MISE_BIN = String.raw`C:\Users\Administrator\.local\bin\mise.exe`;

  /**
   PowerShell preamble to activate mise-managed Node.
   Uses `mise which node | Split-Path` to get the bin directory because
   `mise where` returns the install root (missing the `bin` subdirectory).
   Guest agent runs as SYSTEM so mise installs to systemprofile, which is fine.
   */
  const MISE =
    `$env:PATH = ((& "${MISE_BIN}" which node 2>$null) | Split-Path) + ";" + $env:PATH`;

  /** Guest command to run vmsync via mise-managed Node on Windows. */
  const VMSYNC = `${MISE}; node ${BUNDLE}`;

  // beforeAll setup
  await safeDestroy(VM,);
  await mvm({
    args: ['create', VM, '--image', 'windows',],
    timeout: CREATE_TIMEOUT_MS,
  },);

  // Push the tsdown bundle via virtiofs (instant)
  await mvm({ args: ['push', VM, BUNDLE_PATH, 'index.mjs',], },);

  // Install Visual C++ runtime (required by mise.exe, not present on Server Core).
  await execInVm({
    vmName: VM,
    command: [
      '$ProgressPreference = "SilentlyContinue"',
      String
        .raw`Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vc_redist.x64.exe" -OutFile "$env:TEMP\vc_redist.x64.exe"`,
      String
        .raw`Start-Process -FilePath "$env:TEMP\vc_redist.x64.exe" -ArgumentList "/install","/quiet","/norestart" -Wait`,
    ]
      .join('; ',),
    timeout: EXEC_TIMEOUT_MS,
  },);

  // Download and install mise at test runtime.
  // Resolves the latest version tag via GitHub redirect, then downloads
  // the versioned zip to ensure we always get the actual latest release.
  await execInVm({
    vmName: VM,
    command: [
      '$ProgressPreference = "SilentlyContinue"',
      String.raw`$env:HOME = "C:\Users\Administrator"`,
      String.raw`$dir = "C:\Users\Administrator\.local\bin"`,
      'New-Item -ItemType Directory -Path $dir -Force | Out-Null',
      '$r = Invoke-WebRequest -Uri "https://github.com/jdx/mise/releases/latest" -MaximumRedirection 0 -UseBasicParsing -ErrorAction SilentlyContinue',
      '$version = ($r.Headers.Location -split "/tag/")[1]',
      '$url = "https://github.com/jdx/mise/releases/download/$version/mise-$version-windows-x64.zip"',
      String
        .raw`Invoke-WebRequest -Uri $url -OutFile "$env:TEMP\mise.zip" -UseBasicParsing`,
      String
        .raw`Expand-Archive -Path "$env:TEMP\mise.zip" -DestinationPath "$env:TEMP\mise" -Force`,
      String
        .raw`Copy-Item "$env:TEMP\mise\mise\bin\mise.exe" (Join-Path $dir "mise.exe") -Force`,
      String.raw`Remove-Item "$env:TEMP\mise.zip","$env:TEMP\mise" -Recurse -Force`,
    ]
      .join('; ',),
    timeout: EXEC_TIMEOUT_MS,
  },);

  // Use mise to install Node (handles arch detection).
  // ErrorActionPreference=Continue prevents PowerShell from treating
  // mise's stderr progress output as a terminating error.
  await execInVm({
    vmName: VM,
    command:
      `$ErrorActionPreference = "Continue"; & "${MISE_BIN}" use -g node@latest 2>$null; exit $LASTEXITCODE`,
    timeout: EXEC_TIMEOUT_MS,
  },);

  await using _cleanup = { [Symbol.asyncDispose]: () => safeDestroy(VM,), };
  await describe({
    name: 'vmsync lifecycle (Windows)',
    children: [
      it({
        name: 'full CLI lifecycle',
        fn: async () => {
          //region --help
          const help = await execInVm({ vmName: VM, command: `${VMSYNC} --help`, },);
          expect(help,).toContain('vmsync',);
          expect(help,).toContain('import',);
          expect(help,).toContain('boot',);
          expect(help,).toContain('sync',);
          expect(help,).toContain('status',);
          expect(help,).toContain('list',);
          expect(help,).toContain('config',);
          //endregion --help

          //region list (empty)
          const emptyList = await execInVm({ vmName: VM, command: `${VMSYNC} list`, },);
          expect(emptyList,).toContain('no managed VMs',);
          //endregion list (empty)

          //region Create config manually (Windows lacks qemu-nbd for full import)
          // Uses newline joins because PowerShell hash literals (@{}) require
          // actual newlines between entries, not semicolons.
          await execInVm({
            vmName: VM,
            command: [
              String.raw`$dir = "$env:USERPROFILE\.local\share\vmsync\test-vm"`,
              'New-Item -ItemType Directory -Path $dir -Force | Out-Null',
              '$config = @{',
              '  name = "test-vm"',
              String.raw`  importedFrom = "C:\test.qcow2"`,
              '  importedAt = "2026-01-01T00:00:00Z"',
              '  diskSizeBytes = 536870912',
              '  boot = @{ memory = "4G"; cpus = 4 }',
              '  state = @{',
              '    synced = $true',
              '    checksums = @{ qcow2 = "sha256:aaa"; vhdx = "sha256:bbb" }',
              '  }',
              '} | ConvertTo-Json -Depth 4',
              String.raw`Set-Content -Path "$dir\vmsync.jsonc" -Value $config`,
            ]
              .join('\n',),
          },);
          //endregion Create config manually

          //region list (populated)
          const populatedList = await execInVm({ vmName: VM,
            command: `${VMSYNC} list`, },);
          expect(populatedList,).toContain('test-vm',);
          expect(populatedList,).toContain('synced',);
          //endregion list (populated)

          //region status
          const status = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          expect(status,).toContain('name:       test-vm',);
          expect(status,).toContain('synced:     true',);
          expect(status,).toContain('last boot:  never',);
          expect(status,).toContain('memory:     4G',);
          expect(status,).toContain('cpus:       4',);
          //endregion status

          //region config update
          const configOutput = await execInVm({
            vmName: VM,
            command: `${VMSYNC} config test-vm --memory 16G --cpus 16`,
          },);
          expect(configOutput,).toContain('memory=16G',);
          expect(configOutput,).toContain('cpus=16',);

          const statusAfterConfig = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          expect(statusAfterConfig,).toContain('memory:     16G',);
          expect(statusAfterConfig,).toContain('cpus:       16',);
          //endregion config update

          //region config partial update (cpus only, memory preserved)
          await execInVm({ vmName: VM, command: `${VMSYNC} config test-vm --cpus 8`, },);
          const statusAfterPartial = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          expect(statusAfterPartial,).toContain('memory:     16G',);
          expect(statusAfterPartial,).toContain('cpus:       8',);
          //endregion config partial update

          //region sync (already synced)
          const syncOutput = await execInVm({ vmName: VM,
            command: `${VMSYNC} sync test-vm`, },);
          expect(syncOutput,).toContain('already synced',);
          //endregion sync (already synced)

          //region detectHypervisor returns hyperv on Windows
          const statusCheck = await execInVm({ vmName: VM,
            command: `${VMSYNC} status test-vm`, },);
          // The VM was never booted, last boot is "never" (hypervisor detection
          // only runs during boot, not status; but the binary runs on Windows,
          // confirming cross-platform compatibility)
          expect(statusCheck,).toContain('last boot:  never',);
          //endregion detectHypervisor
        },
      },),
    ],
  },);
}

//endregion Windows lifecycle tests

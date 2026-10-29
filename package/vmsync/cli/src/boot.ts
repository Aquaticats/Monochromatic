/**
 Boot command; launches a VM on the auto-detected hypervisor,
 waits for shutdown, then triggers incremental sync.
 
 KVM boots use a qcow2 overlay so only changed blocks need syncing.
 Hyper-V boots use the vhdx directly (checkpoint-based overlay is a future optimization).
 
 @module
 */

import { access, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  BYTES_PER_GIB,
  BYTES_PER_MIB,
} from '@monochromatic-dev/module-const/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  detectHypervisor,
  readConfig,
  vmDir,
  writeConfig,
} from './config.ts';
import { createOverlay, } from './qemu-img.ts';
import { spawn, } from './spawn.ts';
import {
  syncFromHyperv,
  syncFromKvm,
} from './sync.ts';
import {
  OVERLAY_FILENAME,
  QCOW2_FILENAME,
  VHDX_FILENAME,
} from './types.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

//region OVMF firmware discovery

/**
 Common filesystem paths where OVMF UEFI firmware is installed.
 Checked in order; the first accessible path wins.
 */
const OVMF_SEARCH_PATHS: readonly string[] = [
  '/usr/share/edk2/ovmf/OVMF_CODE.fd',
  '/usr/share/OVMF/OVMF_CODE.fd',
  '/usr/share/edk2-ovmf/x64/OVMF_CODE.fd',
  '/usr/share/qemu/OVMF_CODE.fd',
  '/usr/share/ovmf/OVMF_CODE.fd',
];

/**
 Locates the OVMF UEFI firmware file on the current system.
 
 @returns Absolute path to OVMF_CODE.fd
 
 @throws Error when no OVMF firmware is found in any known location
 
 @example
 ```ts
 const fw = await findOvmf();
 // "/usr/share/edk2/ovmf/OVMF_CODE.fd"
 ```
 */
async function findOvmf(): Promise<string> {
  /**
   Tagged logger so OVMF-discovery entries are scoped to `findOvmf` in the output.
   */
  const rl = tagged({
    tag: findOvmf.name,
    l,
  },);

  /**
   Check all candidates concurrently and return the first accessible one.
   */
  const results = await Promise.all(
    OVMF_SEARCH_PATHS.map(
      async function checkCandidate(candidate,) {
        try {
          await access(candidate,);
          return candidate;
        }
        catch (error) {
          if (!(Error.isError(error,)))
            throw error;

          rl.debug(`not found: ${candidate}`,);
          return undefined;
        }
      },
    ),
  );

  /**
   First accessible firmware path.
   */
  const found = results.find(
    function isFound(r,) {
      return r !== undefined;
    },
  );

  if (found === undefined) {
    throw new Error(
      'OVMF UEFI firmware not found. Install edk2-ovmf (Fedora/RHEL), ovmf (Debian/Ubuntu), or qemu-uefi-aarch64 (Alpine).',
    );
  }

  rl.info(`found OVMF at ${found}`,);
  return found;
}

//endregion OVMF firmware discovery

//region KVM boot

/**
 Boots a VM via QEMU/KVM with a qcow2 overlay for change tracking.
 
 1. Creates a fresh overlay via {@link createOverlay}, backed by the qcow2 base
 2. Launches QEMU with UEFI, virtio devices, and NAT networking
 3. Blocks until the QEMU process exits (user shuts down the VM)
 4. Triggers incremental sync via {@link syncFromKvm}, from overlay to vhdx
 
 @param name - VM name
 
 @throws Error when QEMU or OVMF is unavailable, or when sync fails
 
 @example
 ```ts
 await bootKvm('alpine');
 ```
 */
async function bootKvm(name: string,): Promise<void> {
  /**
   Tagged logger so KVM-boot entries are scoped to `bootKvm` in the output.
   */
  const rl = tagged({
    tag: bootKvm.name,
    l,
  },);

  /**
   VM directory.
   */
  const dir = vmDir(name,);
  /**
   Current configuration for boot parameters.
   */
  const config = await readConfig(name,);
  /**
   Path to the qcow2 base image.
   */
  const qcow2Path = join(
    dir,
    QCOW2_FILENAME,
  );
  /**
   Path for the transient overlay.
   */
  const overlayPath = join(
    dir,
    OVERLAY_FILENAME,
  );

  rl.info('creating boot overlay',);
  await createOverlay({
    overlayPath,
    backingPath: qcow2Path,
  },);

  /**
   Path to the OVMF UEFI firmware.
   */
  const ovmfPath = await findOvmf();

  /**
   QEMU launch arguments: UEFI, KVM acceleration, virtio devices, NAT.
   */
  const qemuArgs: readonly string[] = [
    '-enable-kvm',
    '-m',
    config.boot
      .memory,
    '-smp',
    String(config.boot
      .cpus,),
    '-cpu',
    'host',
    '-bios',
    ovmfPath,
    '-drive',
    `if=virtio,file=${overlayPath},format=qcow2`,
    '-device',
    'virtio-gpu-pci',
    '-device',
    'virtio-net-pci,netdev=net0',
    '-netdev',
    'user,id=net0',
    '-device',
    'virtio-keyboard-pci',
    '-device',
    'virtio-mouse-pci',
  ];

  rl.info(`launching QEMU: qemu-system-x86_64 ${qemuArgs.join(' ',)}`,);
  console.log(
    `booting "${name}" via KVM (close the VM window or shut down the guest to sync)`,
  );

  //region Update state before boot
  config.state
    .lastBootHypervisor = 'kvm';
  config.state
    .lastBootAt = new Date().toISOString();
  config.state
    .synced = false;
  await writeConfig({
    name,
    config,
  },);
  //endregion Update state before boot

  await spawn({
    command: 'qemu-system-x86_64',
    args: [...qemuArgs,],
  },);

  rl.info('QEMU exited, starting sync',);
  await syncFromKvm(name,);
}

//endregion KVM boot

//region Hyper-V boot

/**
 Boots a VM via Hyper-V on Windows.
 
 1. Creates a temporary Hyper-V VM definition pointing to the managed vhdx
 2. Configures Gen2, disables Secure Boot, sets NAT via Default Switch
 3. Starts the VM and waits for it to stop
 4. Removes the VM definition (preserving the vhdx)
 5. Triggers sync via {@link syncFromHyperv}, from vhdx to qcow2
 
 @param name - VM name
 
 @throws Error when Hyper-V is unavailable or PowerShell commands fail
 
 @example
 ```ts
 await bootHyperv('alpine');
 ```
 */
async function bootHyperv(name: string,): Promise<void> {
  /**
   Tagged logger so Hyper-V-boot entries are scoped to `bootHyperv` in the output.
   */
  const rl = tagged({
    tag: bootHyperv.name,
    l,
  },);

  /**
   VM directory.
   */
  const dir = vmDir(name,);
  /**
   Current configuration.
   */
  const config = await readConfig(name,);
  /**
   Path to the vhdx disk image.
   */
  const vhdxPath = join(
    dir,
    VHDX_FILENAME,
  );

  /**
   Hyper-V VM name prefixed to avoid collisions.
   */
  const hvName = `vmsync-${name}`;

  /**
   Memory in bytes for Hyper-V.
   */
  const memoryBytes = parseMemoryToBytes(config.boot
    .memory,);

  rl.info(`creating Hyper-V VM "${hvName}"`,);

  /**
   PowerShell script that creates, configures, boots, waits, and cleans up the VM.
   */
  const psScript = [
    `New-VM -Name "${hvName}" -MemoryStartupBytes ${
      String(memoryBytes,)
    } -VHDPath "${vhdxPath}" -Generation 2`,
    `Set-VMProcessor -VMName "${hvName}" -Count ${String(config.boot
      .cpus,)}`,
    `Set-VMFirmware -VMName "${hvName}" -EnableSecureBoot Off`,
    `Connect-VMNetworkAdapter -VMName "${hvName}" -SwitchName "Default Switch"`,
    `Start-VM -Name "${hvName}"`,
    `Wait-VM -Name "${hvName}" -For Stopped`,
    `Remove-VM -Name "${hvName}" -Force`,
  ]
    .join('; ',);

  //region Update state before boot
  config.state
    .lastBootHypervisor = 'hyperv';
  config.state
    .lastBootAt = new Date().toISOString();
  config.state
    .synced = false;
  await writeConfig({
    name,
    config,
  },);
  //endregion Update state before boot

  rl.info('launching Hyper-V VM',);
  console.log(`booting "${name}" via Hyper-V (shut down the guest to sync)`,);

  await spawn({
    command: 'powershell',
    args: [
      '-Command',
      psScript,
    ],
  },);

  rl.info('Hyper-V VM stopped, starting sync',);
  await syncFromHyperv(name,);
}

//endregion Hyper-V boot

//region Memory parsing

/**
 Parses a human-readable memory string (e.g. "4G", "2048M") to bytes.
 
 @param memory - Memory string with G or M suffix
 
 @returns Memory in bytes
 
 @throws Error when the format is not recognized
 
 @example
 ```ts
 parseMemoryToBytes('4G');    // 4294967296
 parseMemoryToBytes('2048M'); // 2147483648
 ```
 */
export function parseMemoryToBytes(memory: string,): number {
  /**
   Decimal base for `Number.parseInt`.
   */
  const DECIMAL_RADIX = 10;
  /**
   Reports a malformed input with the same message shape the old regex
   version produced. Centralised so every failure path stays in sync.
   
   @throws Error with the canonical "invalid memory format" message
   
   @example
   ```ts
   fail(); // throws Error("invalid memory format: ...")
   ```
   */
  function fail(): never {
    throw new Error(
      `invalid memory format: "${memory}" (expected e.g. "4G" or "2048M")`,
    );
  }
  /**
   Locates the exclusive end of the leading run of ASCII digits.
   
   @param idx - candidate scan offset; advances while digits are seen
   
   @returns first index whose character is not `0`-`9`
   
   @example
   ```ts
   findDigitsEnd(0); // 4 for memory === '2048M'
   ```
   */
  function findDigitsEnd(idx: number,): number {
    /**
     Scan cursor; starts at `idx` and advances past each leading ASCII digit in one linear pass.
     */
    let cursor = idx;
    while (cursor < memory
      .length) {
      /**
       Char under the cursor; stops the scan when non-digit.
       */
      const c = memory.charAt(cursor,);
      if ((c < '0') || (c > '9'))
        break;
      cursor += 1;
    }
    return cursor;
  }
  /**
   Skips ASCII space and tab characters starting at `idx`.
   
   @param idx - candidate scan offset; advances while whitespace is seen
   
   @returns first index whose character is not space or tab
   
   @example
   ```ts
   skipWhitespace(4); // 5 for memory === '2048 M'
   ```
   */
  function skipWhitespace(idx: number,): number {
    /**
     Scan cursor; starts at `idx` and advances past each space or tab in one linear pass.
     */
    let cursor = idx;
    while (cursor < memory
      .length) {
      /**
       Char under the cursor; stops the skip when non-whitespace.
       */
      const c = memory.charAt(cursor,);
      if ((c !== ' ') && (c !== '\t'))
        break;
      cursor += 1;
    }
    return cursor;
  }
  /**
   Exclusive end of the leading digit run; `0` means no digits were present.
   */
  const digitsEnd = findDigitsEnd(0,);
  if (digitsEnd === 0)
    fail();
  /**
   Digit substring used as the numeric portion.
   */
  const digitsPart = memory.slice(
    0,
    digitsEnd,
  );
  /**
   Cursor positioned at the unit token after any inter-token whitespace.
   */
  const unitStart = skipWhitespace(digitsEnd,);
  /**
   Unit portion of the input; must be exactly one character.
   */
  const unitPart = memory.slice(unitStart,);
  if (unitPart.length
    !== 1)
    fail();
  /**
   Unit suffix, normalized to uppercase.
   */
  const unit = unitPart.toUpperCase();
  if ((unit !== 'G') && (unit !== 'M'))
    fail();
  /**
   Numeric part of the memory string.
   */
  const value = Number.parseInt(
    digitsPart,
    DECIMAL_RADIX,
  );
  return unit === 'G'
    ? value * BYTES_PER_GIB
    : value * BYTES_PER_MIB;
}

//endregion Memory parsing

//region Public boot entry point

/**
 Boots a VM using the hypervisor {@link detectHypervisor} resolves for the current platform.
 Linux uses KVM/QEMU, Windows uses Hyper-V.
 
 After the VM shuts down, changes are automatically synced to the other format.
 
 @param name - VM name to boot
 
 @throws Error when the VM config is missing or the hypervisor is unavailable
 
 @example
 ```ts
 await bootVm('alpine');
 ```
 */
export async function bootVm(name: string,): Promise<void> {
  /**
   Tagged logger so VM-boot entries are scoped to `bootVm` in the output.
   */
  const rl = tagged({
    tag: bootVm.name,
    l,
  },);

  /**
   Detected hypervisor for the current platform.
   */
  const hypervisor = detectHypervisor();
  rl.info(`detected hypervisor: ${hypervisor}`,);

  await (hypervisor === 'kvm'
    ? bootKvm(name,)
    : bootHyperv(name,));
}

//endregion Public boot entry point

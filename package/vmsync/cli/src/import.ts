/**
 Import command; validates a disk image is UEFI-bootable,
 converts it to qcow2 + vhdx, and creates a managed VM directory.
 
 @module
 */

import { mkdir, } from 'node:fs/promises';
import {
  basename,
  join,
  resolve,
} from 'node:path';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/module-logger/ts';

import {
  DEFAULT_CPUS,
  DEFAULT_MEMORY,
  validateName,
  vmDir,
  writeConfig,
} from './config.ts';
import {
  connectDisposable,
  ensureNbdModule,
  findFreeNbdDevice,
} from './nbd.ts';
import {
  checksum,
  convert,
  imageInfo,
} from './qemu-img.ts';
import { spawn, } from './spawn.ts';
import {
  QCOW2_FILENAME,
  VHDX_FILENAME,
  type VmsyncConfig,
} from './types.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

/**
 Validates that a disk image contains an EFI System Partition.
 Uses `qemu-nbd` (via {@link connectDisposable}) to expose the image as a block device, then
 inspects the partition table via `fdisk`.
 
 vmsync only supports UEFI images (Gen2 for Hyper-V, OVMF for KVM).
 Images without an ESP are rejected at import time.
 
 @param imagePath - Absolute path to the disk image
 
 @param format - Image format string from qemu-img info
 
 @throws Error when the image lacks an EFI System Partition
 
 @example
 ```ts
 await validateUefi({ imagePath: '/tmp/alpine.qcow2', format: 'qcow2' });
 ```
 */
async function validateUefi(
  {
    imagePath,
    format,
  }: {
    readonly imagePath: string;
    readonly format: string;
  },
): Promise<void> {
  /**
   Tagged logger so UEFI-validation entries are scoped to `validateUefi` in the output.
   */
  const rl = tagged({
    tag: validateUefi.name,
    l,
  },);
  rl.info('checking for EFI System Partition',);

  await ensureNbdModule();
  /**
   NBD device allocated for this inspection.
   */
  const device = await findFreeNbdDevice();

  /**
   Auto-disposed NBD connection; bound for its side effect of holding the export open.
   */
  await using _conn = await connectDisposable({
    imagePath,
    device,
    readOnly: true,
    format,
  },);

  /**
   fdisk output listing partition types.
   */
  const fdiskOutput = await spawn({
    command: 'fdisk',
    args: [
      '-l',
      device,
    ],
  },);

  if (!fdiskOutput.includes('EFI System',)) {
    throw new Error(
      `image "${imagePath}" does not contain an EFI System Partition. `
        + 'vmsync only supports UEFI images (Hyper-V Gen2 / KVM OVMF).',
    );
  }

  rl.info('EFI System Partition found',);
}

/**
 Whether `c` is a character permitted in a VM-name identifier.
 
 @param c - one-character string to inspect
 
 @returns whether `c` is alphanumeric, underscore, or hyphen
 
 @example
 ```ts
 isNameChar('a'); // true
 isNameChar('.'); // false
 ```
 */
function isNameChar(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'))
    || (c === '_')
    || (c === '-');
}

/**
 Derives a VM name from an image file path by stripping the extension.
 
 Mirrors the original `.replace(/\.[^.]+$/, '').replaceAll(/[^a-zA-Z0-9_-]/g, '-')`
 pipeline: drops the last `.<ext>` (only when the dot is not the leading
 character, preserving dotfiles) and rewrites every disallowed character
 to `-`.
 
 @param imagePath - Path to the source image
 
 @returns Sanitized name suitable for use as a VM identifier
 
 @example
 ```ts
 nameFromPath('/tmp/alpine-3.21-cloud.qcow2'); // "alpine-3-21-cloud"
 ```
 */
export function nameFromPath(imagePath: string,): string {
  /**
   File-name portion of the path; the input we sanitise.
   */
  const base = basename(imagePath,);
  /**
   Position of the extension separator; `<= 0` means no extension to strip.
   */
  const dotIdx = base.lastIndexOf('.',);
  /**
   Base name with the last extension dropped, or the base itself when no dot is found.
   */
  const noExt = (dotIdx <= 0)
    ? base
    : base.slice(
      0,
      dotIdx,
    );
  return Array
    .from(
    { length: noExt.length, },
    function sanitiseChar(
      _: undefined,
      idx: number,
    ): string {
        /**
         Char being inspected at this index; replaced when disallowed.
         */
      const c = noExt.charAt(idx,);
      return isNameChar(c,)
        ? c
        : '-';
    },
    )
    .join('',);
}

/**
 Imports a disk image into vmsync management.
 
 1. Detects the image format via {@link imageInfo}
 2. Validates the image contains an EFI System Partition via {@link validateUefi}
 3. Converts to qcow2 and vhdx in a new VM directory
 4. Computes checksums and writes the initial config
 
 @param imagePath - Path to the source disk image (any format qemu-img supports)
 
 @param name - Optional VM name override; defaults to the filename stem
 
 @throws Error when the image is not UEFI-bootable or conversion fails
 
 @example
 ```ts
 await importImage({ imagePath: '/tmp/alpine.qcow2' });
 await importImage({ imagePath: '/tmp/fedora.raw', name: 'fedora-dev' });
 ```
 */
export async function importImage(
  {
    imagePath,
    name,
  }: {
    readonly imagePath: string;
    readonly name?: string;
  },
): Promise<void> {
  /**
   Tagged logger so import entries are scoped to `importImage` in the output.
   */
  const rl = tagged({
    tag: importImage.name,
    l,
  },);

  /**
   Resolved absolute path to the source image.
   */
  const absPath = resolve(imagePath,);
  rl.info(`importing ${absPath}`,);

  /**
   Image metadata from qemu-img.
   */
  const info = await imageInfo(absPath,);
  rl.info(
    `detected format: ${info.format}, virtual size: ${
      String(info['virtual-size'],)
    } bytes`,
  );

  await validateUefi({
    imagePath: absPath,
    format: info.format,
  },);

  /**
   Final VM name, validated for safe characters.
   */
  const vmName = name ?? nameFromPath(absPath,);
  validateName(vmName,);

  /**
   Directory where managed images are stored.
   */
  const dir = vmDir(vmName,);
  await mkdir(
    dir,
    { recursive: true, },
  );

  /**
   Target path for the KVM format.
   */
  const qcow2Path = join(
    dir,
    QCOW2_FILENAME,
  );
  /**
   Target path for the Hyper-V format.
   */
  const vhdxPath = join(
    dir,
    VHDX_FILENAME,
  );

  //region Convert source to both target formats

  await convertSourceImage({
    absPath,
    format: info.format,
    qcow2Path,
    vhdxPath,
    rl,
  },);

  //endregion Convert source to both target formats

  //region Compute checksums and write config

  rl.info('computing checksums',);
  /**
   SHA-256 checksums for both managed formats.
   */
  const [qcow2Hash, vhdxHash,] = await Promise.all([
    checksum(qcow2Path,),
    checksum(vhdxPath,),
  ],);

  /**
   Initial {@link VmsyncConfig}.
   */
  const config: VmsyncConfig = {
    name: vmName,
    importedFrom: absPath,
    importedAt: new Date().toISOString(),
    diskSizeBytes: info['virtual-size'],
    boot: {
      memory: DEFAULT_MEMORY,
      cpus: DEFAULT_CPUS,
    },
    state: {
      synced: true,
      checksums: {
        qcow2: qcow2Hash,
        vhdx: vhdxHash,
      },
    },
  };

  await writeConfig({
    name: vmName,
    config,
  },);
  rl.info(`import complete: "${vmName}" ready in ${dir}`,);
  console.log(`imported "${vmName}" (${info.format} -> qcow2 + vhdx)`,);

  //endregion Compute checksums and write config
}

/**
 Converts the source image to qcow2 and vhdx via {@link convert}, choosing the optimal
 conversion path based on the source format.
 
 @param absPath - Absolute path to the source image
 
 @param format - Detected source format
 
 @param qcow2Path - Target path for qcow2 output
 
 @param vhdxPath - Target path for vhdx output
 
 @param rl - {@link Logger} for status output
 */
async function convertSourceImage(
  {
    absPath,
    format,
    qcow2Path,
    vhdxPath,
    rl,
  }: {
    readonly absPath: string;
    readonly format: string;
    readonly qcow2Path: string;
    readonly vhdxPath: string;
    readonly rl: Logger;
  },
): Promise<void> {
  if (format === 'qcow2') {
    rl.info('source is qcow2, copying as base and converting to vhdx',);
    await convert({
      sourcePath: absPath,
      sourceFormat: 'qcow2',
      targetPath: qcow2Path,
      targetFormat: 'qcow2',
    },);
    await convert({
      sourcePath: absPath,
      sourceFormat: 'qcow2',
      targetPath: vhdxPath,
      targetFormat: 'vhdx',
    },);
  }
  else if (format === 'raw') {
    rl.info('source is raw, converting to qcow2 and vhdx',);
    await convert({
      sourcePath: absPath,
      sourceFormat: 'raw',
      targetPath: qcow2Path,
      targetFormat: 'qcow2',
    },);
    await convert({
      sourcePath: absPath,
      sourceFormat: 'raw',
      targetPath: vhdxPath,
      targetFormat: 'vhdx',
    },);
  }
  else {
    rl.info(`source is ${format}, converting via qcow2 intermediate`,);
    await convert({
      sourcePath: absPath,
      sourceFormat: format,
      targetPath: qcow2Path,
      targetFormat: 'qcow2',
    },);
    await convert({
      sourcePath: qcow2Path,
      sourceFormat: 'qcow2',
      targetPath: vhdxPath,
      targetFormat: 'vhdx',
    },);
  }
}

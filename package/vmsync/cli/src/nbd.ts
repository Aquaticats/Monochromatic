/**
 NBD (Network Block Device) management for incremental block-level sync.
 
 Exposes disk images as Linux block devices via `qemu-nbd`,
 then uses `dd` to copy only changed regions between them.
 
 @module
 */

import { access, } from 'node:fs/promises';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/module-logger/ts';

import { spawn, } from './spawn.ts';
import type { QemuMapRegion, } from './types.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

/**
 Maximum NBD devices to search when finding a free slot.
 */
const MAX_NBD_DEVICES = 16;

/**
 Block size for dd transfers (1 MiB).
 */
const DD_BLOCK_SIZE = 1_048_576;

/**
 Bytes per sector for dd block size alignment.
 */
const SECTOR_SIZE = 512;

//region NBD device discovery

/**
 Finds the first unused `/dev/nbdN` device.
 Checks whether each device file exists and is not already connected
 by attempting to read its size via sysfs.
 
 @returns Device path, e.g. `/dev/nbd0`
 
 @throws Error when all NBD device slots are occupied
 
 @example
 ```ts
 const dev = await findFreeNbdDevice();
 // "/dev/nbd3"
 ```
 */
export async function findFreeNbdDevice(): Promise<string> {
  /**
   Function-tagged logger so NBD device discovery is traceable in shared logs.
   */
  const rl = tagged({
    tag: findFreeNbdDevice.name,
    l,
  },);

  /**
   Immutable paths identifying one candidate NBD device.
   */
  type NbdCandidate = Readonly<{
    device: string;
    sysfsSize: string;
  }>;
  /**
   Check all candidate devices concurrently and return the first free one.
   */
  const candidates = Array.from(
    { length: MAX_NBD_DEVICES, },
    function buildCandidate(
      _,
      i,
    ): NbdCandidate {
      return {
        device: `/dev/nbd${String(i,)}`,
        sysfsSize: `/sys/block/nbd${String(i,)}/size`,
      };
    },
  );

  /**
   Check all devices concurrently to find free ones.
   */
  const freeChecks = await Promise.all(
    candidates.map(
      async function checkCandidate(
        {
          device,
          sysfsSize,
        },
      ) {
        /**
         Whether this device is free (not connected).
         */
        const isFree = await checkDeviceFree({
          sysfsSize,
          rl,
        },);
        return isFree ? device : undefined;
      },
    ),
  );

  /**
   First free device from the concurrent checks.
   */
  const freeDevice = freeChecks.find(
    function isDefined(d,) {
      return d !== undefined;
    },
  );

  if (freeDevice !== undefined) {
    rl.info(`found free device: ${freeDevice}`,);
    return freeDevice;
  }

  throw new Error(
    `all ${String(MAX_NBD_DEVICES,)} NBD device slots are occupied`,
  );
}

/**
 Checks whether an NBD device is free by reading its sysfs size.
 
 @param sysfsSize - Path to the sysfs size file
 
 @param rl - {@link Logger} for debug output
 
 @returns True when the device is free
 */
async function checkDeviceFree(
  {
    sysfsSize,
    rl,
  }: {
    readonly sysfsSize: string;
    readonly rl: Logger;
  },
): Promise<boolean> {
  try {
    await access(sysfsSize,);
    /**
     Size value from sysfs; "0" means not connected.
     */
    const sizeStr = await spawn({
      command: 'cat',
      args: [sysfsSize,],
    },);
    return sizeStr.trim()
      === '0';
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug(`no sysfs entry for ${sysfsSize}, assuming free`,);
    return true;
  }
}

//endregion NBD device discovery

//region NBD connection lifecycle

/**
 Disposable NBD connection handle.
 Use with `await using` to ensure automatic disconnection.
 
 @example
 ```ts
 await using conn = await connectDisposable({
   imagePath: '/data/disk.qcow2',
   device: '/dev/nbd0',
   readOnly: true,
   format: 'qcow2',
 });
 // device is automatically disconnected when scope exits
 ```
 */
export type NbdConnection = {
  readonly device: string;
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Connects a disk image to an NBD device via `qemu-nbd`
 and returns a disposable handle for automatic cleanup.
 
 @param imagePath - Absolute path to the disk image
 
 @param device - NBD device path (e.g. `/dev/nbd0`)
 
 @param readOnly - Mount read-only when true
 
 @param format - Image format (e.g. 'qcow2', 'vhdx')
 
 @returns disposable {@link NbdConnection} handle
 
 @throws Error when connection fails (e.g. device busy, permission denied)
 
 @example
 ```ts
 await using conn = await connectDisposable({
   imagePath: '/data/alpine/overlay.qcow2',
   device: '/dev/nbd0',
   readOnly: true,
   format: 'qcow2',
 });
 ```
 */
export async function connectDisposable(
  {
    imagePath,
    device,
    readOnly,
    format,
  }: {
    readonly imagePath: string;
    readonly device: string;
    readonly readOnly: boolean;
    readonly format: string;
  },
): Promise<NbdConnection> {
  /**
   Function-tagged logger so connect/disconnect events are paired in traces.
   */
  const rl = tagged({
    tag: connectDisposable.name,
    l,
  },);

  /**
   Argument list built dynamically based on options.
   */
  const args = [
    '-c',
    device,
    '-f',
    format,
  ];
  if (readOnly)
    args.unshift('-r',);
  args.push(imagePath,);

  rl.info(`connecting ${imagePath} to ${device} (readOnly=${String(readOnly,)})`,);
  await spawn({
    command: 'qemu-nbd',
    args,
  },);
  rl.info(`connected ${device}`,);

  return {
    device,
    [Symbol.asyncDispose]: async function disconnectOnDispose() {
      rl.info(`disconnecting ${device}`,);
      await spawn({
        command: 'qemu-nbd',
        args: [
          '-d',
          device,
        ],
      },);
      rl.info(`disconnected ${device}`,);
    },
  };
}

//endregion NBD connection lifecycle

//region Block-level patching

/**
 Copies changed blocks from a source NBD device to a target NBD device.
 Only regions marked as overlay data (`depth === 0` and `data === true`)
 in the block map are transferred.
 
 @param sourceDevice - NBD device exposing the overlay (read-only)
 
 @param targetDevice - NBD device exposing the target image (read-write)
 
 @param changedRegions - {@link QemuMapRegion} entries from `qemu-img map` with `depth === 0`
 
 @throws Error when a dd transfer fails
 
 @example
 ```ts
 await patchBlocks({
   sourceDevice: '/dev/nbd0',
   targetDevice: '/dev/nbd1',
   changedRegions: regions.filter(r => r.depth === 0 && r.data),
 });
 ```
 */
export async function patchBlocks(
  {
    sourceDevice,
    targetDevice,
    changedRegions,
  }: {
    readonly sourceDevice: string;
    readonly targetDevice: string;
    readonly changedRegions: readonly QemuMapRegion[];
  },
): Promise<void> {
  /**
   Function-tagged logger so per-region copy operations are traceable.
   */
  const rl = tagged({
    tag: patchBlocks.name,
    l,
  },);

  /**
   Total bytes to transfer across all changed regions.
   */
  const totalBytes = changedRegions.reduce(
    function sumLength(
      acc,
      r,
    ) {
      return acc + r
        .length;
    },
    0,
  );
  rl.info(
    `patching ${String(changedRegions.length,)} regions (${
      String(Math.round(totalBytes / DD_BLOCK_SIZE,),)
    } MiB)`,
  );

  /**
   Transfer promises collected for parallel execution.
   */
  const transfers = changedRegions.map(
    function buildTransfer(region,) {
      return transferRegion({
        sourceDevice,
        targetDevice,
        region,
        rl,
      },);
    },
  );
  await Promise.all(transfers,);

  rl.info('block patching complete',);
}

/**
 Transfers a single changed region between NBD devices via dd.
 
 @param sourceDevice - Source NBD device
 
 @param targetDevice - Target NBD device
 
 @param region - {@link QemuMapRegion} descriptor from the block map
 
 @param rl - {@link Logger} for debug output
 
 @throws Error when the region is not sector-aligned or dd fails
 */
async function transferRegion(
  {
    sourceDevice,
    targetDevice,
    region,
    rl,
  }: {
    readonly sourceDevice: string;
    readonly targetDevice: string;
    readonly region: QemuMapRegion;
    readonly rl: Logger;
  },
): Promise<void> {
  if (((region.start
    % SECTOR_SIZE) !== 0) || ((region.length
      % SECTOR_SIZE) !== 0)) {
    throw new Error(
      `region at offset ${String(region.start,)} with length ${
        String(region.length,)
      } is not sector-aligned`,
    );
  }

  /**
   Number of dd-sized blocks, rounding up for partial final blocks.
   */
  const blockCount = Math.ceil(region.length
    / DD_BLOCK_SIZE,);
  /**
   Byte offset for dd skip/seek.
   */
  const byteOffset = region.start;

  rl.debug(
    `dd: offset=${String(byteOffset,)} blocks=${String(blockCount,)}`,
  );

  await spawn({
    command: 'dd',
    args: [
      `if=${sourceDevice}`,
      `of=${targetDevice}`,
      `bs=${String(DD_BLOCK_SIZE,)}`,
      `skip=${String(byteOffset / DD_BLOCK_SIZE,)}`,
      `seek=${String(byteOffset / DD_BLOCK_SIZE,)}`,
      `count=${String(blockCount,)}`,
      'conv=notrunc',
      'status=none',
    ],
  },);
}

//endregion Block-level patching

//region NBD module loading

/**
 Loads the `nbd` kernel module if not already loaded.
 
 @throws Error when modprobe fails (e.g. module not available, not root)
 
 @example
 ```ts
 await ensureNbdModule();
 ```
 */
export async function ensureNbdModule(): Promise<void> {
  /**
   Function-tagged logger so module-load attempts surface clearly in traces.
   */
  const rl = tagged({
    tag: ensureNbdModule.name,
    l,
  },);

  try {
    await access('/dev/nbd0',);
    rl.info('nbd module already loaded',);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.info('loading nbd kernel module',);
    await spawn({
      command: 'modprobe',
      args: [
        'nbd',
        'max_part=0',
      ],
    },);
    rl.info('nbd module loaded',);
  }
}

//endregion NBD module loading

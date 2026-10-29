/**
 Incremental sync; applies changed blocks from the last boot session
 to the other disk format via NBD block-level patching.
 
 @module
 */

import { unlink, } from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  readConfig,
  vmDir,
  writeConfig,
} from './config.ts';
import {
  connectDisposable,
  ensureNbdModule,
  findFreeNbdDevice,
  patchBlocks,
} from './nbd.ts';
import {
  blockMap,
  checksum,
  commitOverlay,
  convert,
} from './qemu-img.ts';
import {
  OVERLAY_FILENAME,
  QCOW2_FILENAME,
  type QemuMapRegion,
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

/**
 Syncs changes from a KVM boot session (qcow2 overlay) to the vhdx base image.
 
 1. Reads the overlay's block map to find changed regions
 2. Connects both overlay (read-only) and vhdx (read-write) via NBD
 3. Copies only changed blocks from overlay to vhdx
 4. Commits the overlay into the qcow2 base via {@link commitOverlay}
 5. Removes the overlay file
 6. Updates checksums in the config
 
 @param name - VM name
 
 @throws Error when no overlay exists (VM was not booted via KVM)
 
 @example
 ```ts
 await syncFromKvm('alpine');
 ```
 */
export async function syncFromKvm(name: string,): Promise<void> {
  /**
   Function-tagged logger so post-boot sync steps are traceable per VM.
   */
  const rl = tagged({
    tag: syncFromKvm.name,
    l,
  },);

  /**
   VM directory containing all managed images.
   */
  const dir = vmDir(name,);
  /**
   Path to the transient overlay created before boot.
   */
  const overlayPath = join(
    dir,
    OVERLAY_FILENAME,
  );
  /**
   Path to the qcow2 base image.
   */
  const qcow2Path = join(
    dir,
    QCOW2_FILENAME,
  );
  /**
   Path to the vhdx image to patch.
   */
  const vhdxPath = join(
    dir,
    VHDX_FILENAME,
  );

  rl.info('reading overlay block map',);
  /**
   Full block map of the overlay with backing chain depth info.
   */
  const regions = await blockMap(overlayPath,);

  /**
   Regions at depth 0 with actual data: these were written during the boot session.
   */
  const changedRegions: readonly QemuMapRegion[] = regions.filter(
    function isOverlayData(r,) {
      return (r.depth
        === 0) && r
          .data;
    },
  );

  if (changedRegions.length
    === 0)
    rl.info('no blocks changed during boot, skipping sync',);
  else {
    rl.info(`${String(changedRegions.length,)} regions changed, patching vhdx via NBD`,);
    await patchVhdxFromOverlay({
      overlayPath,
      vhdxPath,
      changedRegions,
    },);
  }

  //region Commit overlay and update checksums

  rl.info('committing overlay into qcow2 base',);
  await commitOverlay(overlayPath,);

  rl.info('removing overlay file',);
  await unlink(overlayPath,);

  rl.info('computing new checksums',);
  /**
   Updated checksums after sync.
   */
  const [qcow2Hash, vhdxHash,] = await Promise.all([
    checksum(qcow2Path,),
    checksum(vhdxPath,),
  ],);

  /**
   Current config to update with new state.
   */
  const config = await readConfig(name,);
  config.state
    .synced = true;
  config.state
    .checksums = {
      qcow2: qcow2Hash,
      vhdx: vhdxHash,
    };
  await writeConfig({
    name,
    config,
  },);

  rl.info('sync from KVM complete',);
  console.log(
    `synced "${name}" (overlay -> vhdx, ${
      String(changedRegions.length,)
    } regions patched)`,
  );

  //endregion Commit overlay and update checksums
}

/**
 Patches the vhdx image with changed blocks from the qcow2 overlay via NBD, using
 {@link patchBlocks} for the block-level copy.
 
 @param overlayPath - Path to the qcow2 overlay
 
 @param vhdxPath - Path to the vhdx to patch
 
 @param changedRegions - Block map regions that were written in the overlay
 */
async function patchVhdxFromOverlay(
  {
    overlayPath,
    vhdxPath,
    changedRegions,
  }: {
    readonly overlayPath: string;
    readonly vhdxPath: string;
    readonly changedRegions: readonly QemuMapRegion[];
  },
): Promise<void> {
  await ensureNbdModule();

  /**
   NBD device for the overlay (source, read-only).
   */
  const sourceDevicePath = await findFreeNbdDevice();
  /**
   Disposable NBD connection for the source overlay.
   
   Bound to `_sourceConn` so `await using` triggers automatic disconnect when
   this scope exits; the binding is intentionally unused beyond lifetime control.
   */
  await using _sourceConn = await connectDisposable({
    imagePath: overlayPath,
    device: sourceDevicePath,
    readOnly: true,
    format: 'qcow2',
  },);

  /**
   NBD device for the vhdx (target, read-write).
   */
  const targetDevicePath = await findFreeNbdDevice();
  /**
   Disposable NBD connection for the target vhdx.
   
   Bound to `_targetConn` so `await using` triggers automatic disconnect when
   this scope exits; the binding is intentionally unused beyond lifetime control.
   */
  await using _targetConn = await connectDisposable({
    imagePath: vhdxPath,
    device: targetDevicePath,
    readOnly: false,
    format: 'vhdx',
  },);

  await patchBlocks({
    sourceDevice: sourceDevicePath,
    targetDevice: targetDevicePath,
    changedRegions,
  },);
}

/**
 Syncs changes from a Hyper-V boot session to the qcow2 base image.
 
 Hyper-V writes directly to the vhdx (no overlay mechanism in MVP).
 This performs a full checksum comparison to detect changes,
 then reconverts the entire vhdx to qcow2.
 
 For future optimization: use Hyper-V checkpoints to create differencing
 disks (.avhdx) that enable the same overlay-based incremental sync as KVM.
 
 @param name - VM name
 
 @example
 ```ts
 await syncFromHyperv('alpine');
 ```
 */
export async function syncFromHyperv(name: string,): Promise<void> {
  /**
   Function-tagged logger so post-boot sync steps are traceable per VM.
   */
  const rl = tagged({
    tag: syncFromHyperv.name,
    l,
  },);

  /**
   VM directory.
   */
  const dir = vmDir(name,);
  /**
   Path to the qcow2 image to patch.
   */
  const qcow2Path = join(
    dir,
    QCOW2_FILENAME,
  );
  /**
   Path to the vhdx that was just booted.
   */
  const vhdxPath = join(
    dir,
    VHDX_FILENAME,
  );

  /**
   Current config with pre-boot checksums.
   */
  const config = await readConfig(name,);

  rl.info('computing post-boot vhdx checksum',);
  /**
   Post-boot checksum to compare against stored value.
   */
  const newVhdxHash = await checksum(vhdxPath,);

  if (newVhdxHash
    === config
      .state
      .checksums
      .vhdx) {
    rl.info('vhdx unchanged, skipping sync',);
    config.state
      .synced = true;
    await writeConfig({
      name,
      config,
    },);
    return;
  }

  rl.info('vhdx changed, performing full conversion to qcow2',);

  /**
   Full vhdx-to-qcow2 conversion.
   Hyper-V does not produce an overlay, so block-level incremental sync
   requires checkpoint-based differencing disks; a future optimization.
   For MVP, full conversion of a 100GB vhdx takes ~5 minutes.
   */
  await convert({
    sourcePath: vhdxPath,
    sourceFormat: 'vhdx',
    targetPath: qcow2Path,
    targetFormat: 'qcow2',
  },);

  /**
   Updated qcow2 checksum after conversion.
   */
  const newQcow2Hash = await checksum(qcow2Path,);

  config.state
    .synced = true;
  config.state
    .checksums = {
      qcow2: newQcow2Hash,
      vhdx: newVhdxHash,
    };
  await writeConfig({
    name,
    config,
  },);

  rl.info('sync from Hyper-V complete',);
  console.log(`synced "${name}" (vhdx -> qcow2, full conversion)`,);
}

/**
 Syncs a VM after boot, auto-detecting which hypervisor was used.
 
 @param name - VM name
 
 @throws Error when the VM is already synced or config is missing
 
 @example
 ```ts
 await syncVm('alpine');
 ```
 */
export async function syncVm(name: string,): Promise<void> {
  /**
   Function-tagged logger so dispatch and per-hypervisor sync share traces.
   */
  const rl = tagged({
    tag: syncVm.name,
    l,
  },);

  /**
   Current config to determine last boot hypervisor.
   */
  const config = await readConfig(name,);

  if (config.state
    .synced) {
    rl.info(`"${name}" is already synced`,);
    console.log(`"${name}" is already synced`,);
    return;
  }

  if (config.state
    .lastBootHypervisor
    === 'kvm')
    await syncFromKvm(name,);
  else if (config.state
    .lastBootHypervisor
    === 'hyperv')
    await syncFromHyperv(name,);
  else {
    throw new Error(
      `cannot sync "${name}": no recorded boot hypervisor`,
    );
  }
}

import {
  access,
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  createSeedIso,
  NO_SEED_ISO,
} from './cloud-init.ts';
import {
  SHARED_DIR_NAME,
  validateName,
  VMS_DIR,
} from './config.ts';
import { domainXml, } from './domain-xml.ts';
import { exec, } from './exec.ts';
import { chooseFileTransferRoute, } from './file-transfer-route.ts';
import {
  readVmMeta,
  writeVmMeta,
} from './meta.ts';
import {
  CUSTOM_GUEST_DEFAULTS,
  resolveImage,
} from './registry.ts';
import { qemuImg, } from './qemu-img.ts';
import { waitForGuestAgent, } from './virsh-wait.ts';
import {
  defineVm,
  startVm,
} from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Clones an existing VM by copying its disk and creating a new cloud-init seed.
 The new instance-id in the seed ISO triggers cloud-init to re-run,
 updating the hostname on the cloned disk. Preserves the source VM's
 image identifier so the correct guest config is used for cloud-init.
 
 For Windows VMs, hostname is set via guest agent after boot instead of
 cloud-init since Windows does not support the NoCloud datasource.
 
 @param destination - Destination VM name
 
 @param source - Source VM name to clone from
 
 @throws Error when source disk is missing or clone fails
 
 @example
 ```ts
 await clone({ destination: 'dev-02', source: 'dev-01' });
 ```
 */
export async function clone(
  {
    destination,
    source,
  }: {
    readonly destination: string;
    readonly source: string;
  },
): Promise<void> {
  validateName(source,);
  validateName(destination,);
  /**
   Logger scoped to this clone call so step logs are namespaced.
   */
  const rl = tagged({
    tag: clone.name,
    l,
  },);

  /**
   Source VM directory; holds the disk and metadata that get copied.
   */
  const srcVmDir = join(
    VMS_DIR,
    source,
  );
  /**
   Destination VM directory; created below before the qemu-img copy.
   */
  const dstVmDir = join(
    VMS_DIR,
    destination,
  );

  rl.info(`cloning VM ${source} to ${destination}`,);
  await mkdir(
    dstVmDir,
    { recursive: true, },
  );

  /**
   Source qcow2 path; existence is checked next to detect a missing source VM.
   */
  const srcDiskPath = join(
    srcVmDir,
    'disk.qcow2',
  );
  /**
   Destination qcow2 path; `qemu-img convert` writes the cloned image here.
   */
  const dstDiskPath = join(
    dstVmDir,
    'disk.qcow2',
  );

  try {
    await access(srcDiskPath,);
  }
  catch (error) {
    /**
     Listing of {@link VMS_DIR} so the error message can surface known VM names to the user.
     */
    const entries = await readdir(VMS_DIR,);
    throw new Error(
      `source VM "${source}" not found (no disk at ${srcDiskPath}). Available VMs: ${
        entries.join(', ',)
      }`,
      { cause: error, },
    );
  }

  rl.info('copying disk (this may take a moment)...',);
  await qemuImg({
    args: [
      'convert',
      '-O',
      'qcow2',
      srcDiskPath,
      dstDiskPath,
    ],
  },);

  /**
   Source VM metadata; the image field is preserved so the clone inherits the same guest config.
   */
  const meta = await readVmMeta(srcVmDir,);
  /**
   Resolved image record; either a registry spec or a fall-through for custom images.
   */
  const resolved = await resolveImage(meta.image,);
  /**
   Guest config used for cloud-init seeding; defaults applied when the image isn't in the registry.
   */
  const guest = resolved.kind
    === 'registry'
    ? resolved.spec
    : CUSTOM_GUEST_DEFAULTS;

  /**
   How the clone's files will move: the virtiofs share, or the guest agent when libvirt finds no virtiofsd.
   Decided for this host now, not copied from the source VM.
   */
  const fileTransfer = await chooseFileTransferRoute(dstVmDir,);
  /**
   Shared directory exposed to the guest via virtiofs; only created and attached when the share is used.
   */
  const sharedDir = join(
    dstVmDir,
    SHARED_DIR_NAME,
  );
  if (fileTransfer === 'virtiofs') {
    await mkdir(
      sharedDir,
      { recursive: true, },
    );
  }

  /**
   Generated NoCloud seed ISO with a new instance-id so cloud-init reruns on the clone; {@link NO_SEED_ISO} for Windows.
   */
  const seedIso = await createSeedIso({
    guest,
    name: destination,
    sharedMount: fileTransfer === 'virtiofs',
    vmDir: dstVmDir,
  },);
  /**
   Libvirt domain XML for the clone; attaches the cloned disk, seed ISO, and shared dir.
   */
  const xml = domainXml({
    diskPath: dstDiskPath,
    name: destination,
    osFamily: guest.osFamily,
    ...(fileTransfer === 'virtiofs' ? { sharedDir, } : {}),
    ...(seedIso !== NO_SEED_ISO ? { seedIsoPath: seedIso, } : {}),
  },);

  await defineVm({
    vmDir: dstVmDir,
    xml,
  },);
  // Written before the first guest command: exec reads the clone's metadata to pick the guest's shell.
  await writeVmMeta({
    fileTransfer,
    guest,
    image: meta.image,
    vmDir: dstVmDir,
  },);
  await startVm({ name: destination, },);
  await waitForGuestAgent({ name: destination, },);

  // Windows VMs: set hostname via guest agent since cloud-init is not available
  if (guest.osFamily
    === 'windows') {
    rl.info(`setting Windows hostname to ${destination}`,);
    /**
     Result of the `Rename-Computer` invocation; non-zero exit codes are logged but not fatal.
     */
    const result = await exec({
      command: `Rename-Computer -NewName '${destination}' -Force`,
      name: destination,
    },);
    if (result.exitCode
      !== 0) {
      rl.info(
        `hostname change returned exit code ${
          String(result.exitCode,)
        }: ${result.stderr}`,
      );
    }
  }

  rl.info(
    `VM ${destination} is ready (cloned from ${source}). Connect with: mvm shell ${destination}`,
  );
}

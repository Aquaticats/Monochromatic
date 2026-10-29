import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  createSeedIso,
  NO_SEED_ISO,
} from './cloud-init.ts';
import {
  DEFAULT_DISK_SIZE,
  SHARED_DIR_NAME,
  validateName,
  VMS_DIR,
  WINDOWS_DISK_SIZE,
} from './config.ts';
import { domainXml, } from './domain-xml.ts';
import { exec, } from './exec.ts';
import { chooseFileTransferRoute, } from './file-transfer-route.ts';
import { writeVmMeta, } from './meta.ts';
import {
  CUSTOM_GUEST_DEFAULTS,
  DEFAULT_IMAGE,
  resolveImage,
} from './registry.ts';
import { qemuImg, } from './qemu-img.ts';
import { ensureTemplate, } from './template.ts';
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

//region Windows post-boot provisioning

/**
 Sets the hostname on a running Windows VM via the QEMU guest agent.
 Windows VMs do not use cloud-init, so hostname must be configured
 after boot using PowerShell via guest-exec.
 
 @param hostname - Desired hostname
 
 @param name - VM name for guest agent addressing
 
 @example
 ```ts
 await setWindowsHostname({ name: 'win-01', hostname: 'win-01' });
 ```
 */
async function setWindowsHostname({
  hostname,
  name,
}: {
  readonly hostname: string;
  readonly name: string;
},): Promise<void> {
  /**
   Logger scoped to this helper so the rename invocation is namespaced.
   */
  const rl = tagged({
    tag: setWindowsHostname.name,
    l,
  },);
  rl.info(`setting Windows hostname to ${hostname}`,);
  /**
   Result of the `Rename-Computer` invocation; non-zero exit codes are logged but not fatal.
   */
  const result = await exec({
    command: `Rename-Computer -NewName '${hostname}' -Force`,
    name,
  },);
  if (result.exitCode
    !== 0) {
    rl.info(
      `hostname change returned exit code ${String(result.exitCode,)}: ${result.stderr}`,
    );
  }
}

//endregion Windows post-boot provisioning

//region Create

/**
 Creates a new VM from a template image and starts it.
 Resolves the image identifier through the built-in registry or custom template lookup.
 Registry images go through the download-and-template-bake pipeline;
 custom templates are used as backing files directly.
 
 For Linux guests, a cloud-init seed ISO configures hostname and autologin.
 For Windows guests, the hostname is set via guest agent after boot.
 
 @param image - Image identifier (defaults to `ubuntu`)
 
 @param name - VM name (alphanumeric, hyphens, underscores)
 
 @throws Error on invalid name, unknown image, or disk creation failure
 
 @example
 ```ts
 await create({ name: 'dev-01' });
 await create({ image: 'fedora', name: 'build-box' });
 await create({ image: 'windows', name: 'win-test' });
 await create({ image: 'my-custom', name: 'special' });
 ```
 */
export async function create({
  image = DEFAULT_IMAGE,
  name,
}: {
  readonly image?: string;
  readonly name: string;
},): Promise<void> {
  validateName(name,);
  /**
   Logger scoped to this create call so step logs are namespaced.
   */
  const rl = tagged({
    tag: create.name,
    l,
  },);
  /**
   Per-VM scratch directory under {@link VMS_DIR}; holds disk, seed ISO, and shared dir.
   */
  const vmDir = join(
    VMS_DIR,
    name,
  );

  /**
   Resolved image record from registry or custom-template lookup; drives the rest of the pipeline.
   */
  const resolved = await resolveImage(image,);
  rl.info(`creating VM ${name} (image: ${image})`,);
  await mkdir(
    vmDir,
    { recursive: true, },
  );

  /**
   Backing template path; registry images go through the bake pipeline, custom ones are used directly.
   */
  const templateImage = resolved.kind
    === 'registry'
    ? await ensureTemplate(resolved.spec,)
    : resolved.customTemplatePath;

  /**
   Guest config (osFamily, shell, etc.); falls back to {@link CUSTOM_GUEST_DEFAULTS} for custom templates.
   */
  const guest = resolved.kind
    === 'registry'
    ? resolved.spec
    : CUSTOM_GUEST_DEFAULTS;

  /**
   New VM's qcow2 path; created with the resolved template as a backing file.
   */
  const diskPath = join(
    vmDir,
    'disk.qcow2',
  );
  /**
   Disk capacity for the new VM; Windows needs a larger image so it gets bumped up.
   */
  const diskSize = guest.osFamily
    === 'windows' ? WINDOWS_DISK_SIZE : DEFAULT_DISK_SIZE;

  rl.info('creating disk from template image...',);
  await qemuImg({
    args: [
      'create',
      '-f',
      'qcow2',
      '-b',
      templateImage,
      '-F',
      'qcow2',
      diskPath,
      diskSize,
    ],
  },);

  /**
   How this VM's files will move: the virtiofs share, or the guest agent when libvirt finds no virtiofsd.
   Decided before the domain is defined, because a domain with a share it cannot serve does not start.
   */
  const fileTransfer = await chooseFileTransferRoute(vmDir,);
  /**
   Shared directory exposed to the guest via virtiofs; only created and attached when the share is used.
   */
  const sharedDir = join(
    vmDir,
    SHARED_DIR_NAME,
  );
  if (fileTransfer === 'virtiofs') {
    await mkdir(
      sharedDir,
      { recursive: true, },
    );
  }

  /**
   NoCloud seed ISO carrying the user-data and meta-data files for first-boot cloud-init; {@link NO_SEED_ISO} for Windows.
   */
  const seedIso = await createSeedIso({
    guest,
    name,
    sharedMount: fileTransfer === 'virtiofs',
    vmDir,
  },);
  /**
   Libvirt domain XML wiring the disk, seed ISO, and shared dir into a new VM definition.
   */
  const xml = domainXml({
    diskPath,
    name,
    osFamily: guest.osFamily,
    ...(fileTransfer === 'virtiofs' ? { sharedDir, } : {}),
    ...(seedIso !== NO_SEED_ISO ? { seedIsoPath: seedIso, } : {}),
  },);

  await defineVm({
    vmDir,
    xml,
  },);
  await writeVmMeta({
    fileTransfer,
    guest,
    image,
    vmDir,
  },);
  await startVm({ name, },);
  await waitForGuestAgent({ name, },);

  // Windows VMs do not use cloud-init; set hostname via guest agent
  if (guest.osFamily
    === 'windows') {
    await setWindowsHostname({
      hostname: name,
      name,
    },);
  }
  rl.info(`VM ${name} is ready. Connect with: mvm shell ${name}`,);
}

//endregion Create

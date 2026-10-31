#!/usr/bin/env node
/**
 Imports an already-built qcow2 disk image into libvirt as a VM.
 Skips the container build and bootc-image-builder conversion steps.
 Use this after `mise run ...:run` has built the qcow2 at least once.
 
 Run: mise run //package/vm-builder/cli:import
 */
import {
  findRootCached,
  packageNamed,
} from '@monochromatic-dev/fs-path-core/ts';
import { spawn as nodeSpawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  copyFile,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { generateDomainXml, } from './domain-xml.ts';
import { exec, } from './exec.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 Libvirt domain name.
 */
const VM_NAME = 'monochromatic-dev';

/**
 VM memory in MiB (16 GiB).
 */
const VM_MEMORY_MIB = '16384';

/**
 Virtual CPU count.
 */
const VM_VCPUS = '8';

/**
 Resolved absolute path to the vm-builder package directory:
 the nearest ancestor of this file whose manifest names this package.
 */
const PACKAGE_DIR = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/vm-builder-cli',),
},);

/**
 Directory where bootc-image-builder wrote its output.
 */
const OUTPUT_DIR = join(
  PACKAGE_DIR,
  'output',
);

/**
 Path to the qcow2 disk image built by bootc-image-builder.
 */
const BUILD_QCOW2_PATH = join(
  OUTPUT_DIR,
  'qcow2',
  'disk.qcow2',
);

/**
 Path where the qcow2 is copied for libvirt access.
 `/var/lib/libvirt/images/` has SELinux context `virt_image_t`,
 which allows QEMU (running in `svirt_t`) to read and write the image.
 Files in the user's home directory have `user_home_t` which QEMU cannot access.
 */
const LIBVIRT_IMAGES_DIR = '/var/lib/libvirt/images';

/**
 Final qcow2 path under the libvirt images directory for QEMU access.
 */
const QCOW2_PATH = join(
  LIBVIRT_IMAGES_DIR,
  'monochromatic-dev.qcow2',
);

/**
 libvirt session URI: no sudo needed.
 */
const LIBVIRT_URI = 'qemu:///session';

/**
 Spawns a command with inherited stdio for progress feedback.
 
 @param cmd - Executable name
 
 @param args - Arguments passed to the command
 
 @throws When the command exits with a non-zero code
 */
async function run(
  {
    cmd,
    args,
  }: {
    readonly cmd: string;
    readonly args: readonly string[];
  },
): Promise<void> {
  /**
   Spawned child process with inherited stdio; awaited via `once(child, 'close')` for the exit code.
   */
  const child = nodeSpawn(
    cmd,
    [...args,],
    { stdio: 'inherit', },
  );
  await once(
    child,
    'close',
  );
  if (child.exitCode !== 0)
    throw new Error(`${cmd} exited with code ${String(child.exitCode,)}`,);
}

/**
 Destroys and undefines an existing libvirt domain if it exists.
 No-op when the domain is not defined; the existence probe's failure is
 logged via {@link caughtValueText}.
 
 @param name - Libvirt domain name to remove
 */
async function undefineVmIfExists(name: string,): Promise<void> {
  try {
    await exec({
      cmd: 'virsh',
      args: [
        '--connect',
        LIBVIRT_URI,
        'dominfo',
        name,
      ],
    },);
  }
  catch (error) {
    console.warn(
      `[vm-builder] libvirt domain '${name}' existence probe failed; skipping removal: ${caughtValueText(error,)}`,
    );
    return;
  }
  console.log(`[vm-builder] removing existing VM '${name}'...`,);
  /**
   Current domain state from `virsh domstate`; `'running'` requires `destroy` before `undefine`.
   */
  const state = (await exec({
    cmd: 'virsh',
    args: [
      '--connect',
      LIBVIRT_URI,
      'domstate',
      name,
    ],
  },))
    .trim();
  if (state === 'running') {
    await exec({
      cmd: 'virsh',
      args: [
        '--connect',
        LIBVIRT_URI,
        'destroy',
        name,
      ],
    },);
  }
  await exec({
    cmd: 'virsh',
    args: [
      '--connect',
      LIBVIRT_URI,
      'undefine',
      name,
      '--nvram',
    ],
  },);
}

/**
 Imports the qcow2 disk image into libvirt using XML rendered by
 {@link generateDomainXml}.
 
 @param name - Libvirt domain name to create
 */
async function importVm(name: string,): Promise<void> {
  console.log(`[vm-builder] importing '${name}' into libvirt...`,);
  /**
   Domain XML rendered for {@link name}; consumed by `virsh define`.
   */
  const xml = generateDomainXml({
    name,
    memoryMib: VM_MEMORY_MIB,
    vcpus: VM_VCPUS,
    qcow2Path: QCOW2_PATH,
  },);
  /**
   On-disk location of {@link xml}; `virsh define` reads from this path, not stdin.
   */
  const xmlPath = join(
    OUTPUT_DIR,
    'domain.xml',
  );
  await writeFile(
    xmlPath,
    xml,
  );
  await exec({
    cmd: 'virsh',
    args: [
      '--connect',
      LIBVIRT_URI,
      'define',
      xmlPath,
    ],
  },);
}

/**
 Grants the virt-manager Flatpak read-write access to {@link OUTPUT_DIR}.
 No-op if virt-manager is not installed as a Flatpak; the probe's failure
 is logged via {@link caughtValueText}.
 */
async function grantFlatpakAccess(): Promise<void> {
  try {
    await exec({
      cmd: 'flatpak',
      args: [
        'info',
        'org.virt_manager.virt-manager',
      ],
    },);
  }
  catch (error) {
    console.warn(
      `[vm-builder] virt-manager Flatpak probe failed; skipping filesystem override: ${caughtValueText(error,)}`,
    );
    return;
  }
  console.log(
    '[vm-builder] granting virt-manager Flatpak access to output directory...',
  );
  await exec({
    cmd: 'flatpak',
    args: [
      'override',
      '--user',
      `--filesystem=${OUTPUT_DIR}`,
      'org.virt_manager.virt-manager',
    ],
  },);
}

/**
 Copies the built qcow2 to {@link LIBVIRT_IMAGES_DIR} where SELinux
 labels it `virt_image_t`, allowing QEMU to access it.
 Uses `sudo cp` because {@link LIBVIRT_IMAGES_DIR} is root-owned,
 then restores ownership to the current user.
 */
async function copyToLibvirtImages(): Promise<void> {
  console.log(`[vm-builder] copying qcow2 to ${LIBVIRT_IMAGES_DIR}...`,);
  await run({
    cmd: 'sudo',
    args: [
      'cp',
      BUILD_QCOW2_PATH,
      QCOW2_PATH,
    ],
  },);
  /**
   Login name used to restore ownership of the qcow2 after the root-owned `sudo cp`.
   */
  const currentUser = process.env
    .USER
    ?? 'user';
  await run({
    cmd: 'sudo',
    args: [
      'chown',
      `${currentUser}:${currentUser}`,
      QCOW2_PATH,
    ],
  },);
}

await undefineVmIfExists(VM_NAME,);
await copyToLibvirtImages();
await importVm(VM_NAME,);
await grantFlatpakAccess();

console.log(
  `[vm-builder] done. Start the VM with: virsh --connect ${LIBVIRT_URI} start ${VM_NAME}\n  or open virt-manager and double-click ${VM_NAME}`,
);

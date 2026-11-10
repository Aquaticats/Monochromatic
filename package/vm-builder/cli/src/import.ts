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

import {
  DEV_VM_MEMORY_MIB,
  DEV_VM_NAME,
  DEV_VM_VCPUS,
  devVmDiskPaths,
  LIBVIRT_IMAGES_DIRECTORY,
  LIBVIRT_SESSION_URI,
} from './dev-vm.ts';
import { undefineVmIfExists, } from './domain-removal.ts';
import {
  copyDiskImageIntoLibvirt,
  grantFlatpakAccess,
  installDomainIntoLibvirt,
} from './libvirt-import.ts';

/**
 Resolved absolute path to the vm-builder package directory:
 the nearest ancestor of this file whose manifest names this package.
 */
const PACKAGE_DIR = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/vm-builder-cli',),
},);

/**
 Artifact paths of this checkout, including the qcow2 the converter wrote and the copy QEMU opens.
 */
const DISK_PATHS = devVmDiskPaths({
  packageDirectory: PACKAGE_DIR,
  libvirtImagesDirectory: LIBVIRT_IMAGES_DIRECTORY,
},);

/**
 Login name the copy is handed back to after the root-owned `sudo cp`.
 */
const CURRENT_USER = process.env
  .USER
  ?? 'user';

await undefineVmIfExists({
  name: DEV_VM_NAME,
  uri: LIBVIRT_SESSION_URI,
},);
await copyDiskImageIntoLibvirt({
  sourcePath: DISK_PATHS.builderQcow2Path,
  destinationPath: DISK_PATHS.libvirtQcow2Path,
  owner: CURRENT_USER,
},);
await installDomainIntoLibvirt({
  name: DEV_VM_NAME,
  memoryMib: DEV_VM_MEMORY_MIB,
  vcpus: DEV_VM_VCPUS,
  uri: LIBVIRT_SESSION_URI,
  qcow2Path: DISK_PATHS.libvirtQcow2Path,
  outputDirectory: DISK_PATHS.outputDirectory,
},);
await grantFlatpakAccess({ outputDirectory: DISK_PATHS.outputDirectory, },);

console.log(
  `[vm-builder] done. Start the VM with: virsh --connect ${LIBVIRT_SESSION_URI} start ${DEV_VM_NAME}\n  or open virt-manager and double-click ${DEV_VM_NAME}`,
);

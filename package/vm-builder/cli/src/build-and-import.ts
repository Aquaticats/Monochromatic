#!/usr/bin/env node
/**
 Orchestrates the full VM build pipeline:
 1. Build OCI container image from Containerfile (rootful podman, for bootc-image-builder compatibility)
 2. Convert to qcow2 via bootc-image-builder (rootful podman, privileged)
 3. Fix file ownership after the root-privileged steps
 4. Remove existing libvirt domain of the same name if present
 5. Import the qcow2 as a new libvirt domain accessible from virt-manager
 
 Cosign signing is a separate step (`mise run ...:push`) because `cosign sign`
 pushes signatures to a registry. For local-only builds there is no registry;
 signing is only needed before `podman push` to GHCR.
 
 Prerequisites on the host:
 - podman (rootful via sudo, for both image build and conversion)
 - virsh (libvirt-client package)
 
 Run: mise run //package/vm-builder/cli:run
 */
import {
  findRootCached,
  MISE_MONOREPO,
  packageNamed,
} from '@monochromatic-dev/fs-path-core/ts';
import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  bootcConverterArguments,
  imageBuildArguments,
} from './container-arguments.ts';
import {
  DEV_VM_IMAGE_TAG,
  DEV_VM_MEMORY_MIB,
  DEV_VM_NAME,
  DEV_VM_VCPUS,
  devVmDiskPaths,
  LIBVIRT_IMAGES_DIRECTORY,
  LIBVIRT_SESSION_URI,
} from './dev-vm.ts';
import { undefineVmIfExists, } from './domain-removal.ts';
import { execStreaming, } from './exec.ts';
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
 Absolute path to the monorepo root.
 Used as the podman build context so the Containerfile can COPY from
 sibling packages (e.g. `package/config/dotfiles/`).
 */
const MONOREPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/**
 Artifact paths of this checkout: the converter's output and the copy QEMU opens.
 */
const DISK_PATHS = devVmDiskPaths({
  packageDirectory: PACKAGE_DIR,
  libvirtImagesDirectory: LIBVIRT_IMAGES_DIRECTORY,
},);

/**
 Login name that owns the build output after the root-privileged steps.
 */
const CURRENT_USER = process.env
  .USER
  ?? 'user';

/**
 Builds the OCI container image from Containerfile using rootful podman.
 Rootful is required because bootc-image-builder in the next step resolves images
 from rootful storage (`/var/lib/containers/storage`), not user storage.
 
 Build context is the monorepo root so the Containerfile can COPY from
 sibling packages (e.g. `package/config/dotfiles/`).
 
 The resulting image is tagged as {@link DEV_VM_IMAGE_TAG} in the rootful container store.
 */
async function buildImage(): Promise<void> {
  console.log('[vm-builder] building container image (rootful)...',);
  await execStreaming({
    cmd: 'sudo',
    args: [
      ...imageBuildArguments({
        imageTag: DEV_VM_IMAGE_TAG,
        containerfile: join(
          PACKAGE_DIR,
          'Containerfile',
        ),
        ignorefile: join(
          PACKAGE_DIR,
          '.containerignore',
        ),
        contextDirectory: MONOREPO_ROOT,
      },),
    ],
  },);
}

/**
 Converts the container image to a qcow2 disk image using bootc-image-builder.
 
 Runs as root (sudo podman) because bootc-image-builder requires:
 - `--privileged` for device access during disk partitioning
 - Access to `/var/lib/containers/storage` (the rootful podman image store)
 
 Disk layout is controlled by `disk_config/disk.toml`.
 Output is written to the `qcow2/disk.qcow2` path {@link devVmDiskPaths} resolves.
 */
async function convertToQcow2(): Promise<void> {
  console.log('[vm-builder] converting to qcow2 with bootc-image-builder...',);
  await mkdir(
    DISK_PATHS.outputDirectory,
    { recursive: true, },
  );
  /**
   Host path mounted into the bootc-image-builder container at `/config` for `--config /config/disk.toml`.
   */
  const diskConfigDir = join(
    PACKAGE_DIR,
    'disk_config',
  );
  await execStreaming({
    cmd: 'sudo',
    args: [
      ...bootcConverterArguments({
        outputDirectory: DISK_PATHS.outputDirectory,
        diskConfigDirectory: diskConfigDir,
        imageTag: DEV_VM_IMAGE_TAG,
      },),
    ],
  },);
}

/**
 Restores ownership of the output directory to the current user.
 Both `podman build` and bootc-image-builder run as root,
 leaving all output files owned by root:root.
 Restoring ownership lets virt-manager and the QEMU process it launches read the qcow2 as a normal user.
 */
async function fixOwnership(): Promise<void> {
  console.log('[vm-builder] restoring output ownership to current user...',);
  await execStreaming({
    cmd: 'sudo',
    args: [
      'chown',
      '-R',
      `${CURRENT_USER}:${CURRENT_USER}`,
      DISK_PATHS.outputDirectory,
    ],
  },);
}

await buildImage();
await convertToQcow2();
await fixOwnership();
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

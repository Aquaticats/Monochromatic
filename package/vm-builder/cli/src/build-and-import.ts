#!/usr/bin/env node
/**
 Orchestrates the full VM build pipeline:
 1. Build OCI container image from Containerfile (rootful podman, for bootc-image-builder compatibility)
 2. Convert to qcow2 via bootc-image-builder (rootful podman, privileged)
 3. Fix file ownership after the root-privileged steps
 4. Remove existing libvirt domain of the same name if present
 5. Import the qcow2 as a new libvirt domain accessible from virt-manager
 
 Cosign signing is a separate step (`mise run ...:sign`) because `cosign sign`
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
import { spawn as nodeSpawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { generateDomainXml, } from './domain-xml.ts';
import { exec, } from './exec.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 OCI image tag produced by the podman build step.
 */
const IMAGE_TAG = 'localhost/monochromatic-dev:latest';

/**
 Libvirt domain name used for virsh define.
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
 Absolute path to the monorepo root.
 Used as the podman build context so the Containerfile can COPY from
 sibling packages (e.g. `package/config/dotfiles/`).
 */
const MONOREPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/**
 Directory where bootc-image-builder writes its output.
 Created automatically before the conversion step.
 Gitignored; not checked in.
 */
const OUTPUT_DIR = join(
  PACKAGE_DIR,
  'output',
);

/**
 Path to the qcow2 disk image produced by bootc-image-builder.
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
 Current user login name; used to restore ownership after the privileged build step.
 */
const CURRENT_USER = process.env
  .USER
  ?? 'user';

/**
 libvirt session URI: connects to the user's QEMU/KVM daemon (no sudo needed).
 */
const LIBVIRT_URI = 'qemu:///session';

//region Streaming process runner

/**
 Spawns a command with inherited stdio so output streams to the terminal in real time.
 Used for long-running commands (podman build, bootc-image-builder) where progress
 feedback matters.
 
 @param cmd - Executable name
 
 @param args - Arguments passed to the command
 
 @throws When the command exits with a non-zero code
 
 @example
 ```ts
 await run({ cmd: 'podman', args: ['build', '--tag', 'myimage', '.'] });
 ```
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

//endregion Streaming process runner

/**
 Builds the OCI container image from Containerfile using rootful podman.
 Rootful is required because bootc-image-builder in the next step resolves images
 from rootful storage (`/var/lib/containers/storage`), not user storage.
 
 Build context is the monorepo root so the Containerfile can COPY from
 sibling packages (e.g. `package/config/dotfiles/`).
 
 The resulting image is tagged as {@link IMAGE_TAG} in the rootful container store.
 */
async function buildImage(): Promise<void> {
  console.log('[vm-builder] building container image (rootful)...',);
  await run({
    cmd: 'sudo',
    args: [
      'podman',
      'build',
      '--tag',
      IMAGE_TAG,
      '--file',
      join(
        PACKAGE_DIR,
        'Containerfile',
      ),
      '--ignorefile',
      join(
        PACKAGE_DIR,
        '.containerignore',
      ),
      MONOREPO_ROOT,
    ],
  },);
}

/**
 Converts the container image to a qcow2 disk image using bootc-image-builder.
 
 Runs as root (sudo podman) because bootc-image-builder requires:
 - `--privileged` for device access during disk partitioning
 - Access to `/var/lib/containers/storage` (the rootful podman image store)
 
 Disk layout is controlled by `disk_config/disk.toml`.
 Output is written to `{@link OUTPUT_DIR}/qcow2/disk.qcow2`.
 */
async function convertToQcow2(): Promise<void> {
  console.log('[vm-builder] converting to qcow2 with bootc-image-builder...',);
  await mkdir(
    OUTPUT_DIR,
    { recursive: true, },
  );
  /**
   Host path mounted into the bootc-image-builder container at `/config` for `--config /config/disk.toml`.
   */
  const diskConfigDir = join(
    PACKAGE_DIR,
    'disk_config',
  );
  await run({
    cmd: 'sudo',
    args: [
      'podman',
      'run',
      '--rm',
      '--privileged',
      '--security-opt',
      'label=type:unconfined_t',
      '--volume',
      `${OUTPUT_DIR}:/output`,
      '--volume',
      '/var/lib/containers/storage:/var/lib/containers/storage',
      '--volume',
      `${diskConfigDir}:/config:ro`,
      'quay.io/centos-bootc/bootc-image-builder:latest',
      '--type',
      'qcow2',
      '--config',
      '/config/disk.toml',
      IMAGE_TAG,
    ],
  },);
}

/**
 Restores ownership of {@link OUTPUT_DIR} to the current user.
 Both `podman build` and bootc-image-builder run as root,
 leaving all output files owned by root:root.
 Restoring ownership lets virt-install read the qcow2 as a normal user.
 */
async function fixOwnership(): Promise<void> {
  console.log('[vm-builder] restoring output ownership to current user...',);
  await run({
    cmd: 'sudo',
    args: [
      'chown',
      '-R',
      `${CURRENT_USER}:${CURRENT_USER}`,
      OUTPUT_DIR,
    ],
  },);
}

/**
 Destroys and undefines an existing libvirt domain if it exists.
 No-op when the domain is not defined; the existence probe's failure is
 logged via {@link caughtValueText}.
 Forcibly destroys a running domain before undefining to avoid an "active domain" error.
 
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
    // Domain not defined; nothing to remove.
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
 Imports the qcow2 disk image into libvirt as a new domain.
 Writes domain XML rendered by {@link generateDomainXml} to a file and
 defines it via `virsh define`.
 The VM is registered but not started; open virt-manager or run
 `virsh --connect qemu:///session start <name>` to boot it.
 
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
 Grants the virt-manager Flatpak read-write access to {@link OUTPUT_DIR}
 so QEMU (launched by the Flatpak) can open and write to the qcow2 disk image.
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
    // virt-manager is not a Flatpak; no override needed.
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
  await run({
    cmd: 'sudo',
    args: [
      'chown',
      `${CURRENT_USER}:${CURRENT_USER}`,
      QCOW2_PATH,
    ],
  },);
}

await buildImage();
await convertToQcow2();
await fixOwnership();
await copyToLibvirtImages();
await undefineVmIfExists(VM_NAME,);
await importVm(VM_NAME,);
await grantFlatpakAccess();

console.log(
  `[vm-builder] done. Start the VM with: virsh --connect ${LIBVIRT_URI} start ${VM_NAME}\n  or open virt-manager and double-click ${VM_NAME}`,
);

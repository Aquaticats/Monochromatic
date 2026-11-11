/**
 Identity and disk paths of the development VM the build pipeline produces.

 Three entry scripts resolve the same names and the same paths, so the domain name, the guest size and every artifact
 path live here once, and a unit test checks them without a hypervisor or a container runtime.
 */
import { join, } from 'node:path';

//region Identity

/**
 Libvirt domain name, and the base name of the qcow2 copy QEMU opens.

 Fixed because the `mise` tasks and the README address the domain by this name.
 */
export const DEV_VM_NAME = 'monochromatic-dev';

/**
 Tag the rootful image build writes into the local container store.
 */
export const DEV_VM_IMAGE_TAG = 'localhost/monochromatic-dev:latest';

/**
 The same image as {@link DEV_VM_IMAGE_TAG} under the registry account the push step publishes to.
 */
export const DEV_VM_REGISTRY_IMAGE_TAG = 'ghcr.io/aquaticat/monochromatic-dev:latest';

/**
 Guest memory in MiB: the 16 GiB the README documents.
 */
export const DEV_VM_MEMORY_MIB = '16384';

/**
 Guest vCPU count, which the README documents beside the memory.
 */
export const DEV_VM_VCPUS = '8';

/**
 libvirt connection URI: the user's session daemon, so `virsh` needs no root connection.
 */
export const LIBVIRT_SESSION_URI = 'qemu:///session';

/**
 Directory QEMU may open a disk image from.

 `/var/lib/libvirt/images/` carries the SELinux context `virt_image_t`, which QEMU (`svirt_t`) reads and writes, while a
 file under a home directory carries `user_home_t`, which it cannot open.
 */
export const LIBVIRT_IMAGES_DIRECTORY = '/var/lib/libvirt/images';

//endregion Identity

//region Disk paths

/**
 Where one run keeps the artifacts it builds and the copy libvirt boots.
 */
export type DevVmDiskPaths = {
  /**
   Directory bootc-image-builder writes into, and the one the domain document is kept in.
   */
  readonly outputDirectory: string;
  /**
   Qcow2 the converter produced, before it is copied for QEMU.
   */
  readonly builderQcow2Path: string;
  /**
   Copy QEMU opens, carrying the SELinux context of its directory.
   */
  readonly libvirtQcow2Path: string;
  /**
   Document `virsh define` reads; `virsh` takes a path, not stdin.
   */
  readonly domainXmlPath: string;
};

/**
 Resolves the artifact paths of one package checkout.

 @param packageDirectory - absolute path of this package, which holds the `output` directory

 @param libvirtImagesDirectory - directory the copy QEMU opens is written into

 @returns Paths the converter, the copy and `virsh define` are given

 @example
 ```ts
 const paths = devVmDiskPaths({
   packageDirectory: '/repo/package/vm-builder/cli',
   libvirtImagesDirectory: LIBVIRT_IMAGES_DIRECTORY,
 },);
 ```
 */
export function devVmDiskPaths(
  {
    packageDirectory,
    libvirtImagesDirectory,
  }: {
    readonly packageDirectory: string;
    readonly libvirtImagesDirectory: string;
  },
): DevVmDiskPaths {
  /**
   Directory every artifact of this checkout is written to.
   */
  const outputDirectory = join(
    packageDirectory,
    'output',
  );
  return {
    outputDirectory,
    builderQcow2Path: join(
      outputDirectory,
      'qcow2',
      'disk.qcow2',
    ),
    libvirtQcow2Path: join(
      libvirtImagesDirectory,
      `${DEV_VM_NAME}.qcow2`,
    ),
    domainXmlPath: join(
      outputDirectory,
      'domain.xml',
    ),
  };
}

//endregion Disk paths

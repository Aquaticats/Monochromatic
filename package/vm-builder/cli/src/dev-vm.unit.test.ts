/**
 Identity and disk paths of the development VM, checked as the entry scripts use them so a moved artifact breaks a test
 instead of a build.
 */

import {
  basename,
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  devVmDiskPaths,
  DEV_VM_IMAGE_TAG,
  DEV_VM_MEMORY_MIB,
  DEV_VM_NAME,
  DEV_VM_REGISTRY_IMAGE_TAG,
  DEV_VM_VCPUS,
  LIBVIRT_IMAGES_DIRECTORY,
  LIBVIRT_SESSION_URI,
} from '../dist/final/node/dev-vm.mjs';

/**
 Package directory every path case resolves against, spelled out so no case depends on where this repository is
 checked out.
 */
const FIXTURE_PACKAGE_DIRECTORY = '/repo/package/vm-builder/cli';

/**
 Builds the paths of one case.

 @param libvirtImagesDirectory - images directory the case needs

 @returns Paths of the fixture package resolved against that directory
 */
function pathsOf(libvirtImagesDirectory: string,): ReturnType<typeof devVmDiskPaths> {
  return devVmDiskPaths({
    packageDirectory: FIXTURE_PACKAGE_DIRECTORY,
    libvirtImagesDirectory,
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: devVmDiskPaths.name,
      children: [
        it({
          name: 'puts every build artifact under the package output directory',
          fn: async () => {
            /**
             Directory the converter writes into.
             */
            const outputDirectory = join(
              FIXTURE_PACKAGE_DIRECTORY,
              'output',
            );
            const paths = pathsOf(LIBVIRT_IMAGES_DIRECTORY,);
            expect(paths.outputDirectory,).toBe(outputDirectory,);
            expect(paths.builderQcow2Path,).toBe(join(
              outputDirectory,
              'qcow2',
              'disk.qcow2',
            ),);
            expect(paths.domainXmlPath,).toBe(join(
              outputDirectory,
              'domain.xml',
            ),);
          },
        },),
        it({
          name: 'names the libvirt copy after the domain and keeps it in the images directory',
          fn: async () => {
            const paths = pathsOf(LIBVIRT_IMAGES_DIRECTORY,);
            expect(dirname(paths.libvirtQcow2Path,),).toBe(LIBVIRT_IMAGES_DIRECTORY,);
            expect(basename(paths.libvirtQcow2Path,),).toBe(`${DEV_VM_NAME}.qcow2`,);
          },
        },),
        it({
          name: 'puts the libvirt copy under the images directory the caller names',
          fn: async () => {
            /**
             Disposable images directory, so this case never spells the host path.
             */
            const libvirtImagesDirectory = join(
              scratchRoot(),
              'images',
            );
            const paths = pathsOf(libvirtImagesDirectory,);
            expect(paths.libvirtQcow2Path,).toBe(join(
              libvirtImagesDirectory,
              `${DEV_VM_NAME}.qcow2`,
            ),);
          },
        },),
      ],
    },),
    describe({
      name: 'the dev VM constants',
      children: [
        it({
          name: 'keeps the domain name the mise tasks and the README address',
          fn: async () => {
            expect(DEV_VM_NAME,).toBe('monochromatic-dev',);
          },
        },),
        it({
          name: 'publishes one image name under the local store and GHCR',
          fn: async () => {
            expect(DEV_VM_IMAGE_TAG,).toBe('localhost/monochromatic-dev:latest',);
            expect(DEV_VM_REGISTRY_IMAGE_TAG,).toBe('ghcr.io/aquaticat/monochromatic-dev:latest',);
          },
        },),
        it({
          name: 'asks for the guest size the README documents',
          fn: async () => {
            /**
             MiB in the 16 GiB the README promises.
             */
            const MIB_PER_GIB = 1_024;
            expect(Number(DEV_VM_MEMORY_MIB,),).toBe(16 * MIB_PER_GIB,);
            expect(DEV_VM_VCPUS,).toBe('8',);
          },
        },),
        it({
          name: 'targets the session daemon so virsh needs no root connection',
          fn: async () => {
            expect(LIBVIRT_SESSION_URI,).toBe('qemu:///session',);
          },
        },),
        it({
          name: 'keeps the libvirt copy in the SELinux-labeled directory QEMU may open',
          fn: async () => {
            expect(LIBVIRT_IMAGES_DIRECTORY,).toBe('/var/lib/libvirt/images',);
          },
        },),
      ],
    },),
  ],
},);

/**
 Puts the built qcow2 into libvirt as a domain: the copy QEMU may open, the definition `virsh define` installs, and the
 Flatpak grant virt-manager needs to reach the disk.

 `import.ts` and `build-and-import.ts` both run these steps, so each command has one owner and one unit test that records
 it instead of running it: this host has no hypervisor, and a test must not create or change a libvirt domain.
 */
import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { generateDomainXml, } from './domain-xml.ts';
import {
  type CommandRunner,
  exec,
} from './exec.ts';

/**
 Copies the built qcow2 where QEMU may open it, then hands the copy to the user running the pipeline.

 Both commands run as root: the destination directory is root-owned, and the root-owned `sudo cp` leaves the copy owned
 by root, which the user's QEMU process could not write.

 @param sourcePath - Qcow2 the converter produced

 @param destinationPath - Path of the copy inside the libvirt images directory

 @param owner - Login name the copy is handed back to

 @param runner - Command runner; tests pass one that records the calls instead of running them

 @throws `SubprocessError` from {@link exec} when the copy or the ownership change fails

 @example
 ```ts
 await copyDiskImageIntoLibvirt({
   sourcePath: '/repo/package/vm-builder/cli/output/qcow2/disk.qcow2',
   destinationPath: '/var/lib/libvirt/images/monochromatic-dev.qcow2',
   owner: 'user',
 },);
 ```
 */
export async function copyDiskImageIntoLibvirt(
  {
    sourcePath,
    destinationPath,
    owner,
    runner = exec,
  }: {
    readonly sourcePath: string;
    readonly destinationPath: string;
    readonly owner: string;
    readonly runner?: CommandRunner;
  },
): Promise<void> {
  console.log(`[vm-builder] copying qcow2 to ${destinationPath}...`,);
  await runner({
    cmd: 'sudo',
    args: [
      'cp',
      sourcePath,
      destinationPath,
    ],
  },);
  await runner({
    cmd: 'sudo',
    args: [
      'chown',
      `${owner}:${owner}`,
      destinationPath,
    ],
  },);
}

/**
 Renders the domain document into the output directory and defines it on the given connection.

 The document is written first because `virsh define` reads a path, and keeping it beside the built image lets a later run
 inspect what the domain was defined from.

 @param name - Libvirt domain name to create

 @param memoryMib - Guest memory in MiB as the renderer takes it

 @param vcpus - Guest vCPU count as the renderer takes it

 @param uri - Libvirt connection URI the definition is made on

 @param qcow2Path - Absolute path of the disk image the domain boots

 @param outputDirectory - Directory the document is written into

 @param runner - Command runner; tests pass one that records the calls instead of defining a domain

 @throws `SubprocessError` from {@link exec} when `virsh define` fails

 @example
 ```ts
 await installDomainIntoLibvirt({
   name: 'monochromatic-dev',
   memoryMib: '16384',
   vcpus: '8',
   uri: 'qemu:///session',
   qcow2Path: '/var/lib/libvirt/images/monochromatic-dev.qcow2',
   outputDirectory: '/repo/package/vm-builder/cli/output',
 },);
 ```
 */
export async function installDomainIntoLibvirt(
  {
    name,
    memoryMib,
    vcpus,
    uri,
    qcow2Path,
    outputDirectory,
    runner = exec,
  }: {
    readonly name: string;
    readonly memoryMib: string;
    readonly vcpus: string;
    readonly uri: string;
    readonly qcow2Path: string;
    readonly outputDirectory: string;
    readonly runner?: CommandRunner;
  },
): Promise<void> {
  console.log(`[vm-builder] importing '${name}' into libvirt...`,);
  /**
   On-disk location of the rendered document; `virsh define` reads from this path, not stdin.
   */
  const domainXmlPath = join(
    outputDirectory,
    'domain.xml',
  );
  await writeFile(
    domainXmlPath,
    generateDomainXml({
      name,
      memoryMib,
      vcpus,
      qcow2Path,
    },),
  );
  await runner({
    cmd: 'virsh',
    args: [
      '--connect',
      uri,
      'define',
      domainXmlPath,
    ],
  },);
}

/**
 Grants the virt-manager Flatpak read-write access to the output directory.

 No-op when virt-manager is not installed as a Flatpak: the probe's failure is logged via {@link caughtValueText} and the
 pipeline carries on, because the domain is defined either way.

 @param outputDirectory - Directory the Flatpak is granted

 @param runner - Command runner; tests pass one that records the calls instead of overriding a Flatpak

 @throws `SubprocessError` from {@link exec} when the probe succeeds and the override fails

 @example
 ```ts
 await grantFlatpakAccess({ outputDirectory: '/repo/package/vm-builder/cli/output', },);
 ```
 */
export async function grantFlatpakAccess(
  {
    outputDirectory,
    runner = exec,
  }: {
    readonly outputDirectory: string;
    readonly runner?: CommandRunner;
  },
): Promise<void> {
  try {
    await runner({
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
  console.log('[vm-builder] granting virt-manager Flatpak access to output directory...',);
  await runner({
    cmd: 'flatpak',
    args: [
      'override',
      '--user',
      `--filesystem=${outputDirectory}`,
      'org.virt_manager.virt-manager',
    ],
  },);
}

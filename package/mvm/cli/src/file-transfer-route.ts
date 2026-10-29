/**
 Decides, before a domain is defined, how its files will move between host
 and guest: through the virtiofs shared directory, or through the guest agent
 on a host where libvirt finds no `virtiofsd`.

 libvirt is asked directly: `virsh domxml-to-native` converts a minimal domain
 holding only the share into a QEMU command line without defining or starting
 anything, and fails with the same "Unable to find a satisfying virtiofsd"
 a real start would. Looking for a `virtiofsd` executable instead would look
 in the wrong place whenever libvirt runs in a sandbox.

 @module
 */

import {
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { hXml as h, } from '@monochromatic-dev/module-hyperscript/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { VM_PREFIX, } from './config.ts';
import {
  sharedMemoryBacking,
  virtiofsShare,
} from './domain-xml-builders.ts';
import type { FileTransferRoute, } from './meta.ts';
import { CommandFailedError, } from './spawn-errors.ts';
import { virsh, } from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Text of libvirt's error when no `virtiofsd` satisfies a virtiofs device
 (`qemuVhostUserFillDomainFS` in libvirt's `src/qemu/qemu_vhost_user.c`).
 */
const NO_VIRTIOFSD = 'Unable to find a satisfying virtiofsd';

/**
 File name of the probe domain's XML inside the VM directory.
 The directory is used because libvirt must be able to read the file, and it reads the VM's domain XML from there too.
 */
const PROBE_FILE_NAME = 'virtiofs-probe.xml';

/**
 Memory of the probe domain in MiB; the domain is never started.
 */
const PROBE_MEMORY_MIB = 64;

/**
 Builds a minimal domain that has the share and its memory backing and nothing else,
 so the conversion can only fail for a reason that concerns the share.

 @param sharedDir - Existing host directory named as the share's source

 @returns Domain XML for the probe

 @example
 ```ts
 probeDomainXml('/vms/dev');
 ```
 */
function probeDomainXml(sharedDir: string,): string {
  return h({
    tag: 'domain',
    attrs: { type: 'kvm', },
    children: [
      h({
        tag: 'name',
        text: `${VM_PREFIX}virtiofs-probe`,
      },),
      h({
        tag: 'memory',
        attrs: { unit: 'MiB', },
        text: String(PROBE_MEMORY_MIB,),
      },),
      h({
        tag: 'vcpu',
        text: '1',
      },),
      sharedMemoryBacking(),
      h({
        tag: 'os',
        children: [
          h({
            tag: 'type',
            attrs: { arch: 'x86_64', },
            text: 'hvm',
          },),
        ],
      },),
      h({
        tag: 'devices',
        children: [virtiofsShare(sharedDir,),],
      },),
    ],
  },);
}

/**
 Chooses how a new VM's files will move between host and guest.

 @param vmDir - Existing VM directory; the probe's XML is written there and removed again, and the directory stands in as the share's source

 @returns `virtiofs` when libvirt can run the share, `guest-agent` when it finds no `virtiofsd`

 @throws {@link LibvirtSessionUnavailableError} when virsh cannot reach libvirt

 @example
 ```ts
 const route = await chooseFileTransferRoute('/vms/dev');
 ```
 */
export async function chooseFileTransferRoute(vmDir: string,): Promise<FileTransferRoute> {
  /**
   Logger scoped to the choice so the route taken is attributable.
   */
  const rl = tagged({
    tag: chooseFileTransferRoute.name,
    l,
  },);
  /**
   Path of the probe's XML.
   */
  const probePath = join(
    vmDir,
    PROBE_FILE_NAME,
  );
  await writeFile(
    probePath,
    probeDomainXml(vmDir,),
  );
  /**
   Removes the probe's XML when the choice is made, whatever it is.
   */
  await using _probeFile = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        probePath,
        { force: true, },
      );
    },
  };
  try {
    await virsh({
      args: [
        'domxml-to-native',
        'qemu-argv',
        '--xml',
        probePath,
      ],
    },);
    rl.debug('libvirt can run a virtiofs share; files move through the shared directory',);
    return 'virtiofs';
  }
  catch (error) {
    if (!(error instanceof CommandFailedError))
      throw error;

    if (error.stderr
      .includes(NO_VIRTIOFSD,)) {
      rl.info(
        'libvirt finds no virtiofsd on this host: the VM is defined without the shared directory, and push and pull go through the guest agent',
      );
      return 'guest-agent';
    }
    rl.warn(
      `could not tell whether libvirt can run a virtiofs share, so the shared directory is kept: ${error.message}`,
    );
    return 'virtiofs';
  }
}

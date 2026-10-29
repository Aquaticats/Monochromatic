/**
 VirtIO disk bus verification phase for Windows template creation.
 
 After the initial SATA-based Windows installation, switches the VM
 to VirtIO disk bus and verifies Windows boots correctly with the
 VirtIO storage driver.
 
 @module
 */

import { VIRTIO_VERIFY_AGENT_TIMEOUT_MS, } from './config.ts';
import { domainXml, } from './domain-xml.ts';
import { TEMPLATE_VM_NAME, } from './template-shared.ts';
import {
  shutdownVm,
  waitForGuestAgent,
  waitForShutdown,
} from './virsh-wait.ts';
import {
  defineVm,
  startVm,
  undefineVm,
} from './virsh.ts';

/**
 Switches the template VM from SATA to VirtIO disk and verifies it boots.
 
 @param vmDir - VM directory for the domain XML
 
 @param diskPath - Path to the disk image
 
 @param rl - Logger for status messages
 
 @example
 ```ts
 await verifyVirtioBoot({ vmDir: '/vms/win11', diskPath: '/vms/win11/disk.qcow2', rl: console });
 ```
 */
export async function verifyVirtioBoot({
  vmDir,
  diskPath,
  rl,
}: {
  readonly vmDir: string;
  readonly diskPath: string;
  readonly rl: { readonly info: (msg: string,) => void; };
},): Promise<void> {
  rl.info('guest agent ready on SATA, switching to VirtIO disk bus...',);
  await shutdownVm({ name: TEMPLATE_VM_NAME, },);
  await waitForShutdown({ name: TEMPLATE_VM_NAME, },);

  // Redefine VM with VirtIO disk (no CDROMs needed, boot from hard disk)
  await undefineVm({ name: TEMPLATE_VM_NAME, },);
  /**
   Domain XML rebuilt with VirtIO disk for the post-install verify pass.
   */
  const virtioXml = domainXml({
    diskPath,
    name: TEMPLATE_VM_NAME,
    osFamily: 'windows',
  },);
  await defineVm({
    vmDir,
    xml: virtioXml,
  },);
  await startVm({ name: TEMPLATE_VM_NAME, },);

  rl.info('verifying Windows boots with VirtIO disk (waiting for guest agent)...',);
  await waitForGuestAgent({
    name: TEMPLATE_VM_NAME,
    timeoutMs: VIRTIO_VERIFY_AGENT_TIMEOUT_MS,
  },);

  rl.info('VirtIO boot verified, shutting down for template capture...',);
  await shutdownVm({ name: TEMPLATE_VM_NAME, },);
  await waitForShutdown({ name: TEMPLATE_VM_NAME, },);
}

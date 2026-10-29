/**
 Linux template baking pipeline.
 Creates a template from a cloud image by booting a temporary VM
 with cloud-init configured to install qemu-guest-agent, then converting
 the result to a standalone qcow2.
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  createSeedIso,
  NO_SEED_ISO,
} from './cloud-init.ts';
import {
  DEFAULT_DISK_SIZE,
  IMAGES_DIR,
  VMS_DIR,
} from './config.ts';
import { domainXml, } from './domain-xml.ts';
import { ensureImage, } from './image.ts';
import type { LinuxImageSpec, } from './registry.ts';
import { qemuImg, } from './qemu-img.ts';
import {
  TEMPLATE_VM_NAME,
  templateVmGuard,
} from './template-shared.ts';
import {
  shutdownVm,
  waitForGuestAgent,
  waitForShutdown,
} from './virsh-wait.ts';
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

/**
 Timeout for guest agent during Linux template creation.
 Longer than normal because cloud-init needs to install qemu-guest-agent.
 */
const LINUX_TEMPLATE_AGENT_TIMEOUT_MS = 120_000;

/**
 Creates a Linux template from a cloud image by booting a temporary VM
 with cloud-init configured to install qemu-guest-agent, then converting
 the result to a standalone qcow2.
 
 @param spec - Linux image specification from the registry
 
 @returns Absolute path to the baked template qcow2
 
 @throws Error when template creation fails
 
 @example
 ```ts
 const path = await ensureLinuxTemplate(IMAGES['ubuntu'] as LinuxImageSpec);
 ```
 */
export async function ensureLinuxTemplate(spec: LinuxImageSpec,): Promise<string> {
  /**
   Logger scoped to this template-bake call so log lines carry the function name.
   */
  const rl = tagged({
    tag: ensureLinuxTemplate.name,
    l,
  },);
  /**
   Final on-disk path for the baked template qcow2; written after the overlay is flattened.
   */
  const templatePath = join(
    IMAGES_DIR,
    spec.templateFileName,
  );

  rl.info(
    `creating template ${spec.templateFileName} with qemu-guest-agent pre-installed...`,
  );

  /**
   Cached cloud image used as the qcow2 backing file during the bake.
   */
  const baseImage = await ensureImage(spec,);
  /**
   Per-VM scratch directory for the install VM; holds the overlay disk and seed ISO.
   */
  const vmDir = join(
    VMS_DIR,
    TEMPLATE_VM_NAME,
  );
  await mkdir(
    vmDir,
    { recursive: true, },
  );

  /**
   Overlay qcow2 with `baseImage` as its backing file; the bake mutates this overlay.
   */
  const diskPath = join(
    vmDir,
    'disk.qcow2',
  );

  /**
   Disposable guard that tears down the template VM on scope exit, even on early throws.
   */
  await using _cleanup = templateVmGuard(rl,);

  rl.info('creating overlay disk from base image...',);
  await qemuImg({
    args: [
      'create',
      '-f',
      'qcow2',
      '-b',
      baseImage,
      '-F',
      'qcow2',
      diskPath,
      DEFAULT_DISK_SIZE,
    ],
  },);

  /**
   NoCloud seed ISO carrying cloud-init user-data that installs `qemu-guest-agent`; {@link NO_SEED_ISO} for Windows.
   */
  const seedIso = await createSeedIso({
    guest: spec,
    name: TEMPLATE_VM_NAME,
    template: true,
    vmDir,
  },);
  /**
   Libvirt domain XML for the temporary install VM.
   */
  const xml = domainXml({
    diskPath,
    name: TEMPLATE_VM_NAME,
    ...(seedIso !== NO_SEED_ISO ? { seedIsoPath: seedIso, } : {}),
  },);

  await defineVm({
    vmDir,
    xml,
  },);
  await startVm({ name: TEMPLATE_VM_NAME, },);
  await waitForGuestAgent({
    name: TEMPLATE_VM_NAME,
    timeoutMs: LINUX_TEMPLATE_AGENT_TIMEOUT_MS,
  },);

  rl.info('guest agent ready, shutting down template VM...',);
  await shutdownVm({ name: TEMPLATE_VM_NAME, },);
  await waitForShutdown({ name: TEMPLATE_VM_NAME, },);

  rl.info('converting overlay to standalone template image...',);
  await qemuImg({
    args: [
      'convert',
      '-O',
      'qcow2',
      diskPath,
      templatePath,
    ],
  },);

  rl.info(`template image saved to ${templatePath}`,);

  return templatePath;
}

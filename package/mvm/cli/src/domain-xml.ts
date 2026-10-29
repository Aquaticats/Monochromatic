import { hXml as h, } from '@monochromatic-dev/module-hyperscript/ts';

import {
  DEFAULT_MEMORY_MIB,
  DEFAULT_VCPUS,
  VM_PREFIX,
} from './config.ts';
import {
  clockElement,
  commonDevices,
  hypervFeatures,
  ideCdromDevices,
  sharedMemoryBacking,
  virtiofsShare,
} from './domain-xml-builders.ts';
import type { OsFamily, } from './registry.ts';

//region CDROM type

/**
 Path to a CDROM ISO to attach as an IDE device.
 
 @example
 ```ts
 const cdroms: ReadonlyArray<CdromSpec> = [
 
   { path: '/path/to/windows.iso' },
   { path: '/path/to/virtio-win.iso' },
 ];
 ```
 */
export type CdromSpec = {
  /**
   Absolute path to the ISO file.
   */
  readonly path: string;
};

//endregion CDROM type

//region Domain XML generator

/**
 Generates a libvirt domain XML definition for a KVM virtual machine.
 Supports both Linux and Windows guests with OS-specific optimizations,
 assembled from {@link commonDevices} plus the builders below.
 
 For Linux guests: configures virtio disk and NIC, serial console,
 and an optional cloud-init seed CDROM.
 
 For Windows guests: adds Hyper-V enlightenments via {@link hypervFeatures},
 a localtime clock via {@link clockElement}, and optional IDE CDROMs via
 {@link ideCdromDevices} for installation media (Windows ISO, autounattend,
 virtio-win ISO).
 
   OS family, boot device, and additional CDROMs
 
 @param bootDev - Boot device: `hd` or `cdrom`
 
 @param cdroms - Additional IDE CDROMs
 
 @param diskBus - Bus type for the primary disk (`virtio` or `sata`)
 
 @param diskPath - Absolute path to the VM disk image
 
 @param name - VM name without the mvm- prefix
 
 @param osFamily - Guest OS family
 
 @param seedIsoPath - Absolute path to the cloud-init seed ISO
 
 @returns Complete libvirt domain XML string
 
 @example
 ```ts
 // Linux VM with cloud-init seed
 domainXml({ name: 'dev', diskPath: '/path/disk.qcow2', seedIsoPath: '/path/seed.iso' });
 
 // Windows VM (normal boot)
 domainXml({ name: 'win', diskPath: '/path/disk.qcow2', osFamily: 'windows' });
 
 // Windows template creation (boot from ISO)
 domainXml({
 
   name: 'template',
   diskPath: '/path/disk.qcow2',
 
   osFamily: 'windows',
   bootDev: 'cdrom',
 
   cdroms: [{ path: '/path/win.iso' }, { path: '/path/autounattend.iso' }, { path: '/path/virtio.iso' }],
 });
 ```
 */
export function domainXml(
  {
    bootDev = 'hd',
    cdroms = [],
    diskBus = 'virtio',
    diskPath,
    name,
    osFamily = 'linux',
    seedIsoPath,
    sharedDir,
  }: {
    /**
     Boot device: `hd` for normal operation, `cdrom` for ISO-based installation.
     */
    readonly bootDev?: 'cdrom' | 'hd';
    /**
     Additional IDE CDROMs (Windows ISO, autounattend, virtio-win).
     */
    readonly cdroms?: readonly CdromSpec[];
    /**
     Bus type for the primary disk.
     Use `virtio` for production VMs (best performance).
     Use `sata` during Windows template creation to avoid the Server 2025
     SAN policy (policy 4: offline shared bus) which makes VirtIO disks
     appear offline in WinPE, blocking unattended installation.
     */
    readonly diskBus?: 'sata' | 'virtio';
    /**
     Absolute path to the VM disk image.
     */
    readonly diskPath: string;
    /**
     VM name without the mvm- prefix.
     */
    readonly name: string;
    /**
     Guest OS family for platform-specific optimizations.
     */
    readonly osFamily?: OsFamily;
    /**
     Absolute path to the cloud-init seed ISO (Linux only, omitted for Windows).
     */
    readonly seedIsoPath?: string;
    /**
     Absolute path to a host directory shared via virtiofs.
     Omitted on a host where libvirt finds no `virtiofsd`: the domain then has
     neither the share nor the shared memory backing the share needs.
     */
    readonly sharedDir?: string;
  },
): string {
  /**
   Mutable buffer because Windows pushes Hyper-V enlightenments on top of ACPI.
   */
  const features = [h({ tag: 'acpi', },),];
  if (osFamily === 'windows')
    features.push(hypervFeatures(),);

  /**
   Device name prefix depends on bus type: vda for virtio, sda for sata.
   */
  const diskDev = diskBus === 'virtio' ? 'vda' : 'sda';

  /**
   Mutable buffer because optional seed, virtiofs, and CDROM blocks extend the base disk.
   */
  const devices: string[] = [
    h({
      tag: 'disk',
      attrs: {
        type: 'file',
        device: 'disk',
      },
      children: [
        h({
          tag: 'driver',
          attrs: {
            name: 'qemu',
            type: 'qcow2',
          },
        },),
        h({
          tag: 'source',
          attrs: { file: diskPath, },
        },),
        h({
          tag: 'target',
          attrs: {
            dev: diskDev,
            bus: diskBus,
          },
        },),
      ],
    },),
  ];

  // Cloud-init NoCloud seed ISO for Linux VMs
  if (seedIsoPath !== undefined) {
    devices.push(
      h({
        tag: 'disk',
        attrs: {
          type: 'file',
          device: 'cdrom',
        },
        children: [
          h({
            tag: 'driver',
            attrs: {
              name: 'qemu',
              type: 'raw',
            },
          },),
          h({
            tag: 'source',
            attrs: { file: seedIsoPath, },
          },),
          h({
            tag: 'target',
            attrs: {
              dev: 'sdb',
              bus: 'sata',
            },
          },),
          h({ tag: 'readonly', },),
        ],
      },),
    );
  }

  // virtiofs shared directory for host-guest file transfer
  if (sharedDir !== undefined) {
    devices.push(virtiofsShare(sharedDir,),);
  }

  // IDE CDROMs for Windows template creation (Windows ISO, autounattend, virtio-win)
  devices.push(
    ...ideCdromDevices(cdroms,),
    ...commonDevices(osFamily,)
  );

  /**
   Top-level domain children before devices.
   */
  const domainChildren: string[] = [
    h({
      tag: 'name',
      text: `${VM_PREFIX}${name}`,
    },),
    h({
      tag: 'memory',
      attrs: { unit: 'MiB', },
      text: String(DEFAULT_MEMORY_MIB,),
    },),
    h({
      tag: 'vcpu',
      text: String(DEFAULT_VCPUS,),
    },),
  ];

  // virtiofs requires shared memory backed by memfd
  if (sharedDir !== undefined) {
    domainChildren.push(sharedMemoryBacking(),);
  }

  return h({
    tag: 'domain',
    attrs: { type: 'kvm', },
    children: [
      ...domainChildren,
      h({
        tag: 'os',
        children: [
          h({
            tag: 'type',
            attrs: { arch: 'x86_64', },
            text: 'hvm',
          },),
          h({
            tag: 'boot',
            attrs: { dev: bootDev, },
          },),
        ],
      },),
      // Bun requires AVX
      h({
        tag: 'cpu',
        attrs: { mode: 'host-passthrough', },
      },),
      // ACPI enables graceful shutdown during template baking (template.ts)
      h({
        tag: 'features',
        children: features,
      },),
      clockElement(osFamily,),
      h({
        tag: 'devices',
        children: devices,
      },),
    ],
  },);
}

//endregion Domain XML generator

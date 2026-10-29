/**
 Internal helper functions for building libvirt domain XML fragments.
 Provides Hyper-V enlightenments, clock configuration, IDE CDROM
 device generation, and common device elements used by the main
 {@link domainXml} generator.
 */

import { hXml as h, } from '@monochromatic-dev/module-hyperscript/ts';

import type { OsFamily, } from './registry.ts';

//region Hyper-V enlightenments

/**
 Generates libvirt XML elements for Hyper-V enlightenments.
 These are performance optimizations recognized by the Windows kernel
 that reduce virtualization overhead for timer handling, interrupt
 processing, and spinlock contention.
 
 @returns Array of XML element strings for the `<features>` block
 
 @example
 ```ts
 hypervFeatures(); // => h({ tag: 'hyperv', children: [...] })
 ```
 */
export function hypervFeatures(): string {
  return h({
    tag: 'hyperv',
    attrs: { mode: 'custom', },
    children: [
      h({
        tag: 'relaxed',
        attrs: { state: 'on', },
      },),
      h({
        tag: 'vapic',
        attrs: { state: 'on', },
      },),
      h({
        tag: 'spinlocks',
        attrs: {
          state: 'on',
          retries: '8191',
        },
      },),
    ],
  },);
}

//endregion Hyper-V enlightenments

//region Clock configuration

/**
 Generates the clock XML element appropriate for the guest OS.
 Linux uses UTC offset; Windows expects localtime with a Hyper-V
 reference clock for accurate timekeeping.
 
 @param osFamily - Guest OS family
 
 @returns XML string for the `<clock>` element
 
 @example
 ```ts
 clockElement('windows'); // => '<clock offset="localtime">...'
 clockElement('linux');   // => '<clock offset="utc"/>'
 ```
 */
export function clockElement(osFamily: OsFamily,): string {
  if (osFamily === 'windows') {
    return h({
      tag: 'clock',
      attrs: { offset: 'localtime', },
      children: [
        h({
          tag: 'timer',
          attrs: {
            name: 'hypervclock',
            present: 'yes',
          },
        },),
        h({
          tag: 'timer',
          attrs: {
            name: 'hpet',
            present: 'no',
          },
        },),
      ],
    },);
  }
  return h({
    tag: 'clock',
    attrs: { offset: 'utc', },
  },);
}

//endregion Clock configuration

//region CDROM devices

/**
 Generates IDE CDROM device elements for the given ISO paths.
 Assigns sequential IDE device names (hda, hdb, hdc, hdd).
 
 @param cdroms - Array of objects with ISO paths
 
 @returns Array of XML strings for disk elements
 
 @example
 ```ts
 ideCdromDevices([{ path: '/tmp/win.iso' }]); // => ['<disk type="file" device="cdrom">...']
 ```
 */
export function ideCdromDevices(
  cdroms: readonly { readonly path: string; }[],
): readonly string[] {
  /**
   IDE device name sequence: hda through hdd.
   */
  const ideDevNames = [
    'hda',
    'hdb',
    'hdc',
    'hdd',
  ];
  return cdroms.map(function buildCdromElement(
    cdrom,
    index,
  ) {
    /**
     Sequential IDE slot for this CDROM, undefined past the bus's four-slot limit.
     */
    const devName = ideDevNames[index];
    if (devName === undefined) {
      throw new Error(
        `Too many CDROMs: maximum ${String(ideDevNames.length,)} supported`,
      );
    }
    return h({
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
          attrs: { file: cdrom.path, },
        },),
        h({
          tag: 'target',
          attrs: {
            dev: devName,
            bus: 'ide',
          },
        },),
        h({ tag: 'readonly', },),
      ],
    },);
  },);
}

//endregion CDROM devices

//region Common devices

/**
 Generates common VM device elements shared across all domain configurations:
 SLIRP user-mode networking, QEMU guest agent channel, serial console,
 and Windows-specific VGA/tablet devices.
 
 @param osFamily - Guest OS family for platform-specific devices
 
 @returns Array of XML strings for common device elements
 
 @example
 ```ts
 commonDevices('linux');   // => [network, channel, serial, console]
 commonDevices('windows'); // => [network, channel, serial, console, video, tablet]
 ```
 */
export function commonDevices(osFamily: OsFamily,): readonly string[] {
  /**
   Mutable buffer because Windows appends VGA and tablet on top of the base list.
   */
  const devices: string[] = [
    // SLIRP user-mode networking for outbound internet without bridge setup
    h({
      tag: 'interface',
      attrs: { type: 'user', },
      children: [
        h({
          tag: 'model',
          attrs: { type: 'virtio', },
        },),
      ],
    },),
    // Guest agent channel for command execution via `virsh qemu-agent-command`
    h({
      tag: 'channel',
      attrs: { type: 'unix', },
      children: [
        h({
          tag: 'target',
          attrs: {
            type: 'virtio',
            name: 'org.qemu.guest_agent.0',
          },
        },),
      ],
    },),
    // Serial console for interactive shell via `virsh console` (primarily for Linux)
    h({
      tag: 'serial',
      attrs: { type: 'pty', },
      children: [
        h({
          tag: 'target',
          attrs: { port: '0', },
        },),
      ],
    },),
    h({
      tag: 'console',
      attrs: { type: 'pty', },
      children: [
        h({
          tag: 'target',
          attrs: {
            type: 'serial',
            port: '0',
          },
        },),
      ],
    },),
  ];

  // Windows requires a VGA device for the WinPE installer and OOBE
  // Also useful for debugging via `virsh screenshot`
  if (osFamily === 'windows') {
    devices.push(
      h({
        tag: 'video',
        children: [
          h({
            tag: 'model',
            attrs: {
              type: 'vga',
              vram: '16384',
            },
          },),
        ],
      },),
      // Tablet input device to prevent mouse pointer offset
      h({
        tag: 'input',
        attrs: {
          type: 'tablet',
          bus: 'usb',
        },
      },),
    );
  }

  return devices;
}

//endregion Common devices

//region virtiofs share

/**
 Generates the virtiofs device that exposes a host directory to the guest
 under the mount tag `mvm-shared`. libvirt starts a `virtiofsd` for it, so a
 domain with this device only starts on a host where libvirt finds one.

 @param sharedDir - Absolute path to the host directory to share

 @returns XML string for the `<filesystem>` element

 @example
 ```ts
 virtiofsShare('/vms/dev/shared'); // => '<filesystem type="mount" accessmode="passthrough">...'
 ```
 */
export function virtiofsShare(sharedDir: string,): string {
  return h({
    tag: 'filesystem',
    attrs: {
      type: 'mount',
      accessmode: 'passthrough',
    },
    children: [
      h({
        tag: 'driver',
        attrs: { type: 'virtiofs', },
      },),
      h({
        tag: 'source',
        attrs: { dir: sharedDir, },
      },),
      h({
        tag: 'target',
        attrs: { dir: 'mvm-shared', },
      },),
    ],
  },);
}

/**
 Generates the shared memory backing a virtiofs device requires:
 guest memory in a memfd that `virtiofsd` can map.

 @returns XML string for the `<memoryBacking>` element

 @example
 ```ts
 sharedMemoryBacking(); // => '<memoryBacking><source type="memfd" /><access mode="shared" /></memoryBacking>'
 ```
 */
export function sharedMemoryBacking(): string {
  return h({
    tag: 'memoryBacking',
    children: [
      h({
        tag: 'source',
        attrs: { type: 'memfd', },
      },),
      h({
        tag: 'access',
        attrs: { mode: 'shared', },
      },),
    ],
  },);
}

//endregion virtiofs share

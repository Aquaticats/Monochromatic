import { join, } from 'node:path';

import { IMAGES_DIR, } from './config.ts';
import { pathExists, } from './path-exists.ts';

//region OS family and guest config types

/**
 Top-level OS family discriminant for guest configuration dispatch.
 Determines which provisioning, exec, and domain XML paths are used.
 
 @example
 ```ts
 const family: OsFamily = 'windows';
 ```
 */
export type OsFamily = 'linux' | 'windows';

/**
 Init system used by a Linux guest OS, determines how services are managed
 in cloud-init runcmd directives.
 
 @example
 ```ts
 const system: InitSystem = 'systemd';
 ```
 */
export type InitSystem = 'openrc' | 'systemd';

/**
 Linux guest OS configuration for cloud-init provisioning.
 Discriminated from {@link WindowsGuestConfig} by `osFamily: 'linux'`.
 
 @example
 ```ts
 const guest: LinuxGuestConfig = {
   osFamily: 'linux',
   defaultUser: 'ubuntu',
   initSystem: 'systemd',
   shell: '/bin/bash',
 };
 ```
 */
export type LinuxGuestConfig = {
  /**
   Discriminant identifying this as a Linux guest.
   */
  readonly osFamily: 'linux';
  /**
   Default login user created by cloud-init for this distro.
   */
  readonly defaultUser: string;
  /**
   Init system for service management (systemd or openrc).
   */
  readonly initSystem: InitSystem;
  /**
   Login shell path for the default user.
   */
  readonly shell: string;
};

/**
 Windows guest OS configuration for unattended provisioning.
 Discriminated from {@link LinuxGuestConfig} by `osFamily: 'windows'`.
 Windows guests use the QEMU guest agent for post-boot hostname configuration
 instead of cloud-init.
 
 @example
 ```ts
 const guest: WindowsGuestConfig = {
   osFamily: 'windows',
   defaultUser: 'Administrator',
   shell: 'powershell.exe',
 };
 ```
 */
export type WindowsGuestConfig = {
  /**
   Discriminant identifying this as a Windows guest.
   */
  readonly osFamily: 'windows';
  /**
   Default admin user created during unattended install.
   */
  readonly defaultUser: string;
  /**
   Shell executable for guest-exec commands (powershell.exe or cmd.exe).
   */
  readonly shell: string;
};

/**
 Discriminated union of guest OS configurations.
 Check `osFamily` to narrow to the correct variant.
 
 @example
 ```ts
 function getShell(guest: GuestConfig): string {
   if (guest.osFamily === 'linux') {
     return guest.shell; // LinuxGuestConfig
   }
   return guest.shell; // WindowsGuestConfig
 }
 ```
 */
export type GuestConfig = LinuxGuestConfig | WindowsGuestConfig;

//endregion OS family and guest config types

//region Image spec types

/**
 Linux cloud image specification for download, caching, and template baking.
 The `fileName` refers to a qcow2 cloud image downloaded from `url`.
 
 @example
 ```ts
 const spec: LinuxImageSpec = IMAGES['ubuntu'] as LinuxImageSpec;
 spec.initSystem; // => 'systemd'
 ```
 */
export type LinuxImageSpec = LinuxGuestConfig & {
  /**
   Cached qcow2 cloud image filename under `~/.local/share/mvm/images/`.
   */
  readonly fileName: string;
  /**
   Template filename derived from this image (e.g. `template-ubuntu.qcow2`).
   */
  readonly templateFileName: string;
  /**
   Remote URL to download the cloud image from.
   */
  readonly url: string;
};

/**
 Windows image specification for ISO-based unattended template baking.
 The `fileName` refers to a Windows evaluation ISO downloaded from `url`.
 Template creation installs Windows from the ISO using an autounattend answer file.
 
 @example
 ```ts
 const spec: WindowsImageSpec = IMAGES['windows'] as WindowsImageSpec;
 spec.imageIndex; // => 1 (Server Core)
 ```
 */
export type WindowsImageSpec = WindowsGuestConfig & {
  /**
   Cached ISO filename under `~/.local/share/mvm/images/`.
   */
  readonly fileName: string;
  /**
   Template filename derived from this image (e.g. `template-windows.qcow2`).
   */
  readonly templateFileName: string;
  /**
   Remote URL to download the evaluation ISO from.
   */
  readonly url: string;
  /**
   WIM image index for unattended install (1-based, selects the OS edition).
   */
  readonly imageIndex: number;
};

/**
 Discriminated union of image specifications.
 Linux images are qcow2 cloud images; Windows images are evaluation ISOs.
 Check `osFamily` to narrow to the correct variant.
 
 @example
 ```ts
 const spec = IMAGES['ubuntu'];
 if (spec.osFamily === 'linux') {
   spec.initSystem; // narrowed to LinuxImageSpec
 }
 ```
 */
export type ImageSpec = LinuxImageSpec | WindowsImageSpec;

//endregion Image spec types

//region Virtio-win shared resource

/**
 Stable download URL for the latest virtio-win ISO from the Fedora project.
 Contains VirtIO storage/network drivers and the QEMU guest agent installer
 for Windows guests. Redirects to the latest versioned filename on download.
 */
export const VIRTIO_WIN_URL =
  'https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso';

/**
 Cached filename for the virtio-win ISO under `~/.local/share/mvm/images/`.
 */
export const VIRTIO_WIN_FILENAME = 'virtio-win.iso';

//endregion Virtio-win shared resource

//region Built-in image registry

/**
 Built-in registry of supported cloud images.
 Each entry maps a shorthand name to a full image specification.
 Linux entries are qcow2 cloud images; the Windows entry is an evaluation ISO.
 
 @example
 ```ts
 const fedora = IMAGES['fedora'];
 // => { url: 'https://download.fedoraproject.org/...', defaultUser: 'fedora', ... }
 ```
 */
export const IMAGES: Readonly<Record<string, ImageSpec>> = {
  alpine: {
    osFamily: 'linux',
    defaultUser: 'alpine',
    fileName: 'nocloud_alpine-3.23.3-x86_64-bios-cloudinit-r0.qcow2',
    initSystem: 'openrc',
    shell: '/bin/ash',
    templateFileName: 'template-alpine.qcow2',
    url:
      'https://dl-cdn.alpinelinux.org/alpine/v3.23/releases/cloud/nocloud_alpine-3.23.3-x86_64-bios-cloudinit-r0.qcow2',
  },
  fedora: {
    osFamily: 'linux',
    defaultUser: 'fedora',
    fileName: 'Fedora-Cloud-Base-Generic-43-1.6.x86_64.qcow2',
    initSystem: 'systemd',
    shell: '/bin/bash',
    templateFileName: 'template-fedora.qcow2',
    url:
      'https://download.fedoraproject.org/pub/fedora/linux/releases/43/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-43-1.6.x86_64.qcow2',
  },
  ubuntu: {
    osFamily: 'linux',
    defaultUser: 'ubuntu',
    fileName: 'noble-server-cloudimg-amd64.img',
    initSystem: 'systemd',
    shell: '/bin/bash',
    templateFileName: 'template-ubuntu.qcow2',
    url: 'https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img',
  },
  windows: {
    osFamily: 'windows',
    defaultUser: 'Administrator',
    fileName: 'windows-server-2025-eval.iso',
    imageIndex: 1,
    shell: 'powershell.exe',
    templateFileName: 'template-windows.qcow2',
    url:
      'https://go.microsoft.com/fwlink/?linkid=2293312&clcid=0x409&culture=en-us&country=us',
  },
};

/**
 Default image shorthand when `--image` is not specified.
 */
export const DEFAULT_IMAGE = 'ubuntu';

/**
 Fallback guest configuration for custom template images.
 Uses root with systemd since the actual OS is unknown.
 
 @example
 ```ts
 createSeedIso({ name: 'vm', guest: CUSTOM_GUEST_DEFAULTS, vmDir: '...' });
 ```
 */
export const CUSTOM_GUEST_DEFAULTS: LinuxGuestConfig = {
  osFamily: 'linux',
  defaultUser: 'root',
  initSystem: 'systemd',
  shell: '/bin/sh',
};

//endregion Built-in image registry

//region Resolution

/**
 Resolves an image identifier to a full image spec.
 Checks the built-in registry first. If not found, looks for a custom template
 file named `<identifier>.qcow2` in the images directory.
 
 @param identifier - Registry shorthand (e.g. `ubuntu`, `windows`) or custom template name
 
 @returns Image spec for registry images, or `{ customTemplatePath }` for user-provided templates
 
 @throws Error when identifier matches neither the registry nor a custom template file
 
 @example
 ```ts
 const result = await resolveImage('ubuntu');
 // => { kind: 'registry', spec: { url: '...', ... } }
 
 const win = await resolveImage('windows');
 // => { kind: 'registry', spec: { osFamily: 'windows', imageIndex: 1, ... } }
 
 const custom = await resolveImage('my-custom');
 // => { kind: 'custom', customTemplatePath: '/home/user/.local/share/mvm/images/my-custom.qcow2' }
 ```
 */
export async function resolveImage(identifier: string,): Promise<ResolvedImage> {
  /**
   Registry lookup; primary resolution path before the custom-template fallback.
   */
  const spec = IMAGES[identifier];
  if (spec !== undefined) {
    return {
      kind: 'registry',
      spec,
    };
  }

  /**
   Candidate path for a user-supplied template under the images directory.
   */
  const customPath = join(
    IMAGES_DIR,
    `${identifier}.qcow2`,
  );
  if (await pathExists(customPath,)) {
    return {
      customTemplatePath: customPath,
      kind: 'custom',
    };
  }

  /**
   Listed in the error message so an unknown identifier shows the valid choices.
   */
  const available = Object.keys(IMAGES,)
    .join(', ',);
  throw new Error(
    `unknown image "${identifier}". Built-in images: ${available}. `
      + `For custom images, place a qcow2 template at ${customPath}`,
  );
}

/**
 Discriminated union returned by {@link resolveImage}.
 Registry images go through the download-and-template-bake pipeline.
 Custom images are used directly as backing templates.
 
 @example
 ```ts
 const resolved = await resolveImage('ubuntu');
 if (resolved.kind === 'registry') {
   resolved.spec.url; // download URL
 }
 ```
 */
export type ResolvedImage =
  | {
    customTemplatePath: string;
    kind: 'custom';
  }
    | {
      kind: 'registry';
      spec: ImageSpec;
    };

//endregion Resolution

import { homedir, } from 'node:os';
import { join, } from 'node:path';

/**
 Prefix applied to all VM names in libvirt to avoid collisions.
 */
export const VM_PREFIX = 'mvm-';

/**
 Root data directory for VM images and disks.
 */
export const DATA_DIR: string = join(
  homedir(),
  '.local',
  'share',
  'mvm',
);

/**
 Directory for cached base cloud images.
 */
export const IMAGES_DIR: string = join(
  DATA_DIR,
  'images',
);

/**
 Directory containing per-VM subdirectories with disks and metadata.
 */
export const VMS_DIR: string = join(
  DATA_DIR,
  'vms',
);

/**
 Default VM memory allocation in MiB. Making it large for builds.
 */
export const DEFAULT_MEMORY_MIB = 8_192;

/**
 Default number of virtual CPUs.
 */
export const DEFAULT_VCPUS = 4;

/**
 Default root disk size for new Linux VMs.
 */
export const DEFAULT_DISK_SIZE = '20G';

/**
 Default root disk size for Windows VMs (Server 2025 requires 32 GB minimum).
 */
export const WINDOWS_DISK_SIZE = '40G';

/**
 Name of the shared directory inside each VM directory, exposed via virtiofs.
 */
export const SHARED_DIR_NAME = 'shared';

/**
 Mount point for the virtiofs share inside Linux guests.
 */
export const GUEST_MOUNT_POINT = '/mnt/shared';

/**
 Mount point for the virtiofs share inside Windows guests (WinFSP + virtiofs driver).
 */
export const WINDOWS_GUEST_MOUNT_POINT = 'Z:\\';

/**
 libvirt connection URI targeting the user session QEMU/KVM daemon.
 */
export const LIBVIRT_URI = 'qemu:///session';

/**
 Timeout for guest agent during Windows template creation (40 minutes).
 Windows unattended install takes 15-30 minutes: OS installation,
 first boot, OOBE, and guest agent installation via FirstLogonCommands.
 */
export const WINDOWS_TEMPLATE_AGENT_TIMEOUT_MS = 2_400_000;

/**
 Timeout for guest agent during VirtIO disk bus verification (5 minutes).
 After switching from SATA to VirtIO, Windows needs to detect the new
 disk controller and load the viostor driver on boot.
 */
export const VIRTIO_VERIFY_AGENT_TIMEOUT_MS = 300_000;

/**
 Checks whether `c` is an ASCII alphanumeric character.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is `[A-Za-z0-9]`
 
 @example
 ```ts
 isAlphaNum('a'); // true
 isAlphaNum('-'); // false
 ```
 */
function isAlphaNum(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'));
}

/**
 Checks whether `c` is allowed in non-leading positions of a VM name.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is alphanumeric, underscore, or hyphen
 
 @example
 ```ts
 isNameBodyChar('_'); // true
 isNameBodyChar('.'); // false
 ```
 */
function isNameBodyChar(c: string,): boolean {
  return isAlphaNum(c,)
    || (c === '_')
    || (c === '-');
}

/**
 Checks whether `name` matches the original regex
 `/^[A-Za-z0-9][A-Za-z0-9_-]*$/`.
 
 Empty input fails the leading-alphanumeric requirement; all subsequent
 characters must be alphanumeric, underscore, or hyphen.
 
 @param name - candidate VM name
 
 @returns whether name is a valid VM identifier
 
 @example
 ```ts
 isValidVmName('my-vm-01'); // true
 isValidVmName('-leading'); // false
 isValidVmName('');         // false
 ```
 */
function isValidVmName(name: string,): boolean {
  if (name.length
    === 0)
    return false;
  if (!isAlphaNum(name.charAt(0,),))
    return false;
  for (const c of name.slice(1,)) {
    if (!isNameBodyChar(c,))
      return false;
  }
  return true;
}

/**
 Validates a VM name contains only safe characters.
 
 @param name - VM name to validate
 
 @throws Error when name contains invalid characters
 
 @example
 ```ts
 validateName('my-vm-01'); // OK
 validateName('../evil');  // throws
 ```
 */
export function validateName(name: string,): void {
  if (!isValidVmName(name,)) {
    throw new Error(
      `invalid VM name: ${name} (must start with alphanumeric, contain only alphanumerics, hyphens, underscores)`,
    );
  }
}

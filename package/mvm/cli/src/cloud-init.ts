import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  templateRuncmd,
  vmAutologin,
} from './cloud-init-init-systems.ts';
import { createIso, } from './iso9660.ts';
import type {
  GuestConfig,
  LinuxGuestConfig,
} from './registry.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Sentinel returned by {@link createSeedIso} for Windows guests, which
 provision via the guest agent instead of a cloud-init seed ISO.
 A unique symbol models "no seed ISO" without a nullish union.
 
 @example
 ```ts
 const seed = await createSeedIso({ guest, name, vmDir });
 if (seed !== NO_SEED_ISO) attachSeedIso(seed);
 ```
 */
export const NO_SEED_ISO: unique symbol = Symbol(
  'returned when a guest needs no cloud-init seed ISO',
);

/**
 Narrows a {@link GuestConfig} to {@link LinuxGuestConfig} after the caller
 has already ruled out Windows guests via an early return.
 
 @param guest - Guest config known to be Linux at this call site
 
 @returns The same config narrowed to LinuxGuestConfig
 
 @example
 ```ts
 if (guest.osFamily === 'windows') return;
 const linux = asLinux(guest);
 linux.initSystem; // safe
 ```
 */
function asLinux(guest: GuestConfig,): LinuxGuestConfig {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- caller has already ruled out Windows via early return
  return guest as LinuxGuestConfig;
}

//region User-data generators

/**
 Generates cloud-init user-data for VM instances.
 Sets up the default user with passwordless sudo and serial console autologin.
 
 @param guest - Guest config for distro-specific cloud-init settings
 
 @param name - VM hostname
 
 @param sharedMount - Whether the VM has the virtiofs share, which the guest then mounts at `/mnt/shared`
 
 @returns Cloud-init user-data string
 
 @example
 ```ts
 vmUserData({ name: 'my-vm', guest: IMAGES['ubuntu'], sharedMount: true });
 ```
 */
function vmUserData({
  guest,
  name,
  sharedMount,
}: {
  readonly guest: GuestConfig;
  readonly name: string;
  readonly sharedMount: boolean;
},): string {
  /**
   Guest config narrowed to Linux; lets us read `initSystem`, `shell`, and `defaultUser`.
   */
  const linux = asLinux(guest,);
  return `#cloud-config
hostname: ${name}
users:
  - name: ${linux.defaultUser}
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: ${linux.shell}
${
    sharedMount
      ? `mounts:
  - [mvm-shared, /mnt/shared, virtiofs, "defaults,nofail", "0", "0"]
runcmd:
  - ["mkdir", "-p", "/mnt/shared"]
  - ["mount", "-a"]
`
      : ''
  }${
    vmAutologin({
      initSystem: linux.initSystem,
      user: linux.defaultUser,
    },)
  }`;
}

/**
 Generates cloud-init user-data for template creation.
 Installs qemu-guest-agent so the template image has it pre-baked,
 avoiding package downloads on every VM boot.
 
 @param guest - Guest config for distro-specific cloud-init settings
 
 @param name - Template VM hostname
 
 @returns Cloud-init user-data string with qemu-guest-agent installation
 
 @example
 ```ts
 templateUserData({ name: 'template-setup', guest: IMAGES['ubuntu'] });
 ```
 */
function templateUserData(
  {
    guest,
    name,
  }: {
    readonly guest: GuestConfig;
    readonly name: string;
  },
): string {
  /**
   Guest config narrowed to Linux; lets us read `initSystem`, `shell`, and `defaultUser`.
   */
  const linux = asLinux(guest,);
  return `#cloud-config
hostname: ${name}
users:
  - name: ${linux.defaultUser}
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: ${linux.shell}
packages:
  - qemu-guest-agent
${templateRuncmd(linux.initSystem,)}`;
}

//endregion User-data generators

//region Seed ISO generation

/**
 Generates a cloud-init NoCloud seed ISO with user-data and meta-data.
 Uses the built-in ISO9660 generator instead of external tools like genisoimage.
 
 Configures auto-login on the serial console (ttyS0) so `virsh console`
 drops directly into a shell without SSH or passwords.
 
 Windows guests return `undefined` because they do not use cloud-init.
 Hostname is set via the QEMU guest agent after boot instead.
 
   guest config for distro-specific cloud-init, and whether this is for template creation
 
 @param guest - Guest config for distro-specific cloud-init settings
 
 @param name - VM name used as hostname
 
 @param sharedMount - Whether the VM has the virtiofs share to mount; defaults to true, false for a VM defined without the share
 
 @param template - Whether this is a template creation (installs qemu-guest-agent)
 
 @param vmDir - Directory to write the seed ISO into
 
 @returns Absolute path to the generated seed ISO, or `undefined` for Windows guests
 
 @example
 ```ts
 const seedPath = await createSeedIso({ name: 'my-vm', guest: IMAGES['ubuntu'], vmDir: '/path/to/vm' });
 // seedPath => '/path/to/vm/seed.iso'
 
 const winSeed = await createSeedIso({ name: 'win-vm', guest: IMAGES['windows'], vmDir: '/path/to/vm' });
 // winSeed => NO_SEED_ISO
 ```
 */
export async function createSeedIso({
  guest,
  name,
  sharedMount = true,
  template = false,
  vmDir,
}: {
  readonly guest: GuestConfig;
  readonly name: string;
  readonly sharedMount?: boolean;
  readonly template?: boolean;
  readonly vmDir: string;
},): Promise<string | typeof NO_SEED_ISO> {
  if (guest.osFamily
    === 'windows') {
    /**
     Logger for the Windows skip-path; namespaced so the info line is attributable.
     */
    const rl = tagged({
      tag: createSeedIso.name,
      l,
    },);
    rl.info('skipping seed ISO for Windows guest (uses guest agent for provisioning)',);
    return NO_SEED_ISO;
  }

  /**
   Logger scoped to this function so the "created seed ISO" message is attributable.
   */
  const rl = tagged({
    tag: createSeedIso.name,
    l,
  },);

  /**
   Shared text encoder used for both user-data and meta-data byte payloads.
   */
  const encoder = new TextEncoder();
  /**
   UTF-8 user-data payload picked from template or VM variant depending on `template`.
   */
  const userData = encoder.encode(
    template
      ? templateUserData({
        guest,
        name,
      },)
      : vmUserData({
        guest,
        name,
        sharedMount,
      },),
  );

  /**
   UTF-8 meta-data payload; carries `instance-id` so cloud-init reruns when the id changes.
   */
  const metaData = encoder.encode(
    `instance-id: ${name}
local-hostname: ${name}
`,
  );

  /**
   Generated ISO9660 image carrying `user-data` and `meta-data` under the `cidata` volume.
   */
  const iso = createIso({
    files: [
      {
        data: userData,
        name: 'user-data',
      },
      {
        data: metaData,
        name: 'meta-data',
      },
    ],
    volumeId: 'cidata',
  },);

  /**
   Output path of the seed ISO; attached as a CDROM by the domain XML.
   */
  const seedPath = join(
    vmDir,
    'seed.iso',
  );
  await writeFile(
    seedPath,
    iso,
  );
  rl.info(`created seed ISO at ${seedPath}`,);
  return seedPath;
}

//endregion Seed ISO generation

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  CUSTOM_GUEST_DEFAULTS,
  DEFAULT_IMAGE,
  type GuestConfig,
  type OsFamily,
  resolveImage,
} from './registry.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region VM metadata type

/**
 Persistent metadata stored alongside each VM in `meta.json`.
 Captures guest OS properties needed by exec and clone without
 re-resolving through the image registry.
 
 @example
 ```ts
 const meta: VmMeta = await readVmMeta('/home/user/.local/share/mvm/vms/dev-01');
 meta.osFamily; // => 'linux'
 ```
 */
export type VmMeta = {
  /**
   Registry image identifier (e.g. `ubuntu`, `windows`, or custom name).
   */
  image: string;
  /**
   OS family discriminant for exec shell and domain XML dispatch.
   */
  osFamily: OsFamily;
  /**
   Shell executable path or name for guest-exec commands.
   */
  shell: string;
  /**
   Default login user for this guest OS.
   */
  defaultUser: string;
  /**
   ISO 8601 timestamp of VM creation.
   */
  createdAt: string;
  /**
   How files move between host and guest: the virtiofs shared directory, or the
   guest agent for a VM defined without the share. Absent in metadata written
   before the guest agent route existed, which means the shared directory.
   */
  fileTransfer?: FileTransferRoute;
};

/**
 Channel a VM's files travel through.

 @example
 ```ts
 const route: FileTransferRoute = 'guest-agent';
 ```
 */
export type FileTransferRoute = 'guest-agent' | 'virtiofs';

//endregion VM metadata type

//region Write

/**
 Writes VM metadata to `meta.json` in the VM directory.
 Also writes the legacy `image` text file for backwards compatibility.
 
 @param fileTransfer - How files move between host and guest; defaults to the virtiofs shared directory
 
 @param guest - Guest config for OS family, shell, and default user
 
 @param image - Registry image identifier
 
 @param vmDir - VM directory to write metadata into
 
 @example
 ```ts
 await writeVmMeta({
 
   vmDir: '/home/user/.local/share/mvm/vms/dev-01',
 
   image: 'ubuntu',
 
   guest: IMAGES['ubuntu'],
 });
 ```
 */
export async function writeVmMeta({
  fileTransfer = 'virtiofs',
  guest,
  image,
  vmDir,
}: {
  readonly fileTransfer?: FileTransferRoute;
  readonly guest: GuestConfig;
  readonly image: string;
  readonly vmDir: string;
},): Promise<void> {
  /**
   Logger scoped to this writer so legacy-file fallbacks log with context.
   */
  const rl = tagged({
    tag: writeVmMeta.name,
    l,
  },);
  /**
   Metadata record persisted to `meta.json`; captures the guest config snapshot at creation time.
   */
  const meta: VmMeta = {
    createdAt: new Date().toISOString(),
    defaultUser: guest.defaultUser,
    fileTransfer,
    image,
    osFamily: guest.osFamily,
    shell: guest.shell,
  };

  /**
   Path of the metadata file inside the VM directory; chosen once and reused in the debug log.
   */
  const metaPath = join(
    vmDir,
    'meta.json',
  );
  await writeFile(
    metaPath,
    JSON.stringify(
      meta,
      null,
      2,
    ),
  );
  rl.debug(`wrote VM metadata to ${metaPath}`,);

  // Legacy compatibility: also write the image text file
  await writeFile(
    join(
      vmDir,
      'image',
    ),
    image,
  );
}

//endregion Write

//region Read

/**
 Reads VM metadata from `meta.json` in the VM directory.
 Falls back to the legacy `image` text file and derives metadata
 from the registry for VMs created before meta.json was introduced.
 
 @param vmDir - Absolute path to the VM directory
 
 @returns Resolved VM metadata
 
 @example
 ```ts
 const meta = await readVmMeta('/home/user/.local/share/mvm/vms/dev-01');
 if (meta.osFamily === 'windows') {
 
   // use PowerShell for exec
 }
 ```
 */
export async function readVmMeta(vmDir: string,): Promise<VmMeta> {
  /**
   Logger scoped to this reader so legacy-fallback messages are namespaced.
   */
  const rl = tagged({
    tag: readVmMeta.name,
    l,
  },);

  // Try meta.json first
  try {
    /**
     Raw `meta.json` contents read from disk; parsed as {@link VmMeta} below.
     */
    const content = await readFile(
      join(
        vmDir,
        'meta.json',
      ),
      'utf8',
    );
    /**
     Parsed metadata record; trusted because it was written by {@link writeVmMeta}.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted local meta.json written by writeVmMeta
    const meta = JSON.parse(content,) as VmMeta;
    rl.debug(`read VM metadata from ${vmDir}/meta.json`,);
    return meta;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug('meta.json not found, falling back to legacy image file',);
  }

  // Fall back to legacy image text file
  /**
   Image identifier resolved from the legacy `image` text file, or {@link DEFAULT_IMAGE} when missing.
   */
  const image = await (async function readLegacyImage(): Promise<string> {
    try {
      /**
       Raw legacy file contents; trimmed because old writers added a trailing newline.
       */
      const content = await readFile(
        join(
          vmDir,
          'image',
        ),
        'utf8',
      );
      return content.trim();
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      rl.debug('legacy image file not found, assuming ubuntu',);
      return DEFAULT_IMAGE;
    }
  })();

  /**
   Image record resolved from the legacy identifier; registry or custom.
   */
  const resolved = await resolveImage(image,);
  /**
   Guest config used to fill the synthetic {@link VmMeta}; defaults for custom images.
   */
  const guest = resolved.kind
    === 'registry'
    ? resolved.spec
    : CUSTOM_GUEST_DEFAULTS;

  return {
    createdAt: '',
    defaultUser: guest.defaultUser,
    image,
    osFamily: guest.osFamily,
    shell: guest.shell,
  };
}

//endregion Read

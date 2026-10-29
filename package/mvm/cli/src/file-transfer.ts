/**
 File transfer between host and guest VM.

 A VM created on a host where libvirt can run `virtiofsd` has a `shared/`
 directory on the host exposed inside the guest at `/mnt/shared` (Linux) or
 `Z:\` (Windows via WinFSP + virtiofs driver). Push and pull copy files
 to/from this shared directory on the host; the guest sees changes
 immediately through the virtiofs mount.

 A VM created on a host without `virtiofsd` has no shared directory. Push and
 pull then read and write the guest's own files through the QEMU guest agent,
 and the guest path must be absolute. Each VM's metadata records which of the
 two applies.

 @module
 */

import {
  copyFile,
  mkdir,
  readFile,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  GUEST_MOUNT_POINT,
  SHARED_DIR_NAME,
  validateName,
  VM_PREFIX,
  VMS_DIR,
  WINDOWS_GUEST_MOUNT_POINT,
} from './config.ts';
import {
  pullThroughAgent,
  pushThroughAgent,
} from './guest-file-transfer.ts';
import { readVmMeta, } from './meta.ts';
import type { OsFamily, } from './registry.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Guest paths without a shared directory

/**
 The guest path given for a VM without a shared directory is not absolute,
 so it does not name one file in the guest.

 @example
 ```ts
 try {
   await pushFile({ name: 'dev-01', hostPath: './setup.sh', guestPath: 'setup.sh' });
 }
 catch (error) {
   if (error instanceof GuestPathError) console.error(error.guestPath);
 }
 ```
 */
export class GuestPathError extends Error {
  /**
   Guest path that was given.
   */
  readonly guestPath: string;

  /**
   @param guestPath - Guest path that was given, quoted in the message

   @param name - VM name without the mvm- prefix, named in the message

   @param osFamily - Guest OS family, which decides the example path
   */
  constructor({
    guestPath,
    name,
    osFamily,
  }: {
    readonly guestPath: string;
    readonly name: string;
    readonly osFamily: OsFamily;
  },) {
    super(
      [
        `VM ${name} has no shared directory, because libvirt found no virtiofsd when it was created; its files move through the guest agent, which reads and writes the guest's own files.`,
        `The guest path must therefore be the file's full path in the guest, and "${guestPath}" is not.`,
        osFamily === 'windows'
          ? `Give a path such as C:\\Windows\\Temp\\${basename(guestPath,)}; its directory must exist in the guest.`
          : `Give a path such as /tmp/${basename(guestPath,)}; its directory must exist in the guest.`,
      ].join('\n',),
    );
    this.name = 'GuestPathError';
    this.guestPath = guestPath;
  }
}

/**
 Checks whether `c` is an ASCII letter, as a Windows drive letter is.

 @param c - Single-character string

 @returns Whether `c` is in `[A-Za-z]`

 @example
 ```ts
 isAsciiLetter('C'); // true
 isAsciiLetter('1'); // false
 ```
 */
function isAsciiLetter(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z')) || ((c >= 'A') && (c <= 'Z'));
}

/**
 Checks whether a path names one file in the guest regardless of any working directory:
 `/...` on Linux; `X:\...`, `X:/...` or a `\\server\share` path on Windows.

 @param guestPath - Path given by the caller

 @param osFamily - Guest OS family

 @returns Whether the path is absolute for that guest

 @example
 ```ts
 isAbsoluteGuestPath({ guestPath: 'C:\\Temp\\a.txt', osFamily: 'windows' }); // true
 isAbsoluteGuestPath({ guestPath: 'a.txt', osFamily: 'linux' });             // false
 ```
 */
function isAbsoluteGuestPath({
  guestPath,
  osFamily,
}: {
  readonly guestPath: string;
  readonly osFamily: OsFamily;
},): boolean {
  if (osFamily !== 'windows') {
    return guestPath.startsWith('/',);
  }
  if (guestPath.startsWith(String.raw`\\`,)) {
    return true;
  }
  /**
   Character after the drive letter and colon; a separator there makes the path absolute.
   */
  const separator = guestPath.charAt(2,);
  return isAsciiLetter(guestPath.charAt(0,),)
    && (guestPath.charAt(1,) === ':')
    && ((separator === '\\') || (separator === '/'));
}

//endregion Guest paths without a shared directory

//region Push (host -> guest)

/**
 Pushes a file from the host filesystem into a running VM.

 For a VM with the virtiofs shared directory, the file is copied into the
 VM's shared directory on the host.
 The guest can access it at {@link GUEST_MOUNT_POINT}`/{filename}` (Linux)
 or {@link WINDOWS_GUEST_MOUNT_POINT}`\{filename}` (Windows).
 Only the file name of `guestPath` is used: the file is placed at
 `shared/{basename}` and the caller should use the returned guest path to
 reference it.

 For a VM without the shared directory, the file is written to `guestPath`
 itself through the guest agent; the path must be absolute and its directory
 must exist.

 @param name - VM name without the mvm- prefix

 @param hostPath - Absolute or relative path on the host to read from

 @param guestPath - Desired filename or path inside the guest.
 With the shared directory, a bare filename is enough; without it, the full path in the guest.

 @returns Absolute path inside the guest where the file is accessible

 @throws Error when the VM directory does not exist

 @throws {@link GuestPathError} when the VM has no shared directory and `guestPath` is not absolute

 @throws {@link GuestFileTransferError} when the copy through the guest agent does not complete

 @example
 ```ts
 const guestPath = await pushFile({
   name: 'dev-01',
   hostPath: '/tmp/setup.sh',
   guestPath: 'setup.sh',
 });
 // guestPath => '/mnt/shared/setup.sh'
 ```
 */
export async function pushFile(
  {
    name,
    hostPath,
    guestPath,
  }: {
    readonly name: string;
    readonly hostPath: string;
    readonly guestPath: string;
  },
): Promise<string> {
  validateName(name,);
  /**
   Tagged logger so push entries are scoped to {@link pushFile} in the output.
   */
  const rl = tagged({
    tag: pushFile.name,
    l,
  },);

  /**
   Per-VM directory holding metadata and, when the VM has one, the shared directory.
   */
  const vmDir = join(
    VMS_DIR,
    name,
  );
  /**
   VM metadata: which channel files travel through, and the guest mount point's OS family.
   */
  const meta = await readVmMeta(vmDir,);

  if (meta.fileTransfer === 'guest-agent') {
    if (!isAbsoluteGuestPath({
      guestPath,
      osFamily: meta.osFamily,
    },)) {
      throw new GuestPathError({
        guestPath,
        name,
        osFamily: meta.osFamily,
      },);
    }
    await pushThroughAgent({
      domain: `${VM_PREFIX}${name}`,
      guestPath,
      hostPath,
    },);
    rl.info(`pushed ${hostPath} -> ${guestPath} (through the guest agent)`,);
    return guestPath;
  }

  /**
   Host-side shared directory for this VM.
   */
  const sharedDir = join(
    vmDir,
    SHARED_DIR_NAME,
  );
  await mkdir(
    sharedDir,
    { recursive: true, },
  );

  /**
   Filename to use inside the shared directory.
   */
  const filename = basename(guestPath,);

  /**
   Full path on the host inside the shared directory.
   */
  const sharedHostPath = join(
    sharedDir,
    filename,
  );

  rl.info(`pushing ${hostPath} -> ${sharedHostPath}`,);
  await copyFile(
    hostPath,
    sharedHostPath,
  );

  /**
   Absolute path the guest will use to read the pushed file, branched on OS.
   */
  const guestFilePath = meta.osFamily
    === 'windows'
    ? `${WINDOWS_GUEST_MOUNT_POINT}${filename}`
    : `${GUEST_MOUNT_POINT}/${filename}`;

  rl.info(`pushed ${hostPath} -> ${guestFilePath} (via ${sharedHostPath})`,);
  return guestFilePath;
}

//endregion Push (host -> guest)

//region Pull (guest -> host)

/**
 Pulls a file from a running VM to the host filesystem.

 For a VM with the virtiofs shared directory, the guest writes files to its
 shared mount, and this function reads them from the corresponding host-side
 shared directory; only the file name of `guestPath` is used.

 For a VM without the shared directory, the file at `guestPath` itself is
 read through the guest agent; the path must be absolute.

 @param name - VM name without the mvm- prefix

 @param guestPath - Filename or path relative to the shared mount inside the guest,
 or the full path in the guest for a VM without the shared directory

 @returns File content as a Buffer

 @throws Error when the file does not exist in the shared directory

 @throws {@link GuestPathError} when the VM has no shared directory and `guestPath` is not absolute

 @throws {@link GuestFileTransferError} when the read through the guest agent does not complete

 @example
 ```ts
 const content = await pullFile({
   name: 'dev-01',
   guestPath: 'output.txt',
 });
 console.log(content.toString('utf8'));
 ```
 */
export async function pullFile(
  {
    name,
    guestPath,
  }: {
    readonly name: string;
    readonly guestPath: string;
  },
): Promise<Buffer> {
  validateName(name,);
  /**
   Tagged logger so pull entries are scoped to {@link pullFile} in the output.
   */
  const rl = tagged({
    tag: pullFile.name,
    l,
  },);

  /**
   Per-VM directory holding metadata and, when the VM has one, the shared directory.
   */
  const vmDir = join(
    VMS_DIR,
    name,
  );
  /**
   VM metadata: which channel files travel through.
   */
  const meta = await readVmMeta(vmDir,);

  if (meta.fileTransfer === 'guest-agent') {
    if (!isAbsoluteGuestPath({
      guestPath,
      osFamily: meta.osFamily,
    },)) {
      throw new GuestPathError({
        guestPath,
        name,
        osFamily: meta.osFamily,
      },);
    }
    return await pullThroughAgent({
      domain: `${VM_PREFIX}${name}`,
      guestPath,
    },);
  }

  /**
   Host-side shared directory for this VM.
   */
  const sharedDir = join(
    vmDir,
    SHARED_DIR_NAME,
  );

  /**
   Filename to read from the shared directory.
   */
  const filename = basename(guestPath,);

  /**
   Full path on the host.
   */
  const sharedHostPath = join(
    sharedDir,
    filename,
  );

  rl.info(`pulling ${sharedHostPath} (guest: ${guestPath})`,);
  /**
   File payload read from the shared mount; returned to the caller as a Buffer.
   */
  const content = await readFile(sharedHostPath,);
  rl.info(`pulled ${String(content.length,)} bytes from ${sharedHostPath}`,);

  return content;
}

//endregion Pull (guest -> host)

/**
 Reads the process's mountinfo and udev's UUID links, read-only, to answer which filesystem holds a path.

 @module
 */

import {
  readdir,
  readFile,
  realpath,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  BY_UUID_FOLDER,
  MOUNTINFO_MOUNT_POINT_FIELD,
  MOUNTINFO_PATH,
} from './constants.ts';
import { errorCodeOf, } from './error-code-of.ts';

/**
 One mount of the process's namespace.
 */
export type Mount = {
  readonly mountPoint: string;
  readonly source: string;
};

/**
 Mountinfo escapes a space, tab, newline or backslash in a path as a three-digit octal sequence; this undoes it.

 @param field - the escaped field
 
 @returns the path
 */
function unescapeField(field: string,): string {
  /**
   The pieces of this step.
   */
  const pieces = field.split('\\',);
  return pieces.map(function decode(
    piece,
    index,
  ) {
    /**
     The octal of this step.
     */
    const octal = piece.slice(
      0,
      OCTAL_ESCAPE_LENGTH,
    );
    return (index > 0)
      && (octal.length === OCTAL_ESCAPE_LENGTH)
      && Array.from(octal,)
        .every(function isOctal(digit,) {
          return '01234567'.includes(digit,);
        },)
      ? String.fromCodePoint(Number.parseInt(
        octal,
        OCTAL_BASE,
      ),) + piece.slice(OCTAL_ESCAPE_LENGTH,)
      : (index > 0 ? `\\${piece}` : piece);
  },)
    .join('',);
}

/**
 A UDEV link name with the device it resolves to.
 */
type ResolvedLink = {
  readonly name: string;
  readonly target: string;
};

/**
 Length of the octal digits of a mountinfo escape.
 */
const OCTAL_ESCAPE_LENGTH = 3;

/**
 Base of the octal digits of a mountinfo escape.
 */
const OCTAL_BASE = 8;

/**
 Parses mountinfo text.

 @param text - the content of `/proc/self/mountinfo`
 
 @returns the mounts in file order

 @example
 ```ts
 import { parseMountInfo, } from '\@monochromatic-dev/repository-link-check';
 
 const result = parseMountInfo(text,);
 ```
 */
export function parseMountInfo(text: string,): readonly Mount[] {
  return text.split('\n',)
    .filter(function notBlank(line,) {
      return line.trim() !== '';
    },)
    .map(function toMount(line,): Mount {
      /**
       The fields of this step.
       */
      const fields = line.split(' ',);
      /**
       The separator of this step.
       */
      const separator = fields.indexOf('-',);
      return {
        mountPoint: unescapeField(fields[MOUNTINFO_MOUNT_POINT_FIELD - 1] ?? '',),
        source: separator === (-1) ? '' : fields[separator + 2] ?? '',
      };
    },);
}

/**
 Finds the mount that holds a path: the one with the longest mount point that is a prefix of the path.

 @param mounts - the mounts
 
 @param path - an absolute path with every link resolved
 
 @returns the mount, or an empty mount when none matches

 @example
 ```ts
 import { mountHolding, } from '\@monochromatic-dev/repository-link-check';
 
 const result = mountHolding({ mounts, path, },);
 ```
 */
export function mountHolding(
  {
    mounts,
    path,
  }: {
    readonly mounts: readonly Mount[];
    readonly path: string;
  },
): Mount {
  /**
   The holding of this step.
   */
  const holding = mounts.filter(function isPrefix({ mountPoint, },) {
    return (mountPoint === '/') || (path === mountPoint)
      || path.startsWith(`${mountPoint}/`,);
  },)
    .toSorted(function longestFirst(
      left,
      right,
    ) {
      return right.mountPoint
        .length
        - left.mountPoint
          .length;
    },);
  return holding[0] ?? {
    mountPoint: '',
    source: '',
  };
}

/**
 Reads the mounts of this process.

 @returns the mounts; empty when mountinfo cannot be read

 @example
 ```ts
 import { readMounts, } from './mountinfo.ts';
 
 const result = await readMounts();
 ```
 */
export async function readMounts(): Promise<readonly Mount[]> {
  try {
    return parseMountInfo(await readFile(
      MOUNTINFO_PATH,
      'utf8',
    ),);
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 Finds the filesystem UUID of the block device a source names, through udev's links.

 @param source - the mount source, for example `/dev/mapper/crypt_sda`
 
 @returns the UUID, or an empty string when the source is not a device or has no udev link

 @example
 ```ts
 import { uuidOfSource, } from './mountinfo.ts';
 
 const result = await uuidOfSource(source,);
 ```
 */
export async function uuidOfSource(source: string,): Promise<string> {
  if (!source.startsWith('/dev/',)) {
    return '';
  }
  try {
    /**
     The device of this step.
     */
    const device = await realpath(source,);
    /**
     The external worktree folder names.
     */
    const names = await readdir(BY_UUID_FOLDER,);
    /**
     The resolved of this step.
     */
    const resolved = await Promise.all(names.map(async function resolve(name,) {
      return {
        name,
        target: await realpath(join(
          BY_UUID_FOLDER,
          name,
        ),),
      };
    },),);
    return resolved.find(function sameDevice({ target, }: ResolvedLink,) {
      return target === device;
    },)
      ?.name
      ?? '';
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
      return '';
    }
    throw error;
  }
}

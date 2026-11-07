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
  return field.replaceAll(/\\([0-7]{3})/gv, function decode(_match, octal: string,) {
    return String.fromCodePoint(Number.parseInt(octal, 8,),);
  },);
}

/**
 Parses mountinfo text.

 @param text - the content of `/proc/self/mountinfo`
 @returns the mounts in file order
 */
export function parseMountInfo(text: string,): readonly Mount[] {
  return text.split('\n',).filter(function notBlank(line,) {
    return line.trim() !== '';
  },).map(function toMount(line,): Mount {
    const fields = line.split(' ',);
    const separator = fields.indexOf('-',);
    return {
      mountPoint: unescapeField(fields[MOUNTINFO_MOUNT_POINT_FIELD - 1] ?? '',),
      source: separator < 0 ? '' : fields[separator + 2] ?? '',
    };
  },);
}

/**
 Finds the mount that holds a path: the one with the longest mount point that is a prefix of the path.

 @param mounts - the mounts
 @param path - an absolute path with every link resolved
 @returns the mount, or an empty mount when none matches
 */
export function mountHolding(
  { mounts, path, }: {
    readonly mounts: readonly Mount[];
    readonly path: string;
  },
): Mount {
  const holding = mounts.filter(function isPrefix({ mountPoint, },) {
    return mountPoint === '/' || path === mountPoint || path.startsWith(`${mountPoint}/`,);
  },).toSorted(function longestFirst(left, right,) {
    return right.mountPoint.length - left.mountPoint.length;
  },);
  return holding[0] ?? { mountPoint: '', source: '', };
}

/**
 Reads the mounts of this process.

 @returns the mounts; empty when mountinfo cannot be read
 */
export async function readMounts(): Promise<readonly Mount[]> {
  try {
    return parseMountInfo(await readFile(MOUNTINFO_PATH, 'utf8',),);
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 Finds the filesystem UUID of the block device a source names, through udev's links.

 @param source - the mount source, for example `/dev/mapper/crypt_sda`
 @returns the UUID, or an empty string when the source is not a device or has no udev link
 */
export async function uuidOfSource(source: string,): Promise<string> {
  if (!source.startsWith('/dev/',)) {
    return '';
  }
  try {
    const device = await realpath(source,);
    const names = await readdir(BY_UUID_FOLDER,);
    const resolved = await Promise.all(names.map(async function resolve(name,) {
      return { name, target: await realpath(join(BY_UUID_FOLDER, name,),), };
    },),);
    return resolved.find(function sameDevice({ target, },) {
      return target === device;
    },)?.name ?? '';
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return '';
    }
    throw error;
  }
}

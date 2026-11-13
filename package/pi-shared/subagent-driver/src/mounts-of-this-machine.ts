/**
 The mount points of this machine, as the kernel lists them, so a directory that is another filesystem's file tree is told from
 one that is a run's own folder.

 A run's scratch folder is emptied when the run ends, and a folder that is a mount point is never emptied: what is mounted there
 belongs to whoever mounted it, and a deletion that crossed into it would delete files this run did not make.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a path this account may not read.
 */
const PERMISSION_DENIED_CODE = 'EACCES';

/**
 The file that lists every mount of this machine.
 */
const MOUNTS_FILE = '/proc/self/mounts';

/**
 The character that starts an escaped byte in a mount's path.
 */
const MOUNT_ESCAPE_START = '\\';

/**
 The number of characters one escape of the mount list takes after its start, which is the octal digits of one byte.
 */
const MOUNT_ESCAPE_LENGTH = 3;

/**
 The characters the kernel writes as an escape in a mount's path: a space, a tab, a newline and a backslash.
 */
const MOUNT_ESCAPES: ReadonlyMap<string, string> = new Map<string, string>([
  [
    '040',
    ' ',
  ],
  [
    '011',
    '\t',
  ],
  [
    '012',
    '\n',
  ],
  [
    '134',
    '\\',
  ],
]);

/**
 Reads one path as the kernel wrote it in the mount list, with the escapes of the characters it writes that way.

 @param written - the path, as the mount list spells it

 @returns the path itself

 @example
 ```ts
 const path = unescapedPathOf({ written: '/mnt/a\\040b', },);
 ```
 */
function unescapedPathOf({ written, }: { readonly written: string },): string {
  /**
   The path's characters, once the escapes are read.
   */
  const characters: string[] = [];
  for (let index = 0; index < written.length; index += 1) {
    /**
     The character at this position.
     */
    const character = written.charAt(index,);
    if (character !== MOUNT_ESCAPE_START) {
      characters.push(character,);
      continue;
    }
    /**
     The digits of the escaped byte, which follow the escape character.
     */
    const digits = written.slice(
      index + 1,
      index + 1
        + MOUNT_ESCAPE_LENGTH,
    );
    /**
     The character that byte stands for, which is nothing for a path that holds a backslash of its own.
     */
    const escaped = MOUNT_ESCAPES.get(digits,);
    if (escaped === undefined) {
      characters.push(character,);
      continue;
    }
    characters.push(escaped,);
    index += digits.length;
  }
  return characters.join('',);
}

/**
 Reads the paths of the mount points one mount list names.

 @param shown - what `/proc/self/mounts` printed

 @returns every mount point's path, in the order the list named them

 @example
 ```ts
 const mounts = mountPathsIn({ shown: '/dev/sda1 / ext4 rw 0 0\n/dev/sdb1 /mnt/a\\040b btrfs rw 0 0\n', },);
 ```
 */
export function mountPathsIn({ shown, }: { readonly shown: string },): readonly string[] {
  return shown
    .split('\n',)
    .flatMap(function pathOfLine(line,) {
      /**
       The line's fields, of which the second is the mount point.
       */
      const [, written,] = line.split(' ',);
      if (written === undefined) {
        return [];
      }
      return [unescapedPathOf({ written, },),];
    },);
}

/**
 Reads the mount points of this machine, from the list the kernel keeps.

 @returns every mount point's path, or none when the list cannot be read, which leaves the device comparison in the emptying as
 the guard that an entry on another filesystem is left where it is

 @example
 ```ts
 const mounts = await mountPointsNow();
 ```
 */
export async function mountPointsNow(): Promise<readonly string[]> {
  try {
    return mountPathsIn({ shown: await readFile(
      MOUNTS_FILE,
      'utf8',
    ), },);
  }
  catch (error) {
    /**
     The code this failure carries.
     */
    const code = errorCodeOf(error,);
    // A machine without the kernel's mount list, or a list this account may not read, still has the device of every entry to
    // compare, so the guard that keeps another filesystem's files is the one that answers and the deletion goes on.
    if ((code === MISSING_PATH_CODE) || (code === PERMISSION_DENIED_CODE)) {
      return [];
    }
    throw error;
  }
}

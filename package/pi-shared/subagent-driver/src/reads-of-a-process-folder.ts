/**
 The reads the driver makes inside one process's folder, gathered so a test hands over its own and no test needs a real process to
 end between two of the driver's steps.

 A real process cannot be made to end between the folder being listed and one record being read: the operating system decides
 when, and a plain file in a test's folder answers `ENOENT` rather than the `ESRCH` a released task gives. So the reads are the
 seam, the way the process folder itself already is: every other read the driver makes of a process is unchanged.

 @module
 */

import {
  readFile,
  readlink,
} from 'node:fs/promises';

/**
 Reads one record inside a process's folder.

 @param path - the record's own path

 @returns the text the folder holds there

 @throws the error the operating system gave, which is the whole of what a caller reads

 @example
 ```ts
 const command = await READS_OF_A_PROCESS_FOLDER.text('/proc/486667/comm');
 ```
 */
export type ReadsAProcessRecord = (path: string) => Promise<string>;

/**
 The reads the driver makes of the records one process's folder keeps.
 */
export type ReadsOfAProcessFolder = {
  /**
   Reads the text of one record, such as `comm`, `cgroup` or `stat`.
   */
  readonly text: ReadsAProcessRecord;

  /**
   Reads what a link names, which is how the folder a process works in is read.
   */
  readonly link: ReadsAProcessRecord;
};

/**
 Reads this machine's process folder, which is what every caller makes of a process unless a test hands over its own.
 */
export const READS_OF_A_PROCESS_FOLDER: ReadsOfAProcessFolder = {
  text: async function textIn(path: string,) {
    return await readFile(
      path,
      'utf8',
    );
  },
  link: async function linkAt(path: string,) {
    return await readlink(path,);
  },
};

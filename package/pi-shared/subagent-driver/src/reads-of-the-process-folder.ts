/**
 One process's own files, read from the folder that holds one entry per process, and the codes a process that has ended answers
 to those reads.

 A process is not one thing to read. It may end between the listing of the process folder and the read of one of its files, and
 the operating system answers that read with a code of its own instead of the file, so every reader of a process folder in this
 package reads through this module and one measurement decides what "this process has gone" means.

 Measured on 2026-11-14, against processes this machine started and ended, every read made against `/proc`:

 - a process gone before its file was opened answers `ENOENT` to a read of `comm`, `stat`, `cgroup`, `cmdline` or `status`, to a
   read of the `cwd` link, to a listing of its own folder, and to a `stat` of its own folder: the path is not there;
 - a process gone after its file was opened answers `ESRCH` to the read of `comm`, `stat`, `cgroup`, `cmdline` and `status`: the
   kernel had the process when the open asked for it and has it no more when the read asks. A scan that walks a process table of a
   working machine meets this on its own, because a process ending between the listing and the read of its files is ordinary;
 - another account's process answers `EACCES` or `EPERM`, and a path component that is a file answers `ENOTDIR`. Neither is
   evidence about work still running, and each reader below decides for itself what such an entry means.

 Before 2026-11-14 only `ENOENT` was read as a process that had gone, so a build that ended a moment before the scan read it failed
 the whole scan: the summary of the run `lock-shim-of-pi-takes-over-a-lock-whose-holder-is-gone-2` recorded
 `ESRCH: no such process, read '/proc/4038785/comm'` as its `workProblem`.

 @module
 */

import {
  readFile,
  readlink,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The codes the operating system gives a read of a process's own file when the process has ended before or during that read.
 */
export const GONE_PROCESS_CODES: ReadonlySet<string> = new Set<string>([
  // The file was opened after the process had gone, so the path under `/proc` no longer names it.
  'ENOENT',
  // The file was opened while the process still lived and read once it had gone, which is what one scan of a working process
  // table meets without asking for it.
  'ESRCH',
]);

/**
 Says whether the code of a failed read is the operating system saying that the process has gone.

 @param code - the code the read failed with, or empty text when it carried none

 @returns whether the process the read was about had ended

 @example
 ```ts
 const gone = processHasGone({ code: 'ESRCH', },);
 ```
 */
export function processHasGone({ code, }: { readonly code: string },): boolean {
  return GONE_PROCESS_CODES.has(code,);
}

/**
 A read of a process's own file that failed for a reason which is not that process having gone, so nothing can be said about
 what the process was doing.

 The path and the code are carried apart from the message, so a caller reports both without reading the text of either.
 */
export class ProcessFileUnreadableError extends Error {
  /**
   The file the read was about.
   */
  readonly path: string;

  /**
   The code the read failed with, or empty text when it carried none.
   */
  readonly code: string;

  /**
   Keeps the failure that was caught, so a reader of this one can see what the operating system said.

   @param path - the file the read was about

   @param error - what the read threw
   */
  constructor({
    path,
    error,
  }: {
    readonly path: string;
    readonly error: unknown;
  },) {
    /**
     The code the read failed with.
     */
    const code = errorCodeOf(error,);
    /**
     What a reader of this failure is told: which file, which code, and that the code is no process having gone.
     */
    const message = `${path} could not be read: ${code === '' ? 'the failure carries no code' : code}. `
      + 'That is not a process that has gone, so what the run still has running cannot be read.';
    super(
      message,
      { cause: error, },
    );
    this.name = 'ProcessFileUnreadableError';
    this.path = path;
    this.code = code;
  }
}

/**
 The reads this package makes of one process: where its working folder link points, and the text of one of its own files.

 A test supplies its own, so a scan over a process that ends between two of its reads is checked without a race against a real
 one.

 @example
 ```ts
 const reads: ProcessFolderReads = { workingFolderOf: async (path) => await readlink(path), textOf: async (path) => await readFile(path, 'utf8'), };
 ```
 */
export type ProcessFolderReads = {
  /**
   Where the link a process's folder holds for its working folder points, as `readlink` answers it.
   */
  readonly workingFolderOf: (path: string) => Promise<string>;

  /**
   The text of one of a process's own files, as `readFile` answers it.
   */
  readonly textOf: (path: string) => Promise<string>;
};

/**
 The reads of this machine's process folder, which is `/proc` unless a run names another.
 */
export const readsOfTheProcessFolder: ProcessFolderReads = {
  workingFolderOf: async function workingFolderOf(path,) {
    return await readlink(path,);
  },
  textOf: async function textOf(path,) {
    return await readFile(
      path,
      'utf8',
    );
  },
};

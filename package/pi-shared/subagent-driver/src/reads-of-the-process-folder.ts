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

 A process number is no name for one process. The kernel gives a freed number to a later process, and the control group such a
 process is in says nothing about whose it is: on 2026-11-14 a review of this code proved that a record naming a build at a number
 another process now holds is read as the build still running. So `/proc/<pid>/stat` is read as well, which says both the state the
 kernel gives the process and its start time in clock ticks since boot, and a start time no later process at that number repeats
 (`package/mpm-shadow/cli/src/shadow_identity.rs` reads it the same way for the same reason). Measured with a process this machine
 started and ended: a gone process answers `ENOENT` before the open and `ESRCH` after it, and a process that has ended but whose
 parent has not reaped it still answers, with the state `Z`.

 @module
 */

import { readFileSync, } from 'node:fs';
import {
  readFile,
  readlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
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

/**
 What one line of `/proc/<pid>/stat` says about a process: whether it holds one at all, and which process it describes.
 */
export type LifeOfAProcess =
  | {
    /**
     The line holds a state and a start time.
     */
    readonly found: true;

    /**
     The state the kernel gives it: `R` while it runs, `S` while it sleeps, `Z` once it has ended and its parent has not reaped it.
     */
    readonly state: string;

    /**
     Its start time in clock ticks since boot, which no later process at its number repeats.
     */
    readonly birth: string;
  }
    | {
    /**
     The line holds no state and no start time, so it describes no process this can read.
     */
      readonly found: false;
    };

/**
 Which field after the program's name holds the start time: the twentieth, counted from one.
 */
const BIRTH_FIELD_AFTER_THE_NAME = 19;

/**
 Reads the state and the start time out of one line of `/proc/<pid>/stat`.

 That file puts the program's name in brackets as its second field, and a name may hold spaces and brackets of its own, so the fields
 are counted from the last `)` in the line: the state is the first field after it, and the start time is the twentieth. The kernel
 writes one space between those fields.

 @param statText - one whole line of `/proc/<pid>/stat`

 @returns the state and the birth, or that the line holds neither

 @example
 ```ts
 const life = lifeIn({ statText: '4242 (sleep) S 1 4242 4242 0 -1 4194560 91 0 0 0 0 0 0 0 20 0 1 0 20004601 2797568 185 18446744073709551615', },);
 ```
 */
export function lifeIn({ statText, }: { readonly statText: string },): LifeOfAProcess {
  /**
   Where the program's name ends, which is where the fields a reader can count begin.
   */
  const named = statText.lastIndexOf(')',);
  if (named === (-1)) {
    return { found: false, };
  }
  /**
   The fields after the name, of which the state is the first and the start time the twentieth.
   */
  const fields = statText.slice(named + 1,)
    .trim()
    .split(' ',);
  /**
   The state the kernel gives the process.
   */
  const [state,] = fields;
  /**
   The start time the kernel gave it, in clock ticks since boot.
   */
  const birth = fields[BIRTH_FIELD_AFTER_THE_NAME];
  if (
    (state === undefined)
      || (state === '')
      || (birth === undefined)
      || (birth === '')
  ) {
    return { found: false, };
  }
  return {
    found: true,
    state,
    birth,
  };
}

/**
 The states a process holds once it has ended: a zombie waiting to be reaped, and one the kernel is tearing down.
 */
const ENDED_PROCESS_STATES: ReadonlySet<string> = new Set<string>([
  'Z',
  'X',
]);

/**
 Says whether the state the kernel gives a process is one of a process that has ended.

 A zombie has done no work since it ended: its working-folder link has no target, and nothing it started is still going.

 @param state - the state `/proc/<pid>/stat` gave

 @returns whether the process has ended

 @example
 ```ts
 const ended = processHasEnded({ state: 'Z', },);
 ```
 */
export function processHasEnded({ state, }: { readonly state: string },): boolean {
  return ENDED_PROCESS_STATES.has(state,);
}

/**
 What reading the birth of a process now answered.
 */
export type BirthOfAProcessNow =
  | {
    /**
     The start time the kernel gave it, in clock ticks since boot.
     */
    readonly read: 'birth';
    readonly birth: string;
  }
    | {
    /**
     Why no birth was read, which is a process that has gone as much as a folder this account may not read.
     */
      readonly read: 'unreadable';
      readonly code: string;
    };

/**
 Reads a process's birth from the process folder, now, without waiting.

 The line that records a run's work is written while Pi's events arrive, in the order they arrived, so the birth it carries is read
 where it is written rather than awaited: a promise would let the next event's line be written first, and the number in the feed could
then sit beside a birth read after that number had been reused. It is one read of one short file per process a tool answer names.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param pid - the process number

 @returns the birth, or why there is none

 @example
 ```ts
 const read = birthOfProcessNow({ processFolder: '/proc', pid: 486667, },);
 ```
 */
export function birthOfProcessNow({
  processFolder,
  pid,
}: {
  readonly processFolder: string;
  readonly pid: number;
},): BirthOfAProcessNow {
  /**
   The line of the process's own record that holds its state and its start time.
   */
  const path = join(
    processFolder,
    String(pid,),
    'stat',
  );
  try {
    /**
     What the line says about the process, read now rather than awaited: the feed line this birth belongs to cannot wait for it.
     */
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- the line that records a run's work is written in the order Pi's events arrive, so this read cannot be awaited; see doc/lint/sync-read-of-one-process-line.md
    const statText = readFileSync(
      path,
      'utf8',
    );
    /**
     The life the line describes.
     */
    const life = lifeIn({ statText, },);
    if (!life.found) {
      return {
        read: 'unreadable',
        code: '',
      };
    }
    return {
      read: 'birth',
      birth: life.birth,
    };
  }
  catch (error) {
    /**
     The code the read failed with, which is what this answer says instead of a birth.
     */
    const code = errorCodeOf(error,);
    // A process that has gone, and one this account may not read, keep no birth in the record: the caller leaves the field out, and
    // a scan then says the recorded work is not provably the process at that number.
    return {
      read: 'unreadable',
      code,
    };
  }
}

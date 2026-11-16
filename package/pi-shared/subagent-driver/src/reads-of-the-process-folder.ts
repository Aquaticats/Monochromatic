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
 How many clock ticks `/proc/<pid>/stat` counts a second in, which the kernel reports in hundreds (`USER_HZ`).

 Measured here on 2026-11-15: `/proc/self/stat`'s start time divided by `/proc/uptime`'s seconds was 99.99998 over three rounds.
 */
export const CLOCK_TICKS_PER_SECOND = 100;

/**
 What the boot time a process folder reports looks like.
 */
export type BootTimeInAProcessFolder =
  | {
    /**
     The instant the machine booted, in seconds since the epoch.
     */
    readonly found: true;
    readonly seconds: number;
  }
    | {
    /**
     The folder reported no boot time, so no start time it gives can be placed on the wall clock.
     */
      readonly found: false;
    };

/**
 Reads the instant the machine booted from the line of the process folder that reports it (`btime` in `/proc/stat`).

 A process's own start time counts clock ticks since that instant, so this is what puts it on the wall clock a tool's answer is timed
 by.

 @param statText - the process folder's own line, whose `btime` names the instant

 @returns the instant, or that the line names none

 @example
 ```ts
 const bootedAt = bootTimeIn({ statText: 'cpu  0 0 0 0 0 0 0 0 0 0\nbtime 1700000000\n', },);
 ```
 */
export function bootTimeIn({ statText, }: { readonly statText: string },): BootTimeInAProcessFolder {
  for (const line of statText.split('\n',)) {
    /**
     The words of one line, of which the second is the boot time when the first says so.
     */
    const words = line.trim()
      .split(' ',);
    if ((words[0] !== 'btime') || (words[1] === undefined)) {
      continue;
    }
    /**
     The instant the line names, in seconds since the epoch.
     */
    const seconds = Number(words[1],);
    if (Number.isFinite(seconds,)) {
      return {
        found: true,
        seconds,
      };
    }
  }
  return { found: false, };
}

/**
 Milliseconds in a second, so a start time counted in ticks and a boot time in seconds meet on one clock.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 What placing a process's kernel start time on the wall clock answered.
 */
export type StartInstantOfAProcess =
  | {
    /**
     The instant the process started, in milliseconds since the epoch.
     */
    readonly found: true;
    readonly instant: number;
  }
    | {
    /**
     The process's start time and the machine's boot time cannot be put together, so nothing is placed.
     */
      readonly found: false;
    };

/**
 Places the start time the kernel gave a process on the wall clock, in milliseconds since the epoch.

 @param life - the state and start time the process folder gave for the process

 @param booted - the instant the machine booted, as the process folder reports it

 @returns the instant the process started, or that the two cannot be put together

 @example
 ```ts
 const started = startInstantIn({ life, booted, },);
 ```
 */
export function startInstantIn({
  life,
  booted,
}: {
  readonly life: LifeOfAProcess;
  readonly booted: BootTimeInAProcessFolder;
},): StartInstantOfAProcess {
  if ((!life.found) || (!booted.found)) {
    return { found: false, };
  }
  /**
   The start time as a number of ticks, which the file writes as digits.
   */
  const ticks = Number(life.birth,);
  if (!Number.isFinite(ticks,)) {
    return { found: false, };
  }
  return {
    found: true,
    instant: (booted.seconds * MILLISECONDS_PER_SECOND)
      + ((ticks / CLOCK_TICKS_PER_SECOND) * MILLISECONDS_PER_SECOND),
  };
}

/**
 What reading one line of a process's own record answered.
 */
export type ALineOfAProcess =
  | {
    /**
     The line, as the folder holds it.
     */
    readonly read: true;
    readonly text: string;
  }
    | {
    /**
     Why the read failed, which a caller decides on.
     */
      readonly read: false;
      readonly code: string;
      readonly error: unknown;
    };

/**
 Reads one file of a process's own folder, answering the failure rather than throwing it, so a caller can decide on its code.

 @param path - the file to read

 @param reads - the reads of one process, which a test replaces

 @returns the text, or the failure and the code it carried

 @example
 ```ts
 const line = await aLineOfAProcess({ path: '/proc/12/stat', reads, },);
 ```
 */
export async function aLineOfAProcess({
  path,
  reads,
}: {
  readonly path: string;
  readonly reads: ProcessFolderReads;
},): Promise<ALineOfAProcess> {
  try {
    return {
      read: true,
      text: await reads.textOf(path,),
    };
  }
  catch (error) {
    return {
      read: false,
      code: errorCodeOf(error,),
      error,
    };
  }
}

/**
 Says whether a code means the process folder gave no line this account can use: the number holds nothing, or this account may not
 read it.

 @param code - the code a read failed with

 @returns whether the folder gave no line this run can use

 @example
 ```ts
 const unreadable = theNumberGivesNoLine({ code: 'ENOENT', },);
 ```
 */
export function theNumberGivesNoLine({ code, }: { readonly code: string },): boolean {
  return processHasGone({ code, },)
    || (code === 'EACCES')
    || (code === 'EPERM');
}

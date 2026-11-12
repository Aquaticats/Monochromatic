/**
 The step a launcher takes after its unit ended: an agent that ended its turn while work it started still runs is waited for, then
 resumed with a short note, so the work it left running is read instead of being lost.

 The owner, 2026-11-11: "Ending its turn is good. We don't want unnecessary waste of tokens." An agent may end its turn while a
 build or a reproduction it started keeps running. Before 2026-11-11 the launcher called such a run finished and removed its
 worktree, so a review whose reproduction was still running was never written.

 A process counts as the run's own work only when its working folder is the run's worktree or run folder, or when one whole path
 component of it is the run's own name. A substring match tied five round 211 re-merge launchers to the Kimi review of the same
 branch, whose worktree `pi-kimi-review-<the run's name>-round-<n>` contains that name: they waited on another agent's work and
 would have resumed finished sessions.

 @module
 */

import {
  readdir,
  readFile,
  readlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { entriesOrNothingIn, } from './newest-finished-run-of-a-name.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How many times one run's name may be resumed this way. Each resume is a whole session, and a run that ends its turn five times
 while its own work still runs has something to read rather than something to wait out.
 */
export const WAKE_LIMIT = 5;

/**
 Milliseconds between two looks at whether the run's own work is still running. A build or a reproduction takes minutes, and
 every look walks the process table once.
 */
export const WORK_POLL_MILLISECONDS = 30_000;

/**
 The starts a Kimi run's name carries: a review's and an audit's, which name the review or audit and not the code it reads.
 */
const KIMI_REVIEW_START = 'kimi-review-';

/**
 The start of a Kimi audit's run name.
 */
const KIMI_AUDIT_START = 'kimi-audit-';

/**
 How many of the run's own processes a note names. The head says what is working, and a note of thirty entries stops being read.
 */
export const OWN_WORK_NAMED_IN_A_NOTE = 5;

/**
 One process that works somewhere, read from the folder of process entries.
 */
export type ProcessAtAFolder = {
  /**
   The process number.
   */
  readonly pid: number;

  /**
   The folder it works in.
   */
  readonly folder: string;

  /**
   The program it runs, as the kernel's own name for it.
   */
  readonly command: string;
};

/**
 Reads the name the run's own folders carry, which is the run's name without the review or audit part it starts with.

 @param runName - the run's name

 @returns the component a folder of the run's own work carries

 @example
 ```ts
 const component = ownComponentOf({ runName: 'kimi-review-a-round-1', },);
 ```
 */
export function ownComponentOf({ runName, }: { readonly runName: string },): string {
  if (runName.startsWith(KIMI_REVIEW_START,)) {
    return runName.slice(KIMI_REVIEW_START.length,);
  }
  return runName.startsWith(KIMI_AUDIT_START,)
    ? runName.slice(KIMI_AUDIT_START.length,)
    : runName;
}

/**
 Says whether a working folder is one of the run's own.

 @param folder - the folder a process works in

 @param ownFolders - the run's worktree and run folder, which are its own however they are named

 @param ownComponent - the component a folder of the run's own work carries

 @returns whether the folder is the run's own work

 @example
 ```ts
 const own = isOwnFolder({ folder: '/repo/.claude/worktrees/pi-a', ownFolders: ['/repo/.claude/worktrees/pi-a'], ownComponent: 'a', },);
 ```
 */
export function isOwnFolder({
  folder,
  ownFolders,
  ownComponent,
}: {
  readonly folder: string;
  readonly ownFolders: readonly string[];
  readonly ownComponent: string;
},): boolean {
  /**
   The folder's own components, so a folder whose name merely holds the run's name is no match.
   */
  const components = folder.split('/',);
  return ownFolders.some(function isTheWholeFolder(own,) {
    return (own !== '') && ((folder === own) || folder.startsWith(`${own}/`,));
  },)
    || ((ownComponent !== '') && components.includes(ownComponent,));
}

/**
 Reads the folder every process of this user works in, and the program each runs.

 The process folder is the seam: a test writes its own entries and never reads this machine's process table.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @returns one entry per process whose working folder could be read

 @example
 ```ts
 const processes = await processesAtTheirFolders({ processFolder: '/proc', },);
 ```
 */
export async function processesAtTheirFolders({
  processFolder,
}: {
  readonly processFolder: string;
},): Promise<readonly ProcessAtAFolder[]> {
  /**
   The entries of the process folder, of which a folder that is not there holds none: no process can be read, which is no work
   this run can claim rather than a failure that ends a run.
   */
  const entries = await entriesOrNothingIn(processFolder,);
  /**
   What each process works in, read all at once: one process's answer waits on nothing another's does.
   */
  const read = await Promise.all(entries.map(async function ofEntry(entry: string,): Promise<readonly ProcessAtAFolder[]> {
    if (!isDigitsOnly(entry,)) {
      return [];
    }
    try {
      /**
       The folder the process works in.
       */
      const folder = await readlink(join(
        processFolder,
        entry,
        'cwd',
      ),);
      /**
       The kernel's name for the program it runs, which a process that ended before this read leaves unknown.
       */
      const command = await commandNameIn({
        processFolder,
        entry,
      },);
      return [{
        pid: Number(entry,),
        folder,
        command,
      },];
    }
    catch (error) {
      // Another user's process, or one that ended between the listing and the read, works in no folder this run owns.
      if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (errorCodeOf(error,) === 'ENOTDIR')) {
        return [];
      }
      throw error;
    }
  },));
  return read.flat();
}

/**
 Reads the kernel's name for the program one process runs.

 @param processFolder - the folder holding one entry per process

 @param entry - the process's entry in that folder

 @returns the program's name, or `?` when the process ended before this read

 @example
 ```ts
 const command = await commandNameIn({ processFolder: '/proc', entry: '12', },);
 ```
 */
async function commandNameIn({
  processFolder,
  entry,
}: {
  readonly processFolder: string;
  readonly entry: string;
},): Promise<string> {
  try {
    return (await readFile(
      join(
        processFolder,
        entry,
        'comm',
      ),
      'utf8',
    )).trim();
  }
  catch (error) {
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (errorCodeOf(error,) === 'ENOTDIR')) {
      return '?';
    }
    throw error;
  }
}

/**
 Reads what the run's own processes still run.

 @param runName - the run's name

 @param worktree - the worktree the run works in, which is its own however it is named

 @param runFolder - the run's folder

 @param processes - the processes read from the process folder

 @param excludingPid - the process number not to count, which is the launcher's own

 @returns one `pid command` entry per process still working for this run

 @example
 ```ts
 const working = processesStillWorkingFor({ runName: 'a', worktree: '/w/a', runFolder: '/runs/a-1', processes, excludingPid: 12, },);
 ```
 */
export function processesStillWorkingFor({
  runName,
  worktree,
  runFolder,
  processes,
  excludingPid,
}: {
  readonly runName: string;
  readonly worktree: string;
  readonly runFolder: string;
  readonly processes: readonly ProcessAtAFolder[];
  readonly excludingPid: number;
},): readonly string[] {
  /**
   The run's own folders, which are its own however they are named.
   */
  const ownFolders = [
    worktree,
    runFolder,
  ];
  /**
   The component a folder of the run's own work carries.
   */
  const ownComponent = ownComponentOf({ runName, },);
  return processes.filter(function isTheRunsWork(process,) {
    return (process.pid !== excludingPid) && isOwnFolder({
      folder: process.folder,
      ownFolders,
      ownComponent,
    },);
  },)
    .map(function described(process,) {
      return `${String(process.pid,)} ${process.command}`;
    },);
}

/**
 Reads the note a resumed agent gets, which says what it left running and what to do with it.

 @param stillWorking - the run's own processes that still run, which may be empty when they have ended

 @param wallSeconds - how long the run was allowed to last, which the note names when the work outlasted it

 @returns the note

 @example
 ```ts
 const note = wakePromptOf({ stillWorking: [], wallSeconds: 7_200, },);
 ```
 */
export function wakePromptOf({
  stillWorking,
  wallSeconds,
}: {
  readonly stillWorking: readonly string[];
  readonly wallSeconds: number;
},): string {
  /**
   What the note says about the work, in its first sentence.
   */
  const opening = (stillWorking.length > 0)
    ? `Work you left running when your turn ended is still running after ${String(wallSeconds,)} s: ${stillWorking.slice(
      0,
      OWN_WORK_NAMED_IN_A_NOTE,
    )
      .join(', ',)}.`
    : 'The work you left running when your turn ended has now ended.';
  return `${opening} Read its results and continue your task from where you stopped. Ending your turn again while something runs is fine; you are resumed when it ends.\n`;
}

/**
 Reads the words a launcher is run again with to resume a run's session.

 The prompt and the session are replaced, and the run's folder number is dropped so the resumed run writes a folder of its own
 and never writes over the summary the first run wrote.

 @param commandLine - the launcher's own words

 @param wakePrompt - the note's file, which replaces the run's first prompt

 @param sessionFile - the session file the resumed run continues

 @returns the words for the resumed launch

 @example
 ```ts
 const words = wakeResumeWordsOf({ commandLine: ['--name=a', '--prompt=/runs/a-prompt.md'], wakePrompt: '/runs/a-1/wake-prompt.md', sessionFile: '/sessions/a.jsonl', },);
 ```
 */
export function wakeResumeWordsOf({
  commandLine,
  wakePrompt,
  sessionFile,
}: {
  readonly commandLine: readonly string[];
  readonly wakePrompt: string;
  readonly sessionFile: string;
},): readonly string[] {
  return [
    ...commandLine.filter(function isKept(word,) {
      return (!word.startsWith('--session',))
        && (!word.startsWith('--prompt',))
        && (!word.startsWith('--run=',));
    },),
    `--prompt=${wakePrompt}`,
    `--session=${sessionFile}`,
  ];
}

/**
 Says how long a wait for a run's own work may last: the run's wall clock, so no wait outlives the work it waits for.

 @param wallSeconds - how long a run may last, in seconds

 @returns how long the wait may last, in milliseconds

 @example
 ```ts
 const wait = workWaitMillisecondsOf({ wallSeconds: 600, },);
 ```
 */
export function workWaitMillisecondsOf({ wallSeconds, }: { readonly wallSeconds: number },): number {
  return wallSeconds * MILLISECONDS_PER_SECOND;
}

/**
 What the step after a unit ended found, and what it did.
 */
export type WakeOutcome =
  | {
    /**
     The run left nothing of its own running, so there is nothing to wake.
     */
    readonly step: 'nothing-to-wake';
  }
    | {
    /**
     The run's own work still runs, and the run was not resumed: its worktree must stay as it is.
     */
      readonly step: 'work-still-runs';

    /**
     What still runs, as `pid command` entries.
     */
      readonly ownWork: readonly string[];
    }
    | {
    /**
     The run's own work ended, but its session could not be resumed, so its worktree must stay as it is.
     */
      readonly step: 'could-not-resume';

    /**
     What still runs, as `pid command` entries.
     */
      readonly ownWork: readonly string[];

    /**
     Why the session could not be resumed.
     */
      readonly problem: string;
    }
    | {
    /**
     The run was resumed, and the launcher that resumed it has ended.
     */
      readonly step: 'resumed';

    /**
     The status the resumed launcher ended with, or nothing when a signal ended it.
     */
      readonly status?: number;
    };

/**
 Waits for the run's own work and resumes its session, one wake at a time.

 @param runName - the run's name

 @param wallSeconds - how long the run may last, which bounds the wait

 @param wakesSoFar - how many times this run's name has been resumed already

 @param readOwnWork - reads the processes still working for this run

 @param report - where a line about what was done goes

 @param wait - how the wait passes time

 @param now - where the step reads the time

 @param resume - starts the launch that resumes the session

 @param pollMilliseconds - how long between two looks at the run's own work

 @returns what the step found and did

 @example
 ```ts
 const outcome = await wakeIfItsOwnWorkOutlivesItsTurn({ runName: 'a', wallSeconds: 600, wakesSoFar: 0, readOwnWork, report, wait, now: Date.now, resume, },);
 ```
 */
export async function wakeIfItsOwnWorkOutlivesItsTurn({
  runName,
  wallSeconds,
  wakesSoFar,
  readOwnWork,
  report,
  wait,
  now,
  resume,
  pollMilliseconds = WORK_POLL_MILLISECONDS,
}: {
  readonly runName: string;
  readonly wallSeconds: number;
  readonly wakesSoFar: number;
  readonly readOwnWork: () => Promise<readonly string[]>;
  readonly report: (line: string) => void;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly now: () => number;
  readonly resume: (request: { readonly stillWorking: readonly string[] }) => Promise<WakeResumeAnswer>;
  readonly pollMilliseconds?: number;
},): Promise<WakeOutcome> {
  /**
   What the run's own work still runs now.
   */
  const atFirstLook = await readOwnWork();
  if (atFirstLook.length === 0) {
    return { step: 'nothing-to-wake', };
  }
  /**
   What to tell a reader about those processes.
   */
  const named = atFirstLook.slice(
    0,
    OWN_WORK_NAMED_IN_A_NOTE,
  )
    .join(', ',);
  if (wakesSoFar >= WAKE_LIMIT) {
    report(`${runName}: ended its turn while its own work runs (${named}), and it has been woken ${String(wakesSoFar,)} times, so its worktree is kept`,);
    return {
      step: 'work-still-runs',
      ownWork: atFirstLook,
    };
  }
  report(`${runName}: ended its turn while its own work runs (${named}); waiting for it, then resuming the session`,);
  /**
   What still runs once the wait is over.
   */
  const leftRunning = await workLeftAfterWaiting({
    readOwnWork,
    wait,
    now,
    deadline: now() + workWaitMillisecondsOf({ wallSeconds, },),
    pollMilliseconds,
  },);
  if (leftRunning.length > 0) {
    report(`${runName}: its work still runs after ${String(wallSeconds,)} s (${leftRunning.slice(
      0,
      OWN_WORK_NAMED_IN_A_NOTE,
    )
      .join(', ',)}); resuming the session anyway so it can see why`,);
  }
  /**
   What resuming the session answered.
   */
  const answer = await resume({ stillWorking: leftRunning, },);
  if (!answer.resumed) {
    return {
      step: 'could-not-resume',
      ownWork: leftRunning,
      problem: answer.problem ?? 'the run names no session file',
    };
  }
  return {
    step: 'resumed',
    ...((answer.status === undefined) ? {} : { status: answer.status, }),
  };
}

/**
 What resuming a session answered.
 */
export type WakeResumeAnswer = {
  /**
   Whether the session was resumed.
   */
  readonly resumed: boolean;

  /**
   The status the resumed launcher ended with, or nothing when a signal ended it.
   */
  readonly status?: number;

  /**
   Why the session could not be resumed, when it could not.
   */
  readonly problem?: string;
};

/**
 Reads what the run's own work still runs when the wait is over.

 The wait never outlives the deadline it was given, and a look that finds nothing ends it at once.

 @param readOwnWork - reads the processes still working for this run

 @param wait - how the wait passes time

 @param now - where the wait reads the time

 @param deadline - when the wait must be over, as milliseconds since the epoch

 @param pollMilliseconds - how long between two looks

 @returns what still runs, as `pid command` entries

 @example
 ```ts
 const left = await workLeftAfterWaiting({ readOwnWork, wait, now: Date.now, deadline: 1_000, pollMilliseconds: 30_000, },);
 ```
 */
async function workLeftAfterWaiting({
  readOwnWork,
  wait,
  now,
  deadline,
  pollMilliseconds,
}: {
  readonly readOwnWork: () => Promise<readonly string[]>;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly now: () => number;
  readonly deadline: number;
  readonly pollMilliseconds: number;
},): Promise<readonly string[]> {
  /* oxlint-disable eslint/no-await-in-loop -- a look, then a wait, then the next look: the look is what tells this wait whether
     the run's own work ended. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     How much of the wait is left.
     */
    const left = deadline - now();
    if (left <= 0) {
      return await readOwnWork();
    }
    /**
     How long this wait lasts: the interval, cut to what the deadline leaves.
     */
    const betweenLooks = Math.min(
      pollMilliseconds,
      left,
    );
    /**
     How long this wait lasts, which is never nothing: a wait of no milliseconds would look again at once.
     */
    const thisWait = Math.max(
      1,
      betweenLooks,
    );
    await wait(thisWait,);
    /**
     What the run's own work still runs.
     */
    const working = await readOwnWork();
    if (working.length === 0) {
      return working;
    }
    if (now() >= deadline) {
      return working;
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

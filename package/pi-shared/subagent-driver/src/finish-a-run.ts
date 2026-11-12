/**
 What a launcher does once its unit has ended: the wake of an agent whose own work outlived its turn, and what becomes of the
 worktree.

 The step is kept apart from the launcher that starts the unit so it can be read alone: it is where a run is resumed and where a
 worktree may be removed, and both turn on what the run left running.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  endReasonIn,
  onABranchNow,
  statusOf,
} from './end-of-a-run.ts';
import {
  type RunCommand,
  type ShownCommandEnding,
  runCommandShowingItsOutput,
} from './run-command.ts';
import {
  SessionFileUnreadableError,
  sessionFileOfRun,
} from './session-file-of-a-run.ts';
import {
  type GitRunner,
  removeWorktree,
  worktreeOutcomeOf,
} from './worktree-of-a-run.ts';
import {
  OWN_WORK_NAMED_IN_A_NOTE,
  type WakeOutcome,
  type WakeResumeAnswer,
  processesAtTheirFolders,
  processesStillWorkingFor,
  wakeIfItsOwnWorkOutlivesItsTurn,
  wakePromptOf,
  wakeResumeWordsOf,
} from './wake-a-run-whose-work-outlives-its-turn.ts';

/**
 What finishing one run needs.
 */
export type FinishRequest = {
  /**
   The run's name.
   */
  readonly runName: string;

  /**
   Where the driver wrote this run.
   */
  readonly runFolder: string;

  /**
   Folder the runs live in, which a resumed run's session file is read from.
   */
  readonly runsFolder: string;

  /**
   The status the run's unit ended with, which this step answers with unless the run was resumed.
   */
  readonly unitStatus: number;

  /**
   How long the run may last, which bounds the wait for its own work.
   */
  readonly wallSeconds: number;

  /**
   Whether the run works in the checkout itself, which is never resumed and never removed.
   */
  readonly readOnlyInMain: boolean;

  /**
   The worktree the run worked in.
   */
  readonly worktree: string;

  /**
   The checkout whose worktrees the launcher makes.
   */
  readonly checkoutFolder: string;

  /**
   The folder holding one entry per process, `/proc` on a Linux machine.
   */
  readonly processFolder: string;

  /**
   The launcher's own words, which a resumed run is started with again.
   */
  readonly commandLine: readonly string[];

  /**
   The launcher's entry, which the resumed run is started by.
   */
  readonly launcherEntry: string;

  /**
   How many times this run's name has been resumed after its turn ended.
   */
  readonly wakesSoFar: number;

  /**
   The environment the resumed launcher gets.
   */
  readonly environment: NodeJS.ProcessEnv;

  /**
   How one git question is asked.
   */
  readonly git: GitRunner;

  /**
   How one command is run, which a test replaces.
   */
  readonly run: RunCommand;

  /**
   Where a line about what was done goes.
   */
  readonly report: (line: string) => void;

  /**
   How a wait passes time.
   */
  readonly wait: (milliseconds: number) => Promise<void>;

  /**
   Where the step reads the time.
   */
  readonly now: () => number;
};

/**
 What finishing one run answered.
 */
export type FinishedRun = {
  /**
   The status the launcher exits with.
   */
  readonly status: number;
};

/**
 Reads the processes still working for one run, from the machine's process table.

 @param request - the run and the folder its process table is read from

 @returns one `pid command` entry per process still working for this run

 @example
 ```ts
 const working = await ownWorkStillRunning(request,);
 ```
 */
async function ownWorkStillRunning(request: FinishRequest,): Promise<readonly string[]> {
  return processesStillWorkingFor({
    runName: request.runName,
    worktree: request.worktree,
    runFolder: request.runFolder,
    processes: await processesAtTheirFolders({ processFolder: request.processFolder, },),
    excludingPid: process.pid,
  },);
}

/**
 Reads the note a resumed agent gets and starts the launcher that carries it.

 @param request - the run being finished

 @param stillWorking - the run's own work that still runs

 @returns what resuming the session answered

 @example
 ```ts
 const answer = await resumeTheSession({ request, stillWorking: [], },);
 ```
 */
async function resumeTheSession({
  request,
  stillWorking,
}: {
  readonly request: FinishRequest;
  readonly stillWorking: readonly string[];
},): Promise<WakeResumeAnswer> {
  try {
    /**
     The session file the finished run's summary names, which is what a resumed run continues.
     */
    const sessionFile = await sessionFileOfRun({
      runsFolder: request.runsFolder,
      runFolder: request.runFolder,
    },);
    /**
     Where the note the resumed agent reads is written.
     */
    const wakePrompt = join(
      request.runFolder,
      'wake-prompt.md',
    );
    await writeFile(
      wakePrompt,
      wakePromptOf({
        stillWorking,
        wallSeconds: request.wallSeconds,
      },),
      { encoding: 'utf8', },
    );
    /**
     What the resumed launch answered.
     */
    const resumed: ShownCommandEnding = await runCommandShowingItsOutput({
      command: process.execPath,
      words: [
        request.launcherEntry,
        ...wakeResumeWordsOf({
          commandLine: request.commandLine,
          wakePrompt,
          sessionFile,
        },),
      ],
      environment: {
        ...request.environment,
        PI_RUN_WAKES: String(request.wakesSoFar + 1,),
      },
    },);
    return {
      resumed: true,
      ...((resumed.status === undefined) ? {} : { status: resumed.status, }),
    };
  }
  catch (error) {
    // A run whose summary names no session cannot be resumed; its worktree is kept because its work may still be there.
    if (error instanceof SessionFileUnreadableError) {
      return {
        resumed: false,
        problem: error.message,
      };
    }
    throw error;
  }
}

/**
 Finishes one run: reads why it ended, wakes an agent whose own work outlived its turn, and decides what becomes of its worktree.

 A run that is resumed answers with the status the resumed launcher ended with; every other run answers with the status its unit
 ended with.

 @param request - the run, its worktree, how it ended, and the seams a test replaces

 @returns the status the launcher exits with

 @example
 ```ts
 const finished = await finishTheRun({ runName: 'a', runFolder: '/runs/a-1', unitStatus: 0, wallSeconds: 600, readOnlyInMain: false, worktree: '/repo/.claude/worktrees/pi-a', checkoutFolder: '/repo', processFolder: '/proc', runsFolder: '/runs', commandLine: [], launcherEntry: '/pkg/src/start-pi-run-in-a-worktree.ts', wakesSoFar: 0, environment: process.env, git, run, report: console.log, wait, now: Date.now, },);
 ```
 */
export async function finishTheRun(request: FinishRequest,): Promise<FinishedRun> {
  /**
   Why the run ended, or nothing when it wrote no summary.
   */
  const { endReason, } = await endReasonIn({ folder: request.runFolder, },);

  /**
   What the step after the unit ended found and did, which is nothing at all for a run that ended for any reason but finishing
   its turn.
   */
  const wake: WakeOutcome = ((endReason === 'idle') && (!request.readOnlyInMain))
    ? await wakeIfItsOwnWorkOutlivesItsTurn({
      runName: request.runName,
      wallSeconds: request.wallSeconds,
      wakesSoFar: request.wakesSoFar,
      readOwnWork: async function readOwnWork() {
        return await ownWorkStillRunning(request,);
      },
      report: request.report,
      wait: request.wait,
      now: request.now,
      resume: async function resume({ stillWorking, },): Promise<WakeResumeAnswer> {
        return await resumeTheSession({
          request,
          stillWorking,
        },);
      },
    },)
    : { step: 'nothing-to-wake', };

  if ((wake.step === 'could-not-resume') || (wake.step === 'work-still-runs')) {
    request.report(`${request.runName}: the agent was not resumed (${(wake.step === 'could-not-resume') ? wake.problem : 'its work still runs'})`,);
  }
  if (wake.step === 'resumed') {
    request.report(`${request.runName}: resumed run ended with status ${String(wake.status,)}`,);
    return { status: (wake.status === 0) ? 0 : (wake.status ?? 1), };
  }
  /**
   What the run's own work still runs, which keeps the worktree as it is while it does.
   */
  const ownWorkLeftRunning = ((wake.step === 'could-not-resume') || (wake.step === 'work-still-runs'))
    ? wake.ownWork
    : [];

  /**
   Whether the worktree's HEAD is a branch.
   */
  const onABranch = await onABranchNow({
    readOnlyInMain: request.readOnlyInMain,
    run: request.run,
    worktree: request.worktree,
  },);

  /**
   What is to be done with the worktree, which is nothing while the run's own work still runs in it: a removal would take the
   folder of a build or a reproduction the run left working, and a wake would have nothing to return to.
   */
  const outcome = (ownWorkLeftRunning.length > 0)
    ? {
      action: 'keep' as const,
      line: `worktree kept, the run's own work still runs in it (${ownWorkLeftRunning.slice(
        0,
        OWN_WORK_NAMED_IN_A_NOTE,
      )
        .join(', ',)})`,
    }
    : worktreeOutcomeOf({
      readOnlyInMain: request.readOnlyInMain,
      ...(endReason === undefined ? {} : { endReason, }),
      onABranch,
      ...await statusOf({
        readOnlyInMain: request.readOnlyInMain,
        ...(endReason === undefined ? {} : { endReason, }),
        onABranch,
        run: request.run,
        worktree: request.worktree,
      },),
    },);
  if (outcome.action === 'keep') {
    request.report(`${request.runName}: ${outcome.line}`,);
  }
  if (outcome.action === 'remove') {
    request.report(`${request.runName}: ${await removeWorktree({
      git: request.git,
      checkoutFolder: request.checkoutFolder,
      worktree: request.worktree,
    },)}`,);
  }
  return { status: request.unitStatus, };
}

/**
 The part of a start that runs after the worktree and the run folder exist and before the run's unit is asked for: the folders a unit
 may write, and the branch's own dependency install.

 Why it is one function: everything here can still refuse, and a refusal leaves a worktree and a run folder behind. Each refusal is
 recorded in the run's own folder first, so a person who finds the leftovers and the next start of the same name can read what
 happened rather than guess.

 @module
 */

import {
  type FileSystemOfARun,
  fileSystemOfARun,
} from './file-system-of-a-run.ts';
import { linkDependenciesOrRefuse, } from './dependencies-linked-in-a-unit.ts';
import type { GitRunner, } from './worktree-of-a-run.ts';
import type { RunCommand, } from './run-command.ts';
import { recordARefusedStart, } from './what-a-refused-start-kept.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 What the part of a start after the worktree needs.

 @example
 ```ts
 const fileSystem = await workUntilTheUnitIsAsked({ runName: 'a', checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', branch: 'fix/a', readOnlyInMain: false, git, cpuPercent: 100, memory: '1G', taskCount: 100, sessionPath: '/bin', home: '/home/a', run: runCommandWithChildProcess, report: console.log, });
 ```
 */
export type StartAfterTheWorktree = {
  /**
   The run's name.
   */
  readonly runName: string;

  /**
   The checkout the run's launcher works in.
   */
  readonly checkoutFolder: string;

  /**
   Where the run works.
   */
  readonly worktree: string;

  /**
   The run's own folder, which the start made.
   */
  readonly runFolder: string;

  /**
   The branch the run's work goes on.
   */
  readonly branch: string;

  /**
   Whether the run works in the checkout itself.
   */
  readonly readOnlyInMain: boolean;

  /**
   How one git question is run.
   */
  readonly git: GitRunner;

  /**
   The processor share the units get, in percent.
   */
  readonly cpuPercent: number;

  /**
   The memory the units may hold.
   */
  readonly memory: string;

  /**
   The number of tasks the units may hold.
   */
  readonly taskCount: number;

  /**
   The `PATH` the session's own processes get.
   */
  readonly sessionPath: string;

  /**
   The home folder the units run with.
   */
  readonly home: string;

  /**
   How one command is run.
   */
  readonly run: RunCommand;

  /**
   Where a line about the install is printed.
   */
  readonly report: (line: string) => void;
};

/**
 Reads the folders the run's unit may write, links the worktree's dependencies inside a unit of the run's own, and records any
 refusal in the run's own folder before throwing it.

 @param request - the run, its folders, the caps and the environment

 @returns the folders the run's unit may and may not write

 @throws whatever the step refused with, after the refusal is recorded in the run's own folder

 @example
 ```ts
 const fileSystem = await workUntilTheUnitIsAsked({ runName: 'a', checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', branch: 'fix/a', readOnlyInMain: false, git, cpuPercent: 100, memory: '1G', taskCount: 100, sessionPath: '/bin', home: '/home/a', run: runCommandWithChildProcess, report: console.log, });
 ```
 */
export async function workUntilTheUnitIsAsked(request: StartAfterTheWorktree,): Promise<FileSystemOfARun> {
  try {
    /**
     What the run's unit may and may not write, read once the worktree Git made is there.
     */
    const fileSystem = await fileSystemOfARun({
      git: request.git,
      checkoutFolder: request.checkoutFolder,
      worktree: request.worktree,
      runFolder: request.runFolder,
      worksInTheCheckoutItself: request.readOnlyInMain,
    },);
    if (!request.readOnlyInMain) {
      // The branch's own install scripts run here, so they run inside a unit with this same read-only checkout: a `postinstall`
      // that writes the checkout is refused by the operating system rather than done with the launcher's access. An install that
      // failed refuses the start rather than spending a run on a half-installed tree.
      /**
       What linking the worktree's dependencies answered.
       */
      const linked = await linkDependenciesOrRefuse({
        runName: request.runName,
        checkoutFolder: request.checkoutFolder,
        worktree: request.worktree,
        runFolder: request.runFolder,
        fileSystem,
        cpuPercent: request.cpuPercent,
        memory: request.memory,
        taskCount: request.taskCount,
        pathOfMiseEnvironment: request.sessionPath,
        home: request.home,
        run: request.run,
      },);
      request.report(`dependencies: status ${String(linked.status,)}`,);
    }
    return fileSystem;
  }
  catch (error) {
    await recordARefusedStart({
      runFolder: request.runFolder,
      worktree: request.worktree,
      branch: request.branch,
      // A refusal this start records in its own words, whatever was thrown.
      reason: ((Error.isError(error,)) ? error.message : caughtValueText(error,)),
    },);
    throw error;
  }
}

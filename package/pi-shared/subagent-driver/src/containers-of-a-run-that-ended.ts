/**
 What a run left of its containers, removed once the run's unit has ended.

 Why the launcher and not the unit: a container's `conmon` is a child of the manager's pause process, not of the unit that started it,
 and a container's own process runs outside the unit's cgroup. Measured on this machine: a container started for a run is still
 running after the run's unit is stopped, and after it is killed, and `podman rm --force` from inside the unit's dying cgroup cannot
 stop it. The container the run started carries a label naming the unit, and a process outside that unit, with the account's own
 podman, removes it.

 @module
 */

import type { RunCommand, } from './run-command.ts';
import {
  launcherEndedLineOf,
  unitStateNow,
} from './the-line-a-launcher-ends-with.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 The label every container started through a run's own `podman` carries, whose value is the run's unit.
 */
export const CONTAINER_LABEL_OF_A_RUN: string = 'pi-container-of';

/**
 How many characters of podman's own answer a line about a failed removal carries, which is enough to say why.
 */
const ANSWER_CHARACTERS = 300;

/**
 Two minutes in seconds, which sizes the removal's bound.
 */
/**
 Seconds in a minute, which sizes the removal's bound.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Two minutes in seconds, the bound a removal gets.
 */
const TWO_MINUTES_IN_SECONDS = 2 * SECONDS_PER_MINUTE;

/**
 How long removing what a run left may take, in seconds.

 A bound rather than a measurement: removing a stopped container is a rename and a directory removal, and one that has not answered in
 two minutes is not answering.
 */
export const CONTAINER_CLEANUP_SECONDS: number = TWO_MINUTES_IN_SECONDS;

/**
 Removes every container a run's unit started, naming what it removed.

 @param unitName - the run's unit, which is what its containers are labelled with

 @param run - how one command is run

 @param report - where a line about what was removed is printed

 @returns the number of containers removed, which is zero when the run started none or podman is not there

 @example
 ```ts
 const removed = await removeTheContainersOfARun({ unitName: 'pi-copy-route.service', run: runCommandWithChildProcess, report: console.log, });
 ```
 */
export async function removeTheContainersOfARun({
  unitName,
  run,
  report,
}: {
  /**
   The unit the run ran in, which is what its containers are labelled with.
   */
  readonly unitName: string;

  /**
   How one command is run.
   */
  readonly run: RunCommand;

  /**
   Where a line about what was removed is printed.
   */
  readonly report: (line: string) => void;
},): Promise<number> {
  /**
   The containers the label names, by id.
   */
  const listed = await run({
    command: 'podman',
    words: [
      'ps',
      '--all',
      '--quiet',
      '--filter',
      `label=${CONTAINER_LABEL_OF_A_RUN}=${unitName}`,
    ],
    timeoutMilliseconds: CONTAINER_CLEANUP_SECONDS * MILLISECONDS_PER_SECOND,
  },);
  if (listed.status !== 0) {
    // A machine without podman, or with podman in another storage, has nothing of this run's to remove, and saying so on every run
    // would be noise. What a run started is named by the label, so a reader can find it by hand.
    return 0;
  }
  /**
   One id per container the run left.
   */
  const containers = listed.stdout
    .split('\n',)
    .map(function withoutSpaces(line,) {
      return line.trim();
    },)
    .filter(function holdsSomething(line,) {
      return line !== '';
    },);
  if (containers.length === 0) {
    return 0;
  }
  /**
   What removing them answered.
   */
  const removed = await run({
    command: 'podman',
    words: [
      'rm',
      '--force',
      ...containers,
    ],
    timeoutMilliseconds: CONTAINER_CLEANUP_SECONDS * MILLISECONDS_PER_SECOND,
  },);
  if (removed.status === 0) {
    report(`containers: removed ${String(containers.length,)} a run this left, named by the label ${CONTAINER_LABEL_OF_A_RUN}`,);
    return containers.length;
  }
  report(`containers: ${String(containers.length,)} of this run's are still there, and podman answered ${String(removed.status,)}: ${`${removed.stdout}${removed.stderr}`.trim()
    .slice(
      0,
      ANSWER_CHARACTERS,
    )}`,);
  return 0;
}

/**
 Says what a run left of its containers and what its unit is doing, which is the last pair of lines a launcher prints.

 @param request - the run, its unit and folder, where it worked, what the wait answered, and where a line is printed

 @example
 ```ts
 await sayWhatTheEndLeaves({ runName: 'a', unitName: 'pi-a.service', runFolder: '/runs/a-1', worktree: '/repo/.claude/worktrees/pi-a', ended: 'the-unit-ended', status: 0, run: runCommandWithChildProcess, report: console.log, });
 ```
 */
export async function sayWhatTheEndLeaves(request: {
  /**
   The run's name.
   */
  readonly runName: string;

  /**
   The unit the run ran in.
   */
  readonly unitName: string;

  /**
   The run's own folder.
   */
  readonly runFolder: string;

  /**
   Where the run worked.
   */
  readonly worktree: string;

  /**
   What ended the wait.
   */
  readonly ended: 'the-unit-ended' | 'the-wait-gave-up';

  /**
   The status the unit ended with.
   */
  readonly status: number;

  /**
   How one command is run.
   */
  readonly run: RunCommand;

  /**
   Where a line is printed.
   */
  readonly report: (line: string) => void;
},): Promise<void> {
  await removeTheContainersOfARun({
    unitName: request.unitName,
    run: request.run,
    report: request.report,
  },);
  request.report(launcherEndedLineOf({
    runName: request.runName,
    unitName: request.unitName,
    unitState: await unitStateNow({
      run: request.run,
      unitName: request.unitName,
    },),
    ended: request.ended,
    status: request.status,
    runFolder: request.runFolder,
    worktree: request.worktree,
  },),);
}

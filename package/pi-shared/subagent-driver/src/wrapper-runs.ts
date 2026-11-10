/**
 Which runs serve the Git wrapper and which do not, and the stop file that asks the others to stop.

 The owner, 2026-11-10: "How many times do I need to tell you to focus on the wrapper this session?" A run is the wrapper's when
 its branch names it, and a list of names holds the wrapper-adjacent work the owner ordered although the branch does not say so.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { STOP_FILE_NAME, } from './driver-constants.ts';
import type { NO_RUN_FOLDER, } from './run-folder-of-a-unit.ts';

/**
 The part of a branch name that makes a run the wrapper's work.
 */
export const WRAPPER_BRANCH_PART = 'git-policy';

/**
 The run names that serve the wrapper although their branch does not say so: wrapper-adjacent work the owner ordered.
 */
export const RUNS_KEPT_DESPITE_THEIR_NAMES: readonly string[] = [
  'node-modules-subvolume',
  'zfs-and-btrfs-delete-stalls',
  'lfs-',
  'kimi-',
  'sol-',
  'worktree-',
  'commit-',
  'wrapper-',
  'tokio-',
];

/**
 What the stop file says, in the owner's own reason for asking.
 */
export const STOP_MESSAGE = 'The owner said to focus on the Git wrapper; this run is not wrapper work.\n';

/**
 One run going now, with what it works on.
 */
export type RunToSort = {
  /**
   The run's name, which is its unit's name without the prefix and suffix.
   */
  readonly name: string;

  /**
   The branch its worktree has checked out, or empty text when it has no worktree.
   */
  readonly branch: string;

  /**
   The run folder of the attempt going now, or {@link NO_RUN_FOLDER}.
   */
  readonly runFolder: string | typeof NO_RUN_FOLDER;
};

/**
 Reads whether a run serves the Git wrapper.

 @param name - the run's name

 @param branch - the branch its worktree has checked out

 @returns whether the run is the wrapper's work

 @example
 ```ts
 const kept = isWrapperWork({ name: 'a', branch: 'fix/git-policy-cli-btrfs', });
 ```
 */
export function isWrapperWork({
  name,
  branch,
}: {
  readonly name: string;
  readonly branch: string;
},): boolean {
  return branch.includes(WRAPPER_BRANCH_PART,)
    || RUNS_KEPT_DESPITE_THEIR_NAMES.some(function isOneOfTheNames(kept,) {
      return name.startsWith(kept,);
    },);
}

/**
 Describes one run on one line.

 @param run - the run to describe

 @returns the line, which names the run, its branch and its run folder

 @example
 ```ts
 const line = lineOf({ run: { name: 'a', branch: 'b', runFolder: NO_RUN_FOLDER, }, });
 ```
 */
export function lineOf({
  run,
}: {
  readonly run: RunToSort;
},): string {
  /**
   The branch, or the words that stand for having none.
   */
  const branch = (run.branch === '') ? 'no branch' : run.branch;
  /**
   The run folder's name, or the mark that stands for having none.
   */
  const run_ = ((typeof run.runFolder) === 'symbol') ? '?' : run.runFolder;
  return `${run.name}  [${branch}]  run=${run_}`;
}

/**
 Sorts the runs going now into the wrapper's work and other work.

 @param runs - every run going now

 @returns the lines for the work kept, and the runs that would be asked to stop

 @example
 ```ts
 const sorted = sortRunsForTheWrapper({ runs: [], });
 ```
 */
export function sortRunsForTheWrapper({
  runs,
}: {
  readonly runs: readonly RunToSort[];
},): {
  readonly kept: readonly string[];
  readonly other: readonly RunToSort[];
} {
  /**
   The lines of the runs kept.
   */
  const kept: string[] = [];
  /**
   The runs that would be asked to stop.
   */
  const other: RunToSort[] = [];
  for (const run of runs) {
    if (isWrapperWork({
      name: run.name,
      branch: run.branch,
    },)) {
      kept.push(lineOf({ run, },),);
      continue;
    }
    other.push(run,);
  }
  return {
    kept,
    other,
  };
}

/**
 Writes the stop file into one run's folder, which ends that run early.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param runFolderName - the name of the attempt going now

 @returns the file the run watches

 @throws the error the write gave, such as a run folder that is not there

 @example
 ```ts
 const file = await writeStopFile({ runsFolder: '/runs', runFolderName: 'a-1', });
 ```
 */
export async function writeStopFile({
  runsFolder,
  runFolderName,
}: {
  readonly runsFolder: string;
  readonly runFolderName: string;
},): Promise<string> {
  /**
   The file the driver looks for on every tick.
   */
  const file = join(
    runsFolder,
    runFolderName,
    STOP_FILE_NAME,
  );
  await writeFile(
    file,
    STOP_MESSAGE,
    { encoding: 'utf8', },
  );
  return file;
}

/**
 Asks every run that is not the wrapper's work to stop, when the caller asked for that.

 @param runs - every run going now

 @param apply - whether to write the stop files, rather than only name the runs that would be asked

 @returns the lines to print: the runs kept, then the runs asked to stop

 @throws the error a write gave

 @example
 ```ts
 const lines = await stopRunsNotOnTheWrapper({ runs: [], runsFolder: '/runs', apply: false, });
 ```
 */
export async function stopRunsNotOnTheWrapper({
  runs,
  runsFolder,
  apply,
}: {
  readonly runs: readonly RunToSort[];
  readonly runsFolder: string;
  readonly apply: boolean;
},): Promise<readonly string[]> {
  /**
   The runs sorted into the wrapper's work and other work.
   */
  const sorted = sortRunsForTheWrapper({ runs, },);
  /**
   The other runs that have a folder to be asked in.
   */
  const askable = sorted.other
    .filter(function hasAFolder(run,): boolean {
      return (typeof run.runFolder) !== 'symbol';
    },);
  if (apply) {
    await Promise.all(askable.map(async function askToStop(run,) {
      if ((typeof run.runFolder) === 'symbol') {
        return;
      }
      await writeStopFile({
        runsFolder,
        runFolderName: run.runFolder,
      },);
    },),);
  }
  return [
    `wrapper work kept (${String(sorted.kept
      .length,)}):`,
    ...sorted.kept
      .map(function indented(line,) {
        return `  ${line}`;
      },),
    `other work ${apply ? `asked to stop (${String(askable.length,)} of ${String(sorted.other
      .length,)})` : 'that would be asked to stop'} (${String(sorted.other
        .length,)}):`,
    ...sorted.other
      .map(function indented(line,) {
        return `  ${lineOf({ run: line, },)}`;
      },),
  ];
}

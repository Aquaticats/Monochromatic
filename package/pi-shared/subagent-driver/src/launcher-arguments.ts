/**
 The words the pool gate hands the worktree launcher: the caller's own, with the pool a run was given written into them and a
 resumed run's folder turned into its session file.

 @module
 */

import {
  MODEL_OPTION,
  PROVIDER_OPTION,
  RUNS_FOLDER_OPTION,
  SESSION_OF_OPTION,
  WALL_OPTION,
} from './parse-pool-gate-options.ts';
import type { ProviderPool, } from './pool-limits.ts';

/**
 The option that names the session file a run resumes.
 */
const SESSION_OPTION = '--session=';

/**
 What the launcher is given, and what a reader should be told about it.
 */
export type LauncherArguments = {
  /**
   The launcher's words, in the order they were written.
   */
  readonly words: readonly string[];

  /**
   The options the gate appended, in words, or empty text when it appended nothing.
   */
  readonly addedOptions: string;
};

/**
 Says how long a run on a pool that answers more slowly may last.

 @param wallSeconds - the wall clock the caller asked for

 @param wallFactor - how many times longer a run on the pool may last

 @returns the wall clock the launcher is given, in seconds

 @example
 ```ts
 const doubled = wallSecondsOn({ wallSeconds: 1_800, wallFactor: 2, },);
 ```
 */
export function wallSecondsOn({
  wallSeconds,
  wallFactor,
}: {
  readonly wallSeconds: number;
  readonly wallFactor: number;
},): number {
  return wallSeconds * wallFactor;
}

/**
 Says what the launcher is given: the caller's words, with the pool the run took written into the ones that name it, and the
 options the caller left out appended.

 A run that took the first pool with room gets that pool's provider and model, and its wall clock multiplied by how much slower
 that pool answers. A run that resumes a session, or that names a model only one pool takes, keeps what its caller wrote and no
 wall factor is applied: it stays on the pool it named.

 @param commandLine - the words the caller wrote

 @param pool - the pool the run goes on, when it goes on one

 @param movesTheRun - whether the gate chose the pool rather than the caller

 @param wallSeconds - the wall clock the caller asked for, when it asked for one

 @param sessionFile - the session file of the run this one resumes, when it resumes one

 @param runsFolder - the folder the runs live in

 @returns the launcher's words, and the options that were appended

 @example
 ```ts
 const words = launcherArgumentsOf({ commandLine: ['--name=a',], pool: NEURALWATT_DEEPSEEK_FLEX, movesTheRun: true, wallSeconds: 600, sessionFile: '', runsFolder: '/runs', },);
 ```
 */
export function launcherArgumentsOf({
  commandLine,
  pool,
  movesTheRun,
  wallSeconds,
  sessionFile,
  runsFolder,
}: {
  readonly commandLine: readonly string[];
  readonly pool?: ProviderPool;
  readonly movesTheRun: boolean;
  readonly wallSeconds?: number;
  readonly sessionFile?: string;
  readonly runsFolder: string;
}): LauncherArguments {
  /**
   The words written so far.
   */
  const words = commandLine.map(function rewritten(word,) {
    if (word.startsWith(SESSION_OF_OPTION,)) {
      return `${SESSION_OPTION}${sessionFile ?? ''}`;
    }
    if ((!movesTheRun) || (pool === undefined)) {
      return word;
    }
    if (word.startsWith(PROVIDER_OPTION,)) {
      return `${PROVIDER_OPTION}${pool.provider}`;
    }
    if (word.startsWith(MODEL_OPTION,)) {
      return `${MODEL_OPTION}${pool.model}`;
    }
    return (word.startsWith(WALL_OPTION,) && (wallSeconds !== undefined))
      ? `${WALL_OPTION}${String(wallSecondsOn({
        wallSeconds,
        wallFactor: pool.wallFactor,
      },),)}`
      : word;
  },);
  /**
   The options the caller left out, which the run cannot be started without.
   */
  const added: string[] = [];
  if (movesTheRun && (pool !== undefined)) {
    if (!commandLine.some(function namesProvider(word,) {
      return word.startsWith(PROVIDER_OPTION,);
    },)) {
      added.push(`${PROVIDER_OPTION}${pool.provider}`,);
    }
    if (!commandLine.some(function namesModel(word,) {
      return word.startsWith(MODEL_OPTION,);
    },)) {
      added.push(`${MODEL_OPTION}${pool.model}`,);
    }
  }
  if (!commandLine.some(function namesRunsFolder(word,) {
    return word.startsWith(RUNS_FOLDER_OPTION,);
  },)) {
    added.push(`${RUNS_FOLDER_OPTION}${runsFolder}`,);
  }
  return {
    words: [
      ...words,
      ...added,
    ],
    addedOptions: added.join(' ',),
  };
}

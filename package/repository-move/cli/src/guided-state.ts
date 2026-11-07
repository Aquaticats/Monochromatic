/**
 The runner's state folder: progress records, the cut-over window's clock, timers and the log.

 @module
 */

import {
  appendFile,
  mkdir,
  readdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import { MILLISECONDS_PER_SECOND, } from './constants.ts';
import type {
  GuidedConfig,
  GuidedFlags,
  Stage,
} from './guided-types.ts';

/**
 Path of a file in the runner's state folder.

 @param config - the configuration
 
 @param name - the file name
 
 @returns the path
 */
export function wizFile({
  config,
  name,
}: {
  readonly config: GuidedConfig;
  readonly name: string
},): string {
  return join(
    config.wiz,
    name,
  );
}

/**
 Reads a number from a state file.

 @param path - the file
 
 @returns the number, or zero when the file is absent or empty
 */
export async function readNumber(path: string,): Promise<number> {
  if (!await pathExists(path,)) {
    return 0;
  }
  /**
   The text this step works on.
   */
  const text = (await readFile(
    path,
    'utf8',
  )).trim();
  return text === '' ? 0 : Number(text,);
}

/**
 Seconds since the epoch, whole.

 @returns the seconds
 */
export function nowSeconds(): number {
  return Math.floor(Date.now() / MILLISECONDS_PER_SECOND,);
}

/**
 Tells whether the cut-over window is running: started, not ended, with a stated maximum.

 @param config - the configuration
 
 @returns true while the window runs
 */
export async function windowRunning(config: GuidedConfig,): Promise<boolean> {
  return (await readNumber(wizFile({
    config,
    name: 'window-start',
  },),) > 0)
    && (!await pathExists(wizFile({
      config,
      name: 'window-end',
    },),))
    && (await readNumber(wizFile({
      config,
      name: 'window-max-seconds',
    },),) > 0);
}

/**
 Seconds left in the window; negative once it has passed.

 @param config - the configuration
 
 @returns the seconds
 */
export async function windowRemaining(config: GuidedConfig,): Promise<number> {
  /**
   The start of this step.
   */
  const start = await readNumber(wizFile({
    config,
    name: 'window-start',
  },),);
  /**
   The max of this step.
   */
  const max = await readNumber(wizFile({
    config,
    name: 'window-max-seconds',
  },),);
  return max - (nowSeconds() - start);
}

/**
 Appends a line to the runner's log, unless this is a dry run.

 @param config - the configuration
 
 @param flags - the flags
 
 @param line - the text
 */
export async function logLine(
  {
    config,
    flags,
    line,
  }: {
    readonly config: GuidedConfig;
    readonly flags: GuidedFlags;
    readonly line: string;
  },
): Promise<void> {
  if (!flags.dryRun) {
    await mkdir(
      join(
        config.wiz,
        'logs',
      ),
      { recursive: true, },
    );
    await appendFile(
      wizFile({
        config,
        name: 'logs/wizard.log',
      },),
      `${new Date().toISOString()} ${line}\n`,
    );
  }
}

/**
 Lists what a phase has completed.

 @param config - the configuration
 
 @param phase - the phase
 
 @returns one line per completed stage
 */
export async function listProgress(
  {
    config,
    phase,
  }: {
    readonly config: GuidedConfig;
    readonly phase: string;
  },
): Promise<readonly string[]> {
  /**
   The folder this step works on.
   */
  const folder = join(
    config.wiz,
    'done',
    phase,
  );
  if (!await pathExists(folder,)) {
    return [];
  }
  /**
   The external worktree folder names.
   */
  const names = (await readdir(folder,)).toSorted();
  return await Promise.all(names.map(async function describe(name,) {
    return `${name}  at ${(await readFile(
      join(
        folder,
        name,
      ),
      'utf8',
    )).trim()}`;
  },),);
}

/**
 Adds seconds to, or sets, a timer file.

 @param config - the configuration
 
 @param timer - the timer
 
 @param seconds - the stage's duration
 */
export async function recordTimer(
  {
    config,
    timer,
    seconds,
  }: {
    readonly config: GuidedConfig;
    readonly timer: NonNullable<Stage['timer']>;
    readonly seconds: number;
  },
): Promise<void> {
  /**
   The path this step works on.
   */
  const path = wizFile({
    config,
    name: timer.file,
  },);
  /**
   The prior of this step.
   */
  const prior = timer.mode === 'add' ? await readNumber(path,) : 0;
  await writeFile(
    path,
    `${String(prior + seconds,)}\n`,
  );
}

/**
 Renames the cut-over progress and the window clock aside, so a new attempt starts clean. Nothing is deleted.

 @param config - the configuration
 
 @param suffix - the suffix, a timestamp
 */
export async function renameCutoverStateAside(
  {
    config,
    suffix,
  }: {
    readonly config: GuidedConfig;
    readonly suffix: string;
  },
): Promise<void> {
  for (const name of [
    'done/cutover',
    'window-start',
    'swap-timestamp',
  ]) {
    /**
     The path this step works on.
     */
    const path = wizFile({
      config,
      name,
    },);
    if (await pathExists(path,)) {
      await rename(
        path,
        `${path}.aborted-${suffix}`,
      );
    }
  }
}

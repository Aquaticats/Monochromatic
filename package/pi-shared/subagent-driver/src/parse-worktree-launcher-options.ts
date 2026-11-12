/**
 Reads the command line of the worktree launcher: what makes a run, where it works, and the caps its unit gets.

 @module
 */

import { homedir, } from 'node:os';
import { parseArgs, } from 'node:util';
import { join, } from 'node:path';
import {
  MODEL_OF_A_RUN_NAMING_NONE,
  PROVIDER_OF_A_RUN_NAMING_NONE,
} from './pool-limits.ts';
import {
  type RequiredOption,
  requiredPositiveNumber,
  requiredSeconds,
  requiredText,
} from './required-option-values.ts';

import {
  BASE_OF_A_RUN_NAMING_NONE,
} from './worktree-of-a-run.ts';

/**
 The launcher's command line, for the error message.
 */
export const WORKTREE_LAUNCHER_USAGE: string =
  'start-pi-run-in-a-worktree --name=<run name> --branch=<branch> --prompt=<file> --cpu=<percent> --memory=<size> --tasks=<count> --wall=<seconds> '
    + '[--base=<commit or branch>] [--run=<number>] [--session=<file>] [--provider=<name>] [--model=<name>] [--turn-cap=<n>] '
    + '[--idle-seconds=<n>] [--runs-folder=<folder>] [--checkout-folder=<folder>]';

/**
 The branch name that tells the launcher to run in the checkout itself, without making a worktree.
 */
export const NO_WORKTREE_BRANCH = 'none';

/**
 The folder every worktree of this repository is made under.
 */
const WORKTREE_FOLDER_IN_CHECKOUT: readonly string[] = [
  '.claude',
  'worktrees',
];

/**
 The prefix every worktree of a Pi run carries, so the folder says which run works there.
 */
export const WORKTREE_PREFIX = 'pi-';

/**
 Seconds the driver stays up for a steering message after Pi comes to rest, which is what a coordinator needs to steer a run that
 has settled before it is read. The scratch launcher wrote this value for every run it started.
 */
export const IDLE_SECONDS_OF_A_RUN = 120;

/**
 The parts of the coordinator's runs folder below the home directory of whoever runs this.
 */
const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 A command line the launcher cannot use.
 */
export class WorktreeLauncherOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WorktreeLauncherOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw worktreeLauncherOptionsErrorOf('--name is required.',);
 ```
 */
function worktreeLauncherOptionsErrorOf(message: string,): Error {
  return new WorktreeLauncherOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({ value: values.name, name: 'name', },);
 ```
 */
function optionOf({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string;
},): RequiredOption {
  return {
    value,
    name,
    usage: WORKTREE_LAUNCHER_USAGE,
    errorOf: worktreeLauncherOptionsErrorOf,
  };
}

/**
 What one run was asked for.
 */
export type WorktreeLauncherOptions = {
  /**
   The run's name, which names its unit, its branch's worktree and its run folder.
   */
  readonly name: string;

  /**
   The branch the run's work goes on, or {@link NO_WORKTREE_BRANCH} to work in the checkout itself.
   */
  readonly branch: string;

  /**
   The commit or branch a branch this run creates starts at, which an existing branch of that name keeps its own commits over.
   */
  readonly base: string;

  /**
   The first prompt.
   */
  readonly promptFile: string;

  /**
   The processor share the run's unit gets, in percent.
   */
  readonly cpuPercent: number;

  /**
   The memory the run's unit may hold, as systemd writes a size.
   */
  readonly memory: string;

  /**
   The number of tasks the run's unit may hold.
   */
  readonly taskCount: number;

  /**
   How long the run may last, in seconds.
   */
  readonly wallSeconds: number;

  /**
   The number of the run's folder, or nothing for the next number after every folder of that name.
   */
  readonly runNumber?: number;

  /**
   A session file to resume.
   */
  readonly session?: string;

  /**
   The provider the run asks for.
   */
  readonly provider: string;

  /**
   The model the run asks for.
   */
  readonly model: string;

  /**
   How many model turns the run may start, or nothing to size the cap from finished runs.
   */
  readonly turnCap?: number;

  /**
   Seconds the driver stays up for a steering message after Pi comes to rest.
   */
  readonly idleSeconds: number;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   The checkout the run's worktree is made in, and whose `main` its branch starts from.
   */
  readonly checkoutFolder: string;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @param checkoutFolder - the checkout the command was run from, which is where a worktree is made unless the caller names another

 @returns what the run was asked for

 @throws WorktreeLauncherOptionsError when an option is unknown, missing or unusable

 @example
 ```ts
 const options = parseWorktreeLauncherOptions({ commandLine: ['--name=a', '--branch=b', '--prompt=p', '--cpu=800', '--memory=12G', '--tasks=3000', '--wall=7200',], checkoutFolder: '/repo', },);
 ```
 */
export function parseWorktreeLauncherOptions({
  commandLine,
  checkoutFolder,
}: {
  readonly commandLine: readonly string[];
  readonly checkoutFolder: string;
},): WorktreeLauncherOptions {
  /**
   The options as written.
   */
  const parsed = writtenValuesIn(commandLine,);
  return {
    name: requiredText(optionOf({
      value: parsed.name,
      name: 'name',
    },),),
    branch: requiredText(optionOf({
      value: parsed.branch,
      name: 'branch',
    },),),
    base: parsed.base ?? BASE_OF_A_RUN_NAMING_NONE,
    promptFile: requiredText(optionOf({
      value: parsed.prompt,
      name: 'prompt',
    },),),
    cpuPercent: requiredPositiveNumber(optionOf({
      value: parsed.cpu,
      name: 'cpu',
    },),),
    memory: requiredText(optionOf({
      value: parsed.memory,
      name: 'memory',
    },),),
    taskCount: requiredPositiveNumber(optionOf({
      value: parsed.tasks,
      name: 'tasks',
    },),),
    wallSeconds: requiredSeconds(optionOf({
      value: parsed.wall,
      name: 'wall',
    },),),
    ...(parsed.run === undefined ? {} : { runNumber: requiredPositiveNumber(optionOf({
      value: parsed.run,
      name: 'run',
    },),), }),
    ...(parsed.session === undefined ? {} : { session: parsed.session, }),
    provider: parsed.provider ?? PROVIDER_OF_A_RUN_NAMING_NONE,
    model: parsed.model ?? MODEL_OF_A_RUN_NAMING_NONE,
    ...(parsed['turn-cap'] === undefined ? {} : { turnCap: requiredPositiveNumber(optionOf({
      value: parsed['turn-cap'],
      name: 'turn-cap',
    },),), }),
    idleSeconds: (parsed['idle-seconds'] === undefined)
      ? IDLE_SECONDS_OF_A_RUN
      : requiredSeconds(optionOf({
        value: parsed['idle-seconds'],
        name: 'idle-seconds',
      },),),
    runsFolder: parsed['runs-folder'] ?? join(
      homedir(),
      ...DEFAULT_RUNS_FOLDER_IN_HOME,
    ),
    checkoutFolder: parsed['checkout-folder'] ?? checkoutFolder,
  };
}

/**
 The folder a run's worktree is made in.

 @param checkoutFolder - the checkout the worktree is made in

 @param runName - the run's name

 @returns the worktree's path

 @example
 ```ts
 const folder = worktreeFolderOf({ checkoutFolder: '/repo', runName: 'copy-route', },);
 ```
 */
export function worktreeFolderOf({
  checkoutFolder,
  runName,
}: {
  readonly checkoutFolder: string;
  readonly runName: string;
}): string {
  return join(
    checkoutFolder,
    ...WORKTREE_FOLDER_IN_CHECKOUT,
    `${WORKTREE_PREFIX}${runName}`,
  );
}

/**
 What the command line parser read, by option name, before any value is checked.
 */
type WrittenValues = {
  readonly name?: string;
  readonly branch?: string;
  readonly base?: string;
  readonly prompt?: string;
  readonly cpu?: string;
  readonly memory?: string;
  readonly tasks?: string;
  readonly wall?: string;
  readonly run?: string;
  readonly session?: string;
  readonly provider?: string;
  readonly model?: string;
  readonly 'turn-cap'?: string;
  readonly 'idle-seconds'?: string;
  readonly 'runs-folder'?: string;
  readonly 'checkout-folder'?: string;
};

/**
 Reads the options as they were written, before any of them is checked.

 @param commandLine - arguments after the program name

 @returns the values the parser read, by their option names

 @throws WorktreeLauncherOptionsError when an option is unknown, or a value cannot be parsed

 @example
 ```ts
 const values = writtenValuesIn(['--name=a',],);
 ```
 */
function writtenValuesIn(commandLine: readonly string[],): WrittenValues {
  try {
    /**
     What Node's own parser read.
     */
    const { values, } = parseArgs({
      args: [...commandLine,],
      strict: true,
      allowPositionals: false,
      options: {
        name: { type: 'string', },
        branch: { type: 'string', },
        base: { type: 'string', },
        prompt: { type: 'string', },
        cpu: { type: 'string', },
        memory: { type: 'string', },
        tasks: { type: 'string', },
        wall: { type: 'string', },
        run: { type: 'string', },
        session: { type: 'string', },
        provider: { type: 'string', },
        model: { type: 'string', },
        'turn-cap': { type: 'string', },
        'idle-seconds': { type: 'string', },
        'runs-folder': { type: 'string', },
        'checkout-folder': { type: 'string', },
      },
    },);
    return values;
  }
  catch (error) {
    throw new WorktreeLauncherOptionsError(
      `${String(error,)}\nUsage: ${WORKTREE_LAUNCHER_USAGE}`,
      { cause: error, },
    );
  }
}

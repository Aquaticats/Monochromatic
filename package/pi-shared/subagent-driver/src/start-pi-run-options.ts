/**
 The command line of `start-pi-run-in-a-free-slot`: what run to start, where, on what, and how much of the machine it may use.

 Every value a caller usually leaves alone has a named default here, so the command is one line at a prompt and every hunch
 behind a default is written down beside it.

 @module
 */

import { parseArgs, } from 'node:util';
import {
  type RequiredOption,
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import {
  defaultRepository,
  defaultRunsFolder,
  driverEntryOf,
  promptFileOf,
} from './pi-runs-folders.ts';
import { PI_POOLS, } from './pools-of-pi-runs.ts';

/**
 The command's command line, for the error message.
 */
export const START_USAGE =
  'start-pi-run-in-a-free-slot --name=<name> --branch=<branch> [--prompt=<file>] [--run=<n>] [--provider=<provider>] [--model=<model>] [--cpu=<percent>] [--memory=<size>] [--tasks=<n>] [--wall=<seconds>] [--idle-seconds=<n>] [--turn-cap=<n>] [--session=<file>|--session-of=<run folder>] [--runs-folder=<folder>] [--repository=<folder>] [--driver-entry=<file>]';

/**
 The branch name that means the run works in the repository's own checkout instead of a worktree.
 */
export const NO_WORKTREE_BRANCH = 'none';

/**
 The number of the attempt a start performs when the caller names none.
 */
const DEFAULT_RUN = 1;

/**
 The standard pool, whose provider and model a run that names neither is started on.
 */
const [DEFAULT_POOL,] = PI_POOLS;

/**
 What a coding run is given of the machine when the caller names nothing: a ceiling sized to a real job, not to what the
 command measured. Rules `FHU` and `RXI` of `AGENTS.md` size a real job near what the host has, and these are the values the
 coordinator has started dozens of runs a day with.
 */
const DEFAULT_CPU_PERCENT = 800;

/**
 What a coding run is given of the machine's memory when the caller names nothing.
 */
const DEFAULT_MEMORY = '12G';

/**
 How many tasks a coding run may start when the caller names nothing.
 */
const DEFAULT_TASKS = 3_000;

/**
 How long a coding run may last when the caller names no wall clock. A pool that runs slower multiplies it.
 */
const DEFAULT_WALL_SECONDS = 7_200;

/**
 How long the driver stays up for a steering message after Pi comes to rest, which is what the coordinator's starts used.
 */
const DEFAULT_IDLE_SECONDS = 120;

/**
 A command line the start cannot use.
 */
export class StartPiRunOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'StartPiRunOptionsError';
}

/**
 Builds the error a refused option is answered with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw startOptionsErrorOf('--name is required.',);
 ```
 */
function startOptionsErrorOf(message: string,): Error {
  return new StartPiRunOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({ value: 'a', name: 'name', });
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
    usage: START_USAGE,
    errorOf: startOptionsErrorOf,
  };
}

/**
 What one run of the command was asked for.
 */
export type StartPiRunOptions = {
  /**
   The run's name, which names its unit, its run folder and its worktree.
   */
  readonly name: string;

  /**
   The branch the run works on, or {@link NO_WORKTREE_BRANCH} for the repository's own checkout.
   */
  readonly branch: string;

  /**
   The file holding the first prompt.
   */
  readonly promptFile: string;

  /**
   The number of this attempt, which names its run folder.
   */
  readonly run: number;

  /**
   What Pi runs on.
   */
  readonly provider: string;
  readonly model: string;

  /**
   The ceiling the unit is given.
   */
  readonly cpuPercent: number;
  readonly memory: string;
  readonly tasks: number;

  /**
   How long the run may last before the pool multiplies it.
   */
  readonly wallSeconds: number;

  /**
   How long the driver stays up for a steering message once Pi rests.
   */
  readonly idleSeconds: number;

  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   The checkout that holds this package and the worktrees folder.
   */
  readonly repository: string;

  /**
   The driver entry the unit runs.
   */
  readonly driverEntry: string;

  /**
   A session file to resume.
   */
  readonly session?: string;

  /**
   The run folder of an attempt whose session this run resumes, which keeps the run in that attempt's pool.
   */
  readonly sessionOf?: string;

  /**
   How many turns the run may start, named here instead of sized from the finished runs.
   */
  readonly turnCap?: number;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the run was asked for

 @throws StartPiRunOptionsError when an option is unknown, missing or unusable

 @example
 ```ts
 const options = parseStartPiRunOptions(['--name=a', '--branch=b',],);
 ```
 */
export function parseStartPiRunOptions(commandLine: readonly string[],): StartPiRunOptions {
  try {
    /**
     The options as written.
     */
    const { values, } = parseArgs({
      args: [...commandLine,],
      strict: true,
      allowPositionals: false,
      options: {
        name: { type: 'string', },
        branch: { type: 'string', },
        prompt: { type: 'string', },
        run: { type: 'string', },
        provider: { type: 'string', },
        model: { type: 'string', },
        cpu: { type: 'string', },
        memory: { type: 'string', },
        tasks: { type: 'string', },
        wall: { type: 'string', },
        'idle-seconds': { type: 'string', },
        'turn-cap': { type: 'string', },
        session: { type: 'string', },
        'session-of': { type: 'string', },
        'runs-folder': { type: 'string', },
        repository: { type: 'string', },
        'driver-entry': { type: 'string', },
      },
    },);
    /**
     The run's name, which every folder and unit name is read from.
     */
    const name = requiredText(optionOf({
      value: values.name,
      name: 'name',
    },),);
    /**
     Folder the driver writes one run folder per run into.
     */
    const runsFolder = (values['runs-folder'] === undefined)
      ? defaultRunsFolder({},)
      : requiredText(optionOf({
        value: values['runs-folder'],
        name: 'runs-folder',
      },),);
    /**
     The checkout that holds this package and the worktrees folder.
     */
    const repository = (values.repository === undefined)
      ? defaultRepository({},)
      : requiredText(optionOf({
        value: values.repository,
        name: 'repository',
      },),);
    if ((values.session !== undefined) && (values['session-of'] !== undefined)) {
      throw new StartPiRunOptionsError(`--session and --session-of both name what this run resumes; give one.\nUsage: ${START_USAGE}`,);
    }
    return {
      name,
      branch: requiredText(optionOf({
        value: values.branch,
        name: 'branch',
      },),),
      promptFile: (values.prompt === undefined)
        ? promptFileOf({
          runsFolder,
          name,
        },)
        : requiredText(optionOf({
          value: values.prompt,
          name: 'prompt',
        },),),
      run: (values.run === undefined) ? DEFAULT_RUN : requiredPositiveNumber(optionOf({
        value: values.run,
        name: 'run',
      },),),
      provider: (values.provider === undefined) ? (DEFAULT_POOL?.provider ?? '') : requiredText(optionOf({
        value: values.provider,
        name: 'provider',
      },),),
      model: (values.model === undefined) ? (DEFAULT_POOL?.model ?? '') : requiredText(optionOf({
        value: values.model,
        name: 'model',
      },),),
      cpuPercent: (values.cpu === undefined) ? DEFAULT_CPU_PERCENT : requiredPositiveNumber(optionOf({
        value: values.cpu,
        name: 'cpu',
      },),),
      memory: (values.memory === undefined) ? DEFAULT_MEMORY : requiredText(optionOf({
        value: values.memory,
        name: 'memory',
      },),),
      tasks: (values.tasks === undefined) ? DEFAULT_TASKS : requiredPositiveNumber(optionOf({
        value: values.tasks,
        name: 'tasks',
      },),),
      wallSeconds: (values.wall === undefined) ? DEFAULT_WALL_SECONDS : requiredPositiveNumber(optionOf({
        value: values.wall,
        name: 'wall',
      },),),
      idleSeconds: (values['idle-seconds'] === undefined) ? DEFAULT_IDLE_SECONDS : requiredPositiveNumber(optionOf({
        value: values['idle-seconds'],
        name: 'idle-seconds',
      },),),
      runsFolder,
      repository,
      driverEntry: (values['driver-entry'] === undefined)
        ? driverEntryOf({ repository, },)
        : requiredText(optionOf({
          value: values['driver-entry'],
          name: 'driver-entry',
        },),),
      ...(values['turn-cap'] === undefined ? {} : { turnCap: requiredPositiveNumber(optionOf({
        value: values['turn-cap'],
        name: 'turn-cap',
      },),), }),
      ...(values.session === undefined ? {} : { session: requiredText(optionOf({
        value: values.session,
        name: 'session',
      },),), }),
      ...(values['session-of'] === undefined ? {} : { sessionOf: requiredText(optionOf({
        value: values['session-of'],
        name: 'session-of',
      },),), }),
    };
  }
  catch (error) {
    if (error instanceof StartPiRunOptionsError) {
      throw error;
    }
    throw new StartPiRunOptionsError(
      `${String(error,)}\nUsage: ${START_USAGE}`,
      { cause: error, },
    );
  }
}

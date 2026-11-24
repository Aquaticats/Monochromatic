/**
 What pool a worktree launch's run takes, or the refusal that stops the start before anything is made.

 A command line that names its provider keeps the pool that pair names, and a model one pool takes keeps that pool: such a run is
 refused while its pool takes no new run. A command line that names no provider takes the first open pool in the owner's order, as
 the project decided on 2026-11-23
 (`doc/decision/pi-driver-closed-pools-and-the-worktree-launchers-default.md`), and is never refused for the pool its default
 provider and model name.

 The counts are the pool gate's own (`goingRunsNow`, `talliesOf`), so a slot this command takes is a slot the gate reads as taken
 once the unit exists: the unit's command line names the pool. No second counting mechanism exists beside that one.

 @module
 */

import {
  type GoingRun,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
  goingRunsNow,
} from './going-runs.ts';
import { wallSecondsOn, } from './launcher-arguments.ts';
import { poolOfARunNamingNoProvider, } from './pool-of-a-run-naming-no-provider.ts';
import { NO_SLOT_EXIT_STATUS, } from './pool-gate.ts';
import {
  POOL_LIMITS_VARIABLE,
  PoolLimitsError,
  type PoolMembership,
  type ProviderPool,
  limitsIn,
  poolNamed,
  poolsWithTheseLimits,
} from './pool-limits.ts';
import {
  summaryTextOf,
  talliesOf,
} from './pool-tallies.ts';
import {
  CLOSED_POOL_EXIT_STATUS,
  ClosedPoolRefusalError,
  refuseARunOnAClosedPool,
} from './refusal-of-a-run-on-a-closed-pool.ts';
import type { RunCommand, } from './run-command.ts';
import { PiUnitsUnreadableError, } from './running-pi-units.ts';

/**
 A run no pool a run naming no provider may take has room for, refused with the pools named and the reason each gives.

 A refusal of its own rather than a returned answer, so the command that starts the run cannot go on to make a worktree for it.
 */
export class NoPoolTakesTheRunError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'NoPoolTakesTheRunError';
}

/**
 What the launch's run takes, or why this command starts none.
 */
export type WorktreeLaunchPlacement =
  | {
    /**
     Whether the run takes its pair and not a refusal.
     */
    readonly took: 'a-run';

    /**
     The provider the run's unit and driver are given.
     */
    readonly provider: string;

    /**
     The model the run's unit and driver are given.
     */
    readonly model: string;

    /**
     How long the run may last, in seconds: the caller's own wall clock, or that clock stretched for a pool that answers more
     slowly.
     */
    readonly wallSeconds: number;

    /**
     The lines this command prints about the pool it chose, which are none when the command line named the pair the run takes.
     */
    readonly lines: readonly string[];
  }
    | {
    /**
     Whether the run takes its pair and not a refusal.
     */
      readonly took: 'no-run';

    /**
     The status this command exits with, which is the pool gate's own for a closed pool and for no room.
     */
      readonly status: number;

    /**
     What the run is refused with, one line per thing a reader needs.
     */
      readonly message: string;
    };

/**
 Reads the pools at the limits the caller set, which say which pools take runs, how many, and why a closed one is closed.

 @param environment - the environment the command runs with, whose `PI_RUNS_POOL_LIMITS` sets the pools' limits

 @returns the pools at those limits

 @throws PoolLimitsError when the setting names no pool of the table or holds anything but a whole number, which the caller
 answers by naming the setting and its fault and starting nothing

 @example
 ```ts
 const pools = poolsWithTheCallersLimits({ environment: process.env, },);
 ```
 */
function poolsWithTheCallersLimits({
  environment,
}: {
  readonly environment: Readonly<NodeJS.ProcessEnv>;
},): readonly ProviderPool[] {
  return poolsWithTheseLimits({
    limits: limitsIn({ listed: environment[POOL_LIMITS_VARIABLE] ?? '', },),
  },);
}

/**
 Says what a unit whose own command line could not be read is counted as, which is the line the pool gate prints for the same
 problem.

 @param problem - what the manager answered
 */
function reportAUnitWhosePoolWasNotRead(problem: string,): void {
  process.stderr
    .write(`${problem}\n`,);
}

/**
 Reads the runs going now, counted the way the pool gate counts them: the launchers that have not made their unit yet, and the
 units systemd lists, each with the pool its own command line names.

 @param pools - the pools to count against

 @param processFolder - the folder holding one entry per process

 @param run - how a command is run

 @returns one entry per run going now

 @throws PiUnitsUnreadableError when systemd could not list the units, which the caller answers by refusing the start: a run
 started on a guessed count could take a pool that is full

 @example
 ```ts
 const runs = await runsGoingForTheCount({ pools, processFolder: '/proc', run: runCommandWithChildProcess, },);
 ```
 */
async function runsGoingForTheCount({
  pools,
  processFolder,
  run,
}: {
  readonly pools: readonly ProviderPool[];
  readonly processFolder: string;
  readonly run: RunCommand;
}): Promise<readonly GoingRun[]> {
  return await goingRunsNow({
    run,
    processFolder,
    launcherEntryNames: [
      ...LAUNCHER_ENTRY_NAMES,
      ...REPLACED_LAUNCHER_ENTRY_NAMES,
    ],
    pools,
    onProblem: reportAUnitWhosePoolWasNotRead,
  },);
}

/**
 Places a worktree launch whose command line names no provider on the first open pool of the owner's order.

 This run's own name is left out of the count: the launcher is already a launcher of that name, and the slot it is choosing is the
 one it would hold.

 @param requested - the pool the run's provider and model name, as the table ships it

 @param resumes - whether the run resumes a session, which keeps it on the pool its model names while that pool takes new runs

 @param runName - the run's name, which is left out of the count

 @param wallSeconds - how long the caller asked the run to last

 @param provider - the provider the command line names, or the default this package gave it

 @param model - the model the command line names, or the default this package gave it

 @param environment - the environment the command runs with, whose limits decide which pools are open

 @param processFolder - the folder holding one entry per process

 @param run - how a command is run

 @returns the provider, model and wall clock the run takes, and the line naming the pool

 @throws NoPoolTakesTheRunError when no pool a run naming no provider may take has room, naming each pool and why

 @throws PoolLimitsError when the limits setting cannot be read

 @throws PiUnitsUnreadableError when the runs going now cannot be read

 @example
 ```ts
 const placed = await placeARunNamingNoProvider({ requested, resumes: false, runName: 'copy-route', wallSeconds: 600, provider: 'neuralwatt', model: 'deepseek-v4.1-flash', environment: process.env, processFolder: '/proc', run: runCommandWithChildProcess, },);
 ```
 */
async function placeARunNamingNoProvider({
  requested,
  resumes,
  runName,
  wallSeconds,
  provider,
  model,
  environment,
  processFolder,
  run,
}: {
  readonly requested: PoolMembership;
  readonly resumes: boolean;
  readonly runName: string;
  readonly wallSeconds: number;
  readonly provider: string;
  readonly model: string;
  readonly environment: Readonly<NodeJS.ProcessEnv>;
  readonly processFolder: string;
  readonly run: RunCommand;
},): Promise<WorktreeLaunchPlacement> {
  /**
   The pools at the limits the caller set.
   */
  const pools = poolsWithTheCallersLimits({ environment, },);
  /**
   How many runs each pool holds now, without this run's own name, which is what the owner's order is read against.
   */
  const tallies = talliesOf({
    runs: (await runsGoingForTheCount({
      pools,
      processFolder,
      run,
    },)).filter(function isAnotherRun(going,) {
      return going.name !== runName;
    },),
    pools,
  },);
  /**
   What the run takes.
   */
  const taken = poolOfARunNamingNoProvider({
    requested,
    resumes,
    tallies,
  },);
  if (taken.took === 'no-pool') {
    throw new NoPoolTakesTheRunError(taken.refusal,);
  }
  /**
   The words that say whether this command chose the pool or kept the one the run's own model names.
   */
  const why = taken.movesTheRun
    ? 'the first open pool a run that names no provider may take'
    : 'the pool the run names, which takes new runs';
  /**
   The line this command prints, which names the pool beside every pool's count.
   */
  const line = `pool: ${taken.pool
    .name}, ${why}: ${summaryTextOf(tallies,)}`;
  if (!taken.movesTheRun) {
    return {
      took: 'a-run',
      provider,
      model,
      wallSeconds,
      lines: [line,],
    };
  }
  // A pool that answers more slowly gives the run more wall clock, as it does for a run the pool gate moves here.
  return {
    took: 'a-run',
    provider: taken.pool
      .provider,
    model: taken.pool
      .model,
    wallSeconds: wallSecondsOn({
      wallSeconds,
      wallFactor: taken.pool
        .wallFactor,
    },),
    lines: [line,],
  };
}

/**
 What the launch's run takes, or the refusal this command starts nothing for.

 @param provider - the provider the command line names, or the default this package gave it

 @param model - the model the command line names, or the default this package gave it

 @param namedByTheCommandLine - which of the provider and the model the command line named itself

 @param resumes - whether the run resumes a session

 @param runName - the run's name

 @param wallSeconds - how long the caller asked the run to last

 @param environment - the environment the command runs with, whose `PI_RUNS_POOL_LIMITS` sets the pools' limits

 @param processFolder - the folder holding one entry per process

 @param run - how a command is run

 @returns the run's placement, or the refusal and the status this command exits with

 @throws Error when a failure is neither a refusal this command answers itself nor one a reader can be told about

 @example
 ```ts
 const decided = await poolPlacementOfAWorktreeLaunch({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', namedByTheCommandLine: { provider: false, model: false, }, resumes: false, runName: 'copy-route', wallSeconds: 600, environment: process.env, processFolder: '/proc', run: runCommandWithChildProcess, },);
 ```
 */
export async function poolPlacementOfAWorktreeLaunch({
  provider,
  model,
  namedByTheCommandLine,
  resumes,
  runName,
  wallSeconds,
  environment,
  processFolder,
  run,
}: {
  readonly provider: string;
  readonly model: string;
  readonly namedByTheCommandLine: {
    readonly provider: boolean;
    readonly model: boolean;
  };
  readonly resumes: boolean;
  readonly runName: string;
  readonly wallSeconds: number;
  readonly environment: Readonly<NodeJS.ProcessEnv>;
  readonly processFolder: string;
  readonly run: RunCommand;
},): Promise<WorktreeLaunchPlacement> {
  /**
   The pool the provider and model name, before this command places the run.
   */
  const requested = poolNamed({
    provider,
    model,
  },);
  /**
   Whether this command chooses the run's pool rather than keeping the pair its command line names: a launch that names no
   provider takes the first open pool in the owner's order, and the pools that take such a run are the ones the table marks.
   */
  const poolIsTakenHere = (!namedByTheCommandLine.provider)
    && (requested.holds === 'a-pool')
    && requested.pool
      .takesDefaultRuns;
  if (!poolIsTakenHere) {
    try {
      refuseARunOnAClosedPool({
        provider,
        model,
        namedByTheCommandLine,
        environment,
      },);
    }
    catch (error) {
      if (error instanceof ClosedPoolRefusalError) {
        return {
          took: 'no-run',
          status: CLOSED_POOL_EXIT_STATUS,
          message: error.message,
        };
      }
      // A pool limit this command cannot read refuses the run it could have touched, naming the setting and its fault: a run
      // outside every pool never reaches this, so a broken setting refuses only the runs it could have touched.
      if (error instanceof PoolLimitsError) {
        return {
          took: 'no-run',
          status: 1,
          message: error.message,
        };
      }
      throw error;
    }
    return {
      took: 'a-run',
      provider,
      model,
      wallSeconds,
      lines: [],
    };
  }
  try {
    return await placeARunNamingNoProvider({
      requested,
      resumes,
      runName,
      wallSeconds,
      provider,
      model,
      environment,
      processFolder,
      run,
    },);
  }
  catch (error) {
    // A run no pool has room for is refused with the pools named and the status the pool gate answers for no room.
    if (error instanceof NoPoolTakesTheRunError) {
      return {
        took: 'no-run',
        status: NO_SLOT_EXIT_STATUS,
        message: error.message,
      };
    }
    // A limits setting or a unit listing this command cannot read is named with its fault rather than counted as though the caller
    // had said nothing: a run started on a guessed count could take a pool that is full.
    if ((error instanceof PoolLimitsError) || (error instanceof PiUnitsUnreadableError)) {
      return {
        took: 'no-run',
        status: 1,
        message: error.message,
      };
    }
    throw error;
  }
}

/**
 What the pool gate answers: the counts, that this run is already going, that no pool has room, or the words the launcher is
 given.

 The gate counts and decides; it starts nothing itself, so a caller can ask it for the counts on a turn of its own.

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import {
  type LauncherArguments,
  launcherArgumentsOf,
} from './launcher-arguments.ts';
import {
  AUDIT_RUN_PREFIX,
  MODEL_OF_A_RUN_NAMING_NONE,
  type PoolMembership,
  PROVIDER_OF_A_RUN_NAMING_NONE,
  type ProviderPool,
  poolNamed,
} from './pool-limits.ts';
import {
  type PoolGateOptions,
  POOL_GATE_USAGE,
  PoolGateOptionsError,
} from './parse-pool-gate-options.ts';
import {
  type PoolTallies,
  NO_POOL_HAS_ROOM,
  firstDefaultPoolWithRoom,
  poolHasRoom,
  summaryTextOf,
} from './pool-tallies.ts';

/**
 The status a gate exits with when no pool has room, which the retrying command reads to know it should ask again.
 */
export const NO_SLOT_EXIT_STATUS = 3;

/**
 The status a gate exits with when the run it was asked for is already going.
 */
export const ALREADY_RUNNING_EXIT_STATUS = 4;

/**
 What a run's pool choice is: the pool it goes on, its own provider when no pool holds it, or the fact that none has room.
 */
export type PoolChoice =
  | {
    readonly destination: 'a-pool';
    readonly pool: ProviderPool;
    readonly movesTheRun: boolean;
  }
    | { readonly destination: 'its-own-provider' }
    | { readonly destination: 'no-room' };

/**
 Reads which pool a run may take.

 A run that names a model only one pool takes stays on that pool, and a run that resumes a session stays on the pool it named.
 Every other run takes the first pool of the owner's order that has room, and gets that pool's provider, model and wall clock.

 @param requested - the pool the run's own options name, or that it names none

 @param tallies - how many runs each pool holds

 @param resumes - whether the run resumes a session, which keeps it on the pool it named

 @returns the pool the run goes on, or that none has room

 @example
 ```ts
 const choice = poolChoiceFor({ requested: poolNamed({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', },), tallies, resumes: false, },);
 ```
 */
export function poolChoiceFor({
  requested,
  tallies,
  resumes,
}: {
  readonly requested: PoolMembership;
  readonly tallies: PoolTallies;
  readonly resumes: boolean;
},): PoolChoice {
  if (requested.holds !== 'a-pool') {
    return { destination: 'its-own-provider', };
  }
  if (requested.pool
    .takesDefaultRuns
    && (!resumes)) {
    /**
     The first pool of the owner's order with room.
     */
    const first = firstDefaultPoolWithRoom({ tallies, },);
    return (first === NO_POOL_HAS_ROOM)
      ? { destination: 'no-room', }
      : {
        destination: 'a-pool',
        pool: first,
        movesTheRun: true,
      };
  }
  return poolHasRoom({
    tallies,
    pool: requested.pool,
  },)
    ? {
      destination: 'a-pool',
      pool: requested.pool,
      movesTheRun: false,
    }
    : { destination: 'no-room', };
}

/**
 What the gate answers with.
 */
export type GateOutcome =
  | {
    readonly action: 'counts';
    readonly lines: readonly string[];
  }
    | {
      readonly action: 'already-running';
      readonly status: number;
      readonly lines: readonly string[];
    }
    | {
      readonly action: 'no-room';
      readonly status: number;
      readonly lines: readonly string[];
    }
    | {
      readonly action: 'start';
      readonly status: number;
      readonly lines: readonly string[];
      readonly arguments: LauncherArguments;
    };

/**
 Says what the gate answers, given the runs going now and what it was asked for.

 @param options - what the gate was asked for

 @param runs - the runs going now

 @param tallies - how many runs each pool holds, which were counted from the same runs

 @param sessionFileOf - reads the session file of the finished run this run resumes

 @returns the action, the lines to print, and the launcher's words when it starts one

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs, tallies: talliesOf({ runs, }), sessionFileOf, },);
 ```
 */
export async function gateOutcomeOf({
  options,
  runs,
  tallies,
  sessionFileOf,
}: {
  readonly options: PoolGateOptions;
  readonly runs: readonly GoingRun[];
  readonly tallies: PoolTallies;
  readonly sessionFileOf: (runFolder: string) => Promise<string>;
},): Promise<GateOutcome> {
  /**
   How full every pool is, as every answer names it.
   */
  const summary = summaryTextOf(tallies,);
  // `--counts-only` starts nothing, which is how a coordinator fills the free slots it has just read. It answers the same
  // whoever asks, so it neither needs a run's name nor refuses because a run of that name is already going.
  if (options.countsOnly) {
    return {
      action: 'counts',
      lines: [summary,],
    };
  }
  /**
   The run's name, which every other answer names.
   */
  const runName = options.name;
  if (runName === undefined) {
    throw new PoolGateOptionsError(`--name is required unless only the counts are asked for.\nUsage: ${POOL_GATE_USAGE}`,);
  }
  // An audit runs on another account and holds no slot, so a second run of the same audit name is no conflict.
  if (
    (!runName.startsWith(AUDIT_RUN_PREFIX,))
      && runs.some(function isNamed(run,) {
        return run.name === runName;
      },)
  ) {
    return {
      action: 'already-running',
      status: ALREADY_RUNNING_EXIT_STATUS,
      lines: [`ALREADY RUNNING: ${runName}`,],
    };
  }
  /**
   The pool the run's own options name, or that it names none.
   */
  const requested = poolNamed({
    provider: options.provider ?? PROVIDER_OF_A_RUN_NAMING_NONE,
    model: options.model ?? MODEL_OF_A_RUN_NAMING_NONE,
  },);
  /**
   Which pool the run may take.
   */
  const choice = poolChoiceFor({
    requested,
    tallies,
    resumes: options.sessionOfRunFolder !== undefined,
  },);
  if (choice.destination === 'no-room') {
    return {
      action: 'no-room',
      status: NO_SLOT_EXIT_STATUS,
      lines: [`NO SLOT: ${summary}, not started: ${runName}`,],
    };
  }
  /**
   The session file the run resumes, when it resumes one.
   */
  const sessionFile = (options.sessionOfRunFolder === undefined)
    ? undefined
    : await sessionFileOf(options.sessionOfRunFolder,);
  return {
    action: 'start',
    status: 0,
    lines: [`slot free: ${summary}; ${runName} goes on ${(choice.destination === 'a-pool') ? choice.pool
      .name : 'its own provider'}`,],
    arguments: launcherArgumentsOf({
      commandLine: options.commandLine,
      movesTheRun: (choice.destination === 'a-pool') && choice.movesTheRun,
      ...((choice.destination === 'a-pool') ? { pool: choice.pool, } : {}),
      ...(options.wallSeconds === undefined ? {} : { wallSeconds: options.wallSeconds, }),
      ...(sessionFile === undefined ? {} : { sessionFile, }),
      runsFolder: options.runsFolder,
    },),
  };
}

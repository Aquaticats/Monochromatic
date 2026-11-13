/**
 What the pool gate answers: the counts, that this run is already going, that no pool has room, or the words the launcher is
 given.

 The gate counts and decides; it starts nothing itself, so a caller can ask it for the counts on a turn of its own.

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import {
  type FreeSpaceAgainstItsFloor,
  freeSpaceIsUnderTheFloor,
  freeSpaceTextOf,
  freeSpaceUnderTheFloorTextOf,
} from './free-space-floor.ts';
import {
  type MachineCap,
  agentsGoingOf,
  machineCapCountsTextOf,
  machineCapRefusalTextOf,
  machineTakesNoMoreAgents,
} from './machine-cap.ts';
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
  poolTakesNoNewRun,
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
 The status a gate exits with when it starts nothing for want of room, which is no pool having a slot free, the machine already
 carrying its cap of agents, or one of the volumes a run fills being under its floor. The retrying command reads it to know it
 should ask again.
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

 A run that names a model only one pool takes stays on that pool, and a run that resumes a session stays on the pool it named
 while that pool takes new runs. A resume whose pool is closed takes the first pool of the owner's order with room instead,
 because staying on a pool that takes no new run would refuse it for ever. Every other run takes that first pool with room, and
 gets its provider, model and wall clock.

 @param requested - the pool the run's own options name, or that it names none

 @param tallies - how many runs each pool holds

 @param resumes - whether the run resumes a session, which keeps it on the pool it named while that pool takes new runs

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
  /**
   The pool the run names, as the table being counted holds it, so a limit a caller set applies to a run naming that pool's own
   model as much as to a run that names none.
   */
  const pool = countedPool({
    pool: requested.pool,
    tallies,
  },);
  if (pool.takesDefaultRuns
    && ((!resumes) || poolTakesNoNewRun({ pool, },))) {
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
    pool,
  },)
    ? {
      destination: 'a-pool',
      pool,
      movesTheRun: false,
    }
    : { destination: 'no-room', };
}

/**
 Reads one pool as the table being counted holds it.

 A count is taken against the pools a caller handed over, which carry the limits a caller set; the pool a run's own words name
 comes from the shipped table, which knows neither that limit nor a closing. Reading the room off the shipped pool is how a
 pool closed through the variable still took runs that named its model.

 @param pool - the pool the run's own words name

 @param tallies - the pools that were counted, with the limits they were given

 @returns the counted pool of that name, or the one given when the table being counted does not hold it

 @example
 ```ts
 const counted = countedPool({ pool: QWEN_PLAN_QWEN4_MAX_PREVIEW, tallies },);
 ```
 */
function countedPool({
  pool,
  tallies,
}: {
  readonly pool: ProviderPool;
  readonly tallies: PoolTallies;
},): ProviderPool {
  /**
   The counted pool of that name, when the table being counted holds one.
   */
  const counted = tallies.pools
    .find(function isTheSamePool(held,) {
      return held.name === pool.name;
    },);
  return counted ?? pool;
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

 @param volumes - the room of each distinct volume a new run cannot work without, with the strictest floor set for it

 @param machineCap - how many agents the machine may carry at once, and how many of those the gate cannot see

 @param sessionFileOf - reads the session file of the finished run this run resumes

 @returns the action, the lines to print, and the launcher's words when it starts one

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs, tallies: talliesOf({ runs, }), volumes, machineCap, sessionFileOf, },);
 ```
 */
export async function gateOutcomeOf({
  options,
  runs,
  tallies,
  volumes,
  machineCap,
  sessionFileOf,
}: {
  readonly options: PoolGateOptions;
  readonly runs: readonly GoingRun[];
  readonly tallies: PoolTallies;
  readonly volumes: readonly FreeSpaceAgainstItsFloor[];
  readonly machineCap: MachineCap;
  readonly sessionFileOf: (runFolder: string) => Promise<string>;
},): Promise<GateOutcome> {
  /**
   How full every pool is, as every answer names it.
   */
  const summary = summaryTextOf(tallies,);
  /**
   Every agent going on the machine, which the cap counts: every Pi run, in a pool or in none, and the agents its caller says it
   has going, which are no Pi run and so appear in neither a launcher nor a unit.
   */
  const agentsGoing = agentsGoingOf({
    runs,
    otherAgentsGoing: machineCap.otherAgentsGoing,
  },);
  // `--counts-only` starts nothing, which is how a coordinator fills the free slots it has just read. It answers the same
  // whoever asks, so it neither needs a run's name nor refuses because a run of that name is already going.
  if (options.countsOnly) {
    return {
      action: 'counts',
      lines: [[
        summary,
        machineCapCountsTextOf({
          agentsGoing,
          cap: machineCap.cap,
        },),
        ...volumes.map(function roomOf(volume,) {
          return freeSpaceTextOf(volume.space,);
        },),
      ].join('; ',),],
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
   Each volume under its floor, said as the refusal says one, so a refusal names every volume that is too full and the run it did
   not start.
   */
  const underTheirFloors = volumes
    .filter(function isUnderItsFloor(volume,) {
      return freeSpaceIsUnderTheFloor(volume,);
    },)
    .map(function saidAsARefusal(room,) {
      return freeSpaceUnderTheFloorTextOf(room,);
    },);
  if (underTheirFloors.length > 0) {
    return {
      action: 'no-room',
      status: NO_SLOT_EXIT_STATUS,
      lines: [`NO SLOT: ${underTheirFloors.join(', and ',)}; not started: ${runName}`,],
    };
  }
  /**
   Which pool the run may take.
   */
  const choice = poolChoiceFor({
    requested,
    tallies,
    resumes: options.sessionOfRunFolder !== undefined,
  },);
  /**
   Whether the machine is already carrying its cap, so one more agent would go over it however much room every pool has.
   */
  const machineIsAtItsCap = machineTakesNoMoreAgents({
    agentsGoing,
    cap: machineCap.cap,
  },);
  if (machineIsAtItsCap || (choice.destination === 'no-room')) {
    /**
     Why the start is refused, each reason in its own words: the machine's cap, and the room of the pools. A run refused by both
     says both, so a caller waiting for room knows which two limits it waits on.
     */
    const reasons: string[] = machineIsAtItsCap
      ? [machineCapRefusalTextOf({
        agentsGoing,
        cap: machineCap.cap,
      },)]
      : [];
    if (choice.destination === 'no-room') {
      reasons.push(machineIsAtItsCap ? `no pool has room (${summary})` : summary,);
    }
    return {
      action: 'no-room',
      status: NO_SLOT_EXIT_STATUS,
      lines: [`NO SLOT: ${reasons.join(', and ',)}; not started: ${runName}`,],
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

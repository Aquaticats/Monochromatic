/**
 What the pool gate answers: the counts, that this run is already going, that no pool has room, or the words the launcher is
 given.

 The gate counts and decides; it starts nothing itself, so a caller can ask it for the counts on a turn of its own.

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import {
  type FreeSpace,
  freeSpaceFloorTextOf,
  freeSpaceIsUnderTheFloor,
  freeSpaceTextOf,
} from './free-space-floor.ts';
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
  membershipOfSelection,
  poolOfThatNameIn,
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
  virtualSelectionHasRoom,
} from './pool-tallies.ts';

/**
 The status a gate exits with when it starts nothing for want of room, which is no pool having a slot free or the build volume
 being under its floor. The retrying command reads it to know it should ask again.
 */
export const NO_SLOT_EXIT_STATUS = 3;

/**
 The status a gate exits with when the run it was asked for is already going.
 */
export const ALREADY_RUNNING_EXIT_STATUS = 4;

/**
 What a run's pool choice is: the pool it goes on, the pools a virtual selection's router may send it to, its own provider when no
 pool holds it, or the fact that none has room.
 */
export type PoolChoice =
  | {
    readonly destination: 'a-pool';
    readonly pool: ProviderPool;
    readonly movesTheRun: boolean;
  }
    | {
    /**
     A virtual selection whose members have room together, which keeps the provider and model the run named and takes its
     router's order.
     */
      readonly destination: 'a-virtual-pool';

    /**
     The pools that selection's router may send the run to, with the limits a caller set.
     */
      readonly pools: readonly ProviderPool[];

    /**
     How much longer the run may last: the largest wall factor of those pools, because the run keeps one wall clock wherever its
     router lands.
     */
      readonly wallFactor: number;
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
  // A run on a virtual selection keeps the provider and model it named, because that pair is what lets its router move the
  // request on: rewriting it into one member's pair would pin the run to that member. It is counted once against the room its
  // members have together, so it may start while they have room between them, however full any one member's own room reads.
  if (requested.holds === 'a-virtual-selection') {
    /**
     The pools the selection's router may take, as the table being counted holds them, so a limit a caller set applies here too.
     */
    const members = requested.pools
      .map(function asTheCountedTableHoldsIt(pool,) {
        return poolOfThatNameIn({
          pool,
          pools: tallies.pools,
        },);
      },);
    if (!virtualSelectionHasRoom({
      tallies,
      virtual: requested.virtual,
    },)) {
      return { destination: 'no-room', };
    }
    return {
      destination: 'a-virtual-pool',
      pools: members,
      wallFactor: members.reduce(
        function largestFactor(
          largest,
          member,
        ) {
          return Math.max(
            largest,
            member.wallFactor,
          );
        },
        1,
      ),
    };
  }
  // A membership that names every pool a run could be on is how a run whose own command line could not be read is counted, never
  // how a run's own words place it: placing such a run would take one of those pools for a fact the driver does not have.
  if (requested.holds === 'some-pools') {
    throw new Error('poolChoiceFor was given a membership that names every pool a run could be on, which counts an unreadable run and places none: its caller reads what the run named',);
  }
  if (requested.holds !== 'a-pool') {
    return { destination: 'its-own-provider', };
  }
  /**
   The pool the run names, as the table being counted holds it, so a limit a caller set applies to a run naming that pool's own
   model as much as to a run that names none.
   */
  const pool = poolOfThatNameIn({
    pool: requested.pool,
    pools: tallies.pools,
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
 Says where a run goes, as the gate's line puts it: one pool's name, the selection a router reads with every pool it may take, or
 the run's own provider.

 @param choice - which pool or selection the run may take

 @param provider - the provider the run named, or the one a run naming none gets

 @param model - the model the run named, or the one a run naming none gets

 @returns the words the line puts after "goes on"

 @example
 ```ts
 const words = goesOnTextOf({ choice, provider: 'neuralwatt', model: 'deepseek-v4.1-flash', },);
 ```
 */
function goesOnTextOf({
  choice,
  provider,
  model,
}: {
  readonly choice: PoolChoice;
  readonly provider: string;
  readonly model: string;
},): string {
  if (choice.destination === 'a-pool') {
    return choice.pool
      .name;
  }
  if (choice.destination === 'a-virtual-pool') {
    /**
     The names of the pools that selection's router may take.
     */
    const names = choice.pools
      .map(function nameOf(pool,) {
        return pool.name;
      },)
      .join(', ',);
    return `${provider}/${model}, whose router may take ${names}`;
  }
  return 'its own provider';
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

 @param freeSpace - the room the volume that holds the repository has, which a run does not start without

 @param sessionFileOf - reads the session file of the finished run this run resumes

 @returns the action, the lines to print, and the launcher's words when it starts one

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs, tallies: talliesOf({ runs, }), freeSpace, sessionFileOf, },);
 ```
 */
export async function gateOutcomeOf({
  options,
  runs,
  tallies,
  freeSpace,
  sessionFileOf,
}: {
  readonly options: PoolGateOptions;
  readonly runs: readonly GoingRun[];
  readonly tallies: PoolTallies;
  readonly freeSpace: FreeSpace;
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
      lines: [`${summary}; ${freeSpaceTextOf(freeSpace,)}`,],
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
  /**
   The pool the run's own options name, the pools a virtual selection's router may send it to, or the fact that it names none.
   */
  const requested = membershipOfSelection({
    provider: options.provider ?? PROVIDER_OF_A_RUN_NAMING_NONE,
    model: options.model ?? MODEL_OF_A_RUN_NAMING_NONE,
  },);
  // The volume that holds the repository, every agent worktree and every build folder filled on 2026-11-11, and every Pi driver
  // then exited at once with no summary, so a run starts only while the room another agent's build needs is free.
  if (freeSpaceIsUnderTheFloor(freeSpace,)) {
    return {
      action: 'no-room',
      status: NO_SLOT_EXIT_STATUS,
      lines: [
        `NO SLOT: ${freeSpaceTextOf(freeSpace,)}, under the floor of ${freeSpaceFloorTextOf()}; not started: ${runName}`,
      ],
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
  /**
   Where the run goes, as the line says it: one pool's name, the selection a router reads, or the run's own provider.
   */
  const goesOn = goesOnTextOf({
    choice,
    provider: options.provider ?? PROVIDER_OF_A_RUN_NAMING_NONE,
    model: options.model ?? MODEL_OF_A_RUN_NAMING_NONE,
  },);
  return {
    action: 'start',
    status: 0,
    lines: [`slot free: ${summary}; ${runName} goes on ${goesOn}`,],
    arguments: launcherArgumentsOf({
      commandLine: options.commandLine,
      movesTheRun: (choice.destination === 'a-pool') && choice.movesTheRun,
      ...((choice.destination === 'a-pool') ? { pool: choice.pool, } : {}),
      ...((choice.destination === 'a-virtual-pool') ? { wallFactor: choice.wallFactor, } : {}),
      ...(options.wallSeconds === undefined ? {} : { wallSeconds: options.wallSeconds, }),
      ...(sessionFile === undefined ? {} : { sessionFile, }),
      runsFolder: options.runsFolder,
    },),
  };
}

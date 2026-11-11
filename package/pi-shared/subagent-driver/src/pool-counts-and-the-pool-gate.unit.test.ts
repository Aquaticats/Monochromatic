/**
 What the runs going now hold of each provider pool, which pool a run may take, and the room the build volume has: the counts,
 the summary line, and the answer the pool gate gives. Everything here is read from the runs, the pools and the room a test
 hands over, so no unit, no process, no systemd and no disk of this machine is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALREADY_RUNNING_EXIT_STATUS,
  BYTES_IN_A_GIBIBYTE,
  type FreeSpace,
  FREE_SPACE_FLOOR_BYTES,
  type GateOutcome,
  type GoingRun,
  type LauncherArguments,
  MODEL_OF_A_RUN_NAMING_NONE,
  MODEL_OPTION,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_KIMI_FLEX,
  NEURALWATT_STANDARD,
  NO_POOL_HAS_ROOM,
  NO_SLOT_EXIT_STATUS,
  OPENCODE_GO,
  POOL_OF_A_RUN_NAMING_NO_POOL,
  PROVIDER_OF_A_RUN_NAMING_NONE,
  PROVIDER_OPTION,
  PROVIDER_POOLS,
  PoolGateOptionsError,
  type PoolGateOptions,
  type PoolMembership,
  type ProviderPool,
  QWEN_PLAN_DEEPSEEK,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
  firstDefaultPoolWithRoom,
  freeSpaceFloorTextOf,
  freeSpaceTextOf,
  gateOutcomeOf,
  launcherArgumentsOf,
  membershipOf,
  poolChoiceFor,
  poolHasRoom,
  poolNamed,
  poolTakesNoNewRun,
  summaryTextOf,
  talliesOf,
} from '../dist/final/node/index.mjs';

/**
 The volume every room in this file is on, which is no volume of this machine.

 */
const TEST_VOLUME = '/a-build-volume-of-the-test';

/**
 One run going now, in the pool a test names.

 @param name - the run's name

 @param pool - the membership to count it with, which is outside every pool when absent

 @returns the run

 @example
 ```ts
 const run = aRun('copy-route');
 ```
 */
function aRun(name: string, pool?: PoolMembership,): GoingRun {
  return {
    name,
    membership: pool ?? { inAPool: false, },
  };
}

/**
 A membership in one pool.

 @param pool - the pool

 @returns the membership

 @example
 ```ts
 const inTheFlexPool = inPool(NEURALWATT_DEEPSEEK_FLEX);
 ```
 */
function inPool(pool: ProviderPool,): PoolMembership {
  return {
    inAPool: true,
    pool,
  };
}

/**
 Runs of one pool, named after their pool so a failure prints which count is wrong.

 @param pool - the pool to fill

 @param count - how many runs to make

 @returns the runs

 @example
 ```ts
 const runs = runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit);
 ```
 */
function runsIn(pool: ProviderPool, count: number,): readonly GoingRun[] {
  return Array.from(
    { length: count, },
    function named(_unused, index,) {
      return aRun(
        `${pool.name}-${String(index,)}`,
        inPool(pool,),
      );
    },
  );
}

/**
 Every pool a run that names no model of its own may take, which is every pool that takes default runs and is open.

 @returns the pools, in the order such a run takes them

 @example
 ```ts
 const pools = openDefaultPools();
 ```
 */
function openDefaultPools(): readonly ProviderPool[] {
  return PROVIDER_POOLS.filter(function takesDefaultRunsAndIsOpen(pool,) {
    return pool.takesDefaultRuns && (!poolTakesNoNewRun({ pool, },));
  },);
}

/**
 One run in every slot of every pool the test hands over.

 @param pools - the pools to fill

 @returns the runs

 @example
 ```ts
 const runs = runsFilling(openDefaultPools());
 ```
 */
function runsFilling(pools: readonly ProviderPool[],): readonly GoingRun[] {
  return pools.flatMap(function everySlot(pool,) {
    return runsIn(
      pool,
      pool.limit,
    );
  },);
}

/**
 The pool a run that names no model of its own takes while every pool is empty, read from the pool table rather than written
 down here.

 @returns the pool

 @throws Error when the table takes no default run at all, so expectations built from it could not hold

 @example
 ```ts
 const pool = theFirstDefaultPool();
 ```
 */
function theFirstDefaultPool(): ProviderPool {
  /**
   What the pool table answers about a machine on which nothing is going.
   */
  const pool = firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: [], },), },);
  if (pool === NO_POOL_HAS_ROOM) {
    throw new Error('no pool takes a default run, so this test cannot expect a pool to take one',);
  }
  return pool;
}

/**
 How many of the runs hold a slot in one pool.

 @param runs - the runs going now

 @param pool - the pool to count

 @returns how many runs of those are in the pool

 @example
 ```ts
 const count = countOf({ runs, pool: QWEN_PLAN_DEEPSEEK });
 ```
 */
function countOf({
  runs,
  pool,
}: {
  readonly runs: readonly GoingRun[];
  readonly pool: ProviderPool;
},): number {
  return runs.filter(function isInThisPool(run,) {
    return run.membership.inAPool && (run.membership.pool.name === pool.name);
  },).length;
}

/**
 Says the counts line names every pool the table ships, in the table's order, each with the count these runs give it and its own
 limit.

 @param line - the counts line, without the room the volume has

 @param runs - the runs the count is read from

 @example
 ```ts
 expectCountsLineNamesEachPool({ line: summaryTextOf(tallies), runs });
 ```
 */
function expectCountsLineNamesEachPool({
  line,
  runs,
}: {
  readonly line: string;
  readonly runs: readonly GoingRun[];
},): void {
  /**
   One entry per pool, in the order the line prints them.
   */
  const entries = line.split(', ',);
  expect(entries.length,).toBe(PROVIDER_POOLS.length,);
  for (const [index, pool,] of PROVIDER_POOLS.entries()) {
    expect(entries[index],).toBe(`${pool.name} ${String(countOf({
      runs,
      pool,
    },),)}/${String(pool.limit,)}`,);
  }
}

/**
 Room on the build volume well above the floor, so a run can start in a test.

 @returns the room, and the volume it is on

 @example
 ```ts
 const room = roomToSpare();
 ```
 */
function roomToSpare(): FreeSpace {
  return {
    volume: TEST_VOLUME,
    freeBytes: FREE_SPACE_FLOOR_BYTES + BYTES_IN_A_GIBIBYTE,
  };
}

/**
 Room on the build volume under the floor, which is the fill of 2026-11-11 that this floor came from.

 @returns the room, and the volume it is on

 @example
 ```ts
 const room = roomUnderTheFloor();
 ```
 */
function roomUnderTheFloor(): FreeSpace {
  return {
    volume: TEST_VOLUME,
    freeBytes: FREE_SPACE_FLOOR_BYTES - 1,
  };
}

/**
 The pool gate's options, as a test writes them.

 @param options - what the test cares about

 @returns the options, with the folder the runs live in filled in

 @example
 ```ts
 const options = gateOptions({ name: 'copy-route', });
 ```
 */
function gateOptions(options: {
  readonly name?: string;
  readonly countsOnly?: boolean;
  readonly provider?: string;
  readonly model?: string;
  readonly wallSeconds?: number;
  readonly sessionOfRunFolder?: string;
  readonly commandLine?: readonly string[];
},): PoolGateOptions {
  return {
    runsFolder: '/runs',
    countsOnly: options.countsOnly ?? false,
    ...(options.name === undefined ? {} : { name: options.name, }),
    ...(options.provider === undefined ? {} : { provider: options.provider, }),
    ...(options.model === undefined ? {} : { model: options.model, }),
    ...(options.wallSeconds === undefined ? {} : { wallSeconds: options.wallSeconds, }),
    ...(options.sessionOfRunFolder === undefined ? {} : { sessionOfRunFolder: options.sessionOfRunFolder, }),
    commandLine: options.commandLine ?? [],
  };
}

/**
 A session file reader that answers with a fixed file.

 @returns the file

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs: [], tallies, sessionFileOf: aSessionFile, freeSpace: roomToSpare(), });
 ```
 */
async function aSessionFile(): Promise<string> {
  return '/sessions/a.jsonl';
}

await describe({
  name: '',
  children: [
    describe({
      name: poolNamed.name,
      children: [
        it({
          name: 'reads the pool a provider and model share, and answers that they share none when they do not',
          fn: async () => {
            expect(poolNamed({
              provider: QWEN_PLAN_DEEPSEEK.provider,
              model: QWEN_PLAN_DEEPSEEK.model,
            },),).toEqual(inPool(QWEN_PLAN_DEEPSEEK,),);
            expect(poolNamed({
              provider: NEURALWATT_KIMI_FLEX.provider,
              model: NEURALWATT_KIMI_FLEX.model,
            },),).toEqual(inPool(NEURALWATT_KIMI_FLEX,),);
            expect(poolNamed({
              provider: QWEN_PLAN_QWEN4_MAX_PREVIEW.provider,
              model: QWEN_PLAN_QWEN4_MAX_PREVIEW.model,
            },),).toEqual(inPool(QWEN_PLAN_QWEN4_MAX_PREVIEW,),);
            expect(poolNamed({
              provider: 'synthetic',
              model: 'moonshotai/Kimi-K3.1-Pro',
            },),).toEqual({ inAPool: false, },);
          },
        },),
        it({
          name: 'reads the provider and model a run names nothing of as standard Neuralwatt, the pool the qwen plan replaced, so the run is placed on the first pool with room',
          fn: async () => {
            expect(poolNamed({
              provider: PROVIDER_OF_A_RUN_NAMING_NONE,
              model: MODEL_OF_A_RUN_NAMING_NONE,
            },),).toEqual(inPool(NEURALWATT_STANDARD,),);
            expect(poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: poolTakesNoNewRun.name,
      children: [
        it({
          name: 'says a pool is closed exactly when a run of its own cannot take it, and the table holds both a closed pool and an open one',
          fn: async () => {
            /**
             What the table holds of no pool at all.
             */
            const empty = talliesOf({ runs: [], },);
            for (const pool of PROVIDER_POOLS) {
              expect(poolTakesNoNewRun({ pool, },),).toBe(!poolHasRoom({
                tallies: empty,
                pool,
              },),);
            }
            expect(PROVIDER_POOLS.some(function isClosed(pool,) {
              return poolTakesNoNewRun({ pool, },);
            },),).toBe(true,);
            expect(PROVIDER_POOLS.some(function isOpen(pool,) {
              return !poolTakesNoNewRun({ pool, },);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'says the pool the qwen plan replaced and the pool whose week is used up take no new run, and the pools that were given room do',
          fn: async () => {
            expect(poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },),).toBe(true,);
            expect(poolTakesNoNewRun({ pool: OPENCODE_GO, },),).toBe(true,);
            expect(poolTakesNoNewRun({ pool: QWEN_PLAN_DEEPSEEK, },),).toBe(false,);
            expect(poolTakesNoNewRun({ pool: NEURALWATT_DEEPSEEK_FLEX, },),).toBe(false,);
            expect(poolTakesNoNewRun({ pool: NEURALWATT_KIMI_FLEX, },),).toBe(false,);
            expect(poolTakesNoNewRun({ pool: QWEN_PLAN_QWEN4_MAX_PREVIEW, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: membershipOf.name,
      children: [
        it({
          name: 'counts a run in the pool its command line names, including a Kimi run that names the flex pool',
          fn: async () => {
            expect(membershipOf({
              runName: 'copy-route',
              pool: inPool(OPENCODE_GO,),
            },),).toEqual(inPool(OPENCODE_GO,),);
            expect(membershipOf({
              runName: 'kimi-review-round-9',
              pool: inPool(NEURALWATT_KIMI_FLEX,),
            },),).toEqual(inPool(NEURALWATT_KIMI_FLEX,),);
          },
        },),
        it({
          name: 'counts no slot for an audit, or for a Kimi run on a provider no pool holds',
          fn: async () => {
            expect(membershipOf({ runName: 'sol-audit-1', }),).toEqual({ inAPool: false, },);
            expect(membershipOf({ runName: 'kimi-review-round-9', }),).toEqual({ inAPool: false, },);
          },
        },),
        it({
          name: 'counts a run that names no pool at all in the fallback pool, which is what a unit whose own command line could not be read is counted in',
          fn: async () => {
            expect(membershipOf({ runName: 'copy-route', }),).toEqual({
              inAPool: true,
              pool: POOL_OF_A_RUN_NAMING_NO_POOL,
            },);
          },
        },),
      ],
    },),
    describe({
      name: talliesOf.name,
      children: [
        it({
          name: 'counts every pool, at nothing when no run holds it, and leaves a run in no pool out',
          fn: async () => {
            /**
             Two runs on the qwen plan's pool and one outside every pool.
             */
            const runs = [
              ...runsIn(QWEN_PLAN_DEEPSEEK, 2,),
              aRun('sol-audit-1',),
            ];
            /**
             What the runs hold of each pool.
             */
            const tallies = talliesOf({ runs, },);
            expect(tallies.counts.get(QWEN_PLAN_DEEPSEEK.name,),).toBe(2,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(0,);
            expectCountsLineNamesEachPool({
              line: summaryTextOf(tallies,),
              runs,
            },);
          },
        },),
      ],
    },),
    describe({
      name: summaryTextOf.name,
      children: [
        it({
          name: 'names every pool the table ships, in the order a run that names no model of its own takes them, each with its own limit',
          fn: async () => {
            expectCountsLineNamesEachPool({
              line: summaryTextOf(talliesOf({ runs: [], },),),
              runs: [],
            },);
          },
        },),
      ],
    },),
    describe({
      name: poolHasRoom.name,
      children: [
        it({
          name: 'says a pool with a slot left has room, one at its limit has none, and a closed pool never has any',
          fn: async () => {
            /**
             A pool an ordinary run goes on, which is open and takes default runs.
             */
            const pool = theFirstDefaultPool();
            expect(poolHasRoom({
              tallies: talliesOf({ runs: runsIn(pool, pool.limit - 1,), },),
              pool,
            },),).toBe(true,);
            expect(poolHasRoom({
              tallies: talliesOf({ runs: runsIn(pool, pool.limit,), },),
              pool,
            },),).toBe(false,);
            for (const closed of PROVIDER_POOLS.filter(function isClosed(candidate,) {
              return poolTakesNoNewRun({ pool: candidate, },);
            },)) {
              expect(poolHasRoom({
                tallies: talliesOf({ runs: [], },),
                pool: closed,
              },),).toBe(false,);
            }
          },
        },),
      ],
    },),
    describe({
      name: firstDefaultPoolWithRoom.name,
      children: [
        it({
          name: 'takes the owner order, which starts with the qwen plan pool and then Neuralwatt flex, and skips a closed pool and a pool that takes only its own model',
          fn: async () => {
            expect(firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: [], },), },),).toEqual(QWEN_PLAN_DEEPSEEK,);
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({ runs: runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit,), },),
            },),).toEqual(NEURALWATT_DEEPSEEK_FLEX,);
          },
        },),
        it({
          name: 'answers that no pool has room once every open pool that takes a default run is full, because the closed ones take no new run',
          fn: async () => {
            /**
             The pools a run that names no model of its own may take.
             */
            const open = openDefaultPools();
            expect(open.length,).toBeGreaterThan(0,);
            expect(firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: runsFilling(open,), },), },),).toBe(NO_POOL_HAS_ROOM,);
          },
        },),
      ],
    },),
    describe({
      name: poolChoiceFor.name,
      children: [
        it({
          name: 'moves a run that names a default model, here the closed pool the launcher defaults name, to the first pool with room, and says it was moved',
          fn: async () => {
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_STANDARD,),
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({
              destination: 'a-pool',
              pool: theFirstDefaultPool(),
              movesTheRun: true,
            },);
          },
        },),
        it({
          name: 'keeps a run that resumes a session on the pool it named while that pool takes new runs, with no wall factor',
          fn: async () => {
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_DEEPSEEK_FLEX,),
              tallies: talliesOf({ runs: [], },),
              resumes: true,
            },),).toEqual({
              destination: 'a-pool',
              pool: NEURALWATT_DEEPSEEK_FLEX,
              movesTheRun: false,
            },);
          },
        },),
        it({
          name: 'moves a resume whose own pool is closed to the first pool with room, because staying on it would refuse the run for ever',
          fn: async () => {
            /**
             Every pool the owner closed, which a resumed run may name.
             */
            const closed = PROVIDER_POOLS.filter(function isClosed(pool,) {
              return pool.takesDefaultRuns && poolTakesNoNewRun({ pool, },);
            },);
            expect(closed.length,).toBeGreaterThan(0,);
            for (const pool of closed) {
              expect(poolChoiceFor({
                requested: inPool(pool,),
                tallies: talliesOf({ runs: [], },),
                resumes: true,
              },),).toEqual({
                destination: 'a-pool',
                pool: theFirstDefaultPool(),
                movesTheRun: true,
              },);
            }
          },
        },),
        it({
          name: 'keeps a run that names the Kimi model on the Kimi pool while it has room, never moving it to a default pool',
          fn: async () => {
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_KIMI_FLEX,),
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({
              destination: 'a-pool',
              pool: NEURALWATT_KIMI_FLEX,
              movesTheRun: false,
            },);
          },
        },),
        it({
          name: 'sends a run whose provider no pool holds on its own provider',
          fn: async () => {
            expect(poolChoiceFor({
              requested: { inAPool: false, },
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({ destination: 'its-own-provider', },);
          },
        },),
        it({
          name: 'says no pool has room when every pool the run could take is full, including the Kimi pool it named',
          fn: async () => {
            /**
             Every pool a default run can take, filled.
             */
            const filled = talliesOf({ runs: runsFilling(openDefaultPools(),), },);
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_STANDARD,),
              tallies: filled,
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_KIMI_FLEX,),
              tallies: talliesOf({ runs: runsIn(NEURALWATT_KIMI_FLEX, NEURALWATT_KIMI_FLEX.limit,), },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
          },
        },),
      ],
    },),
    describe({
      name: launcherArgumentsOf.name,
      children: [
        it({
          name: 'writes the chosen pool into the options the caller gave, doubles the wall clock of a flex pool, and appends what the caller left out',
          fn: async () => {
            /**
             What the launcher is given for a run that named only itself.
             */
            const given: LauncherArguments = launcherArgumentsOf({
              commandLine: ['--name=copy-route', '--wall=600',],
              pool: NEURALWATT_DEEPSEEK_FLEX,
              movesTheRun: true,
              wallSeconds: 600,
              runsFolder: '/runs',
            },);
            expect(given.words,).toEqual([
              '--name=copy-route',
              `--wall=${String(600 * NEURALWATT_DEEPSEEK_FLEX.wallFactor,)}`,
              `${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,
              `${MODEL_OPTION}${NEURALWATT_DEEPSEEK_FLEX.model}`,
              '--runs-folder=/runs',
            ],);
            expect(given.addedOptions,).toContain(`${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,);
          },
        },),
        it({
          name: 'rewrites the provider and model the caller named when the run takes the first free pool',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: [`${PROVIDER_OPTION}${NEURALWATT_STANDARD.provider}`, `${MODEL_OPTION}${NEURALWATT_STANDARD.model}`, '--name=a',],
              pool: OPENCODE_GO,
              movesTheRun: true,
              runsFolder: '/runs',
            },).words,).toEqual([
              `${PROVIDER_OPTION}${OPENCODE_GO.provider}`,
              `${MODEL_OPTION}${OPENCODE_GO.model}`,
              '--name=a',
              '--runs-folder=/runs',
            ],);
          },
        },),
        it({
          name: 'leaves every word alone when the run keeps the pool it named',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: [`${PROVIDER_OPTION}${NEURALWATT_KIMI_FLEX.provider}`, '--name=a', '--wall=60',],
              pool: NEURALWATT_KIMI_FLEX,
              movesTheRun: false,
              wallSeconds: 60,
              runsFolder: '/runs',
            },).words,).toEqual([
              `${PROVIDER_OPTION}${NEURALWATT_KIMI_FLEX.provider}`,
              '--name=a',
              '--wall=60',
              '--runs-folder=/runs',
            ],);
          },
        },),
        it({
          name: 'turns a resumed run folder into its session file, and does not add the folder the caller already named',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: ['--name=a', '--session-of=copy-route-4', '--runs-folder=/elsewhere',],
              movesTheRun: false,
              sessionFile: '/sessions/4.jsonl',
              runsFolder: '/runs',
            },).words,).toEqual([
              '--name=a',
              '--session=/sessions/4.jsonl',
              '--runs-folder=/elsewhere',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: gateOutcomeOf.name,
      children: [
        it({
          name: 'answers the counts when asked for them alone, and needs no name for that',
          fn: async () => {
            /**
             What the runs going now hold of each pool, which is nothing.
             */
            const runs: readonly GoingRun[] = [];
            /**
             What the gate answered a caller that named no run.
             */
            const outcome: GateOutcome = await gateOutcomeOf({
              options: gateOptions({ countsOnly: true, }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('counts',);
            expect(outcome.lines,).toEqual([`${summaryTextOf(talliesOf({ runs, },),)}; ${freeSpaceTextOf(roomToSpare(),)}`,],);
            expectCountsLineNamesEachPool({
              line: (outcome.lines[0] ?? '').split('; ',)[0] ?? '',
              runs,
            },);
          },
        },),
        it({
          name: 'answers the counts even when a run of that name is going, because a count names no run',
          fn: async () => {
            /**
             A run of the name the gate was asked about.
             */
            const runs = [aRun('copy-route', inPool(QWEN_PLAN_DEEPSEEK,),),];
            /**
             What the gate answered while that run was going.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                countsOnly: true,
                name: 'copy-route',
              },),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('counts',);
          },
        },),
        it({
          name: 'answers the counts with the room the build volume has, even while that room is under the floor, because reading the counts starts nothing',
          fn: async () => {
            /**
             What the volume holds while it is under the floor.
             */
            const room = roomUnderTheFloor();
            /**
             What the gate answered while the volume was that full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ countsOnly: true, }),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: room,
            },);
            expect(outcome.action,).toBe('counts',);
            expect(outcome.lines,).toEqual([`${summaryTextOf(talliesOf({ runs: [], },),)}; ${freeSpaceTextOf(room,)}`,],);
          },
        },),
        it({
          name: 'refuses to start anything without a run to name',
          fn: async () => {
            try {
              await gateOutcomeOf({
                options: gateOptions({},),
                runs: [],
                tallies: talliesOf({ runs: [], },),
                sessionFileOf: aSessionFile,
                freeSpace: roomToSpare(),
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(PoolGateOptionsError,);
              expect(String(error,),).toContain('--name is required',);
              return;
            }
            throw new Error('a gate with no run name answered instead of refusing',);
          },
        },),
        it({
          name: 'answers that the run is already going, with the status the retrying start reads as no retry',
          fn: async () => {
            /**
             A run of the name the gate was asked about.
             */
            const runs = [aRun('copy-route', inPool(QWEN_PLAN_DEEPSEEK,),),];
            /**
             What the gate answered about that name.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('already-running',);
            expect((outcome.action === 'already-running') ? outcome.status : 0,).toBe(ALREADY_RUNNING_EXIT_STATUS,);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: copy-route',],);
          },
        },),
        it({
          name: 'answers that the run is already going even while the volume is under the floor, because that answer needs no room',
          fn: async () => {
            /**
             A run of the name the gate was asked about.
             */
            const runs = [aRun('copy-route', inPool(QWEN_PLAN_DEEPSEEK,),),];
            /**
             What the gate answered about that name while the volume was under the floor.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomUnderTheFloor(),
            },);
            expect(outcome.action,).toBe('already-running',);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: copy-route',],);
          },
        },),
        it({
          name: 'refuses to start a run while the volume that holds the repository is under the floor, saying how much room there is and how much a run needs',
          fn: async () => {
            /**
             What the volume holds while it is under the floor.
             */
            const room = roomUnderTheFloor();
            /**
             What the gate answered while the volume was that full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: room,
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines,).toEqual([
              `NO SLOT: ${freeSpaceTextOf(room,)}, under the floor of ${freeSpaceFloorTextOf()}; not started: copy-route`,
            ],);
          },
        },),
        it({
          name: 'starts a second run of an audit name, because an audit holds no slot to conflict with',
          fn: async () => {
            /**
             An audit of the name the gate was asked about.
             */
            const runs = [aRun('sol-audit-1',),];
            /**
             What the gate answered about that audit name.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'sol-audit-1', }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
          },
        },),
        it({
          name: 'answers no slot with the status the retrying start waits on',
          fn: async () => {
            /**
             Every pool a run that names a default model could take, filled.
             */
            const filled = runsFilling(openDefaultPools(),);
            /**
             What the gate answered while every one of them was full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: filled,
              tallies: talliesOf({ runs: filled, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines[0],).toContain(`NO SLOT: ${summaryTextOf(talliesOf({ runs: filled, },),)}`,);
            expect(outcome.lines[0],).toContain('not started: copy-route',);
          },
        },),
        it({
          name: 'starts a run on its own provider when no pool holds that provider, saying so',
          fn: async () => {
            /**
             What the gate answered for a Kimi run on Synthetic.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'kimi-review-round-9',
                provider: 'synthetic',
                model: 'moonshotai/Kimi-K3.1-Pro',
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.lines[0],).toContain('goes on its own provider',);
          },
        },),
        it({
          name: 'reads the session file of the run a run resumes, and hands it to the launcher',
          fn: async () => {
            /**
             What the gate answered for a run that resumes another.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route-2',
                sessionOfRunFolder: 'copy-route-1',
                commandLine: ['--name=copy-route-2', '--session-of=copy-route-1',],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain('--session=/sessions/a.jsonl',);
          },
        },),
        it({
          name: 'starts a resume on the first pool with room when the pool the run resumes on is closed, naming that pool in the launcher words',
          fn: async () => {
            expect(poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },),).toBe(true,);
            /**
             What the gate answered for a run that resumes a run on the closed pool.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route-2',
                provider: NEURALWATT_STANDARD.provider,
                model: NEURALWATT_STANDARD.model,
                sessionOfRunFolder: 'copy-route-1',
                commandLine: [
                  '--name=copy-route-2',
                  `${PROVIDER_OPTION}${NEURALWATT_STANDARD.provider}`,
                  `${MODEL_OPTION}${NEURALWATT_STANDARD.model}`,
                  '--session-of=copy-route-1',
                ],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain(
              `${PROVIDER_OPTION}${theFirstDefaultPool().provider}`,
            );
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain(
              `${MODEL_OPTION}${theFirstDefaultPool().model}`,
            );
          },
        },),
      ],
    },),
  ],
},);

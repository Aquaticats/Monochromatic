/**
 What a run on one of Pi's virtual selections holds, and what the driver answers for it: no slot in any pool, a start whatever the
 members' own counts read, and a line that names the members its router may take.

 The owner, 2026-11-26, asked how the driver's pool gate counts a run on the virtual writer, answered "No count (Recommended)": the
 gate counts only the runs pinned to one pool, and a run on `--provider=rate-limit-fallback --model=writer`, or on the reviewer
 selection the same way, holds no slot in any member pool, because the router moves a refused request on by itself
 (`package/pi-plugin/rate-limit-fallback/README.md`, "Order and refusal policy").

 Everything here is read from the pools, the counts and the runs a test hands over, so no unit, no process, no systemd and no disk
 of this machine is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BYTES_IN_A_GIBIBYTE,
  FREE_SPACE_FLOOR_BYTES,
  type FreeSpace,
  type GateOutcome,
  type GoingRun,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_STANDARD,
  NO_SLOT_EXIT_STATUS,
  NO_VIRTUAL_POOL,
  OPENCODE_GO,
  PROVIDER_POOLS,
  QWEN_PLAN_DEEPSEEK,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
  type PoolGateOptions,
  type PoolMembership,
  type ProviderPool,
  VIRTUAL_POOLS,
  VIRTUAL_REVIEWER,
  VIRTUAL_WRITER,
  type VirtualPool,
  gateOutcomeOf,
  membershipOfSelection,
  poolChoiceFor,
  poolHasRoom,
  poolOfARunNamingNoProvider,
  poolOfExecStart,
  poolsWithTheseLimits,
  roomIn,
  summaryTextOf,
  talliesOf,
  virtualPoolNamed,
} from '../dist/final/node/index.mjs';

/**
 The volume every room in this file is on, which is no volume of this machine.
 */
const TEST_VOLUME = '/a-build-volume-of-the-test';

/**
 The provider both virtual selections belong to, which is the plugin's own name for itself.
 */
const VIRTUAL_PROVIDER = 'rate-limit-fallback';

/**
 The folder a gate in this file is asked about, which holds no run.
 */
const A_RUNS_FOLDER = '/runs';

/**
 The names of a selection's member pools, in the selection's own order.

 @param pools - the pools to name

 @returns one name per pool

 @example
 ```ts
 const names = namesOf(VIRTUAL_WRITER.memberPools);
 ```
 */
function namesOf(pools: readonly ProviderPool[],): readonly string[] {
  return pools.map(function nameOf(pool,) {
    return pool.name;
  },);
}

/**
 The largest wall factor of a set of pools, read from those pools rather than written down here.

 @param pools - the pools to read

 @returns the largest factor

 @example
 ```ts
 const factor = largestWallFactorOf(VIRTUAL_WRITER.memberPools);
 ```
 */
function largestWallFactorOf(pools: readonly ProviderPool[],): number {
  return pools.reduce(function largerOf(larger, pool,) {
    return Math.max(
      larger,
      pool.wallFactor,
    );
  }, 1,);
}

/**
 The pools a test counts against, at the limits it set.

 @param limits - the limit each named pool is given, by the pool's name

 @returns the pools, in the owner's order

 @example
 ```ts
 const pools = poolsAt(new Map([[OPENCODE_GO.name, 0]]));
 ```
 */
function poolsAt(limits: ReadonlyMap<string, number>,): readonly ProviderPool[] {
  return poolsWithTheseLimits({ limits, },);
}

/**
 One run going now, which its own command line places on a virtual selection.

 @param name - the run's name

 @param virtual - the selection its command line names

 @returns the run

 @example
 ```ts
 const run = aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER);
 ```
 */
function aRunOnAVirtualSelection(name: string, virtual: VirtualPool,): GoingRun {
  return {
    name,
    // What a run naming that selection holds, read the way the driver reads a run's own command line.
    membership: membershipOfSelection({
      provider: virtual.provider,
      model: virtual.model,
    },),
  };
}

/**
 Runs on one virtual selection, named after it so a failure prints which count is wrong.

 @param virtual - the selection they name

 @param count - how many runs to make

 @returns the runs

 @example
 ```ts
 const runs = virtualRunsGoing(VIRTUAL_WRITER, 40);
 ```
 */
function virtualRunsGoing(virtual: VirtualPool, count: number,): readonly GoingRun[] {
  return Array.from(
    { length: count, },
    function named(_unused, index,) {
      return aRunOnAVirtualSelection(
        `${virtual.model}-${String(index,)}`,
        virtual,
      );
    },
  );
}

/**
 One run going now, in the pool a test names.

 @param name - the run's name

 @param pool - the pool its command line names

 @returns the run

 @example
 ```ts
 const run = aRunIn('another', OPENCODE_GO);
 ```
 */
function aRunIn(name: string, pool: ProviderPool,): GoingRun {
  /**
   The membership of a run whose own command line names one pool.
   */
  const membership: PoolMembership = {
    holds: 'a-pool',
    pool,
  };
  return {
    name,
    membership,
  };
}

/**
 Runs of one pool, named after their pool so a failure prints which count is wrong.

 @param pool - the pool to fill

 @param count - how many runs to make

 @returns the runs

 @example
 ```ts
 const runs = runsIn(OPENCODE_GO, OPENCODE_GO.limit);
 ```
 */
function runsIn(pool: ProviderPool, count: number,): readonly GoingRun[] {
  return Array.from(
    { length: count, },
    function named(_unused, index,) {
      return aRunIn(
        `${pool.name}-${String(index,)}`,
        pool,
      );
    },
  );
}

/**
 Room on the build volume well above the floor, so a gate in a test starts a run.

 @returns the room

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
 The pool gate's options for a run that names the virtual writer.

 @param extra - the words this test adds to the run's own

 @returns the options

 @example
 ```ts
 const options = gateOptionsForTheVirtualWriter({ wallSeconds: 600 });
 ```
 */
function gateOptionsForTheVirtualWriter({
  wallSeconds,
}: {
  readonly wallSeconds: number;
},): PoolGateOptions {
  return {
    runsFolder: A_RUNS_FOLDER,
    countsOnly: false,
    name: 'copy-route',
    provider: VIRTUAL_WRITER.provider,
    model: VIRTUAL_WRITER.model,
    wallSeconds,
    commandLine: [
      '--name=copy-route',
      `--provider=${VIRTUAL_WRITER.provider}`,
      `--model=${VIRTUAL_WRITER.model}`,
      `--wall=${String(wallSeconds,)}`,
    ],
  };
}

/**
 A session file reader the gate of this file never calls, because no run here resumes one.

 @returns the file

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs: [], tallies, sessionFileOf: aSessionFile, freeSpace: roomToSpare() });
 ```
 */
async function aSessionFile(): Promise<string> {
  return '/sessions/a.jsonl';
}

await describe({
  name: '',
  children: [
    describe({
      name: virtualPoolNamed.name,
      children: [
        it({
          name: 'reads the virtual writer and the virtual reviewer, and answers that a pair naming neither is no selection at all',
          fn: async () => {
            expect(virtualPoolNamed({
              provider: VIRTUAL_PROVIDER,
              model: VIRTUAL_WRITER.model,
            },),).toEqual(VIRTUAL_WRITER,);
            expect(virtualPoolNamed({
              provider: VIRTUAL_PROVIDER,
              model: VIRTUAL_REVIEWER.model,
            },),).toEqual(VIRTUAL_REVIEWER,);
            expect(virtualPoolNamed({
              provider: OPENCODE_GO.provider,
              model: OPENCODE_GO.model,
            },),).toBe(NO_VIRTUAL_POOL,);
          },
        },),
      ],
    },),
    describe({
      name: 'the member pools of each virtual selection',
      children: [
        it({
          name: 'names the owner\'s three writer pools in the owner\'s order, and no pool the owner closed and no provider named deepseek',
          fn: async () => {
            // `doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The writers' order": "Qwen,
            // opencode-go, flex (Recommended)".
            expect(namesOf(VIRTUAL_WRITER.memberPools,),).toEqual([
              QWEN_PLAN_DEEPSEEK.name,
              OPENCODE_GO.name,
              NEURALWATT_DEEPSEEK_FLEX.name,
            ],);
            for (const selection of VIRTUAL_POOLS) {
              for (const pool of selection.memberPools) {
                expect(pool,).not.toEqual(NEURALWATT_STANDARD,);
                expect(pool.provider,).not.toBe('deepseek',);
                expect(PROVIDER_POOLS,).toContain(pool,);
              }
            }
          },
        },),
        it({
          name: 'starts the reviewer at the qwen plan\'s DeepSeek pool as the owner ordered, and lists no pool for Synthetic, which holds no slot here',
          fn: async () => {
            expect(VIRTUAL_REVIEWER.memberPools[0],).toEqual(QWEN_PLAN_DEEPSEEK,);
            // Synthetic is a reviewer member and belongs to no pool of this table, so it holds no slot: the three pools below are
            // the ones a reviewer run is counted against.
            expect(namesOf(VIRTUAL_REVIEWER.memberPools,),).toEqual([
              QWEN_PLAN_DEEPSEEK.name,
              OPENCODE_GO.name,
              NEURALWATT_DEEPSEEK_FLEX.name,
            ],);
            expect(VIRTUAL_POOLS.length,).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: membershipOfSelection.name,
      children: [
        it({
          name: 'holds a run that names the virtual writer in no pool, naming the members its router may take, and a run that names one pool in that pool alone',
          fn: async () => {
            expect(membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
            },),).toEqual({
              holds: 'a-virtual-selection',
              virtual: VIRTUAL_WRITER,
              pools: [
                QWEN_PLAN_DEEPSEEK,
                OPENCODE_GO,
                NEURALWATT_DEEPSEEK_FLEX,
              ],
            },);
            expect(membershipOfSelection({
              provider: OPENCODE_GO.provider,
              model: OPENCODE_GO.model,
            },),).toEqual({
              holds: 'a-pool',
              pool: OPENCODE_GO,
            },);
            expect(membershipOfSelection({
              provider: 'synthetic',
              model: 'moonshotai/Kimi-K3.1-Pro',
            },),).toEqual({ holds: 'no-pool', },);
          },
        },),
        it({
          name: 'reads the same selection off a unit that names it, which is how a run is placed once its launcher is gone',
          fn: async () => {
            expect(poolOfExecStart({
              shown: `{ path=node ; argv[]=node drive-pi-subagent.ts --provider=${VIRTUAL_WRITER.provider} --model=${VIRTUAL_WRITER.model} ; }`,
            },),).toEqual(membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
            },),);
          },
        },),
      ],
    },),
    describe({
      name: talliesOf.name,
      children: [
        it({
          name: 'counts one run on the virtual writer in no pool, and counts a run pinned to one pool there alone',
          fn: async () => {
            /**
             One run on the virtual writer, and one run pinned to the first pool of its order.
             */
            const runs = [
              aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER,),
              aRunIn('another-writer', QWEN_PLAN_DEEPSEEK,),
            ];
            /**
             What those runs hold of each pool.
             */
            const tallies = talliesOf({ runs, },);
            // Only the pinned run holds a slot: the virtual run is in no pool, so it is counted in none of them.
            expect(tallies.counts.get(QWEN_PLAN_DEEPSEEK.name,),).toBe(1,);
            expect(tallies.counts.get(OPENCODE_GO.name,),).toBe(0,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(0,);
            expect(tallies.counts.get(NEURALWATT_STANDARD.name,),).toBe(0,);
            expect(tallies.counts.get('qwen-token-plan',),).toBe(0,);
            // The line names every pool once and says nothing about the selection, which holds no slot to show.
            expect(summaryTextOf(tallies,).split(', ',).length,).toBe(PROVIDER_POOLS.length,);
            expect(summaryTextOf(tallies,),).not.toContain(VIRTUAL_PROVIDER,);
          },
        },),
        it({
          name: 'counts no run on the virtual writer in any pool, however many of them are going',
          fn: async () => {
            /**
             A run on the virtual writer in every slot the members have together, which the gate counts in none of them.
             */
            const runs = virtualRunsGoing(VIRTUAL_WRITER, VIRTUAL_WRITER.memberPools
              .reduce(function roomOf(sum, pool,) {
                return sum + roomIn(pool,);
              }, 0,),);
            /**
             What those runs hold of each pool.
             */
            const tallies = talliesOf({ runs, },);
            for (const pool of PROVIDER_POOLS) {
              expect(tallies.counts.get(pool.name,),).toBe(0,);
            }
          },
        },),
      ],
    },),
    describe({
      name: poolChoiceFor.name,
      children: [
        it({
          name: 'takes the virtual selection itself with every pool empty, and gives the run the largest wall factor of its members',
          fn: async () => {
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
              },),
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({
              destination: 'a-virtual-pool',
              pools: [
                QWEN_PLAN_DEEPSEEK,
                OPENCODE_GO,
                NEURALWATT_DEEPSEEK_FLEX,
              ],
              wallFactor: largestWallFactorOf(VIRTUAL_WRITER.memberPools,),
            },);
          },
        },),
        it({
          name: 'takes the virtual writer whatever the members\' own counts read, because a run on it holds no slot in any of them',
          fn: async () => {
            /**
             Every slot of every member's own room, taken by runs pinned to it.
             */
            const runs = VIRTUAL_WRITER.memberPools
              .flatMap(function everySlot(pool,) {
                return runsIn(
                  pool,
                  pool.limit,
                );
              },);
            /**
             What the gate answers for a run on the virtual writer then.
             */
            const choice = poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
              },),
              tallies: talliesOf({ runs, },),
              resumes: false,
            },);
            expect(choice,).toEqual({
              destination: 'a-virtual-pool',
              pools: VIRTUAL_WRITER.memberPools,
              wallFactor: largestWallFactorOf(VIRTUAL_WRITER.memberPools,),
            },);
            // The reviewer selection follows the same rule.
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_REVIEWER.provider,
                model: VIRTUAL_REVIEWER.model,
              },),
              tallies: talliesOf({ runs, },),
              resumes: false,
            },),).toEqual({
              destination: 'a-virtual-pool',
              pools: VIRTUAL_REVIEWER.memberPools,
              wallFactor: largestWallFactorOf(VIRTUAL_REVIEWER.memberPools,),
            },);
          },
        },),
        it({
          name: 'takes the virtual writer while a caller has closed a member, and while it has closed every member, because no member\'s room is counted',
          fn: async () => {
            /**
             The shipped table with the last pool of the writer order closed by a caller.
             */
            const oneClosed = poolsAt(new Map([[NEURALWATT_DEEPSEEK_FLEX.name, 0],]),);
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
                pools: oneClosed,
              },),
              tallies: talliesOf({
                runs: [],
                pools: oneClosed,
              },),
              resumes: false,
            },),).toMatchObject({ destination: 'a-virtual-pool', },);
            /**
             The shipped table with every member closed by a caller.
             */
            const allClosed = poolsAt(new Map([
              [QWEN_PLAN_DEEPSEEK.name, 0,],
              [OPENCODE_GO.name, 0,],
              [NEURALWATT_DEEPSEEK_FLEX.name, 0,],
            ],),);
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
                pools: allClosed,
              },),
              tallies: talliesOf({
                runs: [],
                pools: allClosed,
              },),
              resumes: false,
            },),).toMatchObject({ destination: 'a-virtual-pool', },);
          },
        },),
        it({
          name: 'leaves a pool\'s own room to the runs pinned to it, so runs on the virtual writer neither fill a pool nor keep one from taking a pinned run',
          fn: async () => {
            /**
             Only the flex pool open, so the pool's own room is the only question a pinned run asks.
             */
            const pools = poolsAt(new Map([
              [QWEN_PLAN_DEEPSEEK.name, 0,],
              [OPENCODE_GO.name, 0,],
              [NEURALWATT_DEEPSEEK_FLEX.name, 2,],
            ],),);
            /**
             More runs on the virtual writer than every member takes together.
             */
            const tallies = talliesOf({
              runs: virtualRunsGoing(VIRTUAL_WRITER, 40,),
              pools,
            },);
            /**
             The flex pool, as the caller's table holds it.
             */
            const flex = pools
              .find(function isTheFlexPool(pool,) {
                return pool.name === NEURALWATT_DEEPSEEK_FLEX.name;
              },);
            expect(flex,).toBeDefined();
            if (flex === undefined) {
              return;
            }
            // The pool's own count is nothing, because a run on the virtual writer holds no slot in it.
            expect(tallies.counts.get(flex.name,),).toBe(0,);
            expect(poolHasRoom({
              tallies,
              pool: flex,
            },),).toBe(true,);
            // Once the pool's own room is taken by runs pinned to it, it has none left.
            expect(poolHasRoom({
              tallies: talliesOf({
                runs: [
                  ...virtualRunsGoing(VIRTUAL_WRITER, 40,),
                  ...runsIn(flex, roomIn(flex,),),
                ],
                pools,
              },),
              pool: flex,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'refuses to place a virtual selection as a run that names no provider, because such a run keeps the pair it named',
          fn: async () => {
            expect(function placesAVirtualSelection() {
              poolOfARunNamingNoProvider({
                requested: membershipOfSelection({
                  provider: VIRTUAL_WRITER.provider,
                  model: VIRTUAL_WRITER.model,
                },),
                resumes: false,
                tallies: talliesOf({ runs: [], },),
              },);
            },).toThrow('virtual selection',);
          },
        },),
      ],
    },),
    describe({
      name: gateOutcomeOf.name,
      children: [
        it({
          name: 'starts a run on the virtual writer while the first pool of its order is full, with the pair it named and no slot taken in any pool',
          fn: async () => {
            /**
             The wall clock the caller asked for.
             */
            const wallSeconds = 600;
            /**
             Every slot of the first member's own room, taken by runs pinned to it.
             */
            const runs = runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit,);
            /**
             What the gate answered.
             */
            const outcome: GateOutcome = await gateOutcomeOf({
              options: gateOptionsForTheVirtualWriter({ wallSeconds, },),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            // The line names the selection and every pool its router may take, so a reader sees where a refusal moves the request.
            expect(outcome.lines[0],).toContain(`${VIRTUAL_WRITER.provider}/${VIRTUAL_WRITER.model}`,);
            for (const pool of VIRTUAL_WRITER.memberPools) {
              expect(outcome.lines[0],).toContain(pool.name,);
            }
            // The counts the line carries are those of the runs pinned to each pool, which is what the gate counts.
            /**
             The counts of those pinned runs, as the line prints them.
             */
            const countsLine = summaryTextOf(talliesOf({ runs, },),);
            expect(outcome.lines[0],).toContain(countsLine,);
            /**
             The words the launcher is given.
             */
            const words = (outcome.action === 'start') ? outcome.arguments.words : [];
            // The pair the run named is kept: rewriting it into one member's pair would pin the run to that member and lose the
            // router's fallback.
            expect(words,).toContain(`--provider=${VIRTUAL_WRITER.provider}`,);
            expect(words,).toContain(`--model=${VIRTUAL_WRITER.model}`,);
            expect(words,).toContain(`--wall=${String(wallSeconds * largestWallFactorOf(VIRTUAL_WRITER.memberPools,),)}`,);
          },
        },),
        it({
          name: 'answers no slot for a run pinned to a pool whose own room is full, which is the count the gate still makes',
          fn: async () => {
            /**
             Every slot of a pool only its own model takes, held by runs pinned to it.
             */
            const runs = runsIn(QWEN_PLAN_QWEN4_MAX_PREVIEW, QWEN_PLAN_QWEN4_MAX_PREVIEW.limit,);
            /**
             What the gate answered for a run that names that pool.
             */
            const outcome = await gateOutcomeOf({
              options: {
                runsFolder: A_RUNS_FOLDER,
                countsOnly: false,
                name: 'another',
                provider: QWEN_PLAN_QWEN4_MAX_PREVIEW.provider,
                model: QWEN_PLAN_QWEN4_MAX_PREVIEW.model,
                wallSeconds: 600,
                commandLine: ['--name=another',],
              },
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines[0],).toContain(`${QWEN_PLAN_QWEN4_MAX_PREVIEW.name} ${String(QWEN_PLAN_QWEN4_MAX_PREVIEW.limit,)}/${String(roomIn(QWEN_PLAN_QWEN4_MAX_PREVIEW,),)}`,);
          },
        },),
      ],
    },),
  ],
},);

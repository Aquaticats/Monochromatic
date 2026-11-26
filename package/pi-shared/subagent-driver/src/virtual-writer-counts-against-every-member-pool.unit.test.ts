/**
 What a run on Pi's virtual writer holds, and what the driver answers for it: a slot in every pool that plugin's router may send
 the run to, a start while every one of those pools has room, and a refusal that names each pool without room.

 A virtual selection is the plugin's own (`package/pi-plugin/rate-limit-fallback/README.md`): its router starts every request at
 the first member and moves the request on when a member refuses with a 429 before any output. So the run may be taking any
 member's room at its next request, and the driver counts it in all of them
 (`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The virtual writer").

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
  type PoolGateOptions,
  type PoolMembership,
  type ProviderPool,
  VIRTUAL_POOLS,
  VIRTUAL_REVIEWER,
  VIRTUAL_WRITER,
  gateOutcomeOf,
  membershipOfSelection,
  poolChoiceFor,
  poolOfARunNamingNoProvider,
  poolOfExecStart,
  poolsWithTheseLimits,
  poolsWithoutRoom,
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
 The names of the pools a selection's router may take, in the selection's own order.

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
 One run going now, which is on a virtual selection.

 @param name - the run's name

 @param virtual - the selection its command line names

 @returns the run

 @example
 ```ts
 const run = aRunOnTheVirtualWriter('copy-route');
 ```
 */
function aRunOnAVirtualSelection(name: string, virtual: typeof VIRTUAL_WRITER,): GoingRun {
  /**
   What a run naming that selection holds, read the way the driver reads a run's own command line.
   */
  const membership = membershipOfSelection({
    provider: virtual.provider,
    model: virtual.model,
  },);
  expect(membership.holds,).toBe('some-pools',);
  return {
    name,
    membership,
  };
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
          name: 'holds a run that names the virtual writer in every pool its router may take, and a run that names one pool in that pool alone',
          fn: async () => {
            expect(membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
            },),).toEqual({
              holds: 'some-pools',
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
          name: 'reads the same selection off a unit that names it, which is how a run is counted once its launcher is gone',
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
          name: 'counts one run on the virtual writer once in every pool its router may take, and in no other pool',
          fn: async () => {
            /**
             One run on the virtual writer, and one run on a pool no member of it shares.
             */
            const runs = [
              aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER,),
              aRunIn('another-review', QWEN_PLAN_DEEPSEEK,),
            ];
            /**
             What those runs hold of each pool.
             */
            const tallies = talliesOf({ runs, },);
            expect(tallies.counts.get(QWEN_PLAN_DEEPSEEK.name,),).toBe(2,);
            expect(tallies.counts.get(OPENCODE_GO.name,),).toBe(1,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(1,);
            // A pool no member shares, and one only a run naming its own model takes, hold nothing.
            expect(tallies.counts.get(NEURALWATT_STANDARD.name,),).toBe(0,);
            expect(tallies.counts.get('qwen-token-plan',),).toBe(0,);
            // The line a coordinator reads names every pool, so the slot a virtual run holds is visible in each of them.
            expect(summaryTextOf(tallies,),).toContain(`${OPENCODE_GO.name} 1/${String(roomIn(OPENCODE_GO,),)}`,);
          },
        },),
      ],
    },),
    describe({
      name: poolChoiceFor.name,
      children: [
        it({
          name: 'takes the virtual selection itself while every pool its router may take has room, and gives the run the largest wall factor of those pools',
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
          name: 'takes no run on the virtual writer once the first pool its router may take is full, so that no pool takes more runs than its room when a 429 moves the run there',
          fn: async () => {
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
              },),
              tallies: talliesOf({ runs: runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit,), },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
          },
        },),
        it({
          name: 'takes no run on the virtual writer once any member pool is full, and names every one of them when all are',
          fn: async () => {
            /**
             What the virtual writer's router may take, as the driver reads it.
             */
            const requested = membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
            },);
            /**
             Every member pool filled.
             */
            const tallies = talliesOf({
              runs: VIRTUAL_WRITER.memberPools.flatMap(function everySlot(pool,) {
                return runsIn(
                  pool,
                  pool.limit,
                );
              },),
            },);
            expect(poolsWithoutRoom({
              tallies,
              membership: requested,
            },),).toEqual(VIRTUAL_WRITER.memberPools,);
            expect(poolChoiceFor({
              requested,
              tallies,
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
            // One member with room is not enough: a run holds a slot in the others, and one of them would be over its room.
            const oneFull = talliesOf({ runs: runsIn(NEURALWATT_DEEPSEEK_FLEX, NEURALWATT_DEEPSEEK_FLEX.limit,), },);
            expect(poolsWithoutRoom({
              tallies: oneFull,
              membership: requested,
            },),).toEqual([NEURALWATT_DEEPSEEK_FLEX,],);
          },
        },),
        it({
          name: 'counts the pools of a virtual selection as a caller\'s limits leave them, so a pool closed for one command takes no run of it',
          fn: async () => {
            /**
             The shipped table with the last pool of the writer order closed by a caller.
             */
            const pools = poolsAt(new Map([[NEURALWATT_DEEPSEEK_FLEX.name, 0],]),);
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
                pools,
              },),
              tallies: talliesOf({
                runs: [],
                pools,
              },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
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
          name: 'starts a run on the virtual writer with the pair it named and every member pool\'s room, giving it the largest wall factor of those pools',
          fn: async () => {
            /**
             The wall clock the caller asked for.
             */
            const wallSeconds = 600;
            /**
             What the gate answered.
             */
            const outcome: GateOutcome = await gateOutcomeOf({
              options: gateOptionsForTheVirtualWriter({ wallSeconds, },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            // The line names the selection and every pool its router may take, so a reader sees what the run holds.
            expect(outcome.lines[0],).toContain(`${VIRTUAL_WRITER.provider}/${VIRTUAL_WRITER.model}`,);
            for (const pool of VIRTUAL_WRITER.memberPools) {
              expect(outcome.lines[0],).toContain(pool.name,);
            }
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
          name: 'answers no slot with the counts when one pool the virtual writer may take has no room, and starts nothing',
          fn: async () => {
            /**
             Every slot of the last pool of the writer order, taken.
             */
            const runs = runsIn(NEURALWATT_DEEPSEEK_FLEX, NEURALWATT_DEEPSEEK_FLEX.limit,);
            /**
             What the gate answered while that pool was full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptionsForTheVirtualWriter({ wallSeconds: 600, },),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            /**
             The counts line the gate printed beside its refusal.
             */
            const counts = summaryTextOf(talliesOf({ runs, },),);
            expect(outcome.lines[0],).toContain(counts,);
          },
        },),
      ],
    },),
  ],
},);

/**
 What the runs going now hold of each provider pool, and which pool a run may take: the counts, the summary line, and the answer
 the pool gate gives. Everything here is read from the runs and the pools a test hands over, so no unit, no process and no
 systemd is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALREADY_RUNNING_EXIT_STATUS,
  type GateOutcome,
  type GoingRun,
  type LauncherArguments,
  MODEL_OPTION,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_KIMI_FLEX,
  NEURALWATT_STANDARD,
  NO_POOL_HAS_ROOM,
  NO_SLOT_EXIT_STATUS,
  OPENCODE_GO,
  POOL_OF_A_RUN_NAMING_NO_POOL,
  PoolGateOptionsError,
  PROVIDER_OPTION,
  type PoolGateOptions,
  type PoolMembership,
  firstDefaultPoolWithRoom,
  gateOutcomeOf,
  launcherArgumentsOf,
  membershipOf,
  parsePoolGateOptions,
  poolChoiceFor,
  poolHasRoom,
  poolNamed,
  summaryTextOf,
  talliesOf,
} from '../dist/final/node/index.mjs';

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
 const inStandard = inPool(NEURALWATT_STANDARD);
 ```
 */
function inPool(pool: typeof NEURALWATT_STANDARD,): PoolMembership {
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
 const runs = runsIn(NEURALWATT_STANDARD, 20);
 ```
 */
function runsIn(pool: typeof NEURALWATT_STANDARD, count: number,): readonly GoingRun[] {
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
 const outcome = await gateOutcomeOf({ options, runs: [], tallies, sessionFileOf: aSessionFile, });
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
              provider: 'neuralwatt',
              model: 'deepseek-v4.1-flash',
            },),).toEqual(inPool(NEURALWATT_STANDARD,),);
            expect(poolNamed({
              provider: 'neuralwatt',
              model: 'kimi-k3.1-pro-flex',
            },),).toEqual(inPool(NEURALWATT_KIMI_FLEX,),);
            expect(poolNamed({
              provider: 'synthetic',
              model: 'moonshotai/Kimi-K3.1-Pro',
            },),).toEqual({ inAPool: false, },);
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
             Two runs on the standard pool and one outside every pool.
             */
            const tallies = talliesOf({
              runs: [
                ...runsIn(NEURALWATT_STANDARD, 2,),
                aRun('sol-audit-1',),
              ],
            },);
            expect(tallies.counts.get('neuralwatt',),).toBe(2,);
            expect(tallies.counts.get('opencode-go',),).toBe(0,);
            expect(summaryTextOf(tallies,),).toBe('neuralwatt 2/20, neuralwatt-flex 0/22, opencode-go 0/20, neuralwatt-kimi-flex 0/22',);
          },
        },),
      ],
    },),
    describe({
      name: poolHasRoom.name,
      children: [
        it({
          name: 'says a pool has room while it is one below its limit, and none once it is at it',
          fn: async () => {
            /**
             The standard pool one run short of its limit.
             */
            const nearlyFull = talliesOf({ runs: runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit - 1,), },);
            expect(poolHasRoom({
              tallies: nearlyFull,
              pool: NEURALWATT_STANDARD,
            },),).toBe(true,);
            /**
             The standard pool at its limit.
             */
            const full = talliesOf({ runs: runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit,), },);
            expect(poolHasRoom({
              tallies: full,
              pool: NEURALWATT_STANDARD,
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: firstDefaultPoolWithRoom.name,
      children: [
        it({
          name: 'takes the owner order: standard Neuralwatt, then its flex pool, then opencode-go, and never the Kimi pool',
          fn: async () => {
            expect(firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: [], },), },),).toEqual(NEURALWATT_STANDARD,);
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({ runs: runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit,), },),
            },),).toEqual(NEURALWATT_DEEPSEEK_FLEX,);
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({
                runs: [
                  ...runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit,),
                  ...runsIn(NEURALWATT_DEEPSEEK_FLEX, NEURALWATT_DEEPSEEK_FLEX.limit,),
                ],
              },),
            },),).toEqual(OPENCODE_GO,);
          },
        },),
        it({
          name: 'answers that no pool has room once every pool that takes a default run is full',
          fn: async () => {
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({
                runs: [
                  ...runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit,),
                  ...runsIn(NEURALWATT_DEEPSEEK_FLEX, NEURALWATT_DEEPSEEK_FLEX.limit,),
                  ...runsIn(OPENCODE_GO, OPENCODE_GO.limit,),
                ],
              },),
            },),).toBe(NO_POOL_HAS_ROOM,);
          },
        },),
      ],
    },),
    describe({
      name: poolChoiceFor.name,
      children: [
        it({
          name: 'moves a run that names a default model to the first pool with room, and says it was moved',
          fn: async () => {
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_STANDARD,),
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({
              destination: 'a-pool',
              pool: NEURALWATT_STANDARD,
              movesTheRun: true,
            },);
          },
        },),
        it({
          name: 'keeps a run that resumes a session on the pool it named, with no wall factor',
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
            const filled = talliesOf({
              runs: [
                ...runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit,),
                ...runsIn(NEURALWATT_DEEPSEEK_FLEX, NEURALWATT_DEEPSEEK_FLEX.limit,),
                ...runsIn(OPENCODE_GO, OPENCODE_GO.limit,),
              ],
            },);
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
              '--wall=1200',
              `${PROVIDER_OPTION}neuralwatt`,
              `${MODEL_OPTION}deepseek-v4.1-flash-flex`,
              '--runs-folder=/runs',
            ],);
            expect(given.addedOptions,).toContain(`${PROVIDER_OPTION}neuralwatt`,);
          },
        },),
        it({
          name: 'rewrites the provider and model the caller named when the run takes the first free pool',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: [`${PROVIDER_OPTION}neuralwatt`, `${MODEL_OPTION}deepseek-v4.1-flash`, '--name=a',],
              pool: OPENCODE_GO,
              movesTheRun: true,
              runsFolder: '/runs',
            },).words,).toEqual([
              `${PROVIDER_OPTION}opencode-go`,
              `${MODEL_OPTION}deepseek-v4.1-flash`,
              '--name=a',
              '--runs-folder=/runs',
            ],);
          },
        },),
        it({
          name: 'leaves every word alone when the run keeps the pool it named',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: [`${PROVIDER_OPTION}neuralwatt`, '--name=a', '--wall=60',],
              pool: NEURALWATT_KIMI_FLEX,
              movesTheRun: false,
              wallSeconds: 60,
              runsFolder: '/runs',
            },).words,).toEqual([
              `${PROVIDER_OPTION}neuralwatt`,
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
             What the gate answered a caller that named no run.
             */
            const outcome: GateOutcome = await gateOutcomeOf({
              options: gateOptions({ countsOnly: true, }),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
            },);
            expect(outcome.action,).toBe('counts',);
            expect(outcome.lines,).toEqual(['neuralwatt 0/20, neuralwatt-flex 0/22, opencode-go 0/20, neuralwatt-kimi-flex 0/22',],);
          },
        },),
        it({
          name: 'answers the counts even when a run of that name is going, because a count names no run',
          fn: async () => {
            /**
             What the gate answered while a run called copy-route was going.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                countsOnly: true,
                name: 'copy-route',
              },),
              runs: [aRun('copy-route', inPool(NEURALWATT_STANDARD,),),],
              tallies: talliesOf({ runs: [aRun('copy-route', inPool(NEURALWATT_STANDARD,),),], },),
              sessionFileOf: aSessionFile,
            },);
            expect(outcome.action,).toBe('counts',);
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
             What the gate answered about a name that is going.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: [aRun('copy-route', inPool(NEURALWATT_STANDARD,),),],
              tallies: talliesOf({ runs: [aRun('copy-route', inPool(NEURALWATT_STANDARD,),),], },),
              sessionFileOf: aSessionFile,
            },);
            expect(outcome.action,).toBe('already-running',);
            expect((outcome.action === 'already-running') ? outcome.status : 0,).toBe(ALREADY_RUNNING_EXIT_STATUS,);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: copy-route',],);
          },
        },),
        it({
          name: 'starts a second run of an audit name, because an audit holds no slot to conflict with',
          fn: async () => {
            /**
             What the gate answered about an audit name that is going.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'sol-audit-1', }),
              runs: [aRun('sol-audit-1',),],
              tallies: talliesOf({ runs: [aRun('sol-audit-1',),], },),
              sessionFileOf: aSessionFile,
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
            const filled = [
              ...runsIn(NEURALWATT_STANDARD, NEURALWATT_STANDARD.limit,),
              ...runsIn(NEURALWATT_DEEPSEEK_FLEX, NEURALWATT_DEEPSEEK_FLEX.limit,),
              ...runsIn(OPENCODE_GO, OPENCODE_GO.limit,),
            ];
            /**
             What the gate answered while every one of them was full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: filled,
              tallies: talliesOf({ runs: filled, },),
              sessionFileOf: aSessionFile,
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines[0],).toContain('NO SLOT:',);
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
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain('--session=/sessions/a.jsonl',);
          },
        },),
      ],
    },),
  ],
},);

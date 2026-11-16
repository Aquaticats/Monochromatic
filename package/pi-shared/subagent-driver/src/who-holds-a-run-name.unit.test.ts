/**
 Who holds a run's name: the unit the run is going as, the launcher that has not made that unit yet, and never a launcher whose run
 has already ended.

 The process folder and the manager are seams: a test writes the process entries it describes and answers `systemctl` itself, so no
 unit, worktree or Pi run is started.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  ALREADY_RUNNING_EXIT_STATUS,
  type CommandOutput,
  type FreeSpace,
  FREE_SPACE_FLOOR_BYTES,
  type GoingRun,
  LAUNCHER_ENTRY_NAMES,
  type PoolGateOptions,
  type PoolTallies,
  PROVIDER_POOLS,
  gateOutcomeOf,
  goingRunsNow,
  heldTextOf,
  talliesOf,
  unitFactsIn,
} from '../dist/final/node/index.mjs';

/**
 Room on the build volume well above its floor, so nothing here is refused for want of disk.
 */
const ROOM_TO_SPARE: FreeSpace = {
  volume: '/a-build-volume-of-the-test',
  freeBytes: FREE_SPACE_FLOOR_BYTES + (1_024 ** 3),
};

/**
 One answer from `systemctl`.

 @param status - the status the command ended with

 @param stdout - what it printed

 @returns the answer

 @example
 ```ts
 const answer = answered(0, 'active\n');
 ```
 */
function answered(status: number, stdout = '',): CommandOutput {
  return {
    status,
    stdout,
    stderr: '',
  };
}

/**
 A folder a test may write in, with a process folder and a runs folder, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The process folder.
   */
  readonly processes: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, with a process folder and a runs folder, removed when the test ends.

 @returns the folder

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-holder-',),);
  /**
   The process folder.
   */
  const processes = join(path, 'proc',);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  await mkdir(processes, { recursive: true, },);
  await mkdir(runs, { recursive: true, },);
  return {
    path,
    processes,
    runs,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one process into a process folder.

 @param folder - the process folder

 @param pid - the process's number

 @param words - its command line, each word whole

 @example
 ```ts
 await writeProcess({ folder: processes, pid: '21', words: ['/usr/bin/node', '/pkg/src/start-pi-run-in-a-worktree.ts', '--name=a'], });
 ```
 */
async function writeProcess({
  folder,
  pid,
  words,
}: {
  readonly folder: string;
  readonly pid: string;
  readonly words: readonly string[];
},): Promise<void> {
  await mkdir(join(folder, pid,), { recursive: true, },);
  await writeFile(join(folder, pid, 'cmdline',), `${words.join('\0',)}\0`,);
}

/**
 Writes one finished run's folder and summary.

 @param runs - the folder the runs live in

 @param name - the run's name

 @example
 ```ts
 await writeFinishedRun({ runs, name: 'a', });
 ```
 */
async function writeFinishedRun({
  runs,
  name,
}: {
  readonly runs: string;
  readonly name: string;
},): Promise<string> {
  /**
   The summary's path, which a test may give a time of its own.
   */
  const summary = join(
    runs,
    `${name}-1`,
    'summary.json',
  );
  await mkdir(join(
    runs,
    `${name}-1`,
  ), { recursive: true, },);
  await writeFile(
    summary,
    '{"endReason":"idle"}\n',
  );
  return summary;
}

/**
 Answers `systemctl` for the units and the launchers a test describes.

 @param facts - what `systemctl show` prints for each unit, by the unit's name

 @param units - what `systemctl list-units` prints

 @returns how one command is run, and every question it was asked

 @example
 ```ts
 const { run, asked } = manager({ facts: new Map(), units: '' });
 ```
 */
function manager({
  facts,
  units,
}: {
  readonly facts: ReadonlyMap<string, string>;
  readonly units: string;
},): {
  readonly run: (request: { readonly command: string; readonly words: readonly string[] }) => Promise<CommandOutput>;
  readonly asked: readonly string[];
} {
  /**
   Every question asked of the manager.
   */
  const asked: string[] = [];
  return {
    asked,
    run: async function systemctl({ words, },) {
      asked.push(words.join(' ',),);
      if (words.includes('list-units',)) {
        return answered(
          0,
          units,
        );
      }
      /**
       The unit the question is about.
       */
      const unitName = words[2] ?? '';
      return answered(
        0,
        `${facts.get(unitName,) ?? 'LoadState=not-found\nActiveState=inactive\n'}\n`,
      );
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: unitFactsIn.name,
      children: [
        it({
          name: 'reads one entry per property line, and nothing for a line that is not one',
          fn: async () => {
            expect(unitFactsIn({
              shown: 'LoadState=loaded\nActiveState=active\nActiveEnterTimestamp=Sat 2026-11-14 09:42:51 EST\n',
            },),).toEqual({
              LoadState: 'loaded',
              ActiveState: 'active',
              ActiveEnterTimestamp: 'Sat 2026-11-14 09:42:51 EST',
            },);
            expect(unitFactsIn({ shown: 'not a property\n', },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: heldTextOf.name,
      children: [
        it({
          name: 'names the unit or the launcher that holds a run name and since when, and says nothing when the holder is unknown',
          fn: async () => {
            expect(heldTextOf({
              run: {
                name: 'a',
                membership: { holds: 'no-pool', },
                held: {
                  kind: 'unit',
                  unit: 'pi-a.service',
                  state: 'active',
                  since: 'Sat 2026-11-14 09:42:51 EST',
                },
              },
            },),).toBe(', held since Sat 2026-11-14 09:42:51 EST by unit pi-a.service (active)',);
            expect(heldTextOf({
              run: {
                name: 'a',
                membership: { holds: 'no-pool', },
                held: {
                  kind: 'launcher',
                  pid: 1_234,
                },
              },
            },),).toBe(', held by launcher process 1234',);
            expect(heldTextOf({
              run: {
                name: 'a',
                membership: { holds: 'no-pool', },
              },
            },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: goingRunsNow.name,
      children: [
        it({
          name: 'counts a launcher whose name holds only the summary of an earlier attempt, which says nothing about this launcher',
          fn: async () => {
            await using folder = await makeFolder();
            await writeProcess({
              folder: folder.processes,
              pid: '31',
              words: [
                '/usr/bin/node',
                `/pkg/src/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`,
                '--name=a',
              ],
            },);
            /**
             The summary of the attempt that had ended before this launcher began.
             */
            const summary = await writeFinishedRun({
              runs: folder.runs,
              name: 'a',
            },);
            /**
             A moment an hour before the launcher began, which is all that summary can speak for.
             */
            const anHourBefore = new Date(Date.now() - 3_600_000,);
            await utimes(
              summary,
              anHourBefore,
              anHourBefore,
            );
            /**
             What the manager answers.
             */
            const systemd = manager({
              facts: new Map([['pi-a.service', 'LoadState=loaded\nActiveState=inactive\n',],],),
              units: '',
            },);
            /**
             The runs going now.
             */
            const going = await goingRunsNow({
              run: systemd.run,
              processFolder: folder.processes,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
              onProblem: function onProblem(problem,) {
                throw new Error(`nothing was expected to be a problem: ${problem}`);
              },
            },);
            expect(going.map(function nameOf(run,) {
              return run.name;
            },),).toEqual(['a',],);
          },
        },),
        it({
          name: 'counts a launcher whose run has ended, because it may still resume that run and the wake path and the count cannot disagree',
          fn: async () => {
            await using folder = await makeFolder();
            await writeProcess({
              folder: folder.processes,
              pid: '21',
              words: [
                '/usr/bin/node',
                `/pkg/src/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`,
                '--name=a',
              ],
            },);
            /**
             The moment that launcher began, put a minute back so the summary its own run wrote is strictly later.
             */
            const aMinuteBefore = new Date(Date.now() - 60_000,);
            await utimes(
              join(
                folder.processes,
                '21',
              ),
              aMinuteBefore,
              aMinuteBefore,
            );
            await writeFinishedRun({
              runs: folder.runs,
              name: 'a',
            },);
            /**
             What the manager answers, and what it was asked.
             */
            const systemd = manager({
              facts: new Map([['pi-a.service', 'LoadState=loaded\nActiveState=inactive\n',],],),
              units: '',
            },);
            /**
             The runs going now.
             */
            const going = await goingRunsNow({
              run: systemd.run,
              processFolder: folder.processes,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
              onProblem: function onProblem(problem,) {
                throw new Error(`nothing was expected to be a problem: ${problem}`);
              },
            },);
            // A launcher that outlived its run is waiting for work the run left, or removing its worktree, or about to resume
            // that run. Until it exits it is the holder of the name: dropping it lets a replacement start while its own wake path
            // resumes the same run.
            expect(going.map(function nameOf(run,) {
              return run.name;
            },),).toEqual(['a',],);
            expect(going[0]?.held,).toMatchObject({
              kind: 'launcher',
              pid: 21,
            },);
            expect(systemd.asked,).toContain('--user show pi-a.service --property=LoadState --property=ActiveState --property=ActiveEnterTimestamp',);
          },
        },),
        it({
          name: 'counts a launcher that has not made its unit yet, with the moment it began as what holds the name',
          fn: async () => {
            await using folder = await makeFolder();
            await writeProcess({
              folder: folder.processes,
              pid: '22',
              words: [
                '/usr/bin/node',
                `/pkg/src/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`,
                '--name=b',
              ],
            },);
            /**
             What the manager answers.
             */
            const systemd = manager({
              facts: new Map(),
              units: '',
            },);
            /**
             The runs going now.
             */
            const going = await goingRunsNow({
              run: systemd.run,
              processFolder: folder.processes,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
              onProblem: () => {},
            },);
            expect(going.map(function nameOf(run,) {
              return run.name;
            },),).toEqual(['b',],);
            expect(going[0]?.held,).toMatchObject({
              kind: 'launcher',
              pid: 22,
            },);
          },
        },),
        it({
          name: 'reads a launcher whose unit is going as held by that unit, and the run is counted once',
          fn: async () => {
            await using folder = await makeFolder();
            await writeProcess({
              folder: folder.processes,
              pid: '23',
              words: [
                '/usr/bin/node',
                `/pkg/src/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`,
                '--name=c',
              ],
            },);
            /**
             What the manager answers.
             */
            const systemd = manager({
              facts: new Map([[
                'pi-c.service',
                'LoadState=loaded\nActiveState=active\nActiveEnterTimestamp=Sat 2026-11-14 09:42:51 EST\nExecStart={ argv[]=node drive.ts --provider=x --model=y ; }\n',
              ],],),
              units: 'pi-c.service x\n',
            },);
            /**
             The runs going now.
             */
            const going = await goingRunsNow({
              run: systemd.run,
              processFolder: folder.processes,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
              onProblem: () => {},
            },);
            expect(going,).toHaveLength(1,);
            expect(going[0]?.held,).toMatchObject({
              kind: 'unit',
              unit: 'pi-c.service',
              state: 'active',
              since: 'Sat 2026-11-14 09:42:51 EST',
            },);
          },
        },),
      ],
    },),
    describe({
      name: gateOutcomeOf.name,
      children: [
        it({
          name: 'refuses a name that is going with what holds it and since when',
          fn: async () => {
            /**
             The runs going now, and the pool each holds.
             */
            const runs: readonly GoingRun[] = [{
              name: 'a',
              membership: {
                holds: 'a-pool',
                pool: PROVIDER_POOLS[0] ?? (function noPool(): never {
                  throw new Error('the pool table is empty, so this test cannot name a pool',);
                })(),
              },
              held: {
                kind: 'launcher',
                pid: 4_321,
                since: '2026-11-14T14:41:18.000Z',
              },
            },];
            /**
             What the gate was asked.
             */
            const options: PoolGateOptions = {
              countsOnly: false,
              name: 'a',
              commandLine: ['node', 'start-pi-run-when-a-pool-has-room.ts', '--name=a',],
              runsFolder: '/runs',
            };
            /**
             How many runs each pool holds.
             */
            const tallies: PoolTallies = talliesOf({ runs, },);
            /**
             What the gate answered.
             */
            const outcome = await gateOutcomeOf({
              options,
              runs,
              tallies,
              freeSpace: ROOM_TO_SPARE,
              sessionFileOf: async function sessionFileOf() {
                return '/sessions/a.jsonl';
              },
            },);
            if (outcome.action !== 'already-running') {
              throw new Error(`the gate answered ${outcome.action} for a name that is going`,);
            }
            expect(outcome.status,).toBe(ALREADY_RUNNING_EXIT_STATUS,);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: a, held since 2026-11-14T14:41:18.000Z by launcher process 4321',],);
          },
        },),
      ],
    },),
  ],
},);

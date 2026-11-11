/**
 The run-management commands, against the machines and folders a test hands them: which pool a run goes to, which starts count,
 how a run's turn cap is sized, what a queued start answers, what a batch, a report, a note and a stop file do, and the Kimi
 review list with its fallback when the provider rate-limits one.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
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
  type BatchEntry,
  BatchListError,
  briefOf,
  commandLinesUnderProcessFolder,
  countsOfPools,
  countsSummaryOf,
  fallbackAfterARateLimit,
  isReviewed,
  isWrapperWork,
  KIMI_FALLBACK_MODEL,
  KIMI_FALLBACK_PROVIDER,
  KimiReviewListError,
  latestReportsOf,
  launcherIn,
  launchersOf,
  NO_LAUNCHER,
  NO_POOL,
  NO_SLOT_EXIT_STATUS,
  notesToRuns,
  NOT_A_LAUNCHER,
  NO_WORKTREE_BRANCH,
  parseBatchList,
  parseKimiReviewList,
  parseNamedReportOptions,
  parseRunEndOptions,
  PI_POOLS,
  recentRunEndsOf,
  reportOfANamedRun,
  RunEndOptionsError,
  poolNamed,
  poolWithRoom,
  retryAStartUntilASlotFrees,
  runCommandWithChildProcess,
  type RunUnderAPool,
  sizedTurnCap,
  startABatch,
  startAPiRun,
  STOP_MESSAGE,
  stopRunsNotOnTheWrapper,
  type StartPiRunOptions,
  wallSecondsInPool,
} from '../dist/final/node/index.mjs';

/**
 The pool this test treats as the only one a default run may move into first.
 */
const [STANDARD,] = PI_POOLS;

/**
 A pool of a slower provider, which doubles a run's wall clock.
 */
const [, FLEX,] = PI_POOLS;

/**
 What a fake `systemctl`, `git` and `mise` answer with, so no real unit, repository or install is touched.
 */
const NOTHING_ASKED = '';

/**
 A scratch folder for one test, removed when the test ends.
 */
async function scratch(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  const path = await mkdtemp(join(
    scratchRoot(),
    'pi-run-management-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 What a start was asked for, as the tests want it.
 */
function optionsOf({
  runsFolder,
  repository,
  name,
  branch,
  provider,
  model,
}: {
  readonly runsFolder: string;
  readonly repository: string;
  readonly name: string;
  readonly branch: string;
  readonly provider: string;
  readonly model: string;
}): StartPiRunOptions {
  return {
    name,
    branch,
    promptFile: join(
      runsFolder,
      `${name}-prompt.md`,
    ),
    run: 1,
    provider,
    model,
    cpuPercent: 800,
    memory: '12G',
    tasks: 3_000,
    wallSeconds: 600,
    idleSeconds: 120,
    runsFolder,
    repository,
    driverEntry: '/repo/package/pi-shared/subagent-driver/src/drive-pi-subagent.ts',
    turnCap: 8,
  };
}

await describe({
  name: 'run management',
  children: [
    //region The pools a run is counted in
    describe({
      name: countsOfPools.name,
      children: [
        it({
          name: 'counts a run with no start in flight in the pool the coordinator fills by hand',
          fn: async () => {
            const runs: readonly RunUnderAPool[] = [{ name: 'copy-route', launcherPool: NO_LAUNCHER, },];
            expect(countsOfPools({ runs, },).get('opencode-go',),).toBe(1,);
          },
        },),
        it({
          name: 'leaves an audit and a Kimi run on Synthetic out of every pool, and counts a Kimi run its start named',
          fn: async () => {
            const runs: readonly RunUnderAPool[] = [
              { name: 'sol-audit', launcherPool: NO_LAUNCHER, },
              { name: 'kimi-review-a', launcherPool: NO_LAUNCHER, },
              { name: 'kimi-flex-a', launcherPool: PI_POOLS[3] ?? STANDARD, },
            ];
            const counts = countsOfPools({ runs, },);
            expect(counts.get('opencode-go',),).toBe(0,);
            expect(counts.get('neuralwatt-kimi-flex',),).toBe(1,);
          },
        },),
        it({
          name: 'writes every pool with its limit, in the owner\'s order',
          fn: async () => {
            const counts = countsOfPools({ runs: [], },);
            expect(countsSummaryOf({ counts, },),).toBe(
              PI_POOLS.map(function asShown(pool,) {
                return `${pool.name} 0/${String(pool.limit,)}`;
              },)
                .join(', ',),
            );
          },
        },),
      ],
    },),
    describe({
      name: poolWithRoom.name,
      children: [
        it({
          name: 'moves a default run to the first pool with room, in the owner\'s order',
          fn: async () => {
            const counts = new Map<string, number>([
              ['neuralwatt', 20,],
              ['neuralwatt-flex', 22,],
              ['opencode-go', 0,],
            ],);
            expect(poolWithRoom({
              requested: STANDARD ?? PI_POOLS[0],
              resumes: false,
              counts,
            },),).toBe(PI_POOLS[2],);
          },
        },),
        it({
          name: 'keeps a resumed run in the pool it names',
          fn: async () => {
            const counts = new Map<string, number>();
            expect(poolWithRoom({
              requested: FLEX ?? STANDARD,
              resumes: true,
              counts,
            },),).toBe(FLEX,);
          },
        },),
        it({
          name: 'answers no pool when every pool a default run may take is full, and keeps a pool only a named run takes',
          fn: async () => {
            const full = new Map<string, number>(PI_POOLS.map(function atItsLimit(pool,) {
              return [pool.name, pool.limit,] as const;
            },),);
            expect(poolWithRoom({
              requested: STANDARD ?? PI_POOLS[0],
              resumes: false,
              counts: full,
            },),).toBe(NO_POOL,);
            expect(poolWithRoom({
              requested: PI_POOLS[3] ?? STANDARD,
              resumes: false,
              counts: full,
            },),).toBe(PI_POOLS[3],);
          },
        },),
        it({
          name: 'answers the pool a provider and model pair names, and no pool for a model outside the table',
          fn: async () => {
            expect(poolNamed({
              provider: 'synthetic',
              model: 'hf:moonshotai/Kimi-K3.1-Pro',
            },),).toBe(NO_POOL,);
            expect(poolNamed({
              provider: 'neuralwatt',
              model: 'deepseek-v4.1-flash',
            },),).toBe(PI_POOLS[0],);
          },
        },),
        it({
          name: 'doubles the wall clock of a run that went to the slower pool',
          fn: async () => {
            expect(wallSecondsInPool({
              pool: FLEX ?? STANDARD,
              wallSeconds: 600,
            },),).toBe(1_200,);
          },
        },),
      ],
    },),
    //endregion

    //region Starts in flight
    describe({
      name: launcherIn.name,
      children: [
        it({
          name: 'reads a start of a run and the pool its command line names',
          fn: async () => {
            expect(launcherIn({
              process: {
                processId: '7',
                commandLine: [
                  'node',
                  '/repo/src/start-pi-run-in-a-free-slot.ts',
                  '--name=copy-route',
                  '--provider=neuralwatt',
                  '--model=deepseek-v4.1-flash-flex',
                ],
              },
              commandName: 'start-pi-run-in-a-free-slot',
            },),).toEqual({
              name: 'copy-route',
              pool: FLEX,
            },);
          },
        },),
        it({
          name: 'leaves another command, and a start that names no run, out',
          fn: async () => {
            expect(launcherIn({
              process: { processId: '7', commandLine: ['node', '/repo/src/other.ts', '--name=a',], },
              commandName: 'start-pi-run-in-a-free-slot',
            },),).toBe(NOT_A_LAUNCHER,);
            expect(launcherIn({
              process: { processId: '7', commandLine: ['node', '/repo/src/start-pi-run-in-a-free-slot.ts',], },
              commandName: 'start-pi-run-in-a-free-slot',
            },),).toBe(NOT_A_LAUNCHER,);
          },
        },),
      ],
    },),
    describe({
      name: launchersOf.name,
      children: [
        it({
          name: 'keeps every start but this process, which must never count itself',
          fn: async () => {
            const launchers = launchersOf({
              processes: [
                { processId: '9', commandLine: ['node', '/a/start-pi-run-in-a-free-slot.ts', '--name=mine',], },
                { processId: '8', commandLine: ['node', '/a/start-pi-run-in-a-free-slot.ts', '--name=yours',], },
              ],
              commandName: 'start-pi-run-in-a-free-slot',
              ownProcessId: '8',
            },);
            expect([...launchers.keys(),],).toEqual(['mine',],);
          },
        },),
      ],
    },),
    describe({
      name: commandLinesUnderProcessFolder.name,
      children: [
        it({
          name: 'reads a process table, leaving out an entry that is no process number and one whose command line is gone',
          fn: async () => {
            await using folder = await scratch();
            await mkdir(join(folder.path, '12',), { recursive: true, },);
            await writeFile(
              join(
                folder.path,
                '12',
                'cmdline',
              ),
              `${['node', '/a/start.ts', '--name=a',].join('\u{0}',)  }\u{0}`,
            );
            await mkdir(join(folder.path, '13',), { recursive: true, },);
            await writeFile(
              join(folder.path, 'not-a-number',),
              NOTHING_ASKED,
            );
            const processes = await commandLinesUnderProcessFolder({
              folder: folder.path,
              ownProcessId: '99',
            },);
            expect(processes.map(function idOf(process,) {
              return process.processId;
            },),).toEqual(['12',],);
            expect(processes[0]?.commandLine,).toContain('--name=a',);
          },
        },),
      ],
    },),
    //endregion

    //region Turn caps
    describe({
      name: sizedTurnCap.name,
      children: [
        it({
          name: 'sizes a cap a quarter above the longest finished run of the same kind',
          fn: async () => {
            await using folder = await scratch();
            await writeRun({
              runsFolder: folder.path,
              name: 'copy-route-1',
              summary: { endReason: 'idle', turns: 40, },
            },);
            await writeRun({
              runsFolder: folder.path,
              name: 'other-audit-1',
              summary: { endReason: 'idle', turns: 400, },
            },);
            const sized = await sizedTurnCap({
              runsFolder: folder.path,
              runName: 'copy-route',
            },);
            expect(sized.cap,).toBe(500,);
            expect(sized.longest,).toBe(400,);
            expect(sized.sampleKind,).toBe('every kind',);
          },
        },),
        it({
          name: 'reads every kind when too few of one kind have finished, and leaves out a run that did not end on its own terms',
          fn: async () => {
            await using folder = await scratch();
            await writeRun({
              runsFolder: folder.path,
              name: 'sol-a-1',
              summary: { endReason: 'idle', turns: 80, },
            },);
            await writeRun({
              runsFolder: folder.path,
              name: 'sol-b-1',
              summary: { endReason: 'model-error', turns: 800, },
            },);
            const sized = await sizedTurnCap({
              runsFolder: folder.path,
              runName: 'sol-c',
            },);
            expect(sized.cap,).toBe(100,);
            expect(sized.sampleSize,).toBe(1,);
          },
        },),
        it({
          name: 'refuses when no finished run says how many turns the work takes',
          fn: async () => {
            await using folder = await scratch();
            let caught: unknown;
            try {
              await sizedTurnCap({
                runsFolder: folder.path,
                runName: 'copy-route',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('--turn-cap',);
          },
        },),
      ],
    },),
    //endregion

    //region One start
    describe({
      name: startAPiRun.name,
      children: [
        it({
          name: 'refuses with the no-slot status, naming every pool, when the pools are full',
          fn: async () => {
            await using folder = await scratch();
            const outcome = await startAPiRun({
              options: optionsOf({
                runsFolder: folder.path,
                repository: folder.path,
                name: 'copy-route',
                branch: NO_WORKTREE_BRANCH,
                provider: 'neuralwatt',
                model: 'deepseek-v4.1-flash',
              },),
              run: async function nothing() {
                return { status: 0, stdout: '', stderr: '', };
              },
              runToTheEnd: async function never() {
                return { status: 0, stdout: '', stderr: '', };
              },
              launchers: async function full() {
                return new Map(PI_POOLS.flatMap(function filling(pool,) {
                  return Array.from(
                    { length: pool.limit, },
                    function nth(_unused, index,) {
                      return [`filled-${pool.name}-${String(index,)}`, pool,] as const;
                    },
                  );
                },),);
              },
              runningUnits: async function units() {
                return [];
              },
              sizeATurnCap: async function sized() {
                return {
                  cap: 8,
                  longest: 6,
                  sampleSize: 1,
                  kind: 'other',
                  sampleKind: 'every kind',
                  line: 'turn cap 8',
                };
              },
              home: folder.path,
            },);
            expect(outcome.status,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines[0],).toContain('NO SLOT',);
            expect(outcome.lines[0],).toContain('neuralwatt 20/20',);
          },
        },),
        it({
          name: 'refuses a run of a name that is already going',
          fn: async () => {
            await using folder = await scratch();
            const outcome = await startAPiRun({
              options: optionsOf({
                runsFolder: folder.path,
                repository: folder.path,
                name: 'copy-route',
                branch: NO_WORKTREE_BRANCH,
                provider: 'neuralwatt',
                model: 'deepseek-v4.1-flash',
              },),
              run: async function nothing() {
                return { status: 0, stdout: '', stderr: '', };
              },
              runToTheEnd: async function never() {
                return { status: 0, stdout: '', stderr: '', };
              },
              launchers: async function none() {
                return new Map();
              },
              runningUnits: async function units() {
                return [{ name: 'copy-route', unitName: 'pi-copy-route.service', elapsedMilliseconds: 1, mainPid: 2, },];
              },
              sizeATurnCap: async function sized() {
                return {
                  cap: 8,
                  longest: 6,
                  sampleSize: 1,
                  kind: 'other',
                  sampleKind: 'every kind',
                  line: 'turn cap 8',
                };
              },
              home: folder.path,
            },);
            expect(outcome.status,).toBe(4,);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: copy-route',],);
          },
        },),
        it({
          name: 'starts the unit of a run with a free slot, doubling a flex run\'s wall clock, and answers with the unit\'s status',
          fn: async () => {
            await using folder = await scratch();
            const asked: string[][] = [];
            const outcome = await startAPiRun({
              options: optionsOf({
                runsFolder: folder.path,
                repository: folder.path,
                name: 'copy-route',
                branch: NO_WORKTREE_BRANCH,
                provider: 'neuralwatt',
                model: 'deepseek-v4.1-flash',
              },),
              run: async function answers(request,) {
                asked.push([request.command, ...request.words,],);
                if (request.command === 'mise') {
                  return { status: 0, stdout: 'node\n', stderr: '', };
                }
                return { status: 0, stdout: '', stderr: '', };
              },
              runToTheEnd: async function unit(request,) {
                asked.push([request.command, ...request.words,],);
                return { status: 7, stdout: '', stderr: '', };
              },
              launchers: async function standardFull() {
                return new Map(Array.from(
                  { length: (STANDARD ?? PI_POOLS[0])?.limit ?? 20, },
                  function nth(_unused, index,) {
                    return [`filled-${String(index,)}`, STANDARD ?? PI_POOLS[0],] as const;
                  },
                ),);
              },
              runningUnits: async function units() {
                return [];
              },
              sizeATurnCap: async function sized() {
                return {
                  cap: 8,
                  longest: 6,
                  sampleSize: 1,
                  kind: 'other',
                  sampleKind: 'every kind',
                  line: 'turn cap 8: longest of 1 finished runs',
                };
              },
              home: folder.path,
            },);
            expect(outcome.status,).toBe(7,);
            const unit = asked.find(function isTheUnit(words,) {
              return words[0] === 'systemd-run';
            },) ?? [];
            expect(unit,).toContain('--unit=pi-copy-route',);
            expect(unit,).toContain('--wall-seconds=1200',);
            expect(unit,).toContain('--turn-cap=8',);
            expect(outcome.lines.at(-1),).toContain('unit ended with status 7',);
          },
        },),
        it({
          name: 'says a worktree that is not empty rather than emptying it',
          fn: async () => {
            await using folder = await scratch();
            const worktree = join(
              folder.path,
              '.claude',
              'worktrees',
              'pi-copy-route',
            );
            await mkdir(worktree, { recursive: true, },);
            await writeFile(
              join(worktree, 'kept.txt',),
              'someone\'s work',
            );
            let caught: unknown;
            try {
              await startAPiRun({
                options: optionsOf({
                  runsFolder: folder.path,
                  repository: folder.path,
                  name: 'copy-route',
                  branch: 'feat/x',
                  provider: 'neuralwatt',
                  model: 'deepseek-v4.1-flash',
                },),
                run: async function nothing() {
                  return { status: 1, stdout: '', stderr: '', };
                },
                runToTheEnd: async function never() {
                  return { status: 0, stdout: '', stderr: '', };
                },
                launchers: async function none() {
                  return new Map();
                },
                runningUnits: async function units() {
                  return [];
                },
                sizeATurnCap: async function sized() {
                  return {
                    cap: 8,
                    longest: 6,
                    sampleSize: 1,
                    kind: 'other',
                    sampleKind: 'every kind',
                    line: 'turn cap 8',
                  };
                },
                home: folder.path,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toContain('is not empty',);
            expect(await readFile(join(worktree, 'kept.txt',), { encoding: 'utf8', },),).toBe('someone\'s work',);
          },
        },),
      ],
    },),
    //endregion

    //region A queue for a slot
    describe({
      name: retryAStartUntilASlotFrees.name,
      children: [
        it({
          name: 'answers the no-slot status with the last refusal and the deadline when no slot frees',
          fn: async () => {
            let clock = 0;
            const outcome = await retryAStartUntilASlotFrees({
              startOnce: async function refused() {
                return { status: NO_SLOT_EXIT_STATUS, output: 'NO SLOT: neuralwatt 20/20\n', };
              },
              now: function reads() {
                return clock;
              },
              wait: async function pass(milliseconds,) {
                clock += milliseconds;
              },
              deadlineMilliseconds: 5,
              periodMilliseconds: 1,
              describedAs: '--name=copy-route',
            },);
            expect(outcome.status,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.output,).toContain('neuralwatt 20/20',);
            expect(outcome.output,).toContain('no slot freed before the deadline: --name=copy-route',);
          },
        },),
        it({
          name: 'answers the first start that was not refused, at once',
          fn: async () => {
            let attempts = 0;
            const outcome = await retryAStartUntilASlotFrees({
              startOnce: async function tries() {
                attempts += 1;
                return (attempts < 3)
                  ? { status: NO_SLOT_EXIT_STATUS, output: 'NO SLOT\n', }
                  : { status: 0, output: 'slot free\n', };
              },
              now: function reads() {
                return 0;
              },
              wait: async function pass() {
                // A test's wait passes no time, so the queue reads its clock again at once.
              },
              deadlineMilliseconds: 10,
              periodMilliseconds: 1,
              describedAs: '--name=copy-route',
            },);
            expect(outcome.status,).toBe(0,);
            expect(outcome.output,).toBe('slot free\n',);
            expect(attempts,).toBe(3,);
          },
        },),
        it({
          name: 'answers a start that failed with its own status and output',
          fn: async () => {
            const outcome = await retryAStartUntilASlotFrees({
              startOnce: async function fails() {
                return { status: 1, output: 'git could not make the worktree\n', };
              },
              now: function reads() {
                return 0;
              },
              wait: async function pass() {
                // A test's wait passes no time, so the queue reads its clock again at once.
              },
              deadlineMilliseconds: 10,
              periodMilliseconds: 1,
              describedAs: '--name=copy-route',
            },);
            expect(outcome.status,).toBe(1,);
            expect(outcome.output,).toContain('could not make the worktree',);
          },
        },),
      ],
    },),
    //endregion

    //region A batch, a report, a note and a stop
    describe({
      name: startABatch.name,
      children: [
        it({
          name: 'starts every entry and prints one line per entry, in the list\'s order',
          fn: async () => {
            const entries: readonly BatchEntry[] = [
              { name: 'a', branch: 'feat/a', },
              { name: 'b', branch: 'feat/b', },
            ];
            const lines = await startABatch({
              entries,
              startEntry: async function startOne(entry,) {
                return {
                  name: entry.name,
                  status: (entry.name === 'b') ? 1 : 0,
                  lastLine: 'no slot freed before the deadline',
                };
              },
              wait: async function pass() {
                // A test's wait passes no time, so the queue reads its clock again at once.
              },
              staggerMilliseconds: 0,
            },);
            expect(lines,).toEqual(['a: status 0', 'b: status 1 no slot freed before the deadline',],);
          },
        },),
        it({
          name: 'refuses a list that is no array of entries, naming the entry and the field',
          fn: async () => {
            let caught: unknown;
            try {
              parseBatchList({ text: '[{"name":"a"}]', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(BatchListError,);
            expect((caught as Error).message,).toContain('--branch',);
            let notJson: unknown;
            try {
              parseBatchList({ text: 'not json', },);
            }
            catch (error) {
              notJson = error;
            }
            expect(notJson,).toBeInstanceOf(BatchListError,);
          },
        },),
      ],
    },),
    describe({
      name: latestReportsOf.name,
      children: [
        it({
          name: 'prints the newest finished run of each agent, and says when one has finished none',
          fn: async () => {
            await using folder = await scratch();
            await writeRun({
              runsFolder: folder.path,
              name: 'copy-route-1',
              summary: { endReason: 'idle', turns: 3, lastAssistantText: 'the first report', },
            },);
            await writeRun({
              runsFolder: folder.path,
              name: 'copy-route-2',
              summary: { endReason: 'idle', turns: 4, lastAssistantText: 'the second report', },
            },);
            await mkdir(join(folder.path, 'never-finished-1',), { recursive: true, },);
            const lines = await latestReportsOf({
              runsFolder: folder.path,
              names: ['copy-route', 'never-finished',],
              lines: 5,
              width: 80,
            },);
            expect(lines[0],).toBe('=== copy-route-2: idle, 4 turns',);
            expect(lines,).toContain('the second report',);
            expect(lines,).toContain('=== never-finished: no finished run',);
          },
        },),
      ],
    },),
    describe({
      name: reportOfANamedRun.name,
      children: [
        it({
          name: 'prints the named run\'s end, its line count and the lines the caller asked for, cut to the width',
          fn: async () => {
            await using folder = await scratch();
            await writeRun({
              runsFolder: folder.path,
              name: 'copy-route-3',
              summary: {
                endReason: 'model-error',
                modelError: '429 Too many requests',
                turns: 7,
                lastAssistantText: 'first line\nsecond line\nthird line which is long',
              },
            },);
            const lines = await reportOfANamedRun({
              runsFolder: folder.path,
              runFolder: 'copy-route-3',
              from: 2,
              to: 3,
              width: 10,
            },);
            expect(lines[0],).toBe('=== copy-route-3: model-error 429 Too many requests turns 7',);
            expect(lines[1],).toBe('(report has 3 lines)',);
            expect(lines.slice(2,),).toEqual(['second lin', 'third line',],);
          },
        },),
        it({
          name: 'refuses a command line with no run folder, with an unknown option, and with two counts for one option',
          fn: async () => {
            let noRun: unknown;
            try {
              parseNamedReportOptions([],);
            }
            catch (error) {
              noRun = error;
            }
            expect((noRun as Error).message,).toContain('--run',);
            let unknown: unknown;
            try {
              parseNamedReportOptions(['--run=a-1', '--wibble=1',],);
            }
            catch (error) {
              unknown = error;
            }
            expect((unknown as Error).message,).toContain('no option of this command',);
            let twice: unknown;
            try {
              parseNamedReportOptions(['--run=a-1', '--to=1', '--to=2',],);
            }
            catch (error) {
              twice = error;
            }
            expect((twice as Error).message,).toContain('takes one value',);
          },
        },),
      ],
    },),
    describe({
      name: recentRunEndsOf.name,
      children: [
        it({
          name: 'lists the runs that finished in the window, with their end, turns and whether a later attempt exists',
          fn: async () => {
            await using folder = await scratch();
            await writeRun({
              runsFolder: folder.path,
              name: 'copy-route-1',
              summary: { endReason: 'model-error', modelError: '520 from the provider', turns: 12, },
            },);
            await writeRun({
              runsFolder: folder.path,
              name: 'copy-route-2',
              summary: { endReason: 'idle', turns: 4, },
            },);
            await writeRun({
              runsFolder: folder.path,
              name: 'an-old-run-1',
              summary: { endReason: 'idle', turns: 2, },
            },);
            /**
             A moment far before the window, which the old run's summary is given.
             */
            const longAgo = new Date(Date.now() - (10 * 24 * 60 * 60_000),);
            await utimes(
              join(
                folder.path,
                'an-old-run-1',
                'summary.json',
              ),
              longAgo,
              longAgo,
            );
            await mkdir(join(folder.path, 'still-running-1',), { recursive: true, },);
            const lines = await recentRunEndsOf({
              runsFolder: folder.path,
              sinceMinutes: 40,
              endReasons: [],
              now: Date.now(),
            },);
            expect(lines.toSorted(),).toEqual([
              'copy-route-1\tmodel-error\t520 from the provider\tturns=12\tlaterRun=true',
              'copy-route-2\tidle\t\tturns=4\tlaterRun=false',
            ],);
            /**
             The folders the window leaves out and the one that never finished, which the reader must not report as ends.
             */
            const reported = lines.map(function folderOf(line,) {
              return line.split('\t',)[0];
            },);
            expect(reported,).not.toContain('an-old-run-1',);
            expect(reported,).not.toContain('still-running-1',);
          },
        },),
        it({
          name: 'keeps only the ends the caller asked for, and refuses a window that is no number above zero',
          fn: async () => {
            await using folder = await scratch();
            await writeRun({
              runsFolder: folder.path,
              name: 'a-1',
              summary: { endReason: 'model-error', turns: 1, },
            },);
            await writeRun({
              runsFolder: folder.path,
              name: 'b-1',
              summary: { endReason: 'idle', turns: 1, },
            },);
            const lines = await recentRunEndsOf({
              runsFolder: folder.path,
              sinceMinutes: 40,
              endReasons: ['model-error',],
              now: Date.now(),
            },);
            expect(lines.map(function folderOf(line,) {
              return line.split('\t',)[0];
            },),).toEqual(['a-1',],);
            let caught: unknown;
            try {
              parseRunEndOptions(['--since-minutes=zero',],);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(RunEndOptionsError,);
          },
        },),
      ],
    },),
    describe({
      name: notesToRuns.name,
      children: [
        it({
          name: 'publishes a note under a dot name and renames it, so the run reads a whole message',
          fn: async () => {
            await using folder = await scratch();
            await mkdir(join(folder.path, 'copy-route-2',), { recursive: true, },);
            const lines = await notesToRuns({
              runsFolder: folder.path,
              note: 'a correction',
              noteName: 'correction',
              runs: ['copy-route', 'no-such-run',],
              skip: [],
            },);
            expect(lines,).toEqual([
              `sent to ${join(folder.path, 'copy-route-2',)}`,
              'no run folder for no-such-run',
            ],);
            expect(
              await readdir(join(
                folder.path,
                'copy-route-2',
                'inbox',
              ),),
            ).toEqual(['correction',],);
            expect(await readFile(
              join(
                folder.path,
                'copy-route-2',
                'inbox',
                'correction',
              ),
              { encoding: 'utf8', },
            ),).toBe('a correction',);
          },
        },),
      ],
    },),
    describe({
      name: stopRunsNotOnTheWrapper.name,
      children: [
        it({
          name: 'keeps the wrapper\'s work and asks the other work to stop only when the caller asked for that',
          fn: async () => {
            await using folder = await scratch();
            await mkdir(join(folder.path, 'wrapper-run-1',), { recursive: true, },);
            const runs = [
              { name: 'wrapper-run', branch: 'feat/git-policy-cli-x', runFolder: 'wrapper-run-1', },
              { name: 'other-run', branch: 'feat/unrelated', runFolder: 'other-run-1', },
            ];
            expect(isWrapperWork({
              name: 'other-run',
              branch: 'fix/git-policy-cli-y',
            },),).toBe(true,);
            const readOnly = await stopRunsNotOnTheWrapper({
              runs,
              runsFolder: folder.path,
              apply: false,
            },);
            expect(readOnly.join('\n',),).toContain('that would be asked to stop (1)',);
            expect(
              await readdir(join(folder.path, 'wrapper-run-1',),),
            ).toEqual([],);
            await mkdir(join(folder.path, 'other-run-1',), { recursive: true, },);
            const applied = await stopRunsNotOnTheWrapper({
              runs,
              runsFolder: folder.path,
              apply: true,
            },);
            expect(applied.join('\n',),).toContain('asked to stop (1 of 1)',);
            expect(await readFile(
              join(
                folder.path,
                'other-run-1',
                'stop',
              ),
              { encoding: 'utf8', },
            ),).toBe(STOP_MESSAGE,);
          },
        },),
      ],
    },),
    //endregion

    //region A round of Kimi reviews
    describe({
      name: parseKimiReviewList.name,
      children: [
        it({
          name: 'reads a review list, and refuses one whose field is missing',
          fn: async () => {
            expect(parseKimiReviewList({
              text: '[{"name":"a","commit":"c","range":"r","focus":"f"}]',
            },),).toEqual([{ name: 'a', commit: 'c', range: 'r', focus: 'f', },],);
            let caught: unknown;
            try {
              parseKimiReviewList({ text: '[{"name":"a","commit":"c","range":"r"}]', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(KimiReviewListError,);
            expect((caught as Error).message,).toContain('--focus',);
          },
        },),
      ],
    },),
    describe({
      name: fallbackAfterARateLimit.name,
      children: [
        it({
          name: 'moves a review Synthetic rate-limited onto Neuralwatt\'s Kimi pool, resuming the same session',
          fn: async () => {
            const summary = JSON.stringify({
              endReason: 'model-error',
              modelError: '429 Too many requests for this API key',
              rateLimitErrors: 1,
              sessionStats: { sessionFile: '/sessions/big.jsonl', },
            },);
            expect(fallbackAfterARateLimit({ summaryText: summary, },),).toEqual({
              resume: true,
              session: '/sessions/big.jsonl',
              provider: KIMI_FALLBACK_PROVIDER,
              model: KIMI_FALLBACK_MODEL,
            },);
          },
        },),
        it({
          name: 'leaves a review that ended on its own terms, and one that left no summary, where it is',
          fn: async () => {
            expect(fallbackAfterARateLimit({ summaryText: JSON.stringify({ endReason: 'idle', },), },),).toEqual({ resume: false, },);
            expect(fallbackAfterARateLimit({ summaryText: JSON.stringify({
              endReason: 'model-error',
              rateLimitErrors: 0,
              sessionStats: { sessionFile: '/sessions/a.jsonl', },
            },), },),).toEqual({ resume: false, },);
          },
        },),
      ],
    },),
    describe({
      name: isReviewed.name,
      children: [
        it({
          name: 'names a source file the worktree holds, and leaves out tests and stand-ins',
          fn: async () => {
            expect(isReviewed({ path: 'a/src/b.ts', worktreeHasIt: true, },),).toBe(true,);
            expect(isReviewed({ path: 'a/src/b.test.ts', worktreeHasIt: true, },),).toBe(false,);
            expect(isReviewed({ path: 'a/src/native/c_tests.rs', worktreeHasIt: true, },),).toBe(false,);
            expect(isReviewed({ path: 'a/src/stand-in-pi.ts', worktreeHasIt: true, },),).toBe(false,);
            expect(isReviewed({ path: 'a/src/gone.ts', worktreeHasIt: false, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: briefOf.name,
      children: [
        it({
          name: 'names every file the review must read, with no cap',
          fn: async () => {
            const files = Array.from(
              { length: 45, },
              function nth(_unused, index,) {
                return `/repo/src/file-${String(index,)}.ts`;
              },
            );
            const brief = briefOf({
              review: { name: 'a', commit: 'c', range: 'r', focus: 'a wait', },
              filesToRead: files,
              runFolder: '/reviews/a',
              runsFolder: '/runs',
              reviewsFolder: '/reviews',
            },);
            for (const file of files) {
              expect(brief,).toContain(file,);
            }
            expect(brief,).toContain('Look hardest at: a wait',);
            expect(brief,).toContain('Never generate synthetic host load',);
          },
        },),
      ],
    },),
    //endregion

    //region The machinery a start asks
    describe({
      name: runCommandWithChildProcess.name,
      children: [
        it({
          name: 'kills a command that never answers instead of waiting for it',
          fn: async () => {
            const started = Date.now();
            const answer = await runCommandWithChildProcess({
              command: 'sleep',
              words: ['30',],
              timeoutMilliseconds: 100,
            },);
            expect(Date.now() - started,).toBeLessThan(30_000,);
            expect(answer.status,).toBe(-1,);
          },
        },),
      ],
    },),
    //endregion
  ],
},);

/**
 Writes one run folder with the summary a test describes.

 @param runsFolder - the folder to write into

 @param name - the run folder's name

 @param summary - the summary to write

 @example
 ```ts
 await writeRun({ runsFolder: '/runs', name: 'a-1', summary: { endReason: 'idle', turns: 3, }, });
 ```
 */
async function writeRun({
  runsFolder,
  name,
  summary,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly summary: Readonly<Record<string, unknown>>;
},): Promise<void> {
  await mkdir(join(runsFolder, name,), { recursive: true, },);
  await writeFile(
    join(
      runsFolder,
      name,
      'summary.json',
    ),
    JSON.stringify(summary,),
  );
}

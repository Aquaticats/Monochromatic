/**
 The retrying start and the batch, the command lines they read, and the readings the counting does: the runs going now from a
 process folder and a systemd listing, and the small readers of text another program wrote.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
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
  BATCH_DEFAULT_MEMORY,
  BATCH_DEFAULT_WALL_SECONDS,
  BATCH_RETRY_PERIOD_SECONDS,
  BATCH_STAGGER_MILLISECONDS,
  BatchOptionsError,
  type CommandOutput,
  GATE_OUTPUT_TAIL_CHARACTERS,
  LAUNCHER_ENTRY_NAMES,
  type PoolMembership,
  NO_SLOT_EXIT_STATUS,
  PoolGateOptionsError,
  PROVIDER_POOLS,
  poolsHeldBy,
  RetryStartOptionsError,
  batchEntriesIn,
  entryLineOf,
  firstWordOf,
  goingRunsNow,
  isDigitsOnly,
  launcherRunsIn,
  parseBatchOptions,
  parsePoolGateOptions,
  parseRetryStartOptions,
  poolOfExecStart,
  promptFileOf,
  retryStartUntilAPoolHasRoom,
  retryWordsOf,
  startBatch,
  tailKeeper,
  unitRunNamesIn,
  valueOfOption,
} from '../dist/final/node/index.mjs';

/**
 Reads the pools a run's membership names, or nothing when no pool holds it.

 @param run - the run, when a test found one

 @returns the pools' names, in the order the membership names them

 @example
 ```ts
 const pools = poolNamesOf(going[0]);
 ```
 */
function poolNamesOf(run: { readonly membership: PoolMembership },): readonly string[] {
  if (run.membership.holds === 'a-pool') {
    return [run.membership.pool.name,];
  }
  if (run.membership.holds === 'some-pools') {
    return run.membership.pools.map(function nameOf(pool,) {
      return pool.name;
    },);
  }
  return [];
}

/**
 One run of a test's own, outside every pool unless the test gives it one.

 @returns the run

 @example
 ```ts
 const run = aRunOutsideEveryPool();
 ```
 */
function aRunOutsideEveryPool(): { readonly membership: PoolMembership } {
  return { membership: { holds: 'no-pool', }, };
}

/**
 A command's answer, as a stand-in gives it.

 @param status - the status the command ended with

 @param stdout - what it printed

 @param stderr - what it complained about

 @returns the answer

 @example
 ```ts
 const answered = answered(3, 'NO SLOT');
 ```
 */
function answered(status: number, stdout = '', stderr = '',): CommandOutput {
  return {
    status,
    stdout,
    stderr,
  };
}

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-retry-batch-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one process into a stand-in process folder.

 @param folder - the process folder

 @param pid - the process's number

 @param words - the words the kernel split its command line into

 @example
 ```ts
 await writeProcess({ folder: '/proc', pid: '42', words: ['node', 'launcher.ts'], });
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
}): Promise<void> {
  await mkdir(join(folder, pid,), { recursive: true, },);
  await writeFile(join(folder, pid, 'cmdline',), `${words.join('\0',)}\0`,);
}

await describe({
  name: '',
  children: [
    describe({
      name: retryStartUntilAPoolHasRoom.name,
      children: [
        it({
          name: 'answers with what the gate said the first time a pool had room',
          fn: async () => {
            /**
             What the gate answered.
             */
            const retried = await retryStartUntilAPoolHasRoom({
              attempt: async function started() {
                return answered(0, 'slot free: neuralwatt 3/20',);
              },
              now: Date.now,
              wait: async function noWait() {
                // A slot was free on the first ask.
              },
              deadline: Date.now() + 1_000,
              periodMilliseconds: 10,
              words: ['--name=a',],
            },);
            expect(retried.status,).toBe(0,);
            expect(retried.text,).toBe('slot free: neuralwatt 3/20',);
          },
        },),
        it({
          name: 'keeps asking while no slot is free, and answers with the attempt that started the run',
          fn: async () => {
            /**
             How many times the gate was asked.
             */
            let asked = 0;
            /**
             What asking for a slot ended with.
             */
            const retried = await retryStartUntilAPoolHasRoom({
              attempt: async function starting() {
                asked += 1;
                return (asked < 3) ? answered(NO_SLOT_EXIT_STATUS, 'NO SLOT: full',) : answered(0, 'slot free',);
              },
              now: Date.now,
              wait: async function noWait() {
                // The wait passes without spending time.
              },
              deadline: Date.now() + 1_000,
              periodMilliseconds: 10,
              words: ['--name=a',],
            },);
            expect(asked,).toBe(3,);
            expect(retried.status,).toBe(0,);
          },
        },),
        it({
          name: 'ends at its deadline saying so, with the status the caller waits on',
          fn: async () => {
            /**
             When the wait is asked the time, which is past the deadline after the first ask.
             */
            let now = 0;
            /**
             What asking for a slot ended with.
             */
            const retried = await retryStartUntilAPoolHasRoom({
              attempt: async function noSlot() {
                now += 1_000;
                return answered(NO_SLOT_EXIT_STATUS, 'NO SLOT: full',);
              },
              now: function clock() {
                return now;
              },
              wait: async function noWait() {
                // The deadline is already past, so no wait is taken.
              },
              deadline: 500,
              periodMilliseconds: 10,
              words: [
                '--name=a',
                '--branch=feat/a',
              ],
            },);
            expect(retried.status,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(retried.text,).toBe('no slot freed before the deadline: --name=a --branch=feat/a\n',);
          },
        },),
        it({
          name: 'keeps the end of what the gate printed, where the reason it started nothing is',
          fn: async () => {
            /**
             What the gate printed, longer than the tail a caller reads.
             */
            const long = 'x'.repeat(GATE_OUTPUT_TAIL_CHARACTERS,);
            /**
             What asking for a slot ended with.
             */
            const retried = await retryStartUntilAPoolHasRoom({
              attempt: async function failed() {
                return answered(1, long, 'and the reason');
              },
              now: Date.now,
              wait: async function noWait() {
                // The first ask already answered something other than no slot.
              },
              deadline: Date.now() + 1_000,
              periodMilliseconds: 10,
              words: [],
            },);
            expect(retried.status,).toBe(1,);
            expect(retried.text,).toHaveLength(GATE_OUTPUT_TAIL_CHARACTERS,);
            expect(retried.text,).toContain('and the reason',);
          },
        },),
      ],
    },),
    describe({
      name: startBatch.name,
      children: [
        it({
          name: 'starts one entry at a time, staggered, and answers one line per entry in the list order',
          fn: async () => {
            /**
             Every start the batch made, in order, with the milliseconds it waited before the next.
             */
            const starts: string[] = [];
            /**
             Every wait the batch took.
             */
            const waits: number[] = [];
            /**
             The lines the batch answered, and the entries it says failed.
             */
            const { lines, failed, } = await startBatch({
              entries: [
                {
                  name: 'a',
                  branch: 'feat/a',
                },
                {
                  name: 'b',
                  branch: 'feat/b',
                },
              ],
              runsFolder: '/runs',
              deadlineSeconds: 7_200,
              start: async function startOne({ name, },) {
                starts.push(name,);
                return {
                  status: 0,
                  output: '',
                };
              },
              wait: async function passTime(milliseconds,) {
                waits.push(milliseconds,);
              },
            },);
            expect(starts,).toEqual(['a', 'b',],);
            expect(waits,).toEqual([BATCH_STAGGER_MILLISECONDS, BATCH_STAGGER_MILLISECONDS,],);
            expect(lines,).toEqual(['a: status 0', 'b: status 0',],);
            expect(failed,).toEqual([],);
          },
        },),
        it({
          name: 'keeps the whole of what a failed start printed, and names that file on the entry line',
          fn: async () => {
            /**
             Every output the batch asked to keep.
             */
            const kept: { readonly name: string; readonly output: string }[] = [];
            /**
             Where the batch says it kept them.
             */
            const keptAt = '/runs/a-start-failure.log';
            const { lines, failed, } = await startBatch({
              entries: [
                {
                  name: 'a',
                  branch: 'feat/a',
                },
                {
                  name: 'b',
                  branch: 'feat/b',
                },
              ],
              runsFolder: '/runs',
              deadlineSeconds: 7_200,
              start: async function startOne({ name, },) {
                return (name === 'a')
                  ? {
                    status: 1,
                    output: 'Error: it did not start\n    at nowhere\nNode.js v26.13.0\n',
                  }
                  : {
                    status: 0,
                    output: '',
                  };
              },
              wait: async function passTime() {
                // The stagger is not what this test reads.
              },
              keepFailure: async function keep({ name, output, },) {
                kept.push({
                  name,
                  output,
                },);
                return { keptAt, };
              },
            },);
            expect(kept.map(function nameKept(entry,) {
              return entry.name;
            },),).toEqual(['a',],);
            expect(lines,).toEqual([
              `a: status 1 Error: it did not start (all it printed: ${keptAt})`,
              'b: status 0',
            ],);
            expect(failed,).toEqual(['a',],);
          },
        },),
        it({
          name: 'names every entry whose start did not end with status 0, including one that never started',
          fn: async () => {
            const { failed, } = await startBatch({
              entries: [
                {
                  name: 'a',
                  branch: 'feat/a',
                },
                {
                  name: 'b',
                  branch: 'feat/b',
                },
                {
                  name: 'c',
                  branch: 'feat/c',
                },
              ],
              runsFolder: '/runs',
              deadlineSeconds: 7_200,
              start: async function startOne({ name, },) {
                if (name === 'a') {
                  return {
                    status: 0,
                    output: '',
                  };
                }
                if (name === 'b') {
                  return {
                    status: 1,
                    output: 'Error: it did not start\n',
                  };
                }
                // A retry that could not be started at all answers no status, and that is a failed entry too.
                return {
                  output: '',
                  startProblem: 'spawn ENOENT',
                };
              },
              wait: async function passTime() {
                // The stagger is not what this test reads.
              },
            },);
            expect(failed,).toEqual(['b', 'c',],);
          },
        },),
      ],
    },),
    describe({
      name: entryLineOf.name,
      children: [
        it({
          name: 'names the last thing a failed start printed, which is where its reason is',
          fn: async () => {
            expect(entryLineOf({
              name: 'a',
              started: {
                status: 1,
                output: 'worktree add: status 0\ndependencies: status 1\nsomething broke\n',
              },
            },),).toBe('a: status 1 something broke',);
          },
        },),
        it({
          name: 'says a start that never began could not be started, naming why',
          fn: async () => {
            expect(entryLineOf({
              name: 'a',
              started: {
                output: '',
                startProblem: 'spawn ENOENT',
              },
            },),).toBe('a: could not start the retry (spawn ENOENT)',);
            expect(entryLineOf({
              name: 'a',
              started: { output: '', },
            },),).toBe('a: could not start the retry (it answered no status)',);
          },
        },),
        it({
          name: 'says why a start that ended on an uncaught error failed, never the version line Node ends the stack with',
          fn: async () => {
            /**
             What Node prints for an uncaught error: where it happened, the code, the error, its frames, and its own version.
             */
            const crashed = [
              'file:///pkg/src/required-output-of-a-command.ts:58',
              '    throw new CommandAnswerMissingError(',
              '          ^',
              '',
              'CommandAnswerMissingError: git rev-parse --show-toplevel answered 1: not at the root',
              '    at requiredOutputOf (file:///pkg/src/required-output-of-a-command.ts:58:11)',
              'Node.js v26.13.0',
              '',
            ].join('\n',);
            expect(entryLineOf({
              name: 'a',
              started: {
                status: 1,
                output: crashed,
              },
            },),).toBe('a: status 1 CommandAnswerMissingError: git rev-parse --show-toplevel answered 1: not at the root',);
          },
        },),
        it({
          name: 'names the file the whole of a failed start was kept in, and says so plainly when it printed no reason at all',
          fn: async () => {
            expect(entryLineOf({
              name: 'a',
              started: {
                status: 1,
                output: 'a line nobody can act on\n',
              },
              keptAt: '/runs/a-start-failure.log',
            },),).toBe('a: status 1 a line nobody can act on (all it printed: /runs/a-start-failure.log)',);
            expect(entryLineOf({
              name: 'a',
              started: {
                status: 1,
                output: '    at somewhere (file:///x.ts:1:1)\nNode.js v26.13.0\n',
              },
            },),).toBe('a: status 1 it printed no reason',);
          },
        },),
      ],
    },),
    describe({
      name: batchEntriesIn.name,
      children: [
        it({
          name: 'reads the entries of a list, and refuses a file that holds no list',
          fn: async () => {
            expect(batchEntriesIn({
              shown: '[{"name":"a","branch":"feat/a"}]',
              listFile: 'batch.json',
            },),).toEqual([{
              name: 'a',
              branch: 'feat/a',
            },],);
            expect(() => batchEntriesIn({
              shown: '{"name":"a"}',
              listFile: 'batch.json',
            },),).toThrow('holds no JSON array',);
          },
        },),
        it({
          name: 'refuses an entry that names a field this command does not read, naming the field and the entry',
          fn: async () => {
            /**
             A field no version of this command has read.
             */
            const unread = 'baseBranch';
            expect(() => batchEntriesIn({
              shown: `[{"name":"b","branch":"feat/b","${unread}":"main"}]`,
              listFile: 'batch.json',
            },),).toThrow(unread,);
            expect(() => batchEntriesIn({
              shown: `[{"name":"a","branch":"feat/a"},{"name":"b","branch":"feat/b","${unread}":"main"}]`,
              listFile: 'batch.json',
            },),).toThrow('entry 2 (b)',);
            expect(() => batchEntriesIn({
              shown: '["a"]',
              listFile: 'batch.json',
            },),).toThrow('is no JSON object',);
          },
        },),
        it({
          name: 'reads every field a list has always had, the base a run starts at, and the session file a run resumes',
          fn: async () => {
            /**
             One entry naming every field this command reads.
             */
            const entry = {
              name: 'a',
              branch: 'feat/a',
              base: 'main',
              prompt: 'p.md',
              cpu: 1,
              memory: '1G',
              tasks: 1,
              wall: 1,
              run: 1,
              sessionOf: 'a-1',
              session: '/s.jsonl',
              provider: 'neuralwatt',
              model: 'deepseek-v4.1-flash',
            };
            expect(batchEntriesIn({
              shown: JSON.stringify([entry,],),
              listFile: 'batch.json',
            },),).toEqual([entry,],);
            expect(retryWordsOf({
              entry,
              runsFolder: '/runs',
              deadlineSeconds: 60,
            },),).toContain('--session=/s.jsonl',);
          },
        },),
      ],
    },),
    describe({
      name: promptFileOf.name,
      children: [
        it({
          name: 'reads the prompt an entry names, a path as itself and a name under the runs folder, and names one for an entry that names none',
          fn: async () => {
            expect(promptFileOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
                prompt: '/elsewhere/prompt.md',
              },
              runsFolder: '/runs',
            },),).toBe('/elsewhere/prompt.md',);
            expect(promptFileOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
                prompt: 'a-prompt.md',
              },
              runsFolder: '/runs',
            },),).toBe('/runs/a-prompt.md',);
            expect(promptFileOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
              },
              runsFolder: '/runs',
            },),).toBe('/runs/a-prompt.md',);
          },
        },),
      ],
    },),
    describe({
      name: retryWordsOf.name,
      children: [
        it({
          name: 'takes the defaults a batch list has always had, and writes every option an entry names',
          fn: async () => {
            /**
             The words for an entry that names only itself.
             */
            const plain = retryWordsOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
              },
              runsFolder: '/runs',
              deadlineSeconds: 7_200,
            },);
            expect(plain,).toEqual([
              '--deadline-seconds=7200',
              `--period-seconds=${String(BATCH_RETRY_PERIOD_SECONDS,)}`,
              '--',
              '--runs-folder=/runs',
              '--name=a',
              '--branch=feat/a',
              '--prompt=/runs/a-prompt.md',
              '--cpu=800',
              `--memory=${BATCH_DEFAULT_MEMORY}`,
              '--tasks=3000',
              `--wall=${String(BATCH_DEFAULT_WALL_SECONDS,)}`,
            ],);
            expect(retryWordsOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
                run: 4,
                provider: 'opencode-go',
                model: 'deepseek-v4.1-flash',
                sessionOf: 'a-3',
              },
              runsFolder: '/runs',
              deadlineSeconds: 60,
            },),).toContain('--session-of=a-3',);
          },
        },),
        it({
          name: 'hands the start the commit or branch an entry names as its base, and says nothing about a base an entry names none of',
          fn: async () => {
            /**
             The commit a fix for a reviewed branch starts at.
             */
            const base = 'fix/the-branch-that-was-reviewed';
            expect(retryWordsOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
                base,
              },
              runsFolder: '/runs',
              deadlineSeconds: 60,
            },),).toContain(`--base=${base}`,);
            expect(retryWordsOf({
              entry: {
                name: 'a',
                branch: 'feat/a',
              },
              runsFolder: '/runs',
              deadlineSeconds: 60,
            },).filter(function namesTheBase(word,) {
              return word.startsWith('--base=',);
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: parsePoolGateOptions.name,
      children: [
        it({
          name: 'needs no name for the counts alone, and needs one to start a run',
          fn: async () => {
            expect(parsePoolGateOptions(['--counts-only',],).name,).toBeUndefined();
            expect(() => parsePoolGateOptions([],),).toThrow(PoolGateOptionsError,);
          },
        },),
        it({
          name: 'reads the run, its pool, its wall clock, the run it resumes and where the runs live, and keeps every other word for the launcher',
          fn: async () => {
            expect(parsePoolGateOptions([
              '--name=a',
              '--provider=neuralwatt',
              '--model=deepseek-v4.1-flash',
              '--wall=600',
              '--session-of=a-1',
              '--runs-folder=/runs',
              '--branch=feat/a',
            ],),).toEqual({
              name: 'a',
              runsFolder: '/runs',
              countsOnly: false,
              provider: 'neuralwatt',
              model: 'deepseek-v4.1-flash',
              wallSeconds: 600,
              sessionOfRunFolder: 'a-1',
              commandLine: [
                '--name=a',
                '--provider=neuralwatt',
                '--model=deepseek-v4.1-flash',
                '--wall=600',
                '--session-of=a-1',
                '--runs-folder=/runs',
                '--branch=feat/a',
              ],
            },);
          },
        },),
        it({
          name: 'refuses a wall clock it cannot build a deadline from',
          fn: async () => {
            expect(() => parsePoolGateOptions(['--name=a', '--wall=soon',],),).toThrow('--wall must be a number above zero',);
          },
        },),
      ],
    },),
    describe({
      name: parseRetryStartOptions.name,
      children: [
        it({
          name: 'reads its own two durations wherever they stand, and hands the pool gate every other word',
          fn: async () => {
            /**
             What a line of this command is read as, whichever way it is written.
             */
            const meant = {
              deadlineSeconds: 7_200,
              periodSeconds: 30,
              words: ['--name=a',],
            };
            expect(parseRetryStartOptions(['--deadline-seconds=7200', '--period-seconds=30', '--', '--name=a',],),).toEqual(meant,);
            // `mise run //package/pi-shared/subagent-driver:retry-pi-run-start-until-a-pool-has-room -- --deadline-seconds=1
            // --period-seconds=1 -- --name=a` lost its second `--` before the script saw it, so the line without one reads the
            // same way.
            expect(parseRetryStartOptions(['--deadline-seconds=7200', '--period-seconds=30', '--name=a',],),).toEqual(meant,);
            expect(parseRetryStartOptions(['--name=a', '--deadline-seconds=7200', '--period-seconds=30',],),).toEqual(meant,);
          },
        },),
        it({
          name: 'refuses no words for the pool gate, or a duration it cannot use',
          fn: async () => {
            expect(() => parseRetryStartOptions(['--deadline-seconds=1', '--period-seconds=1',],),).toThrow(RetryStartOptionsError,);
            expect(() => parseRetryStartOptions(['--deadline-seconds=1', '--period-seconds=1', '--',],),).toThrow('no pool gate word',);
            expect(() => parseRetryStartOptions(['--deadline-seconds=1', '--', '--name=a',],),).toThrow('--period-seconds is required',);
          },
        },),
      ],
    },),
    describe({
      name: parseBatchOptions.name,
      children: [
        it({
          name: 'reads the list and takes the runs folder and deadline a caller has always had',
          fn: async () => {
            expect(parseBatchOptions(['--batch-file=/runs/b.json',],),).toEqual({
              batchFile: '/runs/b.json',
              runsFolder: join(process.env.HOME ?? '', 'temp', 'agent', 'pi-runs',),
              deadlineSeconds: 7_200,
            },);
            expect(parseBatchOptions([
              '--batch-file=b.json',
              '--runs-folder=/elsewhere',
              '--deadline-seconds=60',
            ],),).toEqual({
              batchFile: 'b.json',
              runsFolder: '/elsewhere',
              deadlineSeconds: 60,
            },);
          },
        },),
        it({
          name: 'refuses a line that names no list, or a deadline it cannot use',
          fn: async () => {
            expect(() => parseBatchOptions([],),).toThrow(BatchOptionsError,);
            expect(() => parseBatchOptions(['--batch-file=b.json', '--deadline-seconds=never',],),).toThrow('--deadline-seconds must be a number above zero',);
          },
        },),
      ],
    },),
    describe({
      name: poolOfExecStart.name,
      children: [
        it({
          name: 'reads the provider and model a unit command line names, written with an equals sign or with a space',
          fn: async () => {
            /**
             The pool the unit's own command line names.
             */
            const named = poolOfExecStart({
              shown: '{ path=node ; argv[]=node x.ts --provider=neuralwatt --model=deepseek-v4.1-flash ; }',
            },);
            expect(poolsHeldBy({ membership: named, },).map(function nameOf(pool,) {
              return pool.name;
            },),).toEqual(['neuralwatt',],);
            expect(poolOfExecStart({
              shown: '{ argv[]=node x.ts --provider neuralwatt --model deepseek-v4.1-flash ; }',
            },).holds,).toBe('a-pool',);
          },
        },),
        it({
          name: 'answers that no pool is named when the unit names no provider or model, or names a provider no pool holds',
          fn: async () => {
            expect(poolOfExecStart({ shown: '{ argv[]=node x.ts ; }', },),).toEqual({ holds: 'no-pool', },);
            expect(poolOfExecStart({
              shown: '{ argv[]=node x.ts --provider=synthetic --model=moonshotai/Kimi-K3.1-Pro ; }',
            },),).toEqual({ holds: 'no-pool', },);
          },
        },),
      ],
    },),
    describe({
      name: unitRunNamesIn.name,
      children: [
        it({
          name: 'reads the run name out of every Pi unit line, and leaves every other line out',
          fn: async () => {
            expect(unitRunNamesIn({
              shown: [
                'pi-copy-route.service loaded active running x',
                'other.service loaded active running x',
                '',
                'pi-second.service loaded activating start x',
              ].join('\n',),
            },),).toEqual(['copy-route', 'second',],);
          },
        },),
      ],
    },),
    describe({
      name: launcherRunsIn.name,
      children: [
        it({
          name: 'finds every launcher process that names a run, with the pool its own options name',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: '11',
              words: [
                '/usr/bin/node',
                `/runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`,
                '--name=copy-route',
                '--provider=opencode-go',
                '--model=deepseek-v4.1-flash',
              ],
            },);
            await writeProcess({
              folder: processes,
              pid: '12',
              words: [
                '/usr/bin/node',
                `/runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`,
                '--provider=neuralwatt',
              ],
            },);
            await writeProcess({
              folder: processes,
              pid: '13',
              words: ['/usr/bin/node', '/runs/another-command.ts', '--name=not-a-run',],
            },);
            await mkdir(join(processes, 'not-a-number',), { recursive: true, },);
            /**
             The launchers found.
             */
            const found = await launcherRunsIn({
              processFolder: processes,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
            },);
            expect(found.map(function nameOf(run,) {
              return run.name;
            },),).toEqual(['copy-route',],);
            expect(poolNamesOf(found[0] ?? aRunOutsideEveryPool(),),).toEqual(['opencode-go',],);
          },
        },),
      ],
    },),
    describe({
      name: goingRunsNow.name,
      children: [
        it({
          name: 'counts a unit whose launcher is gone by the provider its own command line names, and asks nothing about a unit a launcher already accounts for',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: '21',
              words: ['/usr/bin/node', `/runs/${LAUNCHER_ENTRY_NAMES[0] ?? ''}`, '--name=with-launcher',],
            },);
            /**
             Every question asked of systemd.
             */
            const asked: string[][] = [];
            /**
             The runs going now.
             */
            const going = await goingRunsNow({
              run: async function systemd({ words, },) {
                asked.push([...words,],);
                return words.includes('list-units',)
                  ? answered(0, 'pi-with-launcher.service x\npi-no-launcher.service x\n',)
                  : answered(0, '{ argv[]=node x.ts --provider=neuralwatt --model=kimi-k3.1-pro-flex ; }',);
              },
              processFolder: processes,
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
              onProblem: function onProblem() {
                throw new Error('a unit that answered was reported as unreadable',);
              },
            },);
            expect(going.map(function nameOf(run,) {
              return run.name;
            },),).toEqual(['with-launcher', 'no-launcher',],);
            expect(poolNamesOf(going[1] ?? aRunOutsideEveryPool(),),).toEqual(['neuralwatt-kimi-flex',],);
            expect(asked,).toHaveLength(2,);
          },
        },),
        it({
          name: 'reports a unit whose own command line could not be read, and counts it in every pool it could be running on',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A process folder holding no launcher.
             */
            const processes = join(folder.path, 'proc',);
            await mkdir(processes, { recursive: true, },);
            /**
             What the command was told about the unit it could not read.
             */
            const problems: string[] = [];
            /**
             The runs going now.
             */
            const going = await goingRunsNow({
              run: async function systemd({ words, },) {
                return words.includes('list-units',)
                  ? answered(0, 'pi-a.service x\n',)
                  : answered(1, '', 'No such unit',);
              },
              processFolder: join(folder.path, 'proc',),
              launcherEntryNames: LAUNCHER_ENTRY_NAMES,
              onProblem: function onProblem(problem,) {
                problems.push(problem,);
              },
            },);
            expect(going.map(function nameOf(run,) {
              return run.name;
            },),).toEqual(['a',],);
            expect(poolNamesOf(going[0] ?? aRunOutsideEveryPool(),),).toEqual(PROVIDER_POOLS.map(function nameOf(pool,) {
              return pool.name;
            },),);
            expect(problems,).toHaveLength(1,);
            expect(problems[0],).toContain('pool of unit pi-a.service not read',);
          },
        },),
        it({
          name: 'refuses to count anything when systemd cannot list the units',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A process folder holding no launcher.
             */
            const processes = join(folder.path, 'proc',);
            await mkdir(processes, { recursive: true, },);
            try {
              await goingRunsNow({
                run: async function systemd() {
                  return answered(1, '', 'Failed to connect to bus',);
                },
                processFolder: processes,
                launcherEntryNames: LAUNCHER_ENTRY_NAMES,
                onProblem: function onProblem() {
                  throw new Error('a listing nobody read was reported as a unit',);
                },
              },);
            }
            catch (error) {
              expect(String(error,),).toContain('Failed to connect to bus',);
              return;
            }
            throw new Error('a refused listing was counted as no runs',);
          },
        },),
      ],
    },),
    describe({
      name: valueOfOption.name,
      children: [
        it({
          name: 'reads a value written with an equals sign or with a space, and answers nothing for an option the text does not carry',
          fn: async () => {
            expect(valueOfOption({
              text: 'node x.ts --provider=neuralwatt --model=deepseek-v4.1-flash ;',
              option: '--provider',
            },),).toBe('neuralwatt',);
            expect(valueOfOption({
              text: 'node x.ts --provider neuralwatt ;',
              option: '--provider',
            },),).toBe('neuralwatt',);
            expect(valueOfOption({
              text: 'node x.ts --provider-token=abc ;',
              option: '--provider',
            },),).toBe('',);
            expect(valueOfOption({
              text: 'node x.ts ;',
              option: '--provider',
            },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: isDigitsOnly.name,
      children: [
        it({
          name: 'reads digits alone, and nothing for empty text or text holding anything else',
          fn: async () => {
            expect(isDigitsOnly('42',),).toBe(true,);
            expect(isDigitsOnly('',),).toBe(false,);
            expect(isDigitsOnly('4a',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: firstWordOf.name,
      children: [
        it({
          name: 'reads the first word of a padded line, and nothing for a line of spaces',
          fn: async () => {
            expect(firstWordOf('  pi-a.service loaded',),).toBe('pi-a.service',);
            expect(firstWordOf('   ',),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: tailKeeper.name,
      children: [
        it({
          name: 'keeps only the end of what arrived, across both streams',
          fn: async () => {
            /**
             A keeper of four characters.
             */
            const kept = tailKeeper({ characters: 4, },);
            kept.keep(Buffer.from('abc',),);
            kept.keep(Buffer.from('defg',),);
            expect(kept.text(),).toBe('defg',);
          },
        },),
      ],
    },),
  ],
},);

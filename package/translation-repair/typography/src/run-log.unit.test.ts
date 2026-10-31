/**
 Tests for the log a run keeps: where a pass's records go and that no retention sweep can ever delete them. How a
 reader finds and renders them is `glossary/src/corpus-run/run-log-read.unit.test.ts`.

 A RUN'S RECORDS ARE ITS EVIDENCE. The file sink is given the run folder and told never to sweep; the case that holds
 that is paired with a control that gives the same folder the sink's own default bounds and loses the old day, so the
 case is shown able to fail.

 Every file is written under a scratch directory. Cat-themed invention throughout.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { createLogger, } from '@monochromatic-dev/logger-core/ts';
import {
  createFileSink,
  NO_LOG_RETENTION_SWEEP,
} from '@monochromatic-dev/logger-core/ts/node.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contextRoot,
  inEntryLogContext,
  installRunLog,
  openRunLog,
  printRunLine,
  recordRunLine,
  RUN_LOG_SUFFIX,
} from '../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Record file under a run folder's log directory, found by reading the directories the sink made.

 @param runDir - run folder

 @returns Lines of every record file under its log directory, parsed, in name order

 @example
 ```ts
 const records = await recordsUnder({ runDir, });
 ```
 */
async function recordsUnder({ runDir, }: { readonly runDir: string; },): Promise<readonly unknown[]> {
  /**
   Names of everything under the log directory.
   */
  const names = (await readdir(
    join(
      runDir,
      'log',
    ),
    { recursive: true, },
  )).filter(function isRecordFile(name,): boolean {
    return name.endsWith(RUN_LOG_SUFFIX,);
  },)
    .toSorted();
  /**
   Lines of the files.
   */
  const lines: string[] = [];
  for (const name of names) {
    /* oxlint-disable no-await-in-loop -- files read in name order so the records stay in order */
    lines.push(...(await readFile(
      join(
        runDir,
        'log',
        name,
      ),
      'utf8',
    )).split('\n',)
      .filter(function present(line,): boolean {
        return line !== '';
      },),);
    /* oxlint-enable no-await-in-loop */
  }
  return lines.map(function parsed(line,): unknown {
    return JSON.parse(line,);
  },);
}

/**
 Level and message of a parsed record.

 @param record - one record read back from a record file

 @returns Its level and its message

 @example
 ```ts
 const seen = levelAndMessageOf({ level: 'info', message: 'purr', timestamp: 0, },);
 ```
 */
function levelAndMessageOf(record: unknown,): unknown {
  const { level, message, } = record as { readonly level: unknown; readonly message: unknown; };
  return {
    level,
    message,
  };
}

await describe({
  name: 'run log',
  concurrency: 1,
  children: [
    describe({
      name: openRunLog.name,
      concurrency: 1,
      children: [
        it({
          name: 'RECORDS WHAT A CONTEXT ROOT WRITES under the run folder, one JSON line of exactly level, message and '
            + 'timestamp for each, while the log is installed, and what the entry\'s name leads',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            await using log = await openRunLog({ runDir: scratch.path, },);
            using _installed = installRunLog({ log, },);
            /**
             Root logger of a module that the run's lines go through.
             */
            const l = contextRoot({ tag: 'purr', },);

            await inEntryLogContext({
              entry: 'Tabby',
              generation: 'sha256-tree-v1:0',
              attempt: 2,
              run: async function purrs(): Promise<void> {
                l.info('kneads',);
              },
            },);
            await log.flush();

            expect(log.recording,).toBe(true,);
            /**
             What the file holds.
             */
            const records = await recordsUnder({ runDir: scratch.path, },);
            expect(records.length,).toBe(1,);
            expect(Object.keys(records[0] as object,).toSorted(),).toEqual(['level', 'message', 'timestamp',],);
            expect(records[0],).toMatchObject({
              level: 'info',
              message: '[purr] [Tabby] [attempt 2] kneads',
            },);
          },
        },),

        it({
          name: 'WRITES NOTHING TO THE RUN FOLDER once the log is no longer installed, so a root built long ago '
            + 'follows whichever run is current',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            await using log = await openRunLog({ runDir: scratch.path, },);
            /**
             Root logger built before any run log is installed.
             */
            const l = contextRoot({ tag: 'purr', },);
            {
              using _installed = installRunLog({ log, },);
              l.info('while installed',);
              await log.flush();
            }
            l.info('after',);
            await log.flush();

            expect((await recordsUnder({ runDir: scratch.path, },)).map(function messageOf(record,): unknown {
              return (record as { readonly message: unknown; }).message;
            },),).toEqual(['[purr] while installed',],);
          },
        },),

        it({
          name: 'KEEPS A RUN\'S OLD DAY DIRECTORIES: a sweep never deletes the records, and no stamp file appears; the '
            + 'control gives the same folder the sink\'s default bounds and loses the old day, so this case can fail',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            /**
             Lays an old day of records in a run folder's log directory.

             @param runDir - run folder

             @returns Path of the old record file
             */
            async function oldDay({ runDir, }: { readonly runDir: string; },): Promise<string> {
              /**
               Day directory far older than any retention window.
               */
              const day = join(
                runDir,
                'log',
                '2026-01-01',
              );
              await mkdir(
                day,
                { recursive: true, },
              );
              /**
               Record file of that day, named as the sink names one.
               */
              const file = join(
                day,
                '2026-01-01T00-00-00.000Z.00000000-0000-4000-8000-0000000000aa.log.jsonl',
              );
              await writeFile(
                file,
                '{"level":"info","message":"old evidence","timestamp":1767225600000}\n',
              );
              return file;
            }

            await using kept = await scratchDir({ prefix: 'run-log-kept-', },);
            /**
             Old file of the run whose log is opened as a pass opens it.
             */
            const oldKept = await oldDay({ runDir: kept.path, },);
            {
              await using log = await openRunLog({ runDir: kept.path, },);
              using _installed = installRunLog({ log, },);
              contextRoot({ tag: 'purr', },).info('now',);
              await log.flush();
            }
            expect(await readFile(
              oldKept,
              'utf8',
            ),).toContain('old evidence',);
            expect(
              await readdir(join(
                kept.path,
                'log',
              ),),
            ).not.toContain('.logger-retention-sweep',);

            await using swept = await scratchDir({ prefix: 'run-log-swept-', },);
            /**
             Old file of the control run, whose folder a default-bounded sink sweeps.
             */
            const oldSwept = await oldDay({ runDir: swept.path, },);
            {
              /**
               Control sink: the same folder under the default bounds.
               */
              const sink = createFileSink({
                directory: join(
                  swept.path,
                  'log',
                ),
                retention: {
                  maxAgeDays: 14,
                  maxBytes: 536_870_912,
                  maxFiles: 50_000,
                },
                environment: {},
              },);
              const { logger, } = createLogger({ sinks: [sink,], },);
              logger.info('now',);
              await logger.flush();
            }
            await expect(readFile(
              oldSwept,
              'utf8',
            ),).rejects.toThrow();
          },
        },),

        it({
          name: 'SAYS IT IS NOT RECORDING where the run folder cannot be written, and still prints a line',
          fn: async (ctx) => {
            const printed = ctx.sinon.stub(
              console,
              'log',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            /**
             A file where a run folder should be, so no directory can be made under it.
             */
            const notADirectory = join(
              scratch.path,
              'in-the-way',
            );
            await writeFile(
              notADirectory,
              'a file',
            );
            ctx.sinon.stub(
              console,
              'error',
            );
            await using log = await openRunLog({ runDir: notADirectory, },);
            using _installed = installRunLog({ log, },);

            printRunLine({ line: 'START tip=none', },);

            expect(log.recording,).toBe(false,);
            expect(printed.args,).toEqual([['START tip=none',],],);
          },
        },),
      ],
    },),

    describe({
      name: printRunLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS A LINE AS IT WAS and records it whole as the message, so a pass\'s own lines reach the run '
            + 'folder without anyone capturing the console',
          fn: async (ctx) => {
            const printed = ctx.sinon.stub(
              console,
              'log',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            await using log = await openRunLog({ runDir: scratch.path, },);
            using _installed = installRunLog({ log, },);

            printRunLine({ line: 'START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 pending=2', },);
            await log.flush();

            expect(printed.args,).toEqual([['START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 pending=2',],],);
            expect((await recordsUnder({ runDir: scratch.path, },)).map(levelAndMessageOf,),).toEqual([
              {
                level: 'info',
                message: 'START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 pending=2',
              },
            ],);
          },
        },),

        it({
          name: 'PRINTS A LINE AND RECORDS NOTHING where no run log is installed',
          fn: async (ctx) => {
            const printed = ctx.sinon.stub(
              console,
              'log',
            );

            printRunLine({ line: 'DONE', },);

            expect(printed.args,).toEqual([['DONE',],],);
          },
        },),

        it({
          name: 'RECORDS A LINE WITHOUT PRINTING IT, at the level given, and records nothing where no run log is '
            + 'installed',
          fn: async (ctx) => {
            const printedLog = ctx.sinon.stub(
              console,
              'log',
            );
            const printedError = ctx.sinon.stub(
              console,
              'error',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            recordRunLine({
              level: 'error',
              line: 'before any log is installed',
            },);
            await using log = await openRunLog({ runDir: scratch.path, },);
            using _installed = installRunLog({ log, },);

            recordRunLine({
              level: 'error',
              line: 'the pass stopped',
            },);
            await log.flush();

            expect((await recordsUnder({ runDir: scratch.path, },)).map(levelAndMessageOf,),).toEqual([
              {
                level: 'error',
                message: 'the pass stopped',
              },
            ],);
            expect([printedLog.args, printedError.args,],).toEqual([[], [],],);
          },
        },),
      ],
    },),

    describe({
      name: 'a destination that does not answer',
      concurrency: 1,
      children: [
        it({
          name: 'STOPS WAITING for a sink that never says whether its folder is writable, once the bound has passed '
            + '(ledger B1112): the log is open but records nothing, and says it is unanswered and not unwritable, so '
            + 'the pass can refuse naming the folder',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            /**
             Fake timers, so the bound passes at once and the case waits for no clock.
             */
            const clock = ctx.sinon.useFakeTimers({
              toFake: ['setTimeout', 'clearTimeout',],
              shouldClearNativeTimers: true,
            },);
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            /**
             The real sink, whose verification a hung disk would never finish.
             */
            const real = createFileSink({
              directory: join(
                scratch.path,
                'log',
              ),
              retention: NO_LOG_RETENTION_SWEEP,
              environment: {},
            },);
            /**
             The verification a hung disk would never finish, which only the case can settle.
             */
            const held = Promise.withResolvers<boolean>();
            /**
             Whether the open has settled.
             */
            let settled = false;
            /**
             The run log being opened.
             */
            const opening = (async function opens() {
              const opened = await openRunLog({
                runDir: scratch.path,
                fileSinkOf: () => ({
                  ...real,
                  verify: () => held.promise,
                }),
                verifyWithinMs: 5_000,
              },);
              settled = true;
              return opened;
            })();
            await clock.tickAsync(4_999,);
            /**
             Whether the open had settled before the bound.
             */
            const settledEarly = settled;
            await clock.tickAsync(2,);
            await using log = await opening;

            expect(settledEarly,).toBe(false,);
            expect(log.recording,).toBe(false,);
            expect(log.destination,).toBe('unanswered',);
            held.resolve(true,);
          },
        },),

        it({
          name: 'SAYS A SINK THAT ANSWERS is writable, or unwritable, as it always did',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            /**
             A file where the folder's `log` directory would go, which nothing can write under.
             */
            const blocked = join(
              scratch.path,
              'blocked',
            );
            await mkdir(blocked,);
            await writeFile(
              join(
                blocked,
                'log',
              ),
              'a file',
            );
            await using writable = await openRunLog({ runDir: scratch.path, },);
            await using unwritable = await openRunLog({ runDir: blocked, },);

            expect([writable.recording, writable.destination,],).toEqual([true, 'writable',],);
            expect([unwritable.recording, unwritable.destination,],).toEqual([false, 'unwritable',],);
          },
        },),
      ],
    },),

  ],
},);

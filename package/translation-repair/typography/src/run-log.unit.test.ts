/**
 Tests for the log a run keeps: where a pass's records go, that no retention sweep can ever delete them, and how a
 reader finds and renders them.

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
import { createFileSink, } from '@monochromatic-dev/logger-core/ts/node.ts';
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
  readRunLogText,
  renderRunLogLine,
  RUN_LOG_SUFFIX,
  RunLogUnreadableError,
  runLogFilesUnder,
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

            expect(await recordsUnder({ runDir: scratch.path, },),).toMatchObject([
              { message: '[purr] while installed', },
            ],);
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
            expect(await readdir(join(
              kept.path,
              'log',
            ),),).not.toContain('.logger-retention-sweep',);

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
            expect(await recordsUnder({ runDir: scratch.path, },),).toMatchObject([
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
      ],
    },),

    describe({
      name: renderRunLogLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'RENDERS A RECORD as the console sink prints it',
          fn: async () => {
            expect(renderRunLogLine({
              line: '{"level":"warn","message":"[purr] nap","timestamp":1788256800000}',
            },),).toBe('[warn] [2026-09-01T10:00:00.000Z] [purr] nap',);
          },
        },),

        it({
          name: 'HANDS ON AS IT STANDS every line that is not exactly a record: cut off, not JSON, not an object, '
            + 'a missing, extra or mistyped field, a level the logger lacks, a timestamp no date holds',
          fn: async () => {
            for (const line of [
              '{"level":"info","message":"cut off",',
              'a bare line',
              '"a string"',
              '[1,2]',
              '{"level":"info","message":"no stamp"}',
              '{"level":"info","message":"extra","timestamp":0,"entry":"Tabby"}',
              '{"level":"loud","message":"level","timestamp":0}',
              '{"level":"info","message":3,"timestamp":0}',
              '{"level":"info","message":"fraction","timestamp":0.5}',
              '{"level":"info","message":"far","timestamp":9e15}',
            ]) {
              expect(renderRunLogLine({ line, },),).toBe(line,);
            }
          },
        },),
      ],
    },),

    describe({
      name: runLogFilesUnder.name,
      concurrency: 1,
      children: [
        it({
          name: 'FINDS THE FILES under a run folder, its log directory or a day directory in name order, and takes a '
            + 'named record file as it is',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            /**
             Two day directories of one log directory, with a stray file the sink never wrote.
             */
            const log = join(
              scratch.path,
              'log',
            );
            await mkdir(
              join(
                log,
                '2026-10-02',
              ),
              { recursive: true, },
            );
            await mkdir(
              join(
                log,
                '2026-10-01',
              ),
              { recursive: true, },
            );
            await writeFile(
              join(
                log,
                '2026-10-02',
                'b.log.jsonl',
              ),
              '',
            );
            await writeFile(
              join(
                log,
                '2026-10-01',
                'a.log.jsonl',
              ),
              '',
            );
            await writeFile(
              join(
                log,
                '2026-10-01',
                'a.log',
              ),
              '',
            );

            /**
             The two record files, in name order.
             */
            const both = [
              join(
                log,
                '2026-10-01',
                'a.log.jsonl',
              ),
              join(
                log,
                '2026-10-02',
                'b.log.jsonl',
              ),
            ];
            expect(await runLogFilesUnder({ path: scratch.path, },),).toEqual(both,);
            expect(await runLogFilesUnder({ path: log, },),).toEqual(both,);
            expect(await runLogFilesUnder({ path: join(log, '2026-10-02',), },),).toEqual([both[1],],);
            expect(await runLogFilesUnder({ path: both[0] ?? '', },),).toEqual([both[0],],);
          },
        },),

        it({
          name: 'REFUSES a directory with no record file and a path that is neither a directory nor a record file',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            await expect(runLogFilesUnder({ path: scratch.path, },),).rejects.toBeInstanceOf(
              RunLogUnreadableError,
            );
            await expect(runLogFilesUnder({ path: join(scratch.path, 'nowhere.log',), },),).rejects.toBeInstanceOf(
              RunLogUnreadableError,
            );
          },
        },),
      ],
    },),

    describe({
      name: readRunLogText.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS THE RECORDS A RUN KEPT as the lines of console text, one per record',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-', },);
            {
              await using log = await openRunLog({ runDir: scratch.path, },);
              using _installed = installRunLog({ log, },);
              contextRoot({ tag: 'purr', },).info('one',);
              contextRoot({ tag: 'purr', },).warn('two',);
              await log.flush();
            }

            /**
             Text read back.
             */
            const text = await readRunLogText({ path: scratch.path, },);
            /**
             Lines of the text, the closing line feed leaving an empty piece.
             */
            const pieces = text.split('\n',);
            expect(pieces.length,).toBe(3,);
            expect(pieces[0]?.startsWith('[info] [',),).toBe(true,);
            expect(pieces[0]?.endsWith('Z] [purr] one',),).toBe(true,);
            expect(pieces[1]?.startsWith('[warn] [',),).toBe(true,);
            expect(pieces[1]?.endsWith('Z] [purr] two',),).toBe(true,);
            expect(pieces[2],).toBe('',);
          },
        },),
      ],
    },),
  ],
},);

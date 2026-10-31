/**
 Tests for reading the records a run kept: what counts as a record, how one is rendered for the report parsers, where
 the files of a run are and what is said of a path that holds none.

 A RECORD IS EXACTLY WHAT THE LOGGER'S FILE SINK WRITES. Anything else in a record file is handed on as it stands, so a
 report counts it as the damaged or foreign line it is.

 Every file is written under a scratch directory, the records through the run log of `typography/src/run-log.ts`.
 Cat-themed invention throughout.

 @module
 */

import {
  chmod,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { format, } from 'node:util';

import {
  createLogger,
  sinks,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contextRoot,
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  parseRunLogRecord,
  readRunLogLines,
  readRunLogText,
  renderRunLogLine,
  runLogFilesUnder,
  StatedRefusalError,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'run log read',
  concurrency: 1,
  children: [
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
              expect(parseRunLogRecord({ line, },),).toBe('not-a-record',);
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
            await using scratch = await scratchDir({ prefix: 'run-log-read-', },);
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
          name: 'REFUSES a directory with no record file and a path that is neither a directory nor a record file, '
            + 'each in words of its own',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'run-log-read-', },);
            await expect(runLogFilesUnder({ path: scratch.path, },),).rejects.toBeInstanceOf(StatedRefusalError,);
            await expect(runLogFilesUnder({ path: join(scratch.path, 'nowhere.log',), },),).rejects.toBeInstanceOf(
              StatedRefusalError,
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
          name: 'READS THE RECORDS A RUN KEPT as the lines of console text, one per record, and a last line no line '
            + 'feed ends as a line of its own',
          fn: async (ctx) => {
            ctx.sinon.stub(
              console,
              'info',
            );
            await using scratch = await scratchDir({ prefix: 'run-log-read-', },);
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

            /**
             A record file whose process was killed mid-line.
             */
            const cut = join(
              scratch.path,
              'cut.log.jsonl',
            );
            await writeFile(
              cut,
              '{"level":"info","message":"whole","timestamp":0}\n{"level":"info","message":"cut',
            );
            expect(await readRunLogText({ path: cut, },),).toBe(
              '[info] [1970-01-01T00:00:00.000Z] whole\n{"level":"info","message":"cut\n',
            );
          },
        },),
      ],
    },),

    describe({
      name: 'rendering as the console shows',
      concurrency: 1,
      children: [
        it({
          name: 'ESCAPES every control character the console sink escapes, the same way, and none it leaves alone: C0 '
            + 'but the line feed and the tab, DEL and C1, each checked against what the real console sink printed '
            + '(ledger B1118)',
          fn: async (ctx) => {
            /**
             What the console sink prints, stubbed so the case holds it.
             */
            const info = ctx.sinon.stub(
              console,
              'info',
            );
            /**
             Every character from NUL to the end of C1.
             */
            const characters = Array.from(
              { length: 0xA0, },
              function characterAt(_unused, index,): string {
                return String.fromCodePoint(index,);
              },
            );
            /**
             The logger a person watches, with the console sink only.
             */
            const { logger, } = createLogger({ sinks: [sinks.createConsoleSink(),], },);
            for (const character of characters) {
              logger.info(`a${character}b`,);
              // A RUN OF LINES OF ONE LEVEL IS HANDED TO THE CONSOLE AS ONE CALL, so each is settled before the next.
              // oxlint-disable-next-line no-await-in-loop -- one line to a console call, so each call holds one character
              await logger.flush();
            }
            /**
             What the console printed, a call for each character, the message after the bracketed instant.
             */
            const printed = info.args.flatMap(function messagesOf(call,): readonly string[] {
              return [format(...(call as readonly unknown[]),).split('] ',).slice(2,)
                .join('] ',),];
            },);
            expect(printed.length,).toBe(characters.length,);
            characters.forEach(function compare(character, index,): void {
              /**
               The record line as the file sink writes it.
               */
              const line = JSON.stringify({ level: 'info', message: `a${character}b`, timestamp: 0, },);
              /**
               The rendering's message, after the bracketed level and instant.
               */
              const rendered = renderRunLogLine({ line, },).slice('[info] [1970-01-01T00:00:00.000Z] '.length,);
              expect(rendered,).toBe(printed[index],);
            },);
            // The positive control: the characters that must be written out, and the two that stay.
            expect(renderRunLogLine({ line: JSON.stringify({ level: 'info', message: '\u001B[31mred', timestamp: 0, },), },),)
              .toBe(String.raw`[info] [1970-01-01T00:00:00.000Z] \u001B[31mred`,);
            expect(renderRunLogLine({ line: JSON.stringify({ level: 'info', message: 'a\tb\nc', timestamp: 0, },), },),)
              .toBe('[info] [1970-01-01T00:00:00.000Z] a\tb\nc',);
          },
        },),

        it({
          name: 'READS A RECORD HOLDING A LONE SURROGATE that a JSON tool refuses, as the console shows it, and a '
            + 'folder holding it counts one line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'run-log-read-', },);
            /**
             A record file written the way the sink writes one: a lone surrogate leaves `JSON.stringify` as an escape.
             */
            const file = join(
              scratch.path,
              'lone.log.jsonl',
            );
            await writeFile(
              file,
              `${JSON.stringify({ level: 'warn', message: 'cut \uD800 here', timestamp: 0, },)}\n`,
            );
            expect(await readFile(file, 'utf8',),).toContain(String.raw`\ud800`,);
            expect(await readRunLogText({ path: file, },),).toBe('[warn] [1970-01-01T00:00:00.000Z] cut \uD800 here\n',);
          },
        },),
      ],
    },),

    describe({
      name: 'debug and trace records',
      concurrency: 1,
      children: [
        it({
          name: 'LEAVES OUT the debug and trace records the console showed only for a verbose run, so a count over the '
            + 'folder is the count over a console capture, and a debug line that quotes a marker is no marker '
            + '(ledger B1117); asked for them, hands them on',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'run-log-read-', },);
            /**
             A record file of every level, a debug line quoting a marker among them.
             */
            const file = join(
              scratch.path,
              'levels.log.jsonl',
            );
            await writeFile(
              file,
              `${[
                'trace',
                'debug',
                'info',
                'warn',
                'error',
                'fatal',
              ].map(function recordOf(level, index,): string {
                return JSON.stringify({ level, message: `${level} round: ${String(index,)}/3 heard`, timestamp: index, },);
              },).join('\n',)}\nnot a record\n`,
            );
            /**
             What the console showed, in file order.
             */
            const shown = (await readRunLogText({ path: file, },)).split('\n',);
            expect(shown,).toEqual([
              '[info] [1970-01-01T00:00:00.002Z] info round: 2/3 heard',
              '[warn] [1970-01-01T00:00:00.003Z] warn round: 3/3 heard',
              '[error] [1970-01-01T00:00:00.004Z] error round: 4/3 heard',
              '[fatal] [1970-01-01T00:00:00.005Z] fatal round: 5/3 heard',
              'not a record',
              '',
            ],);
            expect((await readRunLogText({ path: file, verbose: true, },)).split('\n',).length,).toBe(8,);
          },
        },),
      ],
    },),

    describe({
      name: readRunLogLines.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS A FILE A LINE AT A TIME: a consumer that stops after the first line never touches a later file, '
            + 'so no file is held whole (ledger B1120), and a line cut across the reader\'s chunks is one line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'run-log-read-', },);
            /**
             A first file of one long record, longer than a read chunk, and a second file that cannot be opened.
             */
            const first = join(
              scratch.path,
              'a.log.jsonl',
            );
            const second = join(
              scratch.path,
              'b.log.jsonl',
            );
            /**
             Message of the long record, many times the size of a chunk.
             */
            const long = 'purr '.repeat(100_000,);
            await writeFile(
              first,
              `${JSON.stringify({ level: 'info', message: long, timestamp: 0, },)}\n`
                + `${JSON.stringify({ level: 'info', message: 'nap', timestamp: 1, },)}\n`,
            );
            await writeFile(
              second,
              'never read\n',
            );
            await chmod(
              second,
              0o000,
            );
            /**
             Lines taken before stopping.
             */
            const taken: string[] = [];
            for await (const line of readRunLogLines({ path: scratch.path, },)) {
              taken.push(line,);
              if (taken.length === 2)
                break;
            }
            expect(taken,).toEqual([
              `[info] [1970-01-01T00:00:00.000Z] ${long}`,
              '[info] [1970-01-01T00:00:00.001Z] nap',
            ],);
          },
        },),
      ],
    },),
  ],
},);

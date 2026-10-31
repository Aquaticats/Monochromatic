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
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

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
  ],
},);

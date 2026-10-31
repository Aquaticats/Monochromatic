/**
 Tests that the cap census reads the records a pass keeps in its run folder.

 A PASS-RUN LOG IS A RECORD FILE NOW: `<run folder>/log/<day>/*.log.jsonl`, whose first records open with the line a
 pass prints at its start. The census finds those files under the directories it is named, opens each only far enough
 to read its head, and reads the calls of a pass-run file whole. A `.log` capture of the console is no log to it.

 THE DIFFERENTIAL CASE: the calls read from a run folder are the calls read from the same lines as console text, and
 the START line is told a pass's from a test suite's by the commit id it holds, as it was in the text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MODEL_CARDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  capCensusLogsUnder,
  capCensusPassRunReading,
  readCapLog,
  reportCapCensus,
} from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  spendLine,
  streamLine,
} from '@monochromatic-dev/translation-repair-provider-stream/ts/corpus-run/cap-census-log.test-fixture.ts';
import { writeRunFolder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/run-log-write.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';

/**
 Seat the fixture calls went to.
 */
const SEAT = SEAT_HYPER_OPENROUTER_UNMEASURED;

/**
 The seat's Hyper id, as its lines name it.
 */
const HYPER_ID = MODEL_CARDS[SEAT].hyper?.id ?? 'no hyper block';

/**
 What a pass prints to open its run, as the logger prints a line of that text: a commit id a repository could give.
 */
const PASS_START = '[info] [2026-09-28T09:59:00.000Z] START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 '
  + 'pipeline=sha256-tree-v1:5d4265e1c0a37b8f9d2e6a4c1b07f3e58d9a2c4b6e1f0a7d3c5b8e92f4a6d1c0 files=156 pending=2 '
  + 'done=0';

/**
 What a test suite prints from a fixture pass: one digit forty times, a commit id no repository gives.
 */
const FIXTURE_PASS_START = `[info] [2026-09-28T09:59:00.000Z] START tip=${'f'.repeat(40,)} pipeline=sha256-tree-v1:9ae5 files=1 `
  + 'pending=1 done=0';

/**
 Lines of the calls a pass-run log holds: one completed Hyper call with its stream.
 */
const CALL_LINES: readonly string[] = [
  streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: HYPER_ID, outcome: 'completed', content: 7, },),
  spendLine({
    stamp: '2026-09-28T10:00:00.020Z',
    tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=100`,
  },),
];

/**
 Path of the one record file a run folder holds.

 @param runDir - run folder written by the fixture

 @returns Path of its file

 @example
 ```ts
 const file = await recordFileOf({ runDir, },);
 ```
 */
async function recordFileOf({ runDir, }: { readonly runDir: string; },): Promise<string> {
  /**
   Everything under the run folder.
   */
  const names = await readdir(
    runDir,
    { recursive: true, },
  );
  /**
   Names of record files.
   */
  const files = names.filter(function isRecordFile(name,): boolean {
    return name.endsWith('.log.jsonl',);
  },);
  return join(
    runDir,
    files[0] ?? 'no record file',
  );
}

await describe({
  name: 'cap-census over run folders',
  concurrency: 1,
  children: [
    it({
      name: 'FINDS THE RECORD FILES under a named directory and takes no `.log` capture of the console for a log',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-run-folders-', },);
        /**
         Run folder of a pass.
         */
        const pass = await writeRunFolder({
          sinon: ctx.sinon,
          dir: scratch.path,
          name: 'corpus-pass-tabby',
          day: '2026-09-28',
          lines: [PASS_START, ...CALL_LINES,],
        },);
        await writeFile(
          join(
            scratch.path,
            'corpus-pass-old.log',
          ),
          'START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 pipeline=x\n',
        );

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [await recordFileOf({ runDir: pass, },),],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),

    it({
      name: 'READS THE CALLS OF A PASS-RUN FILE as it reads them from the same lines as console text, and tells a '
        + 'suite\'s START from a pass\'s by its commit id',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-run-folders-', },);
        /**
         Run folder of a pass and run folder of a suite's fixture pass.
         */
        const pass = await writeRunFolder({
          sinon: ctx.sinon,
          dir: scratch.path,
          name: 'corpus-pass-tabby',
          day: '2026-09-28',
          lines: [PASS_START, ...CALL_LINES,],
        },);
        const suite = await writeRunFolder({
          sinon: ctx.sinon,
          dir: scratch.path,
          name: 'suite-fixture',
          day: '2026-09-28',
          lines: [FIXTURE_PASS_START, ...CALL_LINES,],
        },);

        expect(await capCensusPassRunReading({ path: await recordFileOf({ runDir: pass, },), },),).toEqual(
          readCapLog({ lines: CALL_LINES, },),
        );
        expect(await capCensusPassRunReading({ path: await recordFileOf({ runDir: suite, },), },),).toBe(
          'not-a-pass-run',
        );
      },
    },),

    it({
      name: 'PRINTS THE SAME CENSUS OF A RUN FOLDER as of its lines: one pass-run log, one completed call',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'cap-census-run-folders-', },);
        await writeRunFolder({
          sinon: ctx.sinon,
          dir: scratch.path,
          name: 'corpus-pass-tabby',
          day: '2026-09-28',
          lines: [PASS_START, ...CALL_LINES,],
        },);

        await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

        expect(printed.lines[0],).toBe(
          'cap-census: 1 log, 1 pass-run log, 1 completed call, 0 on ids no card names, 0 paths unreadable, '
            + '0 dangling links with no log name passed over; '
            + 'lines left out for a stamp the logger did not write: 0',
        );
      },
    },),
  ],
},);

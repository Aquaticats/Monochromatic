/**
 Tests that the slice cost report reads the records a pass keeps in its run folder.

 THE DIFFERENTIAL CASE: the report over a run folder is the report the same lines gave as console text, down to the
 count of cost lines and of dropped ones, and a folder with no record file is refused in words, since an empty report
 would read as a run whose slices were free.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { reportSliceCost, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { writeRunFolder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/run-log-write.test-fixture.ts';
import {
  COST_LOG,
  READ_NOTE,
  SPREAD_NOTE,
} from './slice-cost-report.test-fixture.ts';

await describe({
  name: 'slice-cost-report over a run folder',
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS THE REPORT OF A RUN FOLDER\'S RECORDS as it printed the same lines read as text',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'slice-cost-report-folder-', },);
        /**
         Run folder holding the lines of the cost log.
         */
        const runDir = await writeRunFolder({
          sinon: ctx.sinon,
          dir: scratch.path,
          name: 'corpus-pass-tabby',
          day: '2026-08-17',
          lines: COST_LOG.split('\n',)
            .filter(function present(line,): boolean {
              return line !== '';
            },),
        },);

        await reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [runDir,], },), },);

        expect(`${printed.lines.join('\n',)}\n`,).toBe([
          runDir,
          '5 cost lines, 1 dropped',
          '',
          'WHAT A SLICE COSTS, BY THE SIZE OF ITS ORIGINAL',
          '  under     50 chars  slices    2  min/slice   0.75  ms/char  1125.0',
          '  under    200 chars  slices    1  min/slice   2.00  ms/char   800.0',
          '  under    any chars  slices    1  min/slice   6.00  ms/char   120.0',
          '',
          'SPREAD, WHICH THE BANDS AVERAGE AWAY',
          '  cheapest    40 chars    0.50 min  consolidation',
          '  dearest   3000 chars    6.00 min  translate',
          '  ratio     12.0x',
          SPREAD_NOTE,
          READ_NOTE,
          '',
          'BY LANE',
          '  repair        slices    1  total      1.0 min',
          '  translate     slices    2  total      8.0 min',
          '  consolidation slices    1  total      0.5 min',
          '',
        ].join('\n',),);
      },
    },),

    it({
      name: 'REFUSES A FOLDER THAT HOLDS NO RECORD FILE, naming it, since an empty report would read as free slices',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'slice-cost-report-folder-', },);
        /**
         Run folder of a pass that never started.
         */
        const empty = join(
          scratch.path,
          'corpus-pass-nothing',
        );
        await mkdir(
          join(
            empty,
            'log',
          ),
          { recursive: true, },
        );

        /**
         What the report threw.
         */
        const refusal = await rejectionOf({
          promise: reportSliceCost({ line: lineOf({ command: 'slice-cost-report', typed: [empty,], },), },),
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toContain(`${empty} holds no record file (log/<day>/*.log.jsonl)`,);
      },
    },),
  ],
},);

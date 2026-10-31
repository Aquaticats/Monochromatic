/**
 Tests that the quick writer of record files the report cases use writes the bytes the shared logger's file sink
 writes.

 `report-log-write.test-fixture.ts` renders a case's console-form line as a JSON object without starting a logger,
 which keeps hundreds of lines of cases cheap to write; `run-log-write.test-fixture.ts` has the logger write the same
 lines. A record written by hand that the sink would not write is a report case reading a file no run leaves, so the two
 are held side by side here.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scratchDir, } from '../scratch-dir.test-fixture.ts';
import { writeLog, } from './report-log-write.test-fixture.ts';
import { writeRecordFile, } from './run-log-write.test-fixture.ts';

await describe({
  name: writeLog.name,
  concurrency: 1,
  children: [
    it({
      name: 'WRITES THE BYTES THE FILE SINK WRITES for a console-form line, and a foreign or cut line as it stands',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'report-log-write-', },);
        /**
         Lines of a case: two records, a bare line, a cut line and a line with an instant the logger never writes.
         */
        const lines = [
          '[info] [2026-10-23T12:00:00.000Z] [translation-repair] purr',
          String.raw`[warn] [2026-10-23T12:00:01.250Z] [Tabby] [attempt 2] nap with "quotes", a \ and ☃`,
          'a bare line',
          '{"level":"info","message":"cut off",',
          '[info] [2026-10-23T12:00:02] no zone on the instant',
        ];

        /**
         Path the quick writer wrote.
         */
        const quick = await writeLog({
          dir: scratch.path,
          name: 'quick.log',
          lines,
        },);
        /**
         Path the logger wrote.
         */
        const real = await writeRecordFile({
          sinon: ctx.sinon,
          path: join(
            scratch.path,
            'real.log.jsonl',
          ),
          lines,
        },);

        expect(quick.endsWith('quick.log.jsonl',),).toBe(true,);
        expect(await readFile(
          quick,
          'utf8',
        ),).toBe(await readFile(
          real,
          'utf8',
        ),);
      },
    },),
  ],
},);

/**
 Tests for recording what a command says at its end (ledger B1111): the closing lines become info records of the
 installed run log when the scope ends, in the order the command prints them, and nothing is printed or recorded where
 no lines were handed.

 Every folder is made under a scratch directory. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readRunLogText, } from '@monochromatic-dev/translation-repair-glossary';
import {
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { recordingClosing, } from '../../dist/final/node/index.mjs';

await describe({
  name: recordingClosing.name,
  concurrency: 1,
  children: [
    it({
      name: 'RECORDS THE CLOSING LINES AS INFO RECORDS in the order handed when the scope ends, and not before',
      fn: async (ctx) => {
        /**
         The console's output, which the record must not add to.
         */
        const printed = ctx.sinon.stub(
          console,
          'info',
        );
        await using scratch = await scratchDir({ prefix: 'pass-record-closing-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);

        {
          /**
           The record of the closing lines, made when this block ends.
           */
          await using _closing = recordingClosing({
            closing: {
              lines: async function closingLines(): Promise<readonly string[]> {
                return [
                  'SEAT cat asked=1 usable=1 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0',
                  'SEATS DARK: none',
                ];
              },
            },
          },);
          await runLog.flush();
          await expect(readRunLogText({ path: scratch.path, },),).rejects.toThrow('holds no record file',);
        }
        await runLog.flush();

        /**
         Lines of the folder's records.
         */
        const records = (await readRunLogText({ path: scratch.path, },)).split('\n',);
        expect(records.length,).toBe(3,);
        expect(records[0]?.startsWith('[info] [',),).toBe(true,);
        expect(records[0]?.endsWith('Z] SEAT cat asked=1 usable=1 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0',),)
          .toBe(true,);
        expect(records[1]?.endsWith('Z] SEATS DARK: none',),).toBe(true,);
        expect(printed.callCount,).toBe(0,);
      },
    },),

    it({
      name: 'RECORDS NOTHING where no closing lines were handed',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-record-closing-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);

        {
          /**
           The record of nothing, made when this block ends.
           */
          await using _closing = recordingClosing({},);
        }
        await runLog.flush();

        await expect(readRunLogText({ path: scratch.path, },),).rejects.toThrow('holds no record file',);
      },
    },),
  ],
},);

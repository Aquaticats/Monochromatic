/**
 Tests for the logger a pass hands the work of one entry (ledger B1110).

 THE LOGGER IS ROOTED ON THE RUN LOG: a line it writes is a record of the installed run log, tagged with the entry and
 the attempt, and it is the default logger's line while no run log is installed. The run log is installed through the built typography
 package, which the built pass keeps external, so both reach one install.

 Every folder is written under a scratch directory. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  inEntryLogContext,
  installRunLog,
  openRunLog,
} from '@monochromatic-dev/translation-repair-typography';
import { readRunLogText, } from '@monochromatic-dev/translation-repair-glossary';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { entryLoggerOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: entryLoggerOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'WRITES A RECORD OF THE INSTALLED RUN LOG naming the entry and the attempt, at the level it was called at, '
        + 'beside the console',
      fn: async ctx => {
        /**
         The console's warnings, held off the test's output.
         */
        const warned = ctx.sinon.stub(
          console,
          'warn',
        );
        await using scratch = await scratchDir({ prefix: 'pass-entry-logger-', },);
        await using runLog = await openRunLog({ runDir: scratch.path, },);
        using _installed = installRunLog({ log: runLog, },);

        await inEntryLogContext({
          entry: 'Tabby',
          generation: 'sha256-tree-v1:0',
          attempt: 2,
          run: async () => {
            entryLoggerOf({ entryId: 'Tabby', },).warn('entry Tabby: purrs',);
          },
        },);
        await runLog.flush();

        /**
         Lines of the folder's records, the closing line feed leaving an empty piece.
         */
        const records = (await readRunLogText({ path: scratch.path, },)).split('\n',);
        expect(records.length,).toBe(2,);
        expect(records[0]?.startsWith('[warn] [',),).toBe(true,);
        expect(records[0]?.endsWith('Z] [Tabby] [attempt 2] entry Tabby: purrs',),).toBe(true,);
        expect(warned.callCount,).toBe(1,);
      },
    },),

    it({
      name: 'WRITES TO THE DEFAULT LOGGER while no run log is installed, so a runner that keeps no run folder prints as it did',
      fn: async ctx => {
        /**
         The console's warnings, which the default logger writes.
         */
        const warned = ctx.sinon.stub(
          console,
          'warn',
        );

        entryLoggerOf({ entryId: 'Tabby', },).warn('entry Tabby: purrs',);
        await entryLoggerOf({ entryId: 'Tabby', },).flush();

        expect(warned.args.flat().join('\n',),).toContain('[Tabby] entry Tabby: purrs',);
      },
    },),
  ],
},);

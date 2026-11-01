/**
 Tests the fixture that collects the lines an entry's loggers write (ledger B963): it must read every console method the
 logger writes to, since a fixture that read two left the other levels unread and a mutant that dropped one of the two
 survived every case that used it.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  entryLinesWritten,
  linesRepeatingAttempt,
  linesWithoutAttempt,
} from './entry-log-lines.test-fixture.ts';

await describe({
  name: entryLinesWritten.name,
  // SEQUENTIAL: each case diverts the one global `console`.
  concurrency: 1,
  children: [
    it({
      name: 'READS THE LINES THE LOGGER WRITES AT EVERY LEVEL A RUN PRINTS BY DEFAULT: info, warn, error and a fatal '
        + 'line, which goes out as an error (trace and debug lines are filtered out by the default level and are not written)',
      fn: async (ctx) => {
        /**
         Logger tagged with the entry, as an entry's loggers are.
         */
        const l = tagged({ tag: 'Tabby', },);

        /**
         What the fixture collected while the logger wrote one line at each level.
         */
        const lines = await entryLinesWritten({
          sinon: ctx.sinon,
          entryId: 'Tabby',
          run: async () => {
            l.info('purrs',);
            l.warn('hisses',);
            l.error('knocks a cup over',);
            l.fatal('naps forever',);
            await l.flush();
          },
        },);

        /**
         What the logger wrote that the fixture did not return exactly once.
         */
        const notReadOnce = [
          'purrs',
          'hisses',
          'knocks a cup over',
          'naps forever',
        ].filter(function isNotReadOnce(said,): boolean {
          return lines.filter(function saysIt(line,): boolean {
            return line.includes(said,);
          },).length !== 1;
        },);

        expect(notReadOnce,).toStrictEqual([],);
      },
    },),
    it({
      name: 'LEAVES OUT A LINE THAT NAMES ANOTHER ENTRY, an error line included, so a case reads the lines of its own '
        + 'entry only',
      fn: async (ctx) => {
        /**
         Loggers tagged with two entries.
         */
        const tabby = tagged({ tag: 'Tabby', },);
        const marmalade = tagged({ tag: 'Marmalade', },);

        /**
         What the fixture collected for Tabby while both entries wrote.
         */
        const lines = await entryLinesWritten({
          sinon: ctx.sinon,
          entryId: 'Tabby',
          run: async () => {
            marmalade.error('bats at the curtain',);
            tabby.error('stretches',);
            await tabby.flush();
          },
        },);

        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('stretches',);
      },
    },),
    it({
      name: `${linesWithoutAttempt.name} and ${linesRepeatingAttempt.name} tell a line that holds no attempt, or the `
        + 'attempt twice, from one that holds it once',
      fn: async () => {
        /**
         Lines with the attempt once, twice and not at all.
         */
        const lines = [
          '[Tabby] [attempt 2] purrs',
          '[Tabby] [attempt 2] [attempt 2] purrs',
          '[Tabby] purrs',
        ];

        expect(linesWithoutAttempt({
          lines,
          attempt: 2,
        },),).toStrictEqual(['[Tabby] purrs',],);
        expect(linesRepeatingAttempt({
          lines,
          attempt: 2,
        },),).toStrictEqual(['[Tabby] [attempt 2] [attempt 2] purrs',],);
      },
    },),
  ],
},);

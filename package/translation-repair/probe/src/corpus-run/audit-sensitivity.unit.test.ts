/**
 Tests for the audit sensitivity runner: the built command, run with every
 provider key withheld so it can only refuse.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  noKeyRefusal,
  scratchPlaces,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'audit-sensitivity as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES as stated and exits 6 when no provider key is set, before any call and with nothing on stdout',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'audit-sensitivity-', },);

            /**
             What the command wrote with every key withheld.
             */
            const run = await runBuiltCommand({
              command: 'audit-sensitivity',
              packageEntry: PROBE_ENTRY,
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(noKeyRefusal({ command: 'audit-sensitivity', },),);
          },
        },),

        it({
          name: 'REFUSES an argument it does not read as stated and exits 6, naming the argument and the usage',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'audit-sensitivity-', },);

            /**
             What the command wrote given one argument.
             */
            const run = await runBuiltCommand({
              command: 'audit-sensitivity',
              packageEntry: PROBE_ENTRY,
              args: ['--bogus',],
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'audit-sensitivity: --bogus is not a flag this command reads. Usage: audit-sensitivity\n',
            );
          },
        },),
      ],
    },),
  ],
},);

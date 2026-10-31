/**
 Tests the guard of the built commands at the boundary: a child that holds a
 provider key and does not name the command as the one it means to start
 refuses, and the same child naming it (as the command's task does), or a
 child that holds no key, goes on to what each command did before the guard.

 THE KEY IS INVENTED. `WHISKER_API_KEY` is no provider's variable, so a child
 holding it passes the guard's key test and still builds no client; nothing
 here can reach a provider. Every child is started by `runBuiltCommand` of the
 shared child fixture, whose `startedBy` says who the marker names, and the
 cases in this file name a runs directory, a lookup cache and a corpus clone
 under a scratch directory.

 Two commands of this package stand for two of the three kinds: one report
 (`spend-report`) and one census (`cap-census`). The kind that would build a
 client (`probe-sensitivity`) belongs to the probe package, which runs the same
 suite from `task-runner-guard-built.test-fixture.ts` in its own tests. The
 verdict's own cases are in `task-runner-guard.unit.test.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { describe, } from '@monochromatic-dev/test-harness-core/ts';

import { RUN_ENTRY, } from './built-entry.test-fixture.ts';
import {
  type GuardedCommand,
  guardedSuite,
} from './task-runner-guard-built.test-fixture.ts';
import { HYPER_CHEAP, } from './spend-report.test-fixture.ts';

/**
 The two commands of this package the cases run.
 */
const COMMANDS: readonly GuardedCommand[] = [
  {
    command: 'spend-report',
    owner: 'run',
    packageEntry: RUN_ENTRY,
    prepare: async function oneLog(directory: string,): Promise<readonly string[]> {
      /**
       Log of one call.
       */
      const log = join(
        directory,
        'run.log.jsonl',
      );
      await writeFile(
        log,
        `${HYPER_CHEAP}\n`,
      );
      return [log,];
    },
    before: {
      code: 0,
      stdout: [
        'spend-report: 1 log, 1 line, 1 seat',
        'metered seats, priced at rates read 2026-10-11:',
        '  gemma-4-26b-a4b-it: 8.64 credits (100.0%) over 1 call, in 1000000=1.96 out 1000000=6.68',
        'metered run total: 8.64 credits',
        '',
      ].join('\n',),
      stderr: '',
    },
  },
  {
    command: 'cap-census',
    owner: 'run',
    packageEntry: RUN_ENTRY,
    prepare: async function emptyDirectory(directory: string,): Promise<readonly string[]> {
      /**
       Directory holding no log.
       */
      const logs = join(
        directory,
        'logs',
      );
      await mkdir(logs,);
      return [logs,];
    },
    before: {
      code: 0,
      stdout: [
        'cap-census: 0 logs, 0 pass-run logs, 0 completed calls, 0 on ids no card names, 0 paths unreadable, '
          + '0 dangling links with no log name passed over; '
          + 'lines left out for a stamp the logger did not write: 0',
        'A rule reading can differ from a card for reasons that are not the model: a narrower log scope than the '
          + '2026-09-09 table, calls from before the caps, or calls the cap itself cut. Read each provider\'s cut '
          + 'columns before moving a cap; completion-cap.ts records the 2026-09-28 reading.',
        '',
      ].join('\n',),
      stderr: '',
    },
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: COMMANDS.map(function guardedCase(guarded,) {
    return guardedSuite({ guarded, },);
  },),
},);

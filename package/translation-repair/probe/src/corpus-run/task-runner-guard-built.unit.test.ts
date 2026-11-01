/**
 Tests the guard of the built `probe-sensitivity` command at the boundary: a child that holds a provider key and does
 not name the command as the one it means to start refuses, and the same child naming it (as the command's task does),
 or a child that holds no key, goes on to what the command did before the guard.

 THE KEY IS INVENTED. `WHISKER_API_KEY` is no provider's variable, so a child holding it passes the guard's key test
 and still builds no client; nothing here can reach a provider. Every child is started by `runBuiltCommand` of the
 shared child fixture, whose `startedBy` says who the marker names, and the cases in this file name a runs directory, a
 lookup cache and a corpus clone under a scratch directory.

 `probe-sensitivity` stands for the kind of command that would build a client. The report and the census that stand for
 the other two kinds belong to the run package, which runs the same suite from `task-runner-guard-built.test-fixture.ts`
 in its own tests. The verdict's own cases are in `task-runner-guard.unit.test.ts` of the adjudication package.

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
import {
  type GuardedCommand,
  guardedSuite,
  guardRefusal,
  REFUSED_AS_STATED,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/task-runner-guard-built.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  noKeyRefusal,
  scratchPlaces,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 The command of this package the cases run.
 */
const PROBE_SENSITIVITY: GuardedCommand = {
  command: 'probe-sensitivity',
  owner: 'probe',
  packageEntry: PROBE_ENTRY,
  prepare: async function noArguments(): Promise<readonly string[]> {
    return [];
  },
  before: {
    code: REFUSED_AS_STATED,
    stdout: '',
    stderr: noKeyRefusal({ command: 'probe-sensitivity', },),
  },
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    guardedSuite({ guarded: PROBE_SENSITIVITY, },),

    describe({
      name: 'guard order as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a process that holds a key and carries another command\'s marker, naming that command, '
            + 'before it reads an argument it would refuse',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'task-runner-guard-', },);
            expect(await runBuiltCommand({
              command: 'probe-sensitivity',
              packageEntry: PROBE_ENTRY,
              args: ['--bogus',],
              env: {
                ...scratchPlaces({
                  runsDir: scratch.path,
                  scratchDir: scratch.path,
                },),
                WHISKER_API_KEY: 'purr',
              },
              startedBy: { otherCommand: 'spend-report', },
            },),).toEqual({
              code: REFUSED_AS_STATED,
              stdout: '',
              stderr: `${guardRefusal({ command: 'probe-sensitivity', owner: 'probe', },).trimEnd()} `
                + 'TRANSLATION_REPAIR_STARTED_BY is set in this process to spend-report, which names another '
                + 'command.\n',
            },);
          },
        },),
      ],
    },),
  ],
},);

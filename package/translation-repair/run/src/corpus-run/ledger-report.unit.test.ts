/**
 Tests for the ledger report command at its boundary: the built command run in
 a child process over a ledger the case writes.

 The command spends no quota and asks no model, so what only the built command
 can show is what an operator meets: the whole report on stdout, the exit code
 each state leaves, and the refusal of a malformed flag as one line. The
 functions it prints with have their own files.

 THE CHILD CARRIES NO PROVIDER KEY. `runBuiltCommand` removes every variable
 whose name ends in `_API_KEY` from the environment it hands the child, and
 each case points the runs directory at a scratch directory, so nothing here
 reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  NAP_CONTEST,
  PURR_CONTEST,
  SHORTFALL_SENTENCE,
  SUMMARY_POINTER,
  writeContest,
} from './ledger-report.test-fixture.ts';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Name of the first contest file a case writes.
 */
const FIRST_NAME = '2026-08-25T01-00-00.000Z-p1-000001.json';

/**
 Name of the second contest file a case writes.
 */
const SECOND_NAME = '2026-08-25T01-00-00.000Z-p1-000002.json';

await describe({
  name: 'ledger-report as built',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'REPORTS the whole ledger and exits 0 when every contest reads',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'ledger-report-built-', },);
        await writeContest({
          runsDir: scratch.path,
          name: FIRST_NAME,
          contest: NAP_CONTEST,
        },);
        await writeContest({
          runsDir: scratch.path,
          name: SECOND_NAME,
          contest: PURR_CONTEST,
        },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'ledger-report',
          packageEntry: RUN_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            `ledger-report: 2 contests under ${scratch.path}`,
            '1 ballot named nothing, 1 named a candidate the slate did not have',
            '  tabby-1: 2 candidates, 1 chosen, 33.3% of 3 disinterested ballots, 1 self-vote',
            '  calico-2: 2 candidates, 0 chosen, 0.0% of 4 disinterested ballots, 0 self-votes',
            '',
            SUMMARY_POINTER,
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'PRINTS one seat\'s candidates with the judges\' reasons and exits 0 when a model is named',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'ledger-report-built-', },);
        await writeContest({
          runsDir: scratch.path,
          name: FIRST_NAME,
          contest: NAP_CONTEST,
        },);
        await writeContest({
          runsDir: scratch.path,
          name: SECOND_NAME,
          contest: PURR_CONTEST,
        },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'ledger-report',
          packageEntry: RUN_ENTRY,
          args: [
            '--model',
            'tabby-1',
          ],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            `ledger-report: 2 contests under ${scratch.path}`,
            'tabby-1 wrote 2 candidates, 1 chosen',
            '',
            '--- 1 --- CHOSEN --- render the nap passage',
            'The cat naps in the sun.',
            '  siamese-3: keeps the sun and the nap',
            '',
            '--- 2 --- not chosen --- render the purr passage',
            'The purr rolls on.',
            '  (no disinterested judge named this candidate)',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'SAYS NOTHING WAS RECORDED and exits 1 when the run wrote no ledger',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'ledger-report-built-', },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'ledger-report',
          packageEntry: RUN_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 1,
          stdout: [
            `ledger-report: 0 contests under ${scratch.path}`,
            'NOTHING RECORDED. This run wrote no ledger, which is not the same as a run whose models '
              + 'wrote nothing: every run started before candidate-ledger.ts landed has none, and so does '
              + 'any run launched without TRANSLATION_REPAIR_RUNS_DIR set.',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'NAMES the file that would not read and exits 2 while still reporting the contests that did',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'ledger-report-built-', },);
        await writeContest({
          runsDir: scratch.path,
          name: FIRST_NAME,
          contest: NAP_CONTEST,
        },);
        await writeContest({
          runsDir: scratch.path,
          name: SECOND_NAME,
          contest: { task: 'render the purr passage', },
        },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'ledger-report',
          packageEntry: RUN_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 2,
          stdout: [
            `ledger-report: 1 contest under ${scratch.path}`,
            `  UNREADABLE ${SECOND_NAME}: ledger file ${SECOND_NAME} has no usable at`,
            `  1 of 2 ledger files could not be read. ${SHORTFALL_SENTENCE}`,
            '1 ballot named nothing, 0 named a candidate the slate did not have',
            '  tabby-1: 1 candidate, 1 chosen, 50.0% of 2 disinterested ballots, 1 self-vote',
            '  calico-2: 1 candidate, 0 chosen, 0.0% of 3 disinterested ballots, 0 self-votes',
            '',
            SUMMARY_POINTER,
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'REFUSES a model flag with nothing after it as one line and exits 6, before any file is read',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'ledger-report-built-', },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'ledger-report',
          packageEntry: RUN_ENTRY,
          args: ['--model',],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stdout,).toBe('',);
        expect(run.stderr,).toBe(
          'ledger-report: --model needs a value written after it. Usage: ledger-report [--model <seat id>]\n',
        );
      },
    },),
  ],
},);

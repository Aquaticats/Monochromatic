/**
 Tests for the meter report command at its boundary: the built command run in
 a child process over logs the case writes.

 The command spends no quota and asks no model, so what only the built command
 can show is what an operator meets: the whole report on stdout, the exit code a
 log with no reading leaves, and a refusal as one line with the exit code a
 stated refusal leaves. The functions it prints with have their own files.

 THE CHILD CARRIES NO PROVIDER KEY. `runBuiltCommand` removes every variable
 whose name ends in `_API_KEY` from the environment it hands the child, and
 each case points the runs directory at a scratch directory, so nothing here
 reads a real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ASKING_NOTE,
  BOTH_DRY,
  BOTH_WET_AGAIN,
  EVERYTHING_WET,
  LEVEL_NOT_RECORDED,
  NOTHING_RECORDED_LINE,
  SYNTHETIC_DRY,
} from './meter-report.test-fixture.ts';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Exit code the report leaves when its logs held no reading.
 */
const NOTHING_RECORDED = 1;

await describe({
  name: 'meter-report as built',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'REPORTS every provider\'s availability over two logs read as one record and exits 0',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'meter-report-built-', },);
        /**
         Two logs whose readings overlap by one, each with a record that will not read in the first.
         */
        const first = join(
          scratch.path,
          'first.log',
        );
        const second = join(
          scratch.path,
          'second.log',
        );
        await writeFile(
          first,
          [
            EVERYTHING_WET,
            SYNTHETIC_DRY,
            'a line no reader takes',
            'x METERS synthetic=wet hyp',
          ].join('\n',),
        );
        await writeFile(
          second,
          [
            BOTH_DRY,
            BOTH_WET_AGAIN,
            EVERYTHING_WET,
          ].join('\n',),
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'meter-report',
          packageEntry: RUN_ENTRY,
          args: [
            first,
            second,
          ],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'meter-report: logs=2 readings=4 unread=1',
            '  window 2026-08-25T10:00:00.000Z .. 2026-08-25T12:00:00.000Z (2h)',
            ASKING_NOTE,
            '',
            'neuralwatt: wet=0 dry=0 unreadable=0',
            '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
            '  longest outage: none, no reading found this provider out',
            LEVEL_NOT_RECORDED,
            '',
            'synthetic: wet=2 dry=2 unreadable=0',
            '  spendable on 50.0% of readings that answered (2 of 4)',
            '  longest outage: at least 30m, at most 2h (2026-08-25T10:30:00.000Z .. 2026-08-25T11:00:00.000Z)',
            LEVEL_NOT_RECORDED,
            '',
            'bedrock: wet=0 dry=0 unreadable=0',
            '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
            '  longest outage: none, no reading found this provider out',
            LEVEL_NOT_RECORDED,
            '',
            'deepseek: wet=0 dry=0 unreadable=0',
            '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
            '  longest outage: none, no reading found this provider out',
            LEVEL_NOT_RECORDED,
            '',
            'xiaomi: wet=0 dry=0 unreadable=0',
            '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
            '  longest outage: none, no reading found this provider out',
            LEVEL_NOT_RECORDED,
            '',
            'anthropic: wet=0 dry=0 unreadable=0',
            '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
            '  longest outage: none, no reading found this provider out',
            LEVEL_NOT_RECORDED,
            '',
            'hyper: wet=3 dry=1 unreadable=0',
            '  spendable on 75.0% of readings that answered (3 of 4)',
            '  longest outage: at least 0s, at most 1h30m (2026-08-25T11:00:00.000Z .. 2026-08-25T11:00:00.000Z)',
            '  level first 2026-08-25T10:00:00.000Z: hyperBalance=2497',
            '  level last 2026-08-25T12:00:00.000Z: hyperBalance=500',
            '',
            'openrouter: wet=0 dry=0 unreadable=0',
            '  spendable on NO MEASURABLE FRACTION: no reading in this record answered',
            '  longest outage: none, no reading found this provider out',
            LEVEL_NOT_RECORDED,
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'SAYS NOTHING WAS RECORDED and exits 1 when the logs carry no reading',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'meter-report-built-', },);
        /**
         Log with no meter record in it.
         */
        const bare = join(
          scratch.path,
          'bare.log',
        );
        await writeFile(
          bare,
          'the cat slept',
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'meter-report',
          packageEntry: RUN_ENTRY,
          args: [bare,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: NOTHING_RECORDED,
          stdout: [
            'meter-report: logs=1 readings=0 unread=0',
            NOTHING_RECORDED_LINE,
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'REFUSES a log that cannot be read as one line naming it and its code, and exits 6',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'meter-report-built-', },);
        /**
         Path where no log stands.
         */
        const missing = join(
          scratch.path,
          'missing.log',
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'meter-report',
          packageEntry: RUN_ENTRY,
          args: [missing,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `meter-report: the log ${missing} could not be read (ENOENT), so nothing was counted. `
            + 'Name a log a pass, probe or calibration wrote, by a path that exists.\n',
        },);
      },
    },),

    it({
      name: 'REFUSES a command line naming no log as one line and exits 6, before any file is read',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'meter-report-built-', },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'meter-report',
          packageEntry: RUN_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'meter-report: this command needs <log file>. '
            + 'Usage: meter-report <log file> [<log file> ...]\n',
        },);
      },
    },),
  ],
},);

/**
 Tests for the run timing report command at its boundary: the built command
 run in a child process over logs the case writes.

 The command spends no quota and asks no model, so what only the built command
 can show is what an operator meets: the whole report on stdout, and a refusal
 as one line with the exit code a stated refusal leaves. The functions it
 prints with have their own files.

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
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ROUND_NEVER_LINE,
  ROUND_STOOD_LINE,
  timedCallLine,
  UNSTAMPED_CALL_LINE,
  UNTIMED_CALL_LINE,
} from './run-timing-report.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 What the report says of a log that holds no round line.
 */
const NO_ROUND_LINE = 'NO ROUND LINE. This log predates round lines and call durations, so how long each fan-out '
  + 'took and how much of that was spent waiting after quorum are both unrecorded. That is not the same as a run '
  + 'that never waited.';

await describe({
  name: 'run-timing-report as built',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'REPORTS the rounds and the calls in flight of two logs read as one run and exits 0',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-built-', },);
        /**
         Log of the rounds, and log of the calls.
         */
        const rounds = join(
          scratch.path,
          'rounds.log.jsonl',
        );
        const calls = join(
          scratch.path,
          'calls.log.jsonl',
        );
        await writeFile(
          rounds,
          [
            ROUND_STOOD_LINE,
            ROUND_NEVER_LINE,
          ].join('\n',),
        );
        await writeFile(
          calls,
          [
            timedCallLine({
              stamp: '2026-08-25T10:00:10.000Z',
              elapsedMs: 10_000,
            },),
            timedCallLine({
              stamp: '2026-08-25T10:00:15.000Z',
              elapsedMs: 10_000,
            },),
            UNTIMED_CALL_LINE,
            UNSTAMPED_CALL_LINE,
            'a line no timing reader takes',
          ].join('\n',),
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'run-timing-report',
          packageEntry: RUN_ENTRY,
          args: [
            rounds,
            calls,
          ],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'run-timing-report: 2 logs, 7 lines',
            'rounds                 2, 3.52min in total',
            '  without quorum       1',
            '  waiting after quorum 30.00s, 14.2% of round time',
            '  voices never heard   7',
            '1 completion line carries no elapsed field, predating call durations and leaving no interval to '
              + 'report. Any concurrency this report prints describes only the calls that could be timed.',
            'Completion lines whose stamp the logger did not write: 1. No instant places their calls, so any '
              + 'concurrency this report prints leaves them out.',
            'calls in flight        mean 1.33, peak 2',
            '  busy against span    20.00s of calls across 15.00s of run',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'SAYS NO ROUND LINE and NOTHING IN FLIGHT for a log with no timing line, and still exits 0',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-built-', },);
        /**
         Log written before the timing work.
         */
        const old = join(
          scratch.path,
          'old.log.jsonl',
        );
        await writeFile(
          old,
          'the cat slept',
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'run-timing-report',
          packageEntry: RUN_ENTRY,
          args: [old,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'run-timing-report: 1 log, 1 line',
            NO_ROUND_LINE,
            'NOTHING IN FLIGHT: no call in this log carried a duration, so nothing can be counted in flight, '
              + 'which is not the same as a run that made one call at a time.',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'COUNTS the lines a log holds, not the empty piece after its closing newline, and exits 0',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-built-', },);
        /**
         Log of one call, ended by a newline as every log a run wrote is.
         */
        const closed = join(
          scratch.path,
          'closed.log.jsonl',
        );
        await writeFile(
          closed,
          `${
            timedCallLine({
              stamp: '2026-08-25T10:00:10.000Z',
              elapsedMs: 10_000,
            },)
          }\n`,
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'run-timing-report',
          packageEntry: RUN_ENTRY,
          args: [closed,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'run-timing-report: 1 log, 1 line',
            NO_ROUND_LINE,
            'calls in flight        mean 1.00, peak 1',
            '  busy against span    10.00s of calls across 10.00s of run',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'REFUSES a log that cannot be read as one line naming it and its code, and exits 6',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-built-', },);
        /**
         Path where no log stands.
         */
        const missing = join(
          scratch.path,
          'missing.log.jsonl',
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'run-timing-report',
          packageEntry: RUN_ENTRY,
          args: [missing,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `run-timing-report: the log ${missing} could not be read (ENOENT), so nothing was counted. `
            + 'Name a run folder a pass wrote, or one of its .log.jsonl record files, by a path that exists.\n',
        },);
      },
    },),

    it({
      name: 'REFUSES a log named twice as one line naming it and exits 6, since its lines would be counted twice',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-built-', },);
        /**
         Log named twice on the command line.
         */
        const twice = join(
          scratch.path,
          'twice.log.jsonl',
        );
        await writeFile(
          twice,
          'the cat slept',
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'run-timing-report',
          packageEntry: RUN_ENTRY,
          args: [
            twice,
            twice,
          ],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `run-timing-report: the log ${twice} is named more than once, so its lines would be counted `
            + 'twice. Name each log once.\n',
        },);
      },
    },),

    it({
      name: 'REFUSES a command line naming no log as one line and exits 6, before any file is read',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-timing-report-built-', },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'run-timing-report',
          packageEntry: RUN_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'run-timing-report: this command needs <run folder or log file>. '
            + 'Usage: run-timing-report <run folder or log file> [<run folder or log file> ...]\n',
        },);
      },
    },),
  ],
},);

/**
 Tests for the spend report command at its boundary: the built command run in
 a child process over logs the case writes.

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
  CUT_SPEND_LINE,
  HYPER_CHEAP,
  HYPER_DEAR,
  HYPER_RECKONED,
  HYPER_UNPRICED,
  OPENROUTER_COSTED,
  OPENROUTER_UNCOSTED,
  SUBSCRIPTION_CALL,
} from './spend-report.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

await describe({
  name: 'spend-report as built',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'REPORTS what every kind of seat cost, the floors and the reckoned calls, and exits 0',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'spend-report-built-', },);
        /**
         Log of every kind of call.
         */
        const log = join(
          scratch.path,
          'run.log.jsonl',
        );
        await writeFile(
          log,
          [
            HYPER_DEAR,
            HYPER_CHEAP,
            HYPER_UNPRICED,
            SUBSCRIPTION_CALL,
            OPENROUTER_COSTED,
            OPENROUTER_UNCOSTED,
            HYPER_RECKONED,
            CUT_SPEND_LINE,
          ].join('\n',),
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'spend-report',
          packageEntry: RUN_ENTRY,
          args: [log,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'spend-report: 1 log, 8 lines, 5 seats',
            '1 line carried the marker and would not parse, so this report\'s totals are short by whatever those calls cost',
            'metered seats, priced at rates read 2026-10-11:',
            '  qwen3.8-max: 200.00 credits (95.9%) over 2 calls, in 2000000=80.00 out 1000000=120.00, '
              + '1 of them reckoned rather than reported',
            '  gemma-4-26b-a4b-it: 8.64 credits (4.1%) over 1 call, in 1000000=1.96 out 1000000=6.68',
            'metered run total: 208.64 credits',
            'UNPRICED, and not free: 1 metered seat has no row in the price table read 2026-10-11. '
              + 'This report\'s total omits whatever cost this seat would add',
            '  whisker-mini-9: 1 call, in 10 out 20',
            'OpenRouter seats, billed in USD per token, each priced from the cost= its own lines carried:',
            '  minimax/minimax-m3: 0.0625 USD (100.0%) over 2 calls, in 15 out 25, a floor: 1 call carried no cost',
            'OpenRouter run total: 0.0625 USD, never summed with this report\'s credits',
            'subscription seats, which bill no credits and are metered as a percentage of a weekly allowance on '
              + 'the METERS line:',
            '  hf:zai-org/GLM-5.3-Flash: 1 call, in 300 out 40',
            'RECKONED, NOT REPORTED: 1 call was written as reckonings, an attempt abandoned before it finished or '
              + 'a Bedrock attempt at its bound, so the tokens and cost this report shows for such calls are '
              + 'estimates or bounds rather than what the wire said (ledger P14)',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'COUNTS the lines a log holds, not the empty piece after its closing newline, and exits 0',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'spend-report-built-', },);
        /**
         Log of one call, ended by a newline as every log a run wrote is.
         */
        const closed = join(
          scratch.path,
          'closed.log.jsonl',
        );
        await writeFile(
          closed,
          `${HYPER_CHEAP}\n`,
        );

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'spend-report',
          packageEntry: RUN_ENTRY,
          args: [closed,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'spend-report: 1 log, 1 line, 1 seat',
            'metered seats, priced at rates read 2026-10-11:',
            '  gemma-4-26b-a4b-it: 8.64 credits (100.0%) over 1 call, in 1000000=1.96 out 1000000=6.68',
            'metered run total: 8.64 credits',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),

    it({
      name: 'REFUSES a log that cannot be read as one line naming it and its code, and exits 6',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'spend-report-built-', },);
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
          command: 'spend-report',
          packageEntry: RUN_ENTRY,
          args: [missing,],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `spend-report: the log ${missing} could not be read (ENOENT), so nothing was counted. `
            + 'Name a run folder a pass wrote, or one of its .log.jsonl record files, by a path that exists.\n',
        },);
      },
    },),

    it({
      name: 'REFUSES a command line naming no log as one line and exits 6, before any file is read',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'spend-report-built-', },);

        /**
         What the command wrote.
         */
        const run = await runBuiltCommand({
          command: 'spend-report',
          packageEntry: RUN_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'spend-report: this command needs <run folder or log file>. '
            + 'Usage: spend-report <run folder or log file> [<run folder or log file> ...]\n',
        },);
      },
    },),
  ],
},);

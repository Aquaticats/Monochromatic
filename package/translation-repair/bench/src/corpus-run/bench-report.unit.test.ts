/**
 Tests for the roster bench report: the widths a roster sweeps and which one
 it repeats, the file the rows land in, and the token sums on a summary line.

 The report is the only durable record of a width sweep, and it is written
 atomically under the runs directory; the write was unproven until this suite.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  BenchReportError,
  type BenchRow,
  benchWidths,
  summarizeBench,
  writeBenchReport,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { runsDirPointedAt, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/runs-dir-pointed.test-fixture.ts';

/**
 One slice benched at width two, first pass, whose archive already held a
 translation the judges kept; the exchanges it made are the case's to add.
 */
const KEPT_ROW: BenchRow = {
  width: 2,
  pass: 1,
  entryId: 'Kitten',
  index: 0,
  sourceChars: 10,
  incumbentChars: 10,
  translators: ['mao-1',],
  decision: 'accepted',
  keptIncumbent: true,
  voteWeight: 1,
  judgesAvailable: 1,
  ballots: 1,
  abstentions: 0,
  selfVotes: 0,
  round: {
    producers: [],
    ballots: [],
  },
  candidateCount: 1,
  heardTranslators: 1,
  findings: [],
  calls: [],
  ms: 1,
};

/**
 What every exchange of the token case shares; its three counts are the
 case's to add.
 */
const EXCHANGE = {
  schema: 'translate',
  modelId: 'mao-1',
  ms: 1,
  outcome: 'ok',
};

await describe({
  name: '',
  // ONE AT A TIME: a case diverts the process-wide `console.log` and another
  // points the process-wide runs directory across awaits (ledger B79).
  concurrency: 1,
  children: [
    describe({
      name: writeBenchReport.name,
      concurrency: 1,
      children: [
        it({
          name: 'LANDS roster-bench/rows.json under the runs directory it is handed, not the one the environment '
            + 'names, carrying the commit, the widths, the repeat and the roster beside the rows, readable as JSON',
          fn: async () => {
            /**
             Directory the environment names, which the writer must not use.
             */
            await using elsewhere = await scratchDir({ prefix: 'bench-report-elsewhere-', },);
            using pointed = runsDirPointedAt({ path: elsewhere.path, },);
            /**
             Disposable runs directory the writer is handed.
             */
            await using scratch = await scratchDir({ prefix: 'bench-report-', },);
            /**
             Working path of the runs directory.
             */
            const runsDir = scratch.path;

            await writeBenchReport({
              runsDir,
              rows: [],
              headSha: 'a'.repeat(40,),
              widths: [
                3,
                6,
              ],
              repeated: 3,
              roster: ['hf:cat/Cat-A',],
            },);

            /**
             What landed, parsed.
             */
            const landed: unknown = JSON.parse(
              await readFile(join(runsDir, 'roster-bench', 'rows.json',), 'utf8',),
            );

            expect(landed,).toEqual({
              headSha: 'a'.repeat(40,),
              widths: [
                3,
                6,
              ],
              repeated: 3,
              roster: ['hf:cat/Cat-A',],
              rows: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: benchWidths.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SWEEPS every width from two seats to the whole roster and REPEATS the middle one: three of two '
            + 'to four, and the upper of the two middle widths, four, of two to five',
          fn: async () => {
            expect(benchWidths({ roster: ['mao-1', 'mao-2', 'mao-3', 'mao-4',], },),).toEqual({
              widths: [
                2,
                3,
                4,
              ],
              repeated: 3,
            },);
            expect(benchWidths({ roster: ['mao-1', 'mao-2', 'mao-3', 'mao-4', 'mao-5',], },),).toEqual({
              widths: [
                2,
                3,
                4,
                5,
              ],
              repeated: 4,
            },);
          },
        },),

        it({
          name: 'REFUSES a roster of no seat and a roster of one as stated refusals, each by the count of its '
            + 'seats, since neither leaves a width to vary and the roster is the operator\'s to widen',
          fn: async () => {
            /**
             What the empty roster raised.
             */
            const empty = caught(function widthsOfNoSeat(): unknown {
              return benchWidths({ roster: [], },);
            },);
            expect(empty,).toBeInstanceOf(BenchReportError,);
            expect(empty,).toBeInstanceOf(StatedRefusalError,);
            expect(String(empty,),).toBe('BenchReportError: a roster of 0 cannot be benched: nothing to vary',);

            /**
             What the roster of one seat raised.
             */
            const lone = caught(function widthsOfOneSeat(): unknown {
              return benchWidths({ roster: ['mao-1',], },);
            },);
            expect(lone,).toBeInstanceOf(BenchReportError,);
            expect(lone,).toBeInstanceOf(StatedRefusalError,);
            expect(String(lone,),).toBe('BenchReportError: a roster of 1 cannot be benched: nothing to vary',);
          },
        },),
      ],
    },),

    describe({
      name: summarizeBench.name,
      concurrency: 1,
      children: [
        it({
          name: 'SUMS the tokens of every exchange the rows of one width and pass made onto their line, the '
            + 'sending and the answering halves apart, beside how many exchanges there were',
          fn: async (ctx,) => {
            /**
             Two slices of one width and pass: the first made two exchanges,
             the second one, and no two counts are alike.
             */
            const rows: readonly BenchRow[] = [
              {
                ...KEPT_ROW,
                calls: [
                  {
                    ...EXCHANGE,
                    promptTokens: 1,
                    completionTokens: 2,
                    tokens: 3,
                  },
                  {
                    ...EXCHANGE,
                    promptTokens: 100,
                    completionTokens: 200,
                    tokens: 300,
                  },
                ],
              },
              {
                ...KEPT_ROW,
                index: 1,
                calls: [{
                  ...EXCHANGE,
                  promptTokens: 10,
                  completionTokens: 20,
                  tokens: 30,
                },],
              },
            ];
            using capture = divertingConsoleLog({ sinon: ctx.sinon, },);
            summarizeBench({ rows, },);
            expect(capture.lines,).toEqual([
              'BENCH width 2 pass 1: 2 slices (2 with an incumbent), declined 0, kept 2, self-votes 0, '
                + 'self-preference not put (no producer judged its own candidate), calls 3, '
                + 'tokens 333 (in 111, out 222), 1ms per slice',
            ],);
          },
        },),
      ],
    },),
  ],
},);

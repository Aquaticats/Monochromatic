/**
 Tests that the bench summary GROUPS ITS ROWS by the width and pass they were
 recorded at.

 WHAT THE REPORT IS FOR. The bench runs the same slices at more than one
 roster width, more than once, and prints one line per width and pass. Every
 number on a line, the slice count, the declines, the calls, the tokens and
 the milliseconds per slice, is a sum over whichever rows the grouping put
 there. Group them wrong and each line is a confident description of the wrong
 arm, with nothing in the output saying so.

 WHAT WAS MEASURED. On 2026-08-25, inverting the width filter failed no test
 in this package. The lines would then have carried each other's rows, which
 is precisely the comparison the bench exists to make.

 BOTH FILTERS ARE PINNED, since the rows here differ in width AND in pass, and
 the counts are chosen so no two groups hold the same number of rows: a line
 describing another group's rows cannot then coincide with the right answer.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type BenchRow,
  summarizeBench,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

//region Fixtures

/**
 Builds one bench row at a given width and pass.

 Everything the row carries beyond those two is held constant, so a line that
 described the wrong group could only be spotted by its COUNT, which is what
 the cases read.

 @param width - producers seated for the run this row came from

 @param pass - which pass over that width

 @param index - slice position, which keeps the rows distinguishable

 @returns Row shaped as the bench records one

 @example
 ```ts
 const row = rowAt({ width: 2, pass: 1, index: 0, },);
 ```
 */
function rowAt(
  {
    width,
    pass,
    index,
  }: {
    readonly width: number;
    readonly pass: number;
    readonly index: number;
  },
): BenchRow {
  return {
    width,
    pass,
    entryId: 'mittens-window',
    index,
    sourceChars: 9,
    incumbentChars: 34,
    translators: [],
    decision: 'replaced',
    keptIncumbent: false,
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
    ms: 1_000,
  };
}

//endregion Fixtures

await describe({
  name: summarizeBench.name,
  // ONE AT A TIME: its case diverts the process-wide `console.log` (ledger
  // B79).
  concurrency: 1,
  children: [
    it({
      name: 'PUTS EACH ROW UNDER ITS OWN WIDTH AND RUN, since every number on a line is a sum over '
        + 'whichever rows the grouping put there, and a line built from another arm\'s rows describes '
        + 'that arm while naming this one',
      fn: async (ctx) => {
        /**
         Rows at three distinct width-and-pass groups, of sizes two, one and
         three, so no group can be mistaken for another by its count.
         */
        const rows: readonly BenchRow[] = [
          rowAt({ width: 2, pass: 1, index: 0, },),
          rowAt({ width: 2, pass: 1, index: 1, },),
          rowAt({ width: 2, pass: 2, index: 0, },),
          rowAt({ width: 3, pass: 1, index: 0, },),
          rowAt({ width: 3, pass: 1, index: 1, },),
          rowAt({ width: 3, pass: 1, index: 2, },),
        ];

        /**
         Lines the summary printed.
         */
        const lines: string[] = [];

        {
          using capture = divertingConsoleLog({ sinon: ctx.sinon, },);
          summarizeBench({ rows, },);
          expect(capture.lines
            .length,).toBe(3,);
          lines.push(...capture.lines,);
        }

        /**
         Each line cut down to its group and its slice count with the noun
         the count chose, which is the only thing that distinguishes the
         groups here; the group of one slice shows the singular.
         */
        const counted = lines.map(function toCount(line,): string {
          return line.slice(
            0,
            line.indexOf(' (',),
          );
        },);

        expect(counted,).toStrictEqual([
          'BENCH width 2 pass 1: 2 slices',
          'BENCH width 2 pass 2: 1 slice',
          'BENCH width 3 pass 1: 3 slices',
        ],);
      },
    },),
  ],
},);

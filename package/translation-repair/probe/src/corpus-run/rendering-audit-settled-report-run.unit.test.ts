/**
 Tests for the settled report's run over runs written to a throwaway runs
 directory: which run it reads, when it asks for the directory, and the whole
 text it prints.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runSettledReport, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import {
  ARCHIVE,
  OTHER_TEXTS,
  ROSTER,
  rowFor,
  runOver,
  SAME_TEXTS,
  writeRun,
} from './settled-run-file.test-fixture.ts';

/**
 What the report prints over a run of fresh rows nobody audited twice.

 @param path - run the report read, as it prints it

 @param rows - rows the run bought

 @param noun - word for the count of rows

 @param between - text between the band and the closing lines

 @returns Everything the report logged, one line per newline

 @example
 ```ts
 expect(printed.lines.join('\n',),).toBe(reportText({ path, rows: 1, noun: 'subject', between: '', },),);
 ```
 */
function reportText(
  {
    path,
    rows,
    noun,
    between,
  }: {
    readonly path: string;
    readonly rows: number;
    readonly noun: string;
    readonly between: string;
  },
): string {
  /**
   The count as the lines print it.
   */
  const n = String(rows,);
  return `${path}\n${n} ${noun}\n\n`
    + 'THE TWO HALVES, READ APART\n'
    + '  ARCHIVE text  subjects=0  drew a claim=0  claims=0  corroborated=0  agreed=0  near=0  degraded=0\n'
    + `  FRESH   text  subjects=${n}  drew a claim=0  claims=0  corroborated=0  agreed=0  near=0  degraded=0\n\n`
    + 'WHAT A DOCUMENT WOULD CARRY AT THE SAME SLICES\n'
    + `  survives                                    subjects=${n}  claims=0\n`
    + '  A displaced subject was audited on wording no reader of a document would meet. '
    + 'An undecided one is waiting on a decision, not overruled.\n\n'
    + 'WHAT EACH AUDITOR THOUGHT WAS WORTH A CLAIM\n'
    + `  hf:cat/Tabby-1                                   asked=${n} answered=${n} lost=0 spoke on=0 claims=0 dropped=0\n`
    + `  hf:cat/Mouser-1                                  asked=${n} answered=0 lost=${n} spoke on=0 claims=0 dropped=0\n\n`
    + 'RELOCATION CANDIDATES: claim pairs=0 slice pairs=0\n\n'
    + 'INSTRUMENT BAND over texts this run audited twice\n'
    + '  NOTHING PAIRED. Either no text was audited twice, or the rows predate the recorded text identity '
    + 'that pairing needs. No band is quotable from this run, so no comparison in it resolves anything.\n'
    + `${between}\nTWO ENTRIES. Nothing here settles anything about a particular entry, and nothing here may gate `
    + 'what ships: the instrument\'s own error rate is unmeasured.\n'
    + `Archive that run read: ${ARCHIVE}`;
}

/**
 Writes an older run of one row and a newer run of two rows into a directory.

 @param runsDir - throwaway runs directory

 @returns Paths of the two runs

 @example
 ```ts
 const { older, newer, } = await twoRuns({ runsDir: scratch.path, },);
 ```
 */
async function twoRuns(
  { runsDir, }: { readonly runsDir: string; },
): Promise<{ readonly older: string; readonly newer: string; }> {
  return {
    older: await writeRun({
      runsDir,
      stamp: '2026-08-25T01-00-00.000Z',
      body: runOver({
        rows: [rowFor({
          sliceIndex: 0,
          texts: SAME_TEXTS,
        },),],
        roster: ROSTER,
      },),
    },),
    newer: await writeRun({
      runsDir,
      stamp: '2026-08-26T01-00-00.000Z',
      body: runOver({
        rows: [
          rowFor({
            sliceIndex: 0,
            texts: OTHER_TEXTS,
          },),
          rowFor({
            sliceIndex: 1,
            texts: SAME_TEXTS,
          },),
        ],
        roster: ROSTER,
      },),
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runSettledReport.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS THE RUN IT IS NAMED without asking for the runs directory, and says one subject in the singular',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'settled-report-run-', },);
            const { older, } = await twoRuns({ runsDir: scratch.path, },);
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Times the directory was asked for.
             */
            let asked = 0;
            await runSettledReport({
              line: lineOf({
                command: 'rendering-audit-settled-report',
                typed: [
                  '--run',
                  older,
                ],
              },),
              resolveRuns: function unexpected(): Promise<string> {
                asked += 1;
                return Promise.resolve(scratch.path,);
              },
            },);
            expect(asked,).toBe(0,);
            expect(printed.lines.join('\n',),).toBe(reportText({
              path: older,
              rows: 1,
              noun: 'subject',
              between: '',
            },),);
          },
        },),

        it({
          name: 'READS THE NEWEST RUN when none is named, asking for the runs directory once, and says several '
            + 'subjects in the plural',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'settled-report-run-', },);
            const { newer, } = await twoRuns({ runsDir: scratch.path, },);
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Times the directory was asked for.
             */
            let asked = 0;
            await runSettledReport({
              line: lineOf({
                command: 'rendering-audit-settled-report',
                typed: [],
              },),
              resolveRuns: function resolveScratch(): Promise<string> {
                asked += 1;
                return Promise.resolve(scratch.path,);
              },
            },);
            expect(asked,).toBe(1,);
            expect(printed.lines.join('\n',),).toBe(reportText({
              path: newer,
              rows: 2,
              noun: 'subjects',
              between: '',
            },),);
          },
        },),

        it({
          name: 'PAIRS THE RUN WITH AN EARLIER ONE when asked, printing the across-run band and what it left out',
          fn: async (ctx) => {
            await using scratch = await scratchDir({ prefix: 'settled-report-run-', },);
            const {
              older,
              newer,
            } = await twoRuns({ runsDir: scratch.path, },);
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await runSettledReport({
              line: lineOf({
                command: 'rendering-audit-settled-report',
                typed: [
                  '--run',
                  newer,
                  '--against',
                  older,
                ],
              },),
              resolveRuns: function resolveScratch(): Promise<string> {
                return Promise.resolve(scratch.path,);
              },
            },);
            expect(printed.lines.join('\n',),).toBe(reportText({
              path: newer,
              rows: 2,
              noun: 'subjects',
              between: `\nINSTRUMENT BAND over the same subjects in ${older}\n`
                + '  NOTHING PAIRED. Either no text was audited twice, or the rows predate the recorded text '
                + 'identity that pairing needs. No band is quotable from this run, so no comparison in it '
                + 'resolves anything.\n'
                + '  One slot recorded by BOTH runs and the text DISAGREES, so the archive moved between '
                + 'them and these are left out: naptime-20260825/mittens#0\n',
            },),);
          },
        },),
      ],
    },),
  ],
},);

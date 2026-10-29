/**
 Tests for the editor standing read's procedure: which directories it reads,
 the whole report it prints over them, the lines it writes to stderr and the
 exit code it returns.

 THE PROCEDURE RETURNS ITS CODE AND SETS NONE ON THE PROCESS, which the entry
 file does; `editor-standing-read.unit.test.ts` holds what the built command
 prints and exits with.

 STDERR LINES COME IN THE ORDER THE ARCHIVES WERE NAMED, whichever read ends
 first: the artifacts are read side by side, and a line printed from inside
 each read came out in completion order.

 Every artifact is written under a scratch directory and named by path, and
 the runs directory is a function a case scripts, so nothing here reads a
 real run.

 CAPTURING `console.log` AND `console.error` IS PROCESS-WIDE, which is why this
 file runs at `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportStandings, } from '../../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleError, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  carrying,
  JUDGED_SEATED_ROUND,
  JUDGED_STANDING,
  NO_ROUNDS,
  OBSERVATIONAL,
  OFF_ROSTER_ROUND,
  refusedEmpty,
  SEATED_ROUND,
  standingRun,
  summaryOf,
  WITHOUT_ROUNDS,
} from './editor-standing-read.test-fixture.ts';
import {
  captureCodedReport,
  type CodedReportCapture,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-run-capture.test-fixture.ts';

//region Fixtures

/**
 What one run of the procedure printed to both streams, returned and left.
 */
type StandingCapture = CodedReportCapture & {
  /**
   Lines printed with `console.error`, in order.
   */
  readonly said: readonly string[];
};

/**
 Runs the procedure over the directories typed, with both streams diverted
 and the process's exit code held.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param typed - directories typed after the command

 @param resolveRuns - names the runs directory, as the entry's resolver would

 @returns What it printed to each stream, the code it returned and the one it
 left on the process

 @example
 ```ts
 const captured = await standingsOf({ sinon: ctx.sinon, typed: [archive,], resolveRuns, },);
 ```
 */
async function standingsOf(
  {
    sinon,
    typed,
    resolveRuns,
  }: {
    readonly sinon: DisposableSandbox;
    readonly typed: readonly string[];
    readonly resolveRuns: () => Promise<string>;
  },
): Promise<StandingCapture> {
  using said = divertingConsoleError({ sinon, },);

  /**
   What the procedure printed to stdout, returned and left.
   */
  const captured = await captureCodedReport({
    sinon,
    run: async function reported(): Promise<number> {
      return await reportStandings({
        line: lineOf({
          command: 'editor-standing-read',
          typed,
        },),
        resolveRuns,
      },);
    },
  },);
  return {
    ...captured,
    said: said.lines,
  };
}

/**
 A runs directory no case should need, refusing if asked for.

 @returns Never

 @throws Error naming the read no case expected

 @example
 ```ts
 await standingsOf({ sinon, typed: [archive,], resolveRuns: unaskedRuns, },);
 ```
 */
function unaskedRuns(): Promise<string> {
  return Promise.reject(new Error('the runs directory was asked for though the command line named an archive',),);
}

/**
 The report's first two lines over the readings and counts given, as
 `console.log` prints them.

 @param counts - summary line's fields after its name, written as it
 writes them

 @returns Summary and observational note, each a line of its own

 @example
 ```ts
 const opening = openingOf({ counts: 'archives=1 artifacts=1 read=1 ...', },);
 ```
 */
function openingOf({ counts, }: { readonly counts: string; },): readonly string[] {
  return [
    `editor-standing-read: ${counts}`,
    OBSERVATIONAL.trimEnd(),
  ];
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: reportStandings.name,
      concurrency: 1,
      children: [
        it({
          name: 'RETURNS 1 AND LEAVES THE PROCESS CODE UNSET over an archive whose one artifact recorded no round, '
            + 'saying no round was recorded',
          fn: async (ctx,) => {
            await using run = await standingRun({ artifacts: { 'Mittens.json': carrying({ rounds: [], },), }, },);

            expect(await standingsOf({ sinon: ctx.sinon, typed: [run.dir,], resolveRuns: unaskedRuns, },),).toEqual({
              lines: [
                ...openingOf({
                  counts: 'archives=1 artifacts=1 read=1 earlierRoster=0 earlierSchema=0 digestsJudged=0 '
                    + 'digestsUnjudged=1 unjudgedDigestRounds=0',
                },),
                NO_ROUNDS.trimEnd(),
              ],
              returned: 1,
              left: 'unset',
              said: [],
            },);
          },
        },),

        it({
          name: 'RETURNS 0 AND LEAVES THE PROCESS CODE UNSET where a digest carried a judged round, printing its '
            + 'standing after the summary',
          fn: async (ctx,) => {
            await using run = await standingRun({
              artifacts: { 'Mittens.json': carrying({ rounds: [JUDGED_SEATED_ROUND,], },), },
            },);

            expect(await standingsOf({ sinon: ctx.sinon, typed: [run.dir,], resolveRuns: unaskedRuns, },),).toEqual({
              lines: [
                ...openingOf({
                  counts: 'archives=1 artifacts=1 read=1 earlierRoster=0 earlierSchema=0 digestsJudged=1 '
                    + 'digestsUnjudged=0 unjudgedDigestRounds=0',
                },),
                `\nsha256-tree-v1:${'0'.repeat(64,)} over 1 entry`,
                '  EDITOR : 1 judged round from 1 of 1 chunk',
                JUDGED_STANDING.trimEnd(),
                '  REFINER: 0 judged rounds from 0 of 1 chunk',
              ],
              returned: 0,
              left: 'unset',
              said: [],
            },);
          },
        },),

        it({
          name: 'PRINTS EACH JUDGED DIGEST APART, the one of more entries first, and counts a digest whose one round '
            + 'drew no ballot on the summary line alone, returning 0',
          fn: async (ctx,) => {
            await using run = await standingRun({
              artifacts: {
                'Mittens.json': carrying({ rounds: [JUDGED_SEATED_ROUND,], },),
                'Tabby.json': {
                  ...carrying({ rounds: [JUDGED_SEATED_ROUND,], },),
                  id: 'Tabby',
                },
                'Pepper.json': {
                  ...carrying({ rounds: [JUDGED_SEATED_ROUND,], },),
                  id: 'Pepper',
                  pipelineDigest: `sha256-tree-v1:${'1'.repeat(64,)}`,
                },
                'Whiskers.json': {
                  ...carrying({ rounds: [SEATED_ROUND,], },),
                  id: 'Whiskers',
                  pipelineDigest: `sha256-tree-v1:${'2'.repeat(64,)}`,
                },
              },
            },);

            expect(await standingsOf({ sinon: ctx.sinon, typed: [run.dir,], resolveRuns: unaskedRuns, },),).toEqual({
              lines: [
                ...openingOf({
                  counts: 'archives=1 artifacts=4 read=4 earlierRoster=0 earlierSchema=0 digestsJudged=2 '
                    + 'digestsUnjudged=1 unjudgedDigestRounds=1',
                },),
                `\nsha256-tree-v1:${'0'.repeat(64,)} over 2 entries`,
                '  EDITOR : 2 judged rounds from 2 of 2 chunks',
                `      ${SEAT_SYNTHETIC_VISION_WITHHELD}: 100.0% (2 of 2 disinterested ballots, over 2 candidates)`,
                '  REFINER: 0 judged rounds from 0 of 2 chunks',
                `\nsha256-tree-v1:${'1'.repeat(64,)} over 1 entry`,
                '  EDITOR : 1 judged round from 1 of 1 chunk',
                JUDGED_STANDING.trimEnd(),
                '  REFINER: 0 judged rounds from 0 of 1 chunk',
              ],
              returned: 0,
              left: 'unset',
              said: [],
            },);
          },
        },),

        it({
          name: 'READS THE RUNS DIRECTORY THE RESOLVER NAMES when the command line names none, counting no archive '
            + 'named, and asks it only then',
          fn: async (ctx,) => {
            await using run = await standingRun({ artifacts: { 'Mittens.json': carrying({ rounds: [SEATED_ROUND,], },), }, },);

            /**
             How often the procedure asked for the runs directory.
             */
            const asked: string[] = [];

            expect(await standingsOf({
              sinon: ctx.sinon,
              typed: [],
              resolveRuns: function runs(): Promise<string> {
                asked.push(run.dir,);
                return Promise.resolve(run.dir,);
              },
            },),).toEqual({
              lines: [
                ...openingOf({
                  counts: 'archives=0 artifacts=1 read=1 earlierRoster=0 earlierSchema=0 digestsJudged=0 '
                    + 'digestsUnjudged=1 unjudgedDigestRounds=1',
                },),
                '  NO JUDGED ROUNDS. 1 round was recorded here and drew no ballot: a slate of one candidate, which is '
                  + 'what every producer proposing the same wording leaves, needs no vote, and a panel whose every judge '
                  + 'abstained or failed casts none.',
              ],
              returned: 1,
              left: 'unset',
              said: [],
            },);
            expect(asked,).toEqual([run.dir,],);
          },
        },),

        it({
          name: 'COUNTS AN ARTIFACT SETTLED BEFORE ROUNDS WERE RECORDED and one settled under an earlier roster apart '
            + 'from readings, saying the dropped model, and returns 1 naming the earlier roster',
          fn: async (ctx,) => {
            await using run = await standingRun({
              artifacts: {
                'Mittens.json': WITHOUT_ROUNDS,
                'Tabby.json': {
                  ...carrying({ rounds: [OFF_ROSTER_ROUND,], },),
                  id: 'Tabby',
                },
              },
            },);

            expect(await standingsOf({ sinon: ctx.sinon, typed: [run.dir,], resolveRuns: unaskedRuns, },),).toEqual({
              lines: [
                ...openingOf({
                  counts: 'archives=1 artifacts=2 read=0 earlierRoster=1 earlierSchema=1 digestsJudged=0 '
                    + 'digestsUnjudged=0 unjudgedDigestRounds=0',
                },),
                '  NO ROUNDS UNDER THE CURRENT ROSTER. 1 of these artifacts names a model the roster no longer seats, '
                  + 'so it was settled under an earlier one and is not evidence about the models seated now. This is '
                  + 'an absent measurement, not a poor one.',
              ],
              returned: 1,
              left: 'unset',
              said: [
                'editor-standing-read: Tabby.lanes.repair.result.chunks[0].rounds[0].slate[0].producer.modelId names '
                  + 'not-on-roster, which no longer holds a place in the roster, so this record was written under an '
                  + 'earlier seating and cannot be read as current',
              ],
            },);
          },
        },),

        it({
          name: 'SAYS WHY EACH NAMED DIRECTORY COULD NOT BE LISTED, in the order they were named, and returns 1 over '
            + 'no artifact',
          fn: async (ctx,) => {
            await using run = await standingRun({ artifacts: {}, },);

            /**
             Two directories nobody made, named in this order.
             */
            const missing = [
              join(run.dir, 'whiskers',),
              join(run.dir, 'paws',),
            ];

            expect(await standingsOf({ sinon: ctx.sinon, typed: missing, resolveRuns: unaskedRuns, },),).toEqual({
              lines: [
                summaryOf({ archives: 2, artifacts: 0, },).trimEnd(),
                OBSERVATIONAL.trimEnd(),
                NO_ROUNDS.trimEnd(),
              ],
              returned: 1,
              left: 'unset',
              said: missing.map(function saidOf(path,): string {
                return `editor-standing-read: no artifacts under ${path} (ENOENT on artifacts/, ENOENT on the directory `
                  + 'itself)';
              },),
            },);
          },
        },),

        it({
          name: 'SAYS EACH REFUSED ARTIFACT IN THE ORDER ITS ARCHIVE WAS NAMED when the later archive\'s read ends '
            + 'first, so two runs over the same archives say the same lines',
          fn: async (ctx,) => {
            /**
             Two archives, the first holding an artifact large enough that its
             read and parse end well after the second's empty one.
             */
            await using archives = await scratchDirWith({
              prefix: 'editor-standing-order-',
              setup: async function seeded({ path, },): Promise<{ readonly slow: string; readonly quick: string; }> {
                /**
                 Archive named first, its one artifact a long invented purr.
                 */
                const slow = join(
                  path,
                  'slow',
                );

                /**
                 Archive named second, its one artifact an empty object.
                 */
                const quick = join(
                  path,
                  'quick',
                );
                await Promise.all([
                  mkdir(
                    slow,
                    { recursive: true, },
                  ),
                  mkdir(
                    quick,
                    { recursive: true, },
                  ),
                ],);
                await Promise.all([
                  writeFile(
                    join(
                      slow,
                      'Mittens.json',
                    ),
                    JSON.stringify({ purr: 'r'.repeat(32 * 1_024 * 1_024,), },),
                    'utf8',
                  ),
                  writeFile(
                    join(
                      quick,
                      'Tabby.json',
                    ),
                    '{}',
                    'utf8',
                  ),
                ],);
                return {
                  slow,
                  quick,
                };
              },
            },);

            /**
             What the procedure said over both archives, named slow first.
             */
            const captured = await standingsOf({
              sinon: ctx.sinon,
              typed: [
                archives.slow,
                archives.quick,
              ],
              resolveRuns: unaskedRuns,
            },);

            expect({
              said: captured.said,
              lines: captured.lines,
              returned: captured.returned,
            },).toEqual({
              said: [
                refusedEmpty({ path: join(archives.slow, 'Mittens.json',), },).trimEnd(),
                refusedEmpty({ path: join(archives.quick, 'Tabby.json',), },).trimEnd(),
              ],
              lines: [
                summaryOf({ archives: 2, artifacts: 2, },).trimEnd(),
                OBSERVATIONAL.trimEnd(),
                NO_ROUNDS.trimEnd(),
              ],
              returned: 1,
            },);
          },
        },),
      ],
    },),
  ],
},);

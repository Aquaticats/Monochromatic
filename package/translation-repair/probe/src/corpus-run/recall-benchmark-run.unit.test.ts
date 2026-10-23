/**
 Tests for the recall benchmark run over an invented clone, a scripted client
 and a scripted clock, in which no model is called: every critic reports no
 issue, so each seeded entry settles unchanged and every seed stays undetected.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { runRecallBenchmark, } from '../../dist/final/node/index.mjs';
import { criticClient, } from '@monochromatic-dev/translation-repair-provider/ts/critic-scripted-client.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { benchWorld, } from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/bench-world.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Commit the scripted tip reports.
 */
const TIP = 'b'.repeat(40,);

/**
 First sentence of the invented English page, long enough to be seeded.
 */
const DOZING = 'The kitten dozes on the warm windowsill every sunny afternoon.';

/**
 Second sentence of the invented English page, long enough to be seeded.
 */
const TAIL = 'Its tail hangs down to the floor beside the cushion basket.';

/**
 Original page of the one invented entry.
 */
const SOURCE = '小猫在窗台上打盹。\n';

/**
 Instants the scripted clock reads, in order.
 */
const STAMPS = [
  '2026-10-05T10:00:00.000Z',
  '2026-10-05T11:30:00.000Z',
] as const;

/**
 Scripted seams that record what the run asked of them.
 */
type Seams = {
  /**
   How many clients the run built.
   */
  clients: number;

  /**
   How many times the clock was read.
   */
  reads: number;
};

/**
 Builds the run's process-wide seams over a scripted clock and client.

 @param seen - record the seams keep what they were asked in

 @returns The tip reader, the client builder and the clock

 @example
 ```ts
 const seams = scriptedSeams({ seen, },);
 ```
 */
function scriptedSeams(
  { seen, }: { readonly seen: Seams; },
): {
  readonly readTip: () => Promise<string>;
  readonly newClient: () => ReturnType<typeof criticClient>;
  readonly now: () => string;
} {
  return {
    readTip: function readScripted(): Promise<string> {
      return Promise.resolve(TIP,);
    },
    newClient: function buildScripted(): ReturnType<typeof criticClient> {
      seen.clients += 1;
      return criticClient({
        reportFor: function noIssues(): unknown {
          return { issues: [], };
        },
      },);
    },
    now: function readClock(): string {
      /**
       Instant this read is owed.
       */
      const stamp = STAMPS[seen.reads];
      seen.reads += 1;
      if (stamp === undefined)
        throw new Error('the run read the clock more often than the case scripted',);
      return stamp;
    },
  };
}

await describe({
  name: runRecallBenchmark.name,
  concurrency: 1,
  children: [
    it({
      name: 'RUNS the repair loop over the chosen entry, keeps the scorecard under a stamped name in a runs '
        + 'directory it makes, and prints the report',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using world = await benchWorld({
          entries: {
            mittens: {
              source: SOURCE,
              english: `${DOZING} ${TAIL}\n`,
            },
          },
        },);
        const seen: Seams = {
          clients: 0,
          reads: 0,
        };
        const runsDir = join(
          world.runsDir,
          'not-yet',
        );

        await runRecallBenchmark({
          line: lineOf({
            command: 'recall-benchmark',
            typed: [],
          },),
          runsDir,
          pin: world.pin,
          ...scriptedSeams({ seen, },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /**
         File the scorecard was kept in.
         */
        const kept = join(
          runsDir,
          'recall-scorecard',
          `2026-10-05T10-00-00.000Z-${'b'.repeat(8,)}.json`,
        );
        expect(printed.lines,).toEqual([
          `START tip=${TIP} entries=1 seeds=2 perBand={"small":1,"medium":0,"large":0} budget=43200000ms`,
          'SCORECARD dispatched=1 coverage=1.000 planted=2 detected=0 detectionRate=0.000 policyDeclined=0 '
            + 'detectionRateExcludingPolicy=0.000',
          `SCORECARD kept at ${kept}`,
          'DERIVABILITY nonDerivable=0 detectionExcludingUnfair=0.000',
          'REPAIR judged=0 restored=0 partial=0 strict=0.000 lenient=0.000',
        ],);
        expect(seen,).toEqual({
          clients: 1,
          reads: 2,
        },);
        /**
         What the scorecard directory holds.
         */
        const keptNames = await readdir(join(
          runsDir,
          'recall-scorecard',
        ),);
        expect(keptNames,).toEqual(['2026-10-05T10-00-00.000Z-bbbbbbbb.json',],);

        /**
         The scorecard file, parsed.
         */
        const keptRecord: unknown = JSON.parse(await readFile(
          kept,
          'utf8',
        ),);
        expect(keptRecord,).toEqual({
          startedAt: STAMPS[0],
          finishedAt: STAMPS[1],
          tip: TIP,
          corpusSha: world.pin.commitSha,
          callConfig: {
            perCallTimeoutMs: 360_000,
            streamFirstContentMs: 180_000,
            streamIdleMs: 120_000,
            claudeCallTimeoutMs: 460_000,
          },
          entriesPerBand: 3,
          seedsPerEntry: 3,
          scorecard: {
            dispatchedEntries: 1,
            coverage: 1,
            plantedSeeds: 2,
            detectedSeeds: 0,
            seedDetectionRate: 0,
            policyDeclinedSeeds: 0,
            seedDetectionRateExcludingPolicy: 0,
            nonDerivableSeeds: 0,
            seedDetectionRateExcludingUnfair: 0,
            judgedSeeds: 0,
            restoredSeeds: 0,
            partialSeeds: 0,
            seededRepairRate: 0,
            seededRepairRateLenient: 0,
            lexicalUniverse: 2,
            lexicalRestoredSeeds: 0,
            lexicalRepairRate: 0,
            statusCounts: { unchanged: 1, },
          },
          records: [
            {
              entryId: 'mittens',
              outcomeKind: 'ok',
              status: 'unchanged',
              seedJudgments: {
                'seed/omission-0': {
                  verdict: 'absent',
                  judged: false,
                  votes: 0,
                },
                'seed/omission-1': {
                  verdict: 'absent',
                  judged: false,
                  votes: 0,
                },
              },
              seedDerivability: {
                'seed/omission-0': {
                  verdict: 'derivable',
                  judged: false,
                  votes: 0,
                },
                'seed/omission-1': {
                  verdict: 'derivable',
                  judged: false,
                  votes: 0,
                },
              },
              seedGrades: {
                'seed/omission-0': {
                  measurable: true,
                  disappearedWords: 7,
                  returnedWords: 0,
                  restored: false,
                },
                'seed/omission-1': {
                  measurable: true,
                  disappearedWords: 7,
                  returnedWords: 0,
                  restored: false,
                },
              },
              seedDetection: {
                'seed/omission-0': 'undetected',
                'seed/omission-1': 'undetected',
              },
              issueCount: 0,
              resolvedIssueCount: 0,
              detail: '',
              repairedText: ' \n',
            },
          ],
        },);
      },
    },),

    it({
      name: 'PLANS only: builds the client, prints the start and the plan, and keeps no scorecard',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using world = await benchWorld({
          entries: {
            mittens: {
              source: SOURCE,
              english: `${DOZING} ${TAIL}\n`,
            },
          },
        },);
        const seen: Seams = {
          clients: 0,
          reads: 0,
        };

        await runRecallBenchmark({
          line: lineOf({
            command: 'recall-benchmark',
            typed: ['--plan',],
          },),
          runsDir: world.runsDir,
          pin: world.pin,
          ...scriptedSeams({ seen, },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(printed.lines,).toEqual([
          `START tip=${TIP} entries=1 seeds=2 perBand={"small":1,"medium":0,"large":0} budget=43200000ms`,
          `PLAN ok tip=${TIP} client=constructed entries=mittens`,
        ],);
        expect(seen,).toEqual({
          clients: 1,
          reads: 1,
        },);
        expect(await readdir(world.runsDir,),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES --plan over a corpus that offers no entry to seed, before building a client or making its '
        + 'runs directory',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using world = await benchWorld({ entries: {}, },);
        const seen: Seams = {
          clients: 0,
          reads: 0,
        };

        const refusal = await rejectionOf(async function planOverNobody(): Promise<void> {
          await runRecallBenchmark({
            line: lineOf({
              command: 'recall-benchmark',
              typed: ['--plan',],
            },),
            runsDir: join(
              world.runsDir,
              'recall-runs',
            ),
            pin: world.pin,
            ...scriptedSeams({ seen, },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: no entry at the pin can be seeded, so the benchmark would plant no seed and measure '
            + 'nothing; check that the corpus clone and commit it reads hold entries with both pages and a sentence '
            + 'worth deleting',
        );
        expect(printed.lines,).toEqual([
          `START tip=${TIP} entries=0 seeds=0 perBand={"small":0,"medium":0,"large":0} budget=43200000ms`,
        ],);
        expect(seen,).toEqual({
          clients: 0,
          reads: 1,
        },);
        // The runs directory the run was handed is still absent from its parent.
        expect(await readdir(world.runsDir,),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A RUN over a corpus that offers no entry to seed where --plan refuses, before building a '
        + 'client or making its runs directory, so no scorecard is kept',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using world = await benchWorld({ entries: {}, },);
        const seen: Seams = {
          clients: 0,
          reads: 0,
        };

        const refusal = await rejectionOf(async function runOverNobody(): Promise<void> {
          await runRecallBenchmark({
            line: lineOf({
              command: 'recall-benchmark',
              typed: [],
            },),
            runsDir: join(
              world.runsDir,
              'recall-runs',
            ),
            pin: world.pin,
            ...scriptedSeams({ seen, },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: no entry at the pin can be seeded, so the benchmark would plant no seed and measure '
            + 'nothing; check that the corpus clone and commit it reads hold entries with both pages and a sentence '
            + 'worth deleting',
        );
        expect(printed.lines,).toEqual([
          `START tip=${TIP} entries=0 seeds=0 perBand={"small":0,"medium":0,"large":0} budget=43200000ms`,
        ],);
        expect(seen,).toEqual({
          clients: 0,
          reads: 1,
        },);
        // The runs directory the run was handed is still absent from its parent.
        expect(await readdir(world.runsDir,),).toEqual([],);
      },
    },),
  ],
},);

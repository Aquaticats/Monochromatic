/**
 Tests for where a recall benchmark's scorecard is kept.

 WHAT THESE PIN is the loss the store exists to stop: a rerun used to
 overwrite the previous scorecard under one fixed name, so two runs bought to
 be compared left one file. Fixtures are cat-themed invention.

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
} from '@monochromatic-dev/test-harness-core/ts';

import {
  persistRecallScorecard,
  RECALL_SCORECARD_DIR,
  type RecallScorecardRecord,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 A scorecard as the benchmark would hand it over.
 */
const BASE_RECORD: RecallScorecardRecord = {
  startedAt: '2026-08-26T09:00:00.000Z',
  finishedAt: '2026-08-26T21:00:00.000Z',
  tip: 'cafef00dcafef00dcafef00dcafef00dcafef00d',
  corpusSha: 'feedfacefeedfacefeedfacefeedfacefeedface',
  callConfig: { perCallTimeoutMs: 600_000, },
  entriesPerBand: 3,
  seedsPerEntry: 3,
  scorecard: {
    dispatchedEntries: 9,
    plantedSeeds: 27,
    detectedSeeds: 20,
  },
  records: [{ entryId: 'tabby', restored: true, },],
};

/**
 Files the store left under the scorecard directory.

 @param runsDir - runs directory

 @returns Names in directory order

 @example
 ```ts
 const kept = await keptFiles({ runsDir, },);
 ```
 */
async function keptFiles({ runsDir, }: { readonly runsDir: string; },): Promise<readonly string[]> {
  return await readdir(join(
    runsDir,
    RECALL_SCORECARD_DIR,
  ),);
}

await describe({
  name: persistRecallScorecard.name,
  children: [
    it({
      name:
        'writes the scorecard under the scorecard directory, named by the start stamp and the tip, and '
          + 'returns the path, with no colon in the name since an instant rendered verbatim is not a file name '
          + 'everywhere',
      fn: async () => {
        /**
         Fresh runs directory.
         */
        await using scratch = await scratchDir({ prefix: 'recall-scorecard-store-', },);
        const runsDir = scratch.path;

        /**
         Where the store put it.
         */
        const path = await persistRecallScorecard({
          runsDir,
          record: BASE_RECORD,
        },);
        expect(path,).toBe(join(
          runsDir,
          RECALL_SCORECARD_DIR,
          '2026-08-26T09-00-00.000Z-cafef00d.json',
        ),);
        expect(path.includes(':',),).toBe(false,);

        /**
         What the file says.
         */
        const read = JSON.parse(await readFile(
          path,
          'utf8',
        ),) as RecallScorecardRecord;
        expect(read.scorecard,).toEqual(BASE_RECORD.scorecard,);
        expect(read.records,).toEqual(BASE_RECORD.records,);
      },
    },),
    it({
      name:
        'KEEPS BOTH RUNS when the benchmark is run twice, which is the loss this module exists to stop: '
          + 'a twelve-hour run used to replace the previous one under a fixed name with no trace',
      fn: async () => {
        /**
         Fresh runs directory.
         */
        await using scratch = await scratchDir({ prefix: 'recall-scorecard-store-', },);
        const runsDir = scratch.path;

        /**
         First run's path.
         */
        const first = await persistRecallScorecard({
          runsDir,
          record: {
            ...BASE_RECORD,
            records: [{ entryId: 'tabby', restored: true, },],
          },
        },);

        /**
         Second run's path, started later from another build.
         */
        const second = await persistRecallScorecard({
          runsDir,
          record: {
            ...BASE_RECORD,
            startedAt: '2026-08-27T09:00:00.000Z',
            tip: 'beefbeefbeefbeefbeefbeefbeefbeefbeefbeef',
            records: [{ entryId: 'calico', restored: false, },],
          },
        },);
        expect(first,).not
          .toBe(second,);

        /**
         Files left behind: exactly the two runs, no temporary leftovers.
         */
        const kept = await keptFiles({ runsDir, },);
        expect(kept.toSorted(),).toEqual([
          '2026-08-26T09-00-00.000Z-cafef00d.json',
          '2026-08-27T09-00-00.000Z-beefbeef.json',
        ],);

        // Each still says what it said. A surviving file holding the other
        // run's rows would pass a count and fail a reader.
        expect(await readFile(
          first,
          'utf8',
        ),).toContain('tabby',);
        expect(await readFile(
          second,
          'utf8',
        ),).toContain('calico',);
      },
    },),
    it({
      name:
        'SEPARATES two runs of different builds started in the same instant, since the question a rerun '
          + 'answers is whether a change moved the rate, and collapsing before and after into one file answers nothing',
      fn: async () => {
        /**
         Fresh runs directory.
         */
        await using scratch = await scratchDir({ prefix: 'recall-scorecard-store-', },);
        const runsDir = scratch.path;
        await persistRecallScorecard({
          runsDir,
          record: BASE_RECORD,
        },);
        await persistRecallScorecard({
          runsDir,
          record: {
            ...BASE_RECORD,
            tip: 'beefbeefbeefbeefbeefbeefbeefbeefbeefbeef',
          },
        },);
        expect((await keptFiles({ runsDir, },)).length,).toBe(2,);
      },
    },),
  ],
},);

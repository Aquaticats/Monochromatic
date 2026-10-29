/**
 Tests for what the editor standing read prints for one pipeline digest: its
 entry count, each seat's judged rounds beside the rounds that drew no ballot,
 and the standing of every model a ballot was cast over.

 THE READINGS ARE READ, NOT BUILT. Each case writes settled artifacts and reads
 them through `readStandingArtifact`, so every round is one the reader accepts.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  everyStandingRound,
  printDigestStanding,
  readStandingArtifact,
  type StandingReading,
} from '../../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  carrying,
  JUDGED_SEATED_ROUND,
  JUDGED_SELECTION,
  JUDGED_STANDING,
  SEATED_ROUND,
  SEATED_SELECTION,
  standingRun,
} from './editor-standing-read.test-fixture.ts';

//region Fixtures

/**
 Built pipeline every fixture artifact records.
 */
const DIGEST = `sha256-tree-v1:${'0'.repeat(64,)}`;

/**
 The refiner's round a ballot was cast on.
 */
const JUDGED_REFINE_ROUND = {
  ...JUDGED_SEATED_ROUND,
  stage: 'refine',
  envelopeId: 'env/9',
};

/**
 Reads one artifact per chunk list given, each written as its own entry.

 @param chunks - per entry, the rounds its one chunk records

 @returns Readings, in the order given

 @throws Error where an artifact does not read, naming its outcome

 @example
 ```ts
 const readings = await readingsOf({ chunks: [[SEATED_ROUND,],], },);
 ```
 */
async function readingsOf(
  { chunks, }: { readonly chunks: readonly (readonly unknown[])[]; },
): Promise<readonly StandingReading[]> {
  /**
   Entry ids the artifacts are written under, in order.
   */
  const ids = chunks.map(function idAt(_rounds, index,): string {
    return `Whiskers${String(index,)}`;
  },);
  await using run = await standingRun({
    artifacts: Object.fromEntries(chunks.map(function artifactAt(rounds, index,): readonly [string, unknown,] {
      return [
        `${ids[index] ?? ''}.json`,
        {
          ...carrying({ rounds, },),
          id: ids[index],
        },
      ];
    },),),
  },);
  return await Promise.all(ids.map(async function readOne(id,): Promise<StandingReading> {
    /**
     What the reader made of this entry's artifact.
     */
    const outcome = await readStandingArtifact({ path: join(run.dir, 'artifacts', `${id}.json`,), },);
    if (outcome.kind !== 'read')
      throw new Error(`fixture artifact ${id} read as ${outcome.kind}, ${outcome.said.join(' ',)}`,);
    return outcome.reading;
  },),);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: printDigestStanding.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS ONE ENTRY\'S DIGEST in the singular, its editor\'s one judged round from one chunk with the '
            + 'standing it reads, and a refiner with no round',
          fn: async (ctx,) => {
            /**
             One entry whose editor round drew a ballot.
             */
            const readings = await readingsOf({ chunks: [[JUDGED_SEATED_ROUND,],], },);
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printDigestStanding({ group: { digest: DIGEST, readings, }, },);

            expect(printed.lines,).toEqual([
              `\n${DIGEST} over 1 entry`,
              '  EDITOR : 1 judged round from 1 of 1 chunk',
              JUDGED_STANDING.trimEnd(),
              '  REFINER: 0 judged rounds from 0 of 1 chunk',
            ],);
          },
        },),

        it({
          name: 'PRINTS SEVERAL ENTRIES\' DIGEST in the plural, counting at each seat the judged rounds, the chunks '
            + 'they came from and, in the singular and the plural, the rounds that drew no ballot',
          fn: async (ctx,) => {
            /**
             Three entries: an editor round and a refiner round each judged,
             one editor round unjudged, and two more editor rounds unjudged.
             */
            const readings = await readingsOf({
              chunks: [
                [JUDGED_SEATED_ROUND, JUDGED_REFINE_ROUND,],
                [
                  SEATED_ROUND,
                  {
                    ...JUDGED_REFINE_ROUND,
                    envelopeId: 'env/10',
                  },
                  {
                    ...JUDGED_REFINE_ROUND,
                    ballots: [],
                    envelopeId: 'env/11',
                  },
                ],
                [{ ...SEATED_ROUND, envelopeId: 'env/12', },],
              ],
            },);
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printDigestStanding({ group: { digest: DIGEST, readings, }, },);

            expect(printed.lines,).toEqual([
              `\n${DIGEST} over 3 entries`,
              '  EDITOR : 1 judged round from 1 of 3 chunks; 2 rounds drew no ballot',
              JUDGED_STANDING.trimEnd(),
              '  REFINER: 2 judged rounds from 2 of 3 chunks; 1 round drew no ballot',
              `      ${SEAT_SYNTHETIC_VISION_WITHHELD}: 100.0% (2 of 2 disinterested ballots, over 2 candidates)`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: everyStandingRound.name,
      concurrency: 1,
      children: [
        it({
          name: 'LISTS EVERY ROUND ONE ARTIFACT RECORDED, its editor rounds before its refiner rounds, judged or not',
          fn: async () => {
            /**
             One entry whose chunk recorded a refiner round before an editor
             round.
             */
            const readings = await readingsOf({ chunks: [[JUDGED_REFINE_ROUND, SEATED_ROUND,],], },);

            expect(everyStandingRound({ reading: nonNullishOrThrow(readings[0],), },),)
              .toEqual([SEATED_SELECTION, JUDGED_SELECTION,],);
          },
        },),

        it({
          name: 'LISTS NO ROUND for an artifact whose one chunk recorded none',
          fn: async () => {
            /**
             One entry whose chunk recorded no round.
             */
            const readings = await readingsOf({ chunks: [[],], },);

            expect(everyStandingRound({ reading: nonNullishOrThrow(readings[0],), },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

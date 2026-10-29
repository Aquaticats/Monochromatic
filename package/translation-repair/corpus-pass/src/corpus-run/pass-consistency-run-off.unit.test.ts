/**
 Tests for asking the bench once more among the finalists of a split first round, or
 reading its earlier answer: the choices handed back in roster order beside the
 seat that gave each, the key of the run-off stage under which they are stored,
 what a resume reads back, and which stored values are a run-off's record.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildConsistencyRunOffMessages,
  type ConsistencyRendering,
  type ConsistencySubject,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  askRunOff,
  choiceRecordOf,
  type ConsistencyCacheValue,
  consistencyKey,
  storedChoices,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  scriptedJudgeClient,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { recordingCache, } from './consistency-cache.test-fixture.ts';

/**
 The bench asked, in roster order.
 */
const BENCH: readonly RosterModelId[] = [SEAT_HYPER_ONLY, SEAT_BEDROCK_ONLY_TEXT, SEAT_HYPER_VISION,];

/**
 The repeated sentence in three places, the first two of which are the finalists of a split first round.
 */
const SUBJECT: ConsistencySubject = {
  repeat: { kind: 'maximal', wording: '小猫打盹', },
  occurrences: [
    { sliceIndex: 0, paragraph: '小猫打盹。', markStart: 0, markEnd: 4, english: { kind: 'present', text: 'The cat naps.', }, },
    { sliceIndex: 1, paragraph: '小猫打盹。', markStart: 0, markEnd: 4, english: { kind: 'present', text: 'The kitten naps.', }, },
    { sliceIndex: 2, paragraph: '小猫打盹。', markStart: 0, markEnd: 4, english: { kind: 'present', text: 'The tabby naps.', }, },
  ],
};

/**
 Where each place's rendering was located.
 */
const RENDERINGS: readonly ConsistencyRendering[] = [
  { kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 0, },
  { kind: 'located', quote: 'The kitten naps', core: 'The kitten naps', start: 0, },
  { kind: 'located', quote: 'The tabby naps', core: 'The tabby naps', start: 0, },
];

/**
 The places that tied.
 */
const FINALISTS = ['OCCURRENCE 1', 'OCCURRENCE 2',];

/**
 The question the bench is put, as the cache names it.
 */
const KEY = consistencyKey({
  stage: 'run-off',
  messages: buildConsistencyRunOffMessages({ subject: SUBJECT, finalists: FINALISTS, renderings: RENDERINGS, },),
  modelIds: BENCH,
},);

/**
 What a seat does when every seat chooses the second finalist.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = everySeatChooses;
 ```
 */
function everySeatChooses({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  return { kind: 'answers', text: JSON.stringify({ assessment: `${seat} prefers the kitten.`, choice: 'OCCURRENCE 2', },), };
}

/**
 Puts the run-off to the bench.

 @param behave - what each seat does

 @param cache - store

 @param asked - sink for the seat of every call

 @param sheets - sink for the sheet of every call

 @returns What asking came to

 @example
 ```ts
 const result = await runOff({ behave: everySeatChooses, cache, asked: [], sheets: [], },);
 ```
 */
async function runOff(
  {
    behave,
    cache,
    asked,
    sheets,
  }: {
    readonly behave: SeatScript;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly asked: RosterModelId[];
    readonly sheets: string[];
  },
): ReturnType<typeof askRunOff> {
  return await askRunOff({
    client: scriptedJudgeClient({ behave, asked, sheets, },),
    modelIds: BENCH,
    subject: SUBJECT,
    finalists: FINALISTS,
    renderings: RENDERINGS,
    cache,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l: levelCapturingLogger({ lines: [], },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: askRunOff.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EVERY SEAT WITH A SHEET OF THE FINALISTS ONLY AND HANDS BACK THE CHOICES IN ROSTER ORDER, each beside its '
            + 'seat, storing the record under the run-off key',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const asked: RosterModelId[] = [];
            const sheets: string[] = [];
            const result = await runOff({
              behave: everySeatChooses,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              asked,
              sheets,
            },);
            const record = {
              kind: 'choice',
              quorum: 2,
              voices: BENCH.map(function voiceFor(seat,) {
                return { seat, reason: `${seat} prefers the kitten.`, choice: 'OCCURRENCE 2', };
              },),
              findings: [],
            };
            expect({
              result,
              asked: asked.toSorted(),
              keys: persisted.map(function keyOf({ key, },): string {
                return key;
              },),
              showsThird: sheets.some(function namesTheThird(sheet,): boolean {
                return sheet.includes('The tabby naps',);
              },),
            },).toEqual({
              result: { ...record, resumed: false, },
              asked: [...BENCH,].toSorted(),
              keys: [KEY,],
              showsThird: false,
            },);
            expect(JSON.parse(persisted[0]?.serialized ?? 'null',),).toEqual(record,);
          },
        },),
        it({
          name: 'COUNTS NO CHOICE OF A PLACE THAT DID NOT TIE, so a seat that names the third place is a lost voice',
          fn: async () => {
            const result = await runOff({
              behave: function oneStray({ seat, },): SeatBehaviour {
                return (seat === SEAT_HYPER_ONLY)
                  ? { kind: 'answers', text: JSON.stringify({ assessment: 'The third place.', choice: 'OCCURRENCE 3', },), }
                  : everySeatChooses({ seat, },);
              },
              cache: recordingCache({ resumed: new Map(), persisted: [], },),
              asked: [],
              sheets: [],
            },);
            expect(result.voices.map(function seatOf({ seat, },): string {
              return seat;
            },),).not.toContain(SEAT_HYPER_ONLY,);
          },
        },),
        it({
          name: 'READS BACK A STORED RUN-OFF INSTEAD OF ASKING, so a resumed page runs off the same tie the same way',
          fn: async () => {
            const stored: ConsistencyCacheValue = {
              kind: 'choice',
              quorum: 2,
              voices: [
                { seat: SEAT_HYPER_ONLY, reason: 'The kitten.', choice: 'OCCURRENCE 2', },
                { seat: SEAT_BEDROCK_ONLY_TEXT, reason: 'The cat.', choice: 'OCCURRENCE 1', },
              ],
              findings: [],
            };
            const asked: RosterModelId[] = [];
            const result = await runOff({
              behave: everySeatChooses,
              cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
              asked,
              sheets: [],
            },);
            expect({ result, asked, },).toEqual({ result: { ...stored, resumed: true, }, asked: [], },);
          },
        },),
      ],
    },),
    describe({
      name: storedChoices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A CHOICE RECORD AS A LIST OF ONE AND EVERY OTHER KIND OF STORED VALUE AS NONE',
          fn: async () => {
            const choice: ConsistencyCacheValue = { kind: 'choice', quorum: 1, voices: [], findings: [], };
            expect([
              storedChoices(choice,),
              storedChoices({ quorum: 1, voices: [], findings: [], },),
              storedChoices({ kind: 'writers', quorum: 1, voices: [], findings: [], },),
            ],).toEqual([[choice,], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: choiceRecordOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUILDS THE RECORD OF A GATHER OF CHOICES: each model as the seat, the reason and the choice it gave, and the '
            + 'quorum and findings as the gather reported them',
          fn: async () => {
            expect(choiceRecordOf({
              quorum: 2,
              voices: [
                { modelId: SEAT_HYPER_ONLY, value: { assessment: 'The kitten.', choice: 'OCCURRENCE 2', }, },
                { modelId: SEAT_BEDROCK_ONLY_TEXT, value: { assessment: 'The cat.', choice: 'OCCURRENCE 1', }, },
              ],
              findings: ['stage-voice-lost (page-consistency-run-off hyper-vision)',],
            },),).toEqual({
              kind: 'choice',
              quorum: 2,
              voices: [
                { seat: SEAT_HYPER_ONLY, reason: 'The kitten.', choice: 'OCCURRENCE 2', },
                { seat: SEAT_BEDROCK_ONLY_TEXT, reason: 'The cat.', choice: 'OCCURRENCE 1', },
              ],
              findings: ['stage-voice-lost (page-consistency-run-off hyper-vision)',],
            },);
          },
        },),
      ],
    },),
  ],
},);

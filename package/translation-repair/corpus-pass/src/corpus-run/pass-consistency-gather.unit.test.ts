/**
 Tests for putting one question to a bench through the cache, whatever the kind
 of answer: the record handed back in roster order beside the seat that gave
 each voice, what is stored for a resume and what is not, what a resume reads
 back and which stored records are refused, and what a store that refuses costs.
 The kind of answer here is a choice among two places, the shape a run-off hears.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StageRosterRepeatError, } from '@monochromatic-dev/translation-repair-stage';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  consistencyChoiceResponseFormat,
  usableChoiceFor,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  choiceRecordOf,
  type ConsistencyCacheValue,
  type ConsistencyChoiceRecord,
  consistencyKey,
  gatherThroughCache,
  recordFits,
  storedChoices,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
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
 The sheet every seat reads.
 */
const MESSAGES = [{ role: 'user', content: 'Which of the two places words the nap best?', },] as const;

/**
 The question put, as the cache names it.
 */
const KEY = consistencyKey({ stage: 'run-off', messages: MESSAGES, modelIds: BENCH, },);

/**
 The labels a seat may choose between.
 */
const FINALISTS = ['OCCURRENCE 1', 'OCCURRENCE 2',];

/**
 What a seat sends, naming itself in its reason so a voice is told from the others'.

 @param seat - seat answering

 @returns Seat's reply as text

 @example
 ```ts
 const text = replyOf({ seat: SEAT_HYPER_ONLY, },);
 ```
 */
function replyOf({ seat, }: { readonly seat: RosterModelId; },): string {
  return JSON.stringify({ assessment: `${seat} words the first place best.`, choice: 'OCCURRENCE 1', },);
}

/**
 The voice a seat's reply comes to, as the record keeps it.

 @param seat - seat that answered

 @returns Voice

 @example
 ```ts
 const voice = voiceOf({ seat: SEAT_HYPER_ONLY, },);
 ```
 */
function voiceOf({ seat, }: { readonly seat: string; },): ConsistencyChoiceRecord['voices'][number] {
  return { seat, reason: `${seat} words the first place best.`, choice: 'OCCURRENCE 1', };
}

/**
 What a seat does when every seat answers with a usable choice.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = everySeatAnswers;
 ```
 */
function everySeatAnswers({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  return { kind: 'answers', text: replyOf({ seat, },), };
}

/**
 What a seat does when one seat answers and the others choose a place that is not on the sheet.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = oneSeatAnswers;
 ```
 */
function oneSeatAnswers({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  return (seat === SEAT_HYPER_ONLY)
    ? { kind: 'answers', text: replyOf({ seat, },), }
    : { kind: 'answers', text: JSON.stringify({ assessment: 'A place that is not on the sheet.', choice: 'OCCURRENCE 9', },), };
}

/**
 Puts the question to the bench.

 @param behave - what each seat does

 @param cache - store

 @param modelIds - bench, roster by default

 @param lines - sink for the level and text of every line logged

 @param asked - sink for the seat of every call

 @returns What asking came to

 @example
 ```ts
 const result = await gather({ behave: everySeatAnswers, cache, lines: [], asked: [], },);
 ```
 */
async function gather(
  {
    behave,
    cache,
    modelIds = BENCH,
    lines,
    asked,
  }: {
    readonly behave: SeatScript;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly modelIds?: readonly RosterModelId[];
    readonly lines: string[];
    readonly asked: RosterModelId[];
  },
): ReturnType<typeof gatherThroughCache<unknown, ConsistencyChoiceRecord>> {
  return await gatherThroughCache({
    client: scriptedJudgeClient({ behave, asked, sheets: [], },),
    modelIds,
    messages: MESSAGES,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: consistencyChoiceResponseFormat({ labels: FINALISTS, },),
    validate: usableChoiceFor({ labels: FINALISTS, },),
    stage: 'page-test-gather',
    graceMs: HANG_STOP_MS,
    cache,
    key: KEY,
    readStored: storedChoices,
    recordOf: choiceRecordOf,
    l: levelCapturingLogger({ lines, },),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: gatherThroughCache.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EVERY SEAT AND HANDS BACK THE VOICES IN ROSTER ORDER, each beside the seat that gave it, and stores the '
            + 'record whole under the question\'s key',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const asked: RosterModelId[] = [];
            const { record, resumed, } = await gather({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              lines: [],
              asked,
            },);
            const expected = {
              kind: 'choice',
              quorum: 2,
              voices: BENCH.map(function voiceFor(seat,) {
                return voiceOf({ seat, },);
              },),
              findings: [],
            };
            expect({ record, resumed, asked: asked.toSorted(), },).toEqual({
              record: expected,
              resumed: false,
              asked: [...BENCH,].toSorted(),
            },);
            expect(persisted.map(function keyOf(entry,): string {
              return entry.key;
            },),).toEqual([KEY,],);
            expect(JSON.parse(persisted[0]?.serialized ?? 'null',),).toEqual(expected,);
          },
        },),
        it({
          name: 'STORES NOTHING FOR A GATHER THAT MISSED ITS QUORUM, handing back the voice it did hear, so a resume asks again',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const { record, resumed, } = await gather({
              behave: oneSeatAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              lines: [],
              asked: [],
            },);
            expect({ quorum: record.quorum, voices: record.voices, resumed, persisted, },).toEqual({
              quorum: 2,
              voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },),],
              resumed: false,
              persisted: [],
            },);
          },
        },),
        it({
          name: 'READS BACK A STORED RECORD INSTEAD OF ASKING, marking it resumed and saying how many voices it read',
          fn: async () => {
            const stored: ConsistencyChoiceRecord = {
              kind: 'choice',
              quorum: 2,
              voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), voiceOf({ seat: SEAT_BEDROCK_ONLY_TEXT, },),],
              findings: ['stage-voice-lost (page-test-gather minimax-m3)',],
            };
            const lines: string[] = [];
            const asked: RosterModelId[] = [];
            const read = await gather({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
              lines,
              asked,
            },);
            expect({ read, asked, lines, },).toEqual({
              read: { record: stored, resumed: true, },
              asked: [],
              lines: ['info [gatherThroughCache] page-test-gather: resumed 2 voices',],
            },);
          },
        },),
        it({
          name: 'SAYS ONE VOICE, NOT ONE VOICES, WHEN IT READS BACK A GATHER OF ONE VOICE',
          fn: async () => {
            const lines: string[] = [];
            await gather({
              behave: everySeatAnswers,
              cache: recordingCache({
                resumed: new Map([[KEY, { kind: 'choice', quorum: 1, voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },),], findings: [], },],],),
                persisted: [],
              },),
              lines,
              asked: [],
            },);
            expect(lines,).toEqual(['info [gatherThroughCache] page-test-gather: resumed 1 voice',],);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE STORED VALUE IS ANOTHER KIND OF ANSWER, HOLDS A SEAT TWICE OR HOLDS A SEAT THE BENCH DOES '
            + 'NOT HAVE, warning that it cannot be this bench\'s answer',
          fn: async () => {
            /**
             Values the cache may hold under the key that are no answer of this bench.
             */
            const unfit: readonly ConsistencyCacheValue[] = [
              { quorum: 1, voices: [], findings: [], },
              {
                kind: 'choice',
                quorum: 2,
                voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), voiceOf({ seat: SEAT_HYPER_ONLY, },),],
                findings: [],
              },
              {
                kind: 'choice',
                quorum: 2,
                voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), voiceOf({ seat: 'unseated-cat', },),],
                findings: [],
              },
            ];
            const warning = 'warn [gatherThroughCache] page-test-gather: the stored answer cannot be an answer of this bench '
              + '(another kind, a seat heard twice, or a seat the bench does not have), so the question is asked again';
            const results = await Promise.all(unfit.map(async function askedWith(stored,): Promise<unknown> {
              const lines: string[] = [];
              const asked: RosterModelId[] = [];
              const { resumed, } = await gather({
                behave: everySeatAnswers,
                cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
                lines,
                asked,
              },);
              return {
                resumed,
                asked: asked.toSorted(),
                warnings: lines.filter(function namesTheStoredAnswer(line,): boolean {
                  return line.includes('the stored answer',);
                },),
              };
            },),);
            expect(results,).toEqual(unfit.map(function everySeatAskedAgain(): unknown {
              return { resumed: false, asked: [...BENCH,].toSorted(), warnings: [warning,], };
            },),);
          },
        },),
        it({
          name: 'COUNTS THE VOICES IN HAND WHEN THE STORE REFUSES THEM, warning that they are counted all the same',
          fn: async () => {
            const lines: string[] = [];
            const { record, resumed, } = await gather({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map(), persisted: [], refusal: new Error('the cat sat on the disk',), },),
              lines,
              asked: [],
            },);
            expect({
              voices: record.voices.length,
              resumed,
              warned: lines.filter(function namesTheStore(line,): boolean {
                return line.includes('could not be stored',);
              },),
            },).toEqual({
              voices: BENCH.length,
              resumed: false,
              warned: [
                'warn [gatherThroughCache] page-test-gather: the answer could not be stored (refused by Error); it is counted all the same',
              ],
            },);
          },
        },),
        it({
          name: 'REFUSES A BENCH THAT SEATS ONE MODEL TWICE before any seat is asked',
          fn: async () => {
            const asked: RosterModelId[] = [];
            const refusal = await rejectionOf(async function asksTwiceSeated(): Promise<unknown> {
              return await gather({
                behave: everySeatAnswers,
                cache: recordingCache({ resumed: new Map(), persisted: [], },),
                modelIds: [SEAT_HYPER_ONLY, SEAT_HYPER_ONLY,],
                lines: [],
                asked,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StageRosterRepeatError,);
            expect(asked,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: recordFits.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FITS VOICES FROM THE BENCH\'S OWN SEATS, EACH ONCE, and refuses a seat twice or a seat the bench lacks',
          fn: async () => {
            expect([
              recordFits({ record: { quorum: 2, voices: BENCH.map(function voiceFor(seat,) {
                return voiceOf({ seat, },);
              },), findings: [], }, modelIds: BENCH, },),
              recordFits({ record: { quorum: 1, voices: [], findings: [], }, modelIds: BENCH, },),
              recordFits({
                record: { quorum: 2, voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), voiceOf({ seat: SEAT_HYPER_ONLY, },),], findings: [], },
                modelIds: BENCH,
              },),
              recordFits({
                record: { quorum: 1, voices: [voiceOf({ seat: 'unseated-cat', },),], findings: [], },
                modelIds: BENCH,
              },),
            ],).toEqual([true, true, false, false,],);
          },
        },),
      ],
    },),
  ],
},);

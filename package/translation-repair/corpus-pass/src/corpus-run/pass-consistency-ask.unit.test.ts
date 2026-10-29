/**
 Tests for asking the bench once about one repeated wording, or reading its
 earlier answer: the ballots handed back in roster order beside the seat that
 gave each, the quorum and findings of the gather, what is stored for a resume
 and what is not, what a resume reads back, and what a store that refuses
 costs.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createSeatTally,
  gatherStageVoices,
  StageRosterRepeatError,
} from '@monochromatic-dev/translation-repair-stage';
import {
  buildConsistencyMessages,
  consistencyLabels,
  consistencyResponseFormat,
  type ConsistencySubject,
  usableConsistencyBallotFor,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  askConsistencyBench,
  type ConsistencyCacheValue,
  consistencyKey,
  type ConsistencyRecord,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { recordingCache, } from './consistency-cache.test-fixture.ts';
import {
  scriptedJudgeClient,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';

/**
 The bench asked, in roster order.
 */
const BENCH: readonly RosterModelId[] = [SEAT_HYPER_ONLY, SEAT_BEDROCK_ONLY_TEXT, SEAT_HYPER_VISION,];

/**
 The repeated sentence in two places, each with its English.
 */
const SUBJECT: ConsistencySubject = {
  repeat: { kind: 'maximal', wording: '小猫打盹', },
  occurrences: [
    {
      sliceIndex: 0,
      paragraph: '小猫打盹。',
      markStart: 0,
      markEnd: 4,
      english: { kind: 'present', text: 'The cat naps.', },
    },
    {
      sliceIndex: 1,
      paragraph: '小猫打盹。',
      markStart: 0,
      markEnd: 4,
      english: { kind: 'present', text: 'The kitten naps.', },
    },
  ],
};

/**
 The question the bench is put, as the cache names it.
 */
const KEY = consistencyKey({
  stage: 'ballots',
  messages: buildConsistencyMessages({ subject: SUBJECT, },),
  modelIds: BENCH,
},);

/**
 What a seat sends, naming itself in its reason so a ballot is told from the others'.

 @param seat - seat answering

 @returns Seat's reply as text

 @example
 ```ts
 const text = replyOf({ seat: SEAT_HYPER_ONLY, },);
 ```
 */
function replyOf({ seat, }: { readonly seat: RosterModelId; },): string {
  return JSON.stringify({
    assessment: `${seat} words the first place best.`,
    candidateQuotes: [
      { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
      { label: 'OCCURRENCE 2', quote: 'The kitten naps.', },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
  },);
}

/**
 The ballot a seat's reply comes to, as the record keeps it.

 @param seat - seat that answered

 @returns Seat and its ballot

 @example
 ```ts
 const voice = voiceOf({ seat: SEAT_HYPER_ONLY, },);
 ```
 */
function voiceOf({ seat, }: { readonly seat: RosterModelId; },): ConsistencyRecord['voices'][number] {
  return {
    seat,
    reason: `${seat} words the first place best.`,
    quotes: [
      { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
      { label: 'OCCURRENCE 2', quote: 'The kitten naps.', },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
  };
}

/**
 What a seat does when every seat answers with a usable ballot.

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
 Asks the bench about the repeated sentence.

 @param behave - what each seat does

 @param cache - store

 @param modelIds - bench, roster by default

 @param subject - question put, repeated sentence by default

 @param lines - sink for the level and text of every line logged

 @param asked - sink for the seat of every call

 @returns What asking came to

 @example
 ```ts
 const result = await askBench({ behave: everySeatAnswers, cache, lines: [], asked: [], },);
 ```
 */
async function askBench(
  {
    behave,
    cache,
    modelIds = BENCH,
    subject = SUBJECT,
    lines,
    asked,
  }: {
    readonly behave: SeatScript;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly modelIds?: readonly RosterModelId[];
    readonly subject?: ConsistencySubject;
    readonly lines: string[];
    readonly asked: RosterModelId[];
  },
): ReturnType<typeof askConsistencyBench> {
  return await askConsistencyBench({
    client: scriptedJudgeClient({ behave, asked, sheets: [], },),
    modelIds,
    subject,
    cache,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l: levelCapturingLogger({ lines, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 What a seat does when one seat answers with a ballot and the others with a reply the sheet cannot count.

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
    : {
      kind: 'answers',
      text: JSON.stringify({ assessment: 'A place that is not on the sheet.', candidateQuotes: [], choice: 'OCCURRENCE 9', keep: [], },),
    };
}

/**
 What the gather alone makes of the bench, to read beside what asking makes of it.

 @param behave - what each seat does

 @param seatTally - run's record of what each seat did, when a case needs seats it has withheld

 @param benchStart - seat the rotation starts at in place of the sheet's own, for a case that names the order the gather hands its
 voices back in; absent where the case compares the gather with asking, which rotates by the sheet too

 @returns Gather over the same sheet and bench

 @example
 ```ts
 const gathered = await gatheredBy({ behave: everySeatAnswers, },);
 ```
 */
async function gatheredBy(
  {
    behave,
    seatTally,
    benchStart,
  }: {
    readonly behave: SeatScript;
    readonly seatTally?: ReturnType<typeof createSeatTally>;
    readonly benchStart?: number;
  },
): ReturnType<typeof gatherStageVoices<unknown>> {
  return await gatherStageVoices({
    client: scriptedJudgeClient({ behave, asked: [], sheets: [], },),
    modelIds: BENCH,
    messages: buildConsistencyMessages({ subject: SUBJECT, },),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: consistencyResponseFormat({ labels: consistencyLabels({ count: 2, },), },),
    validate: usableConsistencyBallotFor({ labels: consistencyLabels({ count: 2, },), },),
    stage: 'page-consistency',
    l: levelCapturingLogger({ lines: [], },),
    graceMs: HANG_STOP_MS,
    ...((seatTally === undefined) ? {} : { seatTally, }),
    // Conditional spread keeps the pinned seat absent instead of undefined.
    ...((benchStart === undefined) ? {} : { benchStart, }),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: askConsistencyBench.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EVERY SEAT AND HANDS BACK THE BALLOTS IN ROSTER ORDER, each beside the seat that gave it, and stores the '
            + 'gather whole under the question\'s key',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const asked: RosterModelId[] = [];
            const result = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              lines: [],
              asked,
            },);
            // Control: the gather itself hands its voices back in the order the bench is rotated to, not the roster's. The rotation is
            // pinned to the second seat, since the sheet's own start is zero for some wording of the sheet, and then the order is the
            // roster's and the control says nothing.
            const gathered = await gatheredBy({ behave: everySeatAnswers, benchStart: 1, },);
            expect(gathered.voices.map(function seatOf(voice,): string {
              return voice.modelId;
            },),).not.toEqual(BENCH,);
            expect(result,).toEqual({
              quorum: 2,
              voices: BENCH.map(function voiceFor(seat,) {
                return voiceOf({ seat, },);
              },),
              findings: [],
              resumed: false,
            },);
            expect([...asked,].toSorted(),).toEqual([...BENCH,].toSorted(),);
            expect(persisted.map(function keyOf(entry,): string {
              return entry.key;
            },),).toEqual([KEY,],);
            expect(JSON.parse(persisted[0]?.serialized ?? 'null',),).toEqual({
              quorum: 2,
              voices: result.voices,
              findings: [],
            },);
          },
        },),
        it({
          name: 'STORES NOTHING FOR A GATHER THAT MISSED ITS QUORUM, handing back the ballot it did hear with the quorum '
            + 'it needed and the gather\'s findings, so a resume asks again',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const result = await askBench({
              behave: oneSeatAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              lines: [],
              asked: [],
            },);
            const gathered = await gatheredBy({ behave: oneSeatAnswers, },);
            expect(gathered.quorumMet,).toBe(false,);
            expect(result,).toEqual({
              quorum: 2,
              voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },),],
              findings: gathered.findings,
              resumed: false,
            },);
            expect(persisted,).toEqual([],);
          },
        },),
        it({
          name: 'READS BACK A STORED ANSWER INSTEAD OF ASKING, marking it resumed and saying how many voices it read, as the '
            + 'writers\' and the choosing judges\' asking does, since all three share one cache protocol',
          fn: async () => {
            const stored: ConsistencyRecord = {
              quorum: 2,
              voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), voiceOf({ seat: SEAT_BEDROCK_ONLY_TEXT, },),],
              findings: ['stage-voice-lost (page-consistency minimax-m3)',],
            };
            const lines: string[] = [];
            const asked: RosterModelId[] = [];
            const result = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
              lines,
              asked,
            },);
            expect(result,).toEqual({ ...stored, resumed: true, },);
            expect(asked,).toEqual([],);
            expect(lines,).toEqual(['info [askConsistencyBench] [gatherThroughCache] page-consistency: resumed 2 voices',],);
          },
        },),
        it({
          name: 'SAYS ONE VOICE, NOT ONE VOICES, WHEN IT READS BACK A GATHER OF ONE VOICE',
          fn: async () => {
            const stored: ConsistencyRecord = {
              quorum: 1,
              voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },),],
              findings: [],
            };
            const lines: string[] = [];
            await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
              lines,
              asked: [],
            },);
            expect(lines,).toEqual(['info [askConsistencyBench] [gatherThroughCache] page-consistency: resumed 1 voice',],);
          },
        },),
        it({
          name: 'RESUMES THE GATHER IT STORED FOR A BENCH OF TWO, whose quorum is one, asking no seat again',
          fn: async () => {
            const twoSeats = BENCH.slice(0, 2,);
            const twoSeatsKey = consistencyKey({
              stage: 'ballots',
              messages: buildConsistencyMessages({ subject: SUBJECT, },),
              modelIds: twoSeats,
            },);
            const persisted: { key: string; serialized: string; }[] = [];
            const fresh = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              modelIds: twoSeats,
              lines: [],
              asked: [],
            },);
            const stored: ConsistencyRecord = {
              quorum: fresh.quorum,
              voices: fresh.voices,
              findings: fresh.findings,
            };
            expect(persisted.map(function keyOf(entry,): string {
              return entry.key;
            },),).toEqual([twoSeatsKey,],);
            expect(JSON.parse(nonNullishOrThrow(persisted[0],).serialized,),).toEqual(stored,);
            const asked: RosterModelId[] = [];
            const again = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map([[twoSeatsKey, stored,],],), persisted: [], },),
              modelIds: twoSeats,
              lines: [],
              asked,
            },);
            expect({ quorum: fresh.quorum, asked, again, },).toEqual({
              quorum: 1,
              asked: [],
              again: { ...stored, resumed: true, },
            },);
          },
        },),
        it({
          name: 'RESUMES A GATHER THAT CLOSED ON A QUORUM OF ONE BECAUSE THE RUN HAD WITHHELD TWO SEATS OF A BENCH OF THREE, '
            + 'asking no seat again',
          fn: async () => {
            /**
             A tally holding the second and third seats withheld: each threw on all four asks it was given.
             */
            const seatTally = createSeatTally();
            for (const seat of BENCH.slice(1,)) {
              seatTally.record({ modelId: seat, outcome: 'threw', },);
              seatTally.record({ modelId: seat, outcome: 'threw', },);
              seatTally.record({ modelId: seat, outcome: 'threw', },);
              seatTally.record({ modelId: seat, outcome: 'threw', },);
            }
            const gathered = await gatheredBy({ behave: everySeatAnswers, seatTally, },);
            expect({
              quorum: gathered.quorum,
              quorumMet: gathered.quorumMet,
              heard: gathered.voices.map(function seatOf(voice,): string {
                return voice.modelId;
              },),
            },).toEqual({ quorum: 1, quorumMet: true, heard: [SEAT_HYPER_ONLY,], },);
            const stored: ConsistencyRecord = {
              quorum: gathered.quorum,
              voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },),],
              findings: gathered.findings,
            };
            const asked: RosterModelId[] = [];
            const again = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
              lines: [],
              asked,
            },);
            expect({ asked, again, },).toEqual({ asked: [], again: { ...stored, resumed: true, }, },);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE STORED BALLOTS HOLD A SEAT TWICE OR A SEAT THE BENCH DOES NOT HAVE, warning that they cannot '
            + 'be this bench\'s answer',
          fn: async () => {
            /**
             The ballots a gather over this bench cannot have made: one seat heard twice, and a seat from outside the bench.
             */
            const unfit: readonly ConsistencyRecord[] = [
              {
                quorum: 2,
                voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), voiceOf({ seat: SEAT_HYPER_ONLY, },),],
                findings: [],
              },
              {
                quorum: 2,
                voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },), { ...voiceOf({ seat: SEAT_BEDROCK_ONLY_TEXT, },), seat: 'unseated-cat', },],
                findings: [],
              },
            ];
            const warning = 'warn [askConsistencyBench] [gatherThroughCache] page-consistency: the stored answer cannot be an '
              + 'answer of this bench (another kind, a seat heard twice, or a seat the bench does not have), so the question '
              + 'is asked again';
            const results = await Promise.all(unfit.map(async function askedWith(stored,): Promise<unknown> {
              const lines: string[] = [];
              const asked: RosterModelId[] = [];
              const result = await askBench({
                behave: everySeatAnswers,
                cache: recordingCache({ resumed: new Map([[KEY, stored,],],), persisted: [], },),
                lines,
                asked,
              },);
              return {
                resumed: result.resumed,
                asked: asked.toSorted(),
                warnings: lines.filter(function namesTheStoredBallots(line,): boolean {
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
          name: 'KEEPS THE SLOT OF EACH QUOTE OF A REFRAIN AND KEYS THE QUESTION BY THE DEATH AND THE NAMES THE SHEET READS, so a '
            + 'page that records a death never resumes the gather of one that records none',
          fn: async () => {
            const framed: ConsistencySubject = {
              ...SUBJECT,
              repeat: { kind: 'refrain', prefix: '小猫', suffix: '打盹', },
              lifeEnded: true,
              subjectNames: ['Mimi',],
            };
            const persisted: { key: string; serialized: string; }[] = [];
            const result = await askBench({
              behave: function framedSeatAnswers({ seat, },): SeatBehaviour {
                return {
                  kind: 'answers',
                  text: JSON.stringify({
                    assessment: `${seat} words the first place best.`,
                    candidateQuotes: [
                      { label: 'OCCURRENCE 1', quote: 'The cat naps.', slot: 'cat', },
                      { label: 'OCCURRENCE 2', quote: 'The kitten naps.', slot: 'kitten', },
                    ],
                    choice: 'OCCURRENCE 1',
                    keep: [],
                  },),
                };
              },
              cache: recordingCache({ resumed: new Map(), persisted, },),
              subject: framed,
              lines: [],
              asked: [],
            },);
            expect(result.voices.map(function quotesOf(voice,): unknown {
              return voice.quotes;
            },),).toEqual(BENCH.map(function quotesWithSlots(): unknown {
              return [
                { label: 'OCCURRENCE 1', quote: 'The cat naps.', slot: 'cat', },
                { label: 'OCCURRENCE 2', quote: 'The kitten naps.', slot: 'kitten', },
              ];
            },),);
            expect(persisted.map(function keyOf(entry,): string {
              return entry.key;
            },),).toEqual([
              consistencyKey({
                stage: 'ballots',
                messages: buildConsistencyMessages({ subject: framed, },),
                modelIds: BENCH,
                lifeEnded: true,
                subjectNames: ['Mimi',],
              },),
            ],);
            expect(persisted[0]?.key,).not.toBe(consistencyKey({
              stage: 'ballots',
              messages: buildConsistencyMessages({ subject: framed, },),
              modelIds: BENCH,
            },),);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE STORED ANSWER IS FOR ANOTHER BENCH, since the key names the bench as well as the sheet',
          fn: async () => {
            const otherKey = consistencyKey({
              stage: 'ballots',
              messages: buildConsistencyMessages({ subject: SUBJECT, },),
              modelIds: BENCH.slice(0, 2,),
            },);
            const asked: RosterModelId[] = [];
            const result = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({
                resumed: new Map([[otherKey, { quorum: 1, voices: [voiceOf({ seat: SEAT_HYPER_ONLY, },),], findings: [], },],],),
                persisted: [],
              },),
              lines: [],
              asked,
            },);
            expect(result.resumed,).toBe(false,);
            expect(asked.length,).toBe(BENCH.length,);
          },
        },),
        it({
          name: 'COUNTS THE BALLOTS IN HAND WHEN THE STORE REFUSES THEM, warning that they are counted all the same',
          fn: async () => {
            const lines: string[] = [];
            const result = await askBench({
              behave: everySeatAnswers,
              cache: recordingCache({
                resumed: new Map(),
                persisted: [],
                refusal: new Error('the cat sat on the disk',),
              },),
              lines,
              asked: [],
            },);
            expect(result.voices.length,).toBe(BENCH.length,);
            expect(result.resumed,).toBe(false,);
            expect(lines.filter(function namesTheStore(line,): boolean {
              return line.includes('could not be stored',);
            },),).toEqual([
              'warn [askConsistencyBench] [gatherThroughCache] page-consistency: the answer could not be stored (refused by Error); '
                + 'it is counted all the same',
            ],);
          },
        },),
        it({
          name: 'REFUSES A BENCH THAT SEATS ONE MODEL TWICE before any seat is asked',
          fn: async () => {
            const asked: RosterModelId[] = [];
            const refusal = await rejectionOf(async function asksTwiceSeated(): Promise<unknown> {
              return await askBench({
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
  ],
},);

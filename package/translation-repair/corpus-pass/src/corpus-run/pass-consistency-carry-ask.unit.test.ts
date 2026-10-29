/**
 Tests for the two questions of a carry, asked through the cache: the writer
 bench asked for the receiving place's paragraph and the judge bench asked to
 choose among the candidates. Each is handed back in roster order beside the
 seat that gave it, stored under a key of its own stage, read back on a resume,
 and asked again when what is stored is another kind of answer.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  producedVolumeBound,
  type SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor';
import type { ChatJsonRequest, } from '@monochromatic-dev/translation-repair-provider';
import {
  buildCarryChoiceMessages,
  buildCarryWriterMessages,
  type CarryChoiceSubject,
  type CarryWriterSubject,
  type ConsistencyOccurrenceView,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  askCarryChoice,
  askCarryWriters,
  type ConsistencyCacheValue,
  consistencyKey,
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
 The chosen place as the consistency sheet shows it.
 */
const CHOSEN_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 0,
  paragraph: '小猫打盹。',
  markStart: 0,
  markEnd: 4,
  english: { kind: 'present', text: 'The cat naps.', },
};

/**
 The receiving place as the consistency sheet shows it.
 */
const RECEIVING_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 1,
  paragraph: '小猫打盹。',
  markStart: 0,
  markEnd: 4,
  english: { kind: 'present', text: 'The kitten naps.', },
};

/**
 What the writers' sheet shows.
 */
const WRITER_SUBJECT: CarryWriterSubject = {
  repeat: { kind: 'maximal', wording: '小猫打盹', },
  chosen: { view: CHOSEN_VIEW, span: 'The cat naps', paragraph: 'The cat naps.', },
  receiving: [{ view: RECEIVING_VIEW, span: 'The kitten naps', },],
  paragraph: 'The kitten naps.',
  keeping: [],
  lineStructured: false,
};

/**
 What the choosing judges' sheet shows.
 */
const CHOICE_SUBJECT: CarryChoiceSubject = {
  repeat: { kind: 'maximal', wording: '小猫打盹', },
  receiving: [{ view: RECEIVING_VIEW, span: 'The kitten naps', },],
  chosen: { view: CHOSEN_VIEW, span: 'The cat naps', paragraph: 'The cat naps.', },
  keeping: [],
  candidates: [
    { label: 'CANDIDATE 1', current: true, text: 'The kitten naps.', },
    { label: 'CANDIDATE 2', current: false, text: 'The cat naps.', },
  ],
};

/**
 The question the writers are put, as the cache names it.
 */
const WRITERS_KEY = consistencyKey({
  stage: 'writers',
  messages: buildCarryWriterMessages({ subject: WRITER_SUBJECT, },),
  modelIds: BENCH,
},);

/**
 The question the judges are put, as the cache names it.
 */
const CHOICE_KEY = consistencyKey({
  stage: 'choice',
  messages: buildCarryChoiceMessages({ subject: CHOICE_SUBJECT, },),
  modelIds: BENCH,
},);

/**
 What a writer sends, naming itself so a paragraph is told from the others'.

 @param seat - seat answering

 @returns Seat's reply as text

 @example
 ```ts
 const text = paragraphOf({ seat: SEAT_HYPER_ONLY, },);
 ```
 */
function paragraphOf({ seat, }: { readonly seat: RosterModelId; }): string {
  return `The cat naps (${seat}).`;
}

/**
 What a seat does when every writer answers with a paragraph.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = everyWriterAnswers;
 ```
 */
function everyWriterAnswers({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  return { kind: 'answers', text: JSON.stringify({ translation: paragraphOf({ seat, },), },), };
}

/**
 What a seat does when every judge chooses the second candidate.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = everyJudgeChooses;
 ```
 */
function everyJudgeChooses({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  return { kind: 'answers', text: JSON.stringify({ assessment: `${seat} prefers the cat.`, choice: 'CANDIDATE 2', },), };
}

/**
 Puts the writers' question to the bench.

 @param behave - what each seat does

 @param cache - store

 @param subject - question put, cat sheet by default

 @param lines - sink for the level and text of every line logged

 @param asked - sink for the seat of every call

 @returns What asking came to

 @example
 ```ts
 const result = await askWriters({ behave: everyWriterAnswers, cache, lines: [], asked: [], },);
 ```
 */
async function askWriters(
  {
    behave,
    cache,
    subject = WRITER_SUBJECT,
    lines,
    asked,
  }: {
    readonly behave: SeatScript;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly subject?: CarryWriterSubject;
    readonly lines: string[];
    readonly asked: RosterModelId[];
  },
): ReturnType<typeof askCarryWriters> {
  return await askCarryWriters({
    client: scriptedJudgeClient({ behave, asked, sheets: [], },),
    modelIds: BENCH,
    subject,
    cache,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l: levelCapturingLogger({ lines, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Puts the choosing question to the bench.

 @param behave - what each seat does

 @param cache - store

 @param lines - sink for the level and text of every line logged

 @param asked - sink for the seat of every call

 @returns What asking came to

 @example
 ```ts
 const result = await askChoice({ behave: everyJudgeChooses, cache, lines: [], asked: [], },);
 ```
 */
async function askChoice(
  {
    behave,
    cache,
    lines,
    asked,
  }: {
    readonly behave: SeatScript;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly lines: string[];
    readonly asked: RosterModelId[];
  },
): ReturnType<typeof askCarryChoice> {
  return await askCarryChoice({
    client: scriptedJudgeClient({ behave, asked, sheets: [], },),
    modelIds: BENCH,
    subject: CHOICE_SUBJECT,
    cache,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l: levelCapturingLogger({ lines, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: askCarryWriters.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EVERY WRITER AND HANDS BACK THE PARAGRAPHS IN ROSTER ORDER, each beside its seat, and stores the record '
            + 'under the key of the writers\' stage',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const asked: RosterModelId[] = [];
            const result = await askWriters({
              behave: everyWriterAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              lines: [],
              asked,
            },);
            const record = {
              kind: 'writers',
              quorum: 2,
              voices: BENCH.map(function voiceFor(seat,) {
                return { seat, translation: paragraphOf({ seat, },), };
              },),
              findings: [],
            };
            expect({ result, asked: asked.toSorted(), keys: persisted.map(function keyOf({ key, },): string {
              return key;
            },), }).toEqual({
              result: { ...record, resumed: false, },
              asked: [...BENCH,].toSorted(),
              keys: [WRITERS_KEY,],
            },);
            expect(JSON.parse(persisted[0]?.serialized ?? 'null',),).toEqual(record,);
          },
        },),
        it({
          name: 'COUNTS NO WRITER THAT SENDS A PARAGRAPH RENDERING AS NOTHING, so the gather waits for the quorum of the others',
          fn: async () => {
            const result = await askWriters({
              behave: function oneEmpty({ seat, },): SeatBehaviour {
                return (seat === SEAT_HYPER_ONLY)
                  ? { kind: 'answers', text: JSON.stringify({ translation: '  ', },), }
                  : everyWriterAnswers({ seat, },);
              },
              cache: recordingCache({ resumed: new Map(), persisted: [], },),
              lines: [],
              asked: [],
            },);
            expect(result.voices.map(function seatOf({ seat, },): string {
              return seat;
            },),).not.toContain(SEAT_HYPER_ONLY,);
          },
        },),
        it({
          name: 'READS BACK A STORED RECORD INSTEAD OF ASKING, marking it resumed and saying how many voices it read',
          fn: async () => {
            const stored: ConsistencyCacheValue = {
              kind: 'writers',
              quorum: 2,
              voices: [
                { seat: SEAT_HYPER_ONLY, translation: paragraphOf({ seat: SEAT_HYPER_ONLY, },), },
                { seat: SEAT_BEDROCK_ONLY_TEXT, translation: paragraphOf({ seat: SEAT_BEDROCK_ONLY_TEXT, },), },
              ],
              findings: [],
            };
            const lines: string[] = [];
            const asked: RosterModelId[] = [];
            const result = await askWriters({
              behave: everyWriterAnswers,
              cache: recordingCache({ resumed: new Map([[WRITERS_KEY, stored,],],), persisted: [], },),
              lines,
              asked,
            },);
            expect({ result, asked, lines, },).toEqual({
              result: { ...stored, resumed: true, },
              asked: [],
              lines: ['info [gatherThroughCache] page-consistency-writers: resumed 2 voices',],
            },);
          },
        },),
        it({
          name: 'ASKS AGAIN WHEN THE STORED VALUE IS A CHOICE AMONG CANDIDATES, since it is another kind of answer',
          fn: async () => {
            const asked: RosterModelId[] = [];
            const lines: string[] = [];
            const result = await askWriters({
              behave: everyWriterAnswers,
              cache: recordingCache({
                resumed: new Map([[WRITERS_KEY, { kind: 'choice', quorum: 1, voices: [], findings: [], },],],),
                persisted: [],
              },),
              lines,
              asked,
            },);
            expect({
              resumed: result.resumed,
              asked: asked.length,
              warned: lines.filter(function namesTheStoredAnswer(line,): boolean {
                return line.includes('the stored answer cannot be an answer of this bench',);
              },).length,
            },).toEqual({
              resumed: false,
              asked: BENCH.length,
              warned: 1,
            },);
          },
        },),
        it({
          name: 'BOUNDS EACH ANSWER BY THE ORIGINAL IN CODE POINTS, an original shared by two receiving places counted once',
          fn: async () => {
            const original = String.fromCodePoint(0x2_00_00,)
              .repeat(2_000,);
            const view: ConsistencyOccurrenceView = { ...RECEIVING_VIEW, paragraph: original, markEnd: 2, };
            /**
             The bound on the answer of each call, none recorded for a call that set none.
             */
            const bounds: number[] = [];
            const client = scriptedJudgeClient({ behave: everyWriterAnswers, asked: [], sheets: [], },);
            await askCarryWriters({
              client: {
                ...client,
                chatJson: function recording<ValueT,>(request: ChatJsonRequest<ValueT>,) {
                  if (request.maxAnswerChars !== undefined)
                    bounds.push(request.maxAnswerChars,);
                  return client.chatJson(request,);
                },
              },
              modelIds: BENCH,
              subject: {
                ...WRITER_SUBJECT,
                receiving: [{ view, span: 'The kitten naps', }, { view, span: 'the sun', },],
              },
              cache: recordingCache({ resumed: new Map(), persisted: [], },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines: [], },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...new Set(bounds,),],).toEqual([producedVolumeBound({ materialChars: 2_000, },),],);
          },
        },),
        it({
          name: 'KEYS THE QUESTION BY THE DEATH AND THE NAMES THE SHEET READS, so a page that records a death never resumes the '
            + 'writers of one that records none',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            await askWriters({
              behave: everyWriterAnswers,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              subject: { ...WRITER_SUBJECT, lifeEnded: true, subjectNames: ['Mimi',], },
              lines: [],
              asked: [],
            },);
            expect(persisted.map(function keyOf({ key, },): string {
              return key;
            },),).toEqual([
              consistencyKey({
                stage: 'writers',
                messages: buildCarryWriterMessages({
                  subject: { ...WRITER_SUBJECT, lifeEnded: true, subjectNames: ['Mimi',], },
                },),
                modelIds: BENCH,
                lifeEnded: true,
                subjectNames: ['Mimi',],
              },),
            ],);
            expect(persisted[0]?.key,).not.toBe(WRITERS_KEY,);
          },
        },),
      ],
    },),
    describe({
      name: askCarryChoice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EVERY JUDGE AND HANDS BACK THE CHOICES IN ROSTER ORDER, each beside its seat and reason, and stores the '
            + 'record under the key of the choice stage',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const result = await askChoice({
              behave: everyJudgeChooses,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              lines: [],
              asked: [],
            },);
            const record = {
              kind: 'choice',
              quorum: 2,
              voices: BENCH.map(function voiceFor(seat,) {
                return { seat, reason: `${seat} prefers the cat.`, choice: 'CANDIDATE 2', };
              },),
              findings: [],
            };
            expect({ result, keys: persisted.map(function keyOf({ key, },): string {
              return key;
            },), }).toEqual({ result: { ...record, resumed: false, }, keys: [CHOICE_KEY,], },);
            expect(JSON.parse(persisted[0]?.serialized ?? 'null',),).toEqual(record,);
          },
        },),
        it({
          name: 'COUNTS NO CHOICE OF A LABEL THAT IS NOT AMONG THE CANDIDATES, so a seat that names one is a lost voice',
          fn: async () => {
            const result = await askChoice({
              behave: function oneStray({ seat, },): SeatBehaviour {
                return (seat === SEAT_HYPER_ONLY)
                  ? { kind: 'answers', text: JSON.stringify({ assessment: 'A candidate that is not on the sheet.', choice: 'CANDIDATE 9', },), }
                  : everyJudgeChooses({ seat, },);
              },
              cache: recordingCache({ resumed: new Map(), persisted: [], },),
              lines: [],
              asked: [],
            },);
            expect(result.voices.map(function seatOf({ seat, },): string {
              return seat;
            },),).not.toContain(SEAT_HYPER_ONLY,);
          },
        },),
        it({
          name: 'LISTS THE SHEET\'S LABELS AS THE ENUM OF THE CHOICE in the response format of the ask, the first ask and the run-off alike',
          fn: async () => {
            /**
             The response format of every call, in order.
             */
            const formats: NonNullable<ChatJsonRequest<unknown>['responseFormat']>[] = [];
            const client = scriptedJudgeClient({ behave: everyJudgeChooses, asked: [], sheets: [], },);
            /**
             The client, keeping the response format of each call.
             */
            const recording = {
              ...client,
              chatJson: function recordingFormat<ValueT,>(request: ChatJsonRequest<ValueT>,) {
                if (request.responseFormat !== undefined)
                  formats.push(request.responseFormat,);
                return client.chatJson(request,);
              },
            };
            await Promise.all([false, true,].map(function askWith(runOff,): Promise<unknown> {
              return askCarryChoice({
                client: recording,
                modelIds: BENCH,
                subject: CHOICE_SUBJECT,
                cache: recordingCache({ resumed: new Map(), persisted: [], },),
                signal: AbortSignal.timeout(HANG_STOP_MS,),
                exchangeTimeoutMs: HANG_STOP_MS,
                l: levelCapturingLogger({ lines: [], },),
                stageSettings: SCRIPTED_STAGE_SETTINGS,
                runOff,
              },);
            },),);
            expect(formats.length,).toBe(BENCH.length * 2,);
            expect(formats.map(function enumOf(format,): unknown {
              return (format.json_schema.schema.properties as Record<string, { enum?: string[]; }>).choice?.enum;
            },),).toEqual(formats.map(function expected(): unknown {
              return ['CANDIDATE 1', 'CANDIDATE 2',];
            },),);
          },
        },),
        it({
          name: 'ASKS THE RUN-OFF UNDER A STAGE AND A KEY OF ITS OWN, so its answer is stored and read back apart from the first '
            + 'ask\'s, which asked the same bench the same candidates',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const lines: string[] = [];
            await askCarryChoice({
              client: scriptedJudgeClient({ behave: everyJudgeChooses, asked: [], sheets: [], },),
              modelIds: BENCH,
              subject: CHOICE_SUBJECT,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
              runOff: true,
            },);
            /**
             The key the run-off is kept under.
             */
            const runOffKey = consistencyKey({
              stage: 'choice-run-off',
              messages: buildCarryChoiceMessages({ subject: CHOICE_SUBJECT, runOff: true, },),
              modelIds: BENCH,
            },);
            expect(persisted.map(function keyOf({ key, },): string {
              return key;
            },),).toEqual([runOffKey,],);
            expect(runOffKey,).not.toBe(CHOICE_KEY,);
            // A stored record of the run-off is read back from its own key, saying its own stage.
            const resumedLines: string[] = [];
            const resumed = await askCarryChoice({
              client: scriptedJudgeClient({ behave: everyJudgeChooses, asked: [], sheets: [], },),
              modelIds: BENCH,
              subject: CHOICE_SUBJECT,
              cache: recordingCache({
                resumed: new Map([[runOffKey, JSON.parse(persisted[0]?.serialized ?? 'null',) as ConsistencyCacheValue,],],),
                persisted: [],
              },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines: resumedLines, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
              runOff: true,
            },);
            expect({ resumed: resumed.resumed, resumedLines, },).toEqual({
              resumed: true,
              resumedLines: ['info [gatherThroughCache] page-consistency-choice-run-off: resumed 3 voices',],
            },);
          },
        },),
        it({
          name: 'READS BACK A STORED CHOICE INSTEAD OF ASKING, and asks again when the stored value is a writers\' record',
          fn: async () => {
            const stored: ConsistencyCacheValue = {
              kind: 'choice',
              quorum: 2,
              voices: [
                { seat: SEAT_HYPER_ONLY, reason: 'The cat.', choice: 'CANDIDATE 2', },
                { seat: SEAT_BEDROCK_ONLY_TEXT, reason: 'The cat.', choice: 'CANDIDATE 2', },
              ],
              findings: [],
            };
            const readAsked: RosterModelId[] = [];
            const read = await askChoice({
              behave: everyJudgeChooses,
              cache: recordingCache({ resumed: new Map([[CHOICE_KEY, stored,],],), persisted: [], },),
              lines: [],
              asked: readAsked,
            },);
            const againAsked: RosterModelId[] = [];
            const again = await askChoice({
              behave: everyJudgeChooses,
              cache: recordingCache({
                resumed: new Map([[CHOICE_KEY, { kind: 'writers', quorum: 1, voices: [], findings: [], },],],),
                persisted: [],
              },),
              lines: [],
              asked: againAsked,
            },);
            expect({
              read,
              readAsked,
              again: again.resumed,
              againAsked: againAsked.length,
            },).toEqual({
              read: { ...stored, resumed: true, },
              readAsked: [],
              again: false,
              againAsked: BENCH.length,
            },);
          },
        },),
      ],
    },),
  ],
},);

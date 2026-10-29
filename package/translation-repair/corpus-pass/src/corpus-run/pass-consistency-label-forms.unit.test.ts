/**
 Tests for what asking the bench does with a label written in another form (the lead's ruling of 2026-10-15): a
 reply that names exactly one shown label counts as that label and is stored in its canonical form, the sheet names
 the exact labels the reply may use, and the response format of the ask lists them as an `enum`.

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
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  JsonSchemaResponseFormat,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buildConsistencyMessages,
  buildConsistencyRunOffMessages,
  type ConsistencyRendering,
  type ConsistencySubject,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  askConsistencyBench,
  askRunOff,
  choiceRecordOf,
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
 The repeated sentence in three places.
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
 What each seat writes as the label of its choice, in another form of a shown label.
 */
const FORMS: ReadonlyMap<RosterModelId, string> = new Map([
  [SEAT_HYPER_ONLY, 'Occurrence 2',],
  [SEAT_BEDROCK_ONLY_TEXT, ' 2 ',],
  [SEAT_HYPER_VISION, 'occurrence 1',],
]);

/**
 The canonical label each seat's form stands for.
 */
const CANONICAL: ReadonlyMap<RosterModelId, string> = new Map([
  [SEAT_HYPER_ONLY, 'OCCURRENCE 2',],
  [SEAT_BEDROCK_ONLY_TEXT, 'OCCURRENCE 2',],
  [SEAT_HYPER_VISION, 'OCCURRENCE 1',],
]);

/**
 Wraps a client so each response format it is sent is kept.

 @param client - client to wrap
 @param formats - sink for the response format of every call

 @returns Same client, recording the formats

 @example
 ```ts
 const client = formatRecording({ client: scriptedJudgeClient({ behave, asked: [], sheets: [], },), formats: [], },);
 ```
 */
function formatRecording(
  {
    client,
    formats,
  }: {
    readonly client: ReturnType<typeof scriptedJudgeClient>;
    readonly formats: JsonSchemaResponseFormat[];
  },
): ReturnType<typeof scriptedJudgeClient> {
  return {
    ...client,
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      // A call with no response format is one a provider answers in any shape, so the case fails on it.
      if (request.responseFormat === undefined)
        throw new Error('the pass sent a consistency call with no response format',);
      formats.push(request.responseFormat,);
      return client.chatJson(request,);
    },
  };
}

/**
 What a seat sends in the first round, writing every label in another form.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = firstRoundInOtherForms;
 ```
 */
function firstRoundInOtherForms({ seat, }: { readonly seat: RosterModelId; },): SeatBehaviour {
  return {
    kind: 'answers',
    text: JSON.stringify({
      assessment: `${seat} words the first place best.`,
      candidateQuotes: [
        { label: '1', quote: 'The cat naps.', },
        { label: ' occurrence 2', quote: 'The kitten naps.', },
        { label: 'Occurrence 3', quote: 'The tabby naps.', },
      ],
      choice: 'Occurrence 1',
      keep: [' 2 ', 'occurrence 3',],
    },),
  };
}

/**
 What a seat sends in the run-off, writing its choice in another form.

 @param seat - seat asked

 @returns Seat's reply

 @example
 ```ts
 const behave: SeatScript = runOffInOtherForms;
 ```
 */
function runOffInOtherForms({ seat, }: { readonly seat: RosterModelId; }): SeatBehaviour {
  return { kind: 'answers', text: JSON.stringify({ assessment: `${seat} prefers its place.`, choice: FORMS.get(seat,), },), };
}

/**
 Puts the run-off to the bench.

 @param behave - what each seat does
 @param cache - store
 @param formats - sink for the response format of every call
 @param sheets - sink for the sheet of every call

 @returns What asking came to

 @example
 ```ts
 const result = await runOff({ behave: runOffInOtherForms, cache, formats: [], sheets: [], },);
 ```
 */
async function runOff(
  {
    behave,
    cache,
    formats,
    sheets,
  }: {
    readonly behave: SeatScript;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly formats: JsonSchemaResponseFormat[];
    readonly sheets: string[];
  },
): ReturnType<typeof askRunOff> {
  return await askRunOff({
    client: formatRecording({ client: scriptedJudgeClient({ behave, asked: [], sheets, },), formats, },),
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
      name: askConsistencyBench.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STORES A BALLOT THAT WRITES ITS CHOICE, KEPT PLACES AND QUOTE LABELS IN OTHER FORMS WITH THE SHOWN LABELS, and '
            + 'asks with a sheet and a response format that name the labels it may use',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const formats: JsonSchemaResponseFormat[] = [];
            const sheets: string[] = [];
            const result = await askConsistencyBench({
              client: formatRecording({
                client: scriptedJudgeClient({ behave: firstRoundInOtherForms, asked: [], sheets, },),
                formats,
              },),
              modelIds: BENCH,
              subject: SUBJECT,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l: levelCapturingLogger({ lines: [], },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(result.voices.length,).toBe(3,);
            for (const voice of result.voices) {
              expect({
                choice: voice.choice,
                keep: voice.keep,
                labels: voice.quotes.map(function labelOf(quote,): string {
                  return quote.label;
                },),
              },).toEqual({
                choice: 'OCCURRENCE 1',
                keep: ['OCCURRENCE 2', 'OCCURRENCE 3',],
                labels: ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],
              },);
            }
            expect(JSON.parse(persisted[0]?.serialized ?? 'null',) as unknown,).toEqual({
              quorum: 2,
              voices: result.voices,
              findings: result.findings,
            },);
            // The key is still the sheet's, whose messages now name the labels.
            expect(persisted.map(function keyOf(entry,): string {
              return entry.key;
            },),).toEqual([
              consistencyKey({ stage: 'ballots', messages: buildConsistencyMessages({ subject: SUBJECT, },), modelIds: BENCH, },),
            ],);
            for (const sheet of sheets) {
              expect(sheet.includes('exactly: OCCURRENCE 1, OCCURRENCE 2, OCCURRENCE 3.',),).toBe(true,);
            }
            expect(formats.length,).toBeGreaterThan(0,);
            for (const format of formats) {
              const properties = format.json_schema.schema.properties as Record<string, { enum?: string[]; items?: unknown; }>;
              expect(properties.choice?.enum,).toEqual(['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',],);
            }
          },
        },),
      ],
    },),
    describe({
      name: askRunOff.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STORES A CHOICE WRITTEN IN ANOTHER FORM OF A FINALIST WITH THE SHOWN LABEL, and asks with a sheet and a response '
            + 'format that name the finalists only',
          fn: async () => {
            const persisted: { key: string; serialized: string; }[] = [];
            const formats: JsonSchemaResponseFormat[] = [];
            const sheets: string[] = [];
            const result = await runOff({
              behave: runOffInOtherForms,
              cache: recordingCache({ resumed: new Map(), persisted, },),
              formats,
              sheets,
            },);
            expect(
              new Map(result.voices.map(function entryOf(voice,): [string, string] {
                return [voice.seat, voice.choice,];
              },),),
            ).toEqual(CANONICAL,);
            expect(persisted.map(function keyOf(entry,): string {
              return entry.key;
            },),).toEqual([
              consistencyKey({
                stage: 'run-off',
                messages: buildConsistencyRunOffMessages({ subject: SUBJECT, finalists: FINALISTS, renderings: RENDERINGS, },),
                modelIds: BENCH,
              },),
            ],);
            for (const sheet of sheets) {
              expect(sheet.includes('exactly: OCCURRENCE 1, OCCURRENCE 2.',),).toBe(true,);
              expect(sheet.includes('OCCURRENCE 3',),).toBe(false,);
            }
            expect(formats.length,).toBeGreaterThan(0,);
            for (const format of formats) {
              const properties = format.json_schema.schema.properties as Record<string, { enum?: string[]; }>;
              expect(properties.choice?.enum,).toEqual(['OCCURRENCE 1', 'OCCURRENCE 2',],);
            }
          },
        },),
      ],
    },),
    describe({
      name: choiceRecordOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STORES EACH CHOICE AS THE SHOWN LABEL IT NAMES when the labels are given, and as heard when they are not',
          fn: async () => {
            const voices = [
              { modelId: SEAT_HYPER_ONLY, value: { assessment: 'r', choice: 'Candidate 2', }, },
              { modelId: SEAT_BEDROCK_ONLY_TEXT, value: { assessment: 'r', choice: '1', }, },
            ];
            expect(
              choiceRecordOf({ quorum: 2, voices, findings: [], labels: ['CANDIDATE 1', 'CANDIDATE 2',], },).voices.map(
                function choiceOf(voice,): string {
                  return voice.choice;
                },
              ),
            ).toEqual(['CANDIDATE 2', 'CANDIDATE 1',],);
            expect(
              choiceRecordOf({ quorum: 2, voices, findings: [], },).voices.map(function choiceOf(voice,): string {
                return voice.choice;
              },),
            ).toEqual(['Candidate 2', '1',],);
          },
        },),
      ],
    },),
  ],
},);

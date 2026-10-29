/**
 Tests for the reply of a judge who only chooses (the run-off and the choice
 among a receiving place's candidates): the shape, which choices count against
 the labels a sheet showed, and the strict majority of those that count.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  choiceTallyOf,
  type ConsistencyChoiceVoice,
  consistencyChoiceResponseFormat,
  isConsistencyChoiceBallot,
  isConsistencyChoiceWire,
  isUsableChoice,
  readConsistencyChoiceWire,
  usableChoiceFor,
  usableChoicesOf,
} from '../dist/final/node/index.mjs';

/**
 Labels a sheet showed.
 */
const LABELS = ['CANDIDATE 1', 'CANDIDATE 2',] as const;

/**
 Builds one heard choice.

 @param seat - seat that answered

 @param choice - label chosen

 @param reason - reason given

 @returns Choice with its seat

 @example
 ```ts
 const voice = voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },);
 ```
 */
function voiceOf(
  {
    seat,
    choice,
    reason = 'The kitten reads better here.',
  }: {
    readonly seat: string;
    readonly choice: string;
    readonly reason?: string;
  },
): ConsistencyChoiceVoice {
  return {
    seat,
    reason,
    choice,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencyChoiceResponseFormat.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DECLARES THE REASON BEFORE THE CHOICE, both required, nothing else allowed, and lists the labels as an enum',
          fn: async () => {
            const format = consistencyChoiceResponseFormat({ labels: LABELS, },);
            expect(format,).toEqual({
              type: 'json_schema',
              json_schema: {
                name: 'page_consistency_choice',
                schema: {
                  type: 'object',
                  required: ['assessment', 'choice',],
                  additionalProperties: false,
                  properties: {
                    assessment: { type: 'string', },
                    choice: { type: 'string', enum: ['CANDIDATE 1', 'CANDIDATE 2',], },
                  },
                },
              },
            },);
            // The order the keys are written in is the order the reply is asked in.
            expect(JSON.stringify(format.json_schema.schema,),).toBe(
              '{"type":"object","required":["assessment","choice"],"additionalProperties":false,'
                + '"properties":{"assessment":{"type":"string"},"choice":{"type":"string","enum":["CANDIDATE 1","CANDIDATE 2"]}}}',
            );
          },
        },),
      ],
    },),
    describe({
      name: isConsistencyChoiceWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A RECORD WITH A STRING ASSESSMENT AND A STRING CHOICE, and nothing else',
          fn: async () => {
            expect([
              isConsistencyChoiceWire({ assessment: 'r', choice: 'CANDIDATE 1', },),
              isConsistencyChoiceWire({ assessment: 'r', },),
              isConsistencyChoiceWire({ choice: 'CANDIDATE 1', },),
              isConsistencyChoiceWire({ assessment: 1, choice: 'CANDIDATE 1', },),
              isConsistencyChoiceWire({ assessment: 'r', choice: ['CANDIDATE 1',], },),
              isConsistencyChoiceWire('CANDIDATE 1',),
              isConsistencyChoiceWire(null,),
            ],).toEqual([true, false, false, false, false, false, false,],);
          },
        },),
        it({
          name: 'REFUSES A REPLY THAT STILL SPELLS THE REASON `reason`, since the reply key is `assessment` (ledger B762)',
          fn: async () => {
            expect(isConsistencyChoiceWire({ reason: 'r', choice: 'CANDIDATE 1', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isConsistencyChoiceBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A CHOICE AS STORED UNDER THE SPELLING IT WAS STORED UNDER, `reason`, so a cache written before the '
            + 'reply key was renamed still counts (ledger B762), and refuses the reply spelling and the rest',
          fn: async () => {
            expect([
              isConsistencyChoiceBallot({ reason: 'r', choice: 'CANDIDATE 1', },),
              isConsistencyChoiceBallot({ seat: 'a', reason: 'r', choice: 'CANDIDATE 1', },),
              isConsistencyChoiceBallot({ assessment: 'r', choice: 'CANDIDATE 1', },),
              isConsistencyChoiceBallot({ reason: 'r', },),
              isConsistencyChoiceBallot({ reason: 1, choice: 'CANDIDATE 1', },),
              isConsistencyChoiceBallot(null,),
            ],).toEqual([true, true, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: readConsistencyChoiceWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STORES THE ASSESSMENT AS THE REASON and keeps the choice, so what a gather stores is spelled as before',
          fn: async () => {
            expect(readConsistencyChoiceWire({ wire: { assessment: 'It reads better.', choice: 'CANDIDATE 2', }, },),).toEqual({
              reason: 'It reads better.',
              choice: 'CANDIDATE 2',
            },);
          },
        },),
      ],
    },),
    describe({
      name: isUsableChoice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS A CHOICE THAT GIVES A REASON A READER CAN SEE AND NAMES A LABEL THE SHEET SHOWED',
          fn: async () => {
            expect([
              isUsableChoice({ ballot: { reason: 'It reads better.', choice: 'CANDIDATE 2', }, labels: LABELS, },),
              isUsableChoice({ ballot: { reason: '  ', choice: 'CANDIDATE 2', }, labels: LABELS, },),
              isUsableChoice({ ballot: { reason: 'It reads better.', choice: 'CANDIDATE 9', }, labels: LABELS, },),
            ],).toEqual([true, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: usableChoiceFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GUARDS A GATHER so a reply that cannot be counted is asked again like one that is not a choice',
          fn: async () => {
            const validate = usableChoiceFor({ labels: LABELS, },);
            expect([
              validate({ assessment: 'It reads better.', choice: 'CANDIDATE 1', },),
              validate({ assessment: 'It reads better.', choice: 'CANDIDATE 3', },),
              validate({ assessment: '', choice: 'CANDIDATE 1', },),
              validate({ choice: 'CANDIDATE 1', },),
            ],).toEqual([true, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: usableChoicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SELECTS THE CHOICES THAT COUNT, in the order they were heard',
          fn: async () => {
            const voices = [
              voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },),
              voiceOf({ seat: 'b', choice: 'CANDIDATE 9', },),
              voiceOf({ seat: 'c', choice: 'CANDIDATE 1', reason: '', },),
              voiceOf({ seat: 'd', choice: 'CANDIDATE 1', },),
            ];
            expect(usableChoicesOf({ voices, labels: LABELS, },).map(function seatOf(voice,): string {
              return voice.seat;
            },),).toEqual(['a', 'd',],);
          },
        },),
      ],
    },),
    describe({
      name: choiceTallyOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE USABLE CHOICES AND NAMES THE LABEL MORE THAN HALF OF THEM CHOSE, none for a tie',
          fn: async () => {
            const majority = [
              voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },),
              voiceOf({ seat: 'b', choice: 'CANDIDATE 2', },),
              voiceOf({ seat: 'c', choice: 'CANDIDATE 1', },),
              voiceOf({ seat: 'd', choice: 'CANDIDATE 9', },),
            ];
            const tied = [
              voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },),
              voiceOf({ seat: 'b', choice: 'CANDIDATE 1', },),
            ];
            expect([
              choiceTallyOf({ voices: majority, labels: LABELS, },),
              choiceTallyOf({ voices: tied, labels: LABELS, },),
              choiceTallyOf({ voices: [], labels: LABELS, },),
            ],).toEqual([
              { usable: 3, chosen: ['CANDIDATE 2',], },
              { usable: 2, chosen: [], },
              { usable: 0, chosen: [], },
            ],);
          },
        },),
      ],
    },),
  ],
},);

/**
 Tests the scoring behind `reader-measure`: how a candidate reader's
 transcription is held against the readings the pipeline already trusts, and
 the two clauses of the reader rule written on 2026-09-08 (the openrouter
 planning record, "The three Gemma sizes against the seated readers on nine
 pictures").

 THE RULE: a candidate joins the readers when, over the same pictures, each
 reading it produces is corroborated by every seated reader of that picture
 at least as often as the seated reader corroborated least often, and it
 produces a reading no seated reader corroborates no more often than the
 seated reader uncorroborated most often. A call that produces nothing counts
 for neither clause.

 Fixtures are cat-themed invention: three strings that corroborate one another
 and one that shares no trigram with them.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  seatedBandOf,
  candidateRatesOf,
  MINIMUM_PRODUCED_READINGS,
  verdictOf,
} from '../../dist/final/node/index.mjs';

/**
 A transcription of a notice about a lost cat.
 */
const NOTICE = 'Lost tabby cat called Mittens, last seen by the old well. Please call 555 0134.';

/**
 Another model's transcription of the same notice, with a word dropped.
 */
const NOTICE_SHORT = 'Lost tabby cat called Mittens, last seen by the old well.';

/**
 A reading that shares no three-character run with the notice.
 */
const UNRELATED = 'Zyx wvu qj kk zz yy xx.';

/**
 The trusted readings of one picture: two that agree and one that does not.
 */
const SPLIT_RECORD = [
  {
    modelId: 'seated-one',
    text: NOTICE,
  },
  {
    modelId: 'seated-two',
    text: NOTICE_SHORT,
  },
  {
    modelId: 'seated-three',
    text: UNRELATED,
  },
] as const;

/**
 The trusted readings of a picture all three agree on.
 */
const AGREED_RECORD = [
  {
    modelId: 'seated-one',
    text: NOTICE,
  },
  {
    modelId: 'seated-two',
    text: NOTICE_SHORT,
  },
  {
    modelId: 'seated-three',
    text: NOTICE,
  },
] as const;

await describe({
  name: 'reader measure score',
  children: [
    describe({
      name: seatedBandOf.name,
      children: [
        it({
          name: 'READS THE SEATED BAND AS THE LOWEST RATE OF BEING CORROBORATED BY EVERY OTHER READER and the '
            + 'highest rate of being corroborated by none, per reader over the matched pictures',
          fn: async () => {
            await Promise.resolve();
            // seated-one and seated-two agree with each other but not with seated-three on the split picture;
            // seated-three agrees with nobody there, and with both others on the agreed one.
            const band = seatedBandOf({ records: [SPLIT_RECORD, AGREED_RECORD,], },);
            expect(band.floorCorroboratedByAll,).toBe(0.5,);
            expect(band.ceilingCorroboratedByNone,).toBe(0.5,);
            expect(band.readings,).toBe(6,);
          },
        },),

        it({
          name: 'READS A BAND OF FULL AGREEMENT AS A FLOOR OF ONE AND A CEILING OF ZERO',
          fn: async () => {
            await Promise.resolve();
            const band = seatedBandOf({ records: [AGREED_RECORD,], },);
            expect(band.floorCorroboratedByAll,).toBe(1,);
            expect(band.ceilingCorroboratedByNone,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: candidateRatesOf.name,
      children: [
        it({
          name: 'COUNTS A READING CORROBORATED BY EVERY TRUSTED READER, one corroborated by none, and the mean overlap',
          fn: async () => {
            await Promise.resolve();
            const rates = candidateRatesOf({
              rows: [
                {
                  candidate: NOTICE,
                  trusted: AGREED_RECORD,
                },
                {
                  candidate: UNRELATED,
                  trusted: AGREED_RECORD,
                },
                {
                  candidate: NOTICE,
                  trusted: SPLIT_RECORD,
                },
              ],
            },);
            expect(rates.produced,).toBe(3,);
            expect(rates.corroboratedByAll,).toBeCloseTo(1 / 3, 12,);
            expect(rates.corroboratedByNone,).toBeCloseTo(1 / 3, 12,);
            expect(rates.meanOverlap,).toBeGreaterThan(0.3,);
          },
        },),

        it({
          name: 'COUNTS NOTHING FOR NO PRODUCED READING',
          fn: async () => {
            await Promise.resolve();
            expect(candidateRatesOf({ rows: [], },),).toEqual({
              produced: 0,
              corroboratedByAll: 0,
              corroboratedByNone: 0,
              meanOverlap: 0,
            },);
          },
        },),
      ],
    },),

    describe({
      name: verdictOf.name,
      children: [
        it({
          name: 'MEETS BOTH CLAUSES when the candidate is corroborated as often as the weakest seated reader and '
            + 'uncorroborated no more often than the worst',
          fn: async () => {
            await Promise.resolve();
            expect(verdictOf({
              band: {
                floorCorroboratedByAll: 0.8,
                ceilingCorroboratedByNone: 0.1,
                readings: 20,
              },
              candidate: {
                produced: MINIMUM_PRODUCED_READINGS,
                corroboratedByAll: 0.8,
                corroboratedByNone: 0.1,
                meanOverlap: 0.7,
              },
            },),).toEqual({ kind: 'meets-both-clauses', },);
          },
        },),

        it({
          name: 'FAILS THE FIRST CLAUSE below the floor, the second above the ceiling, and names both when both fail',
          fn: async () => {
            await Promise.resolve();
            /**
             Band every case holds the candidate to.
             */
            const band = {
              floorCorroboratedByAll: 0.8,
              ceilingCorroboratedByNone: 0.1,
              readings: 20,
            };
            expect(verdictOf({
              band,
              candidate: {
                produced: 9,
                corroboratedByAll: 0.5,
                corroboratedByNone: 0,
                meanOverlap: 0.5,
              },
            },),).toEqual({ kind: 'fails-clauses', clauses: ['first',], },);
            expect(verdictOf({
              band,
              candidate: {
                produced: 9,
                corroboratedByAll: 0.9,
                corroboratedByNone: 0.3,
                meanOverlap: 0.5,
              },
            },),).toEqual({ kind: 'fails-clauses', clauses: ['second',], },);
            expect(verdictOf({
              band,
              candidate: {
                produced: 9,
                corroboratedByAll: 0.1,
                corroboratedByNone: 0.5,
                meanOverlap: 0.2,
              },
            },),).toEqual({ kind: 'fails-clauses', clauses: ['first', 'second',], },);
          },
        },),

        it({
          name: 'IS INCONCLUSIVE BELOW THE LEAST NUMBER OF PRODUCED READINGS A VERDICT RESTS ON, whatever the rates say',
          fn: async () => {
            await Promise.resolve();
            expect(verdictOf({
              band: {
                floorCorroboratedByAll: 0.8,
                ceilingCorroboratedByNone: 0.1,
                readings: 20,
              },
              candidate: {
                produced: MINIMUM_PRODUCED_READINGS - 1,
                corroboratedByAll: 1,
                corroboratedByNone: 0,
                meanOverlap: 0.9,
              },
            },),).toEqual({ kind: 'inconclusive', produced: MINIMUM_PRODUCED_READINGS - 1, },);
          },
        },),
      ],
    },),
  ],
},);

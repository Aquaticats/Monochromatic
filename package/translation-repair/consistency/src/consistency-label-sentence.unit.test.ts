/**
 Tests for the sentence that ends every sheet asking a judge for a label (the lead's ruling of 2026-10-15): it lists the
 exact labels the reply may use, in the order the sheet shows them, names where in the reply a label goes, and says a
 number alone is not a label.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { labelsSentenceOf, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: labelsSentenceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE LABELS IN THE ORDER SHOWN and names the one field a label goes in',
          fn: async () => {
            expect(labelsSentenceOf({ labels: ['OCCURRENCE 4', 'OCCURRENCE 3',], fields: ['"choice"',], },),).toBe(
              'The labels you may write are exactly: OCCURRENCE 4, OCCURRENCE 3. Write each one as shown, with its word and '
                + 'its number, in "choice"; a number alone is not a label.',
            );
          },
        },),
        it({
          name: 'JOINS TWO FIELDS WITH "and" AND THREE WITH COMMAS before the "and"',
          fn: async () => {
            expect([
              labelsSentenceOf({ labels: ['OCCURRENCE 1',], fields: ['"choice"', '"keep"',], },),
              labelsSentenceOf({ labels: ['OCCURRENCE 1',], fields: ['"choice"', '"keep"', 'each quote\'s "label"',], },),
            ],).toEqual([
              'The labels you may write are exactly: OCCURRENCE 1. Write each one as shown, with its word and its number, in '
                + '"choice" and "keep"; a number alone is not a label.',
              'The labels you may write are exactly: OCCURRENCE 1. Write each one as shown, with its word and its number, in '
                + '"choice", "keep" and each quote\'s "label"; a number alone is not a label.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A SHEET THAT NAMES NO FIELD, rather than writing a sentence that says the label goes nowhere',
          fn: async () => {
            /**
             What the call threw.
             */
            const refusal = caught(function namesNoField(): string {
              return labelsSentenceOf({ labels: ['CANDIDATE 1', 'CANDIDATE 2',], fields: [], },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: a sheet asks for a label and names no field it goes in',
            );
          },
        },),
      ],
    },),
  ],
},);

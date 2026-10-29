/**
 Tests for reading a model's JSON answer when more text follows it.

 LEDGER P8 (the whole-package audit, 2026-09-27): a complete JSON value
 followed by a separator, a sentence of explanation or a stray fence failed
 the whole-text parse and was lost as a schema mismatch, 760 times across the
 run logs and about four voices on each TianqiChen666 run. Every such stored
 reply ended with the provider's stop reason, so the value before the text is
 the model's whole answer.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseAnswerJson, } from '../dist/final/node/index.mjs';

/**
 One reply and the value it must read as.
 */
type TrailingCase = readonly [
  label: string,
  text: string,
  value: unknown,
];

/**
 Replies a complete value opens and more text follows.
 */
const TRAILING_CASES: readonly TrailingCase[] = [
  [
    'a separator and a sentence',
    '{"checks": [{"issue": 1, "verdict": "fixed"}]}\n\n---\nThe cat is fixed.',
    { checks: [{
      issue: 1,
      verdict: 'fixed',
    },], },
  ],
  [
    'a closing brace inside a string',
    '{"reason": "the } sat on the mat", "vote": "supported"} Meow.',
    {
      reason: 'the } sat on the mat',
      vote: 'supported',
    },
  ],
  [
    'an escaped quote before a brace inside a string',
    '{"reason": "she said \\"nap}\\" twice"}\nThat is all.',
    { reason: 'she said "nap}" twice', },
  ],
  [
    'an array',
    '[1, 2, 3]\nThree cats.',
    [
      1,
      2,
      3,
    ],
  ],
  [
    'a stray fence',
    '{"best": 2}\n```',
    { best: 2, },
  ],
];

await describe({
  name: 'reading a JSON answer that more text follows (ledger P8)',
  children: [
    it({
      name: 'READS THE VALUE an answer opens with when prose, a separator or a fence follows it',
      fn: async () => {
        expect(TRAILING_CASES.map(function readingOf({ 0: label, 1: text, },): string {
          /**
           How the reply parsed.
           */
          const attempt = parseAnswerJson({ text, },);
          return `${label}: ${attempt.parsed ? JSON.stringify(attempt.value,) : 'lost'}`;
        },),).toEqual(TRAILING_CASES.map(function expectedOf({ 0: label, 2: value, },): string {
          return `${label}: ${JSON.stringify(value,)}`;
        },),);
      },
    },),
    it({
      name: 'STILL REFUSES an answer whose opening value never closes, since that is a cut answer, and one that '
        + 'opens with prose; an abandoned opening before a whole object is the false-start reader\'s to read',
      fn: async () => {
        expect(parseAnswerJson({ text: '{"cats": [1, 2', },).parsed,).toBe(false,);
        expect(parseAnswerJson({ text: 'The cat says {"nap": 1}', },).parsed,).toBe(false,);
      },
    },),
    it({
      name: 'COUNTS the trailing characters for the log, and reads the LAST of two whole values, the model\'s '
        + 'revision, as the false-start reader always has',
      fn: async () => {
        expect(parseAnswerJson({ text: '{"best": 2}\nThat is my pick.', },),).toEqual({
          parsed: true,
          value: { best: 2, },
          abandoned: 0,
          trailing: 17,
        },);
        expect(parseAnswerJson({ text: '{} {"best": 3}', },),).toEqual({
          parsed: true,
          value: { best: 3, },
          abandoned: 3,
          trailing: 0,
        },);
      },
    },),
  ],
},);

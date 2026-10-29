/**
 Tests for the row the coverage probe keeps for each candidate passage: where
 it sits, what the roster concluded, and what a failed call leaves.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type {
  CoverageAnswer,
  CoverageCandidate,
} from '@monochromatic-dev/translation-repair-archive';
import {
  answeredRow,
  failedRow,
  whereOf,
} from '../../dist/final/node/index.mjs';

/**
 A whole section the matcher paired with nothing.
 */
const SECTION: CoverageCandidate = {
  scale: 'section',
  sourceIndex: 4,
  sourceText: '## 猫\n\n猫猫睡觉。',
};

/**
 A block a paired section left unmatched.
 */
const BLOCK: CoverageCandidate = {
  scale: 'block',
  pairIndex: 2,
  sourceIndex: 7,
  sourceText: '猫猫吃鱼。',
};

/**
 What a roster concluded about the passage.
 */
const ANSWER: CoverageAnswer = {
  verdict: {
    kind: 'partly-carried',
    anchoredFull: 1,
    anchoredPartial: 2,
    absent: 3,
    unanchored: 4,
    misattributed: 0,
    heard: 10,
    asked: 11,
    evidence: ['the cat naps',],
    unanchoredQuotes: ['a cat napped',],
    misattributedQuotes: [],
  },
  findings: ['one seat was silent',],
};

await describe({
  name: 'coverage-probe-row',
  children: [
    describe({
      name: whereOf.name,
      children: [
        it({
          name: 'NAMES a section by its index in the original',
          fn: async () => {
            expect(whereOf({ candidate: SECTION, },),).toBe('section 4',);
          },
        },),
        it({
          name: 'NAMES a block by its section pair and its index in the original',
          fn: async () => {
            expect(whereOf({ candidate: BLOCK, },),).toBe('pair 2 block 7',);
          },
        },),
      ],
    },),
    describe({
      name: answeredRow.name,
      children: [
        it({
          name: 'KEEPS the verdict counts, the quotes and the findings beside where the passage sits and how long it is',
          fn: async () => {
            expect(answeredRow({
              entryId: 'Mittens',
              candidate: SECTION,
              answer: ANSWER,
            },),).toEqual({
              entryId: 'Mittens',
              scale: 'section',
              where: 'section 4',
              sourceChars: 11,
              kind: 'partly-carried',
              anchoredFull: 1,
              anchoredPartial: 2,
              absent: 3,
              unanchored: 4,
              heard: 10,
              asked: 11,
              unanchoredQuotes: ['a cat napped',],
              evidence: ['the cat naps',],
              findings: ['one seat was silent',],
            },);
          },
        },),
        it({
          name: 'KEEPS the scale and place of a block passage',
          fn: async () => {
            expect(answeredRow({
              entryId: 'Tabby',
              candidate: BLOCK,
              answer: ANSWER,
            },),).toEqual({
              entryId: 'Tabby',
              scale: 'block',
              where: 'pair 2 block 7',
              sourceChars: 5,
              kind: 'partly-carried',
              anchoredFull: 1,
              anchoredPartial: 2,
              absent: 3,
              unanchored: 4,
              heard: 10,
              asked: 11,
              unanchoredQuotes: ['a cat napped',],
              evidence: ['the cat naps',],
              findings: ['one seat was silent',],
            },);
          },
        },),
      ],
    },),
    describe({
      name: failedRow.name,
      children: [
        it({
          name: 'KEEPS a failed attempt with every count zero, the roster size as asked, and a marked error in its own words',
          fn: async () => {
            expect(failedRow({
              entryId: 'Mittens',
              candidate: SECTION,
              asked: 8,
              error: new StatedRefusalError({ says: 'the cat left the page', },),
            },),).toEqual({
              entryId: 'Mittens',
              scale: 'section',
              where: 'section 4',
              sourceChars: 11,
              kind: 'failed',
              anchoredFull: 0,
              anchoredPartial: 0,
              absent: 0,
              unanchored: 0,
              heard: 0,
              asked: 8,
              evidence: [],
              unanchoredQuotes: [],
              findings: ['the cat left the page',],
            },);
          },
        },),
        it({
          name: 'NAMES an unmarked error by its class alone, never its message',
          fn: async () => {
            expect(failedRow({
              entryId: 'Mittens',
              candidate: BLOCK,
              asked: 1,
              error: new RangeError('secret whisker text',),
            },).findings,).toEqual(['refused by RangeError',],);
          },
        },),
      ],
    },),
  ],
},);

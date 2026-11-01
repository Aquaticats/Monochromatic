/**
 Tests for locating a claimed defect in a document.

 WHAT THESE PIN is the split that the first version of this instrument did not
 have: a LOCATOR says which occurrence is meant, a FOCUS says what changed,
 and the second is allowed to be short and repeated as long as it is unique
 inside the first. The cases in this file are the ones that decide whether that split
 actually buys anything, so each names the reading it would have had under a
 single-span rule.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { anchorLocatedSpan, } from '../dist/final/node/index.mjs';

/**
 Original carrying a negator TWICE, which is what makes a bare focus quote
 ambiguous and a located one exact.
 */
const SOURCE_TEXT = '三只猫住在书店的阁楼里。她们不吃罐头，也不喝凉牛奶。';

/**
 Rendering of it, carrying an English word that repeats.
 */
const CANDIDATE_TEXT = 'Three cats live in the bookshop attic. They eat canned food, and they drink cold milk.';

await describe({
  name: anchorLocatedSpan.name,
  children: [
    it({
      name:
        'LOCATES a focus that occurs more than once in the document but only once inside its locator, '
          + 'which is the whole reason the two spans are separate: the negator here appears twice, and a '
          + 'rule demanding a document-unique quote would refuse the exact word the claim is about',
      fn: async () => {
        const anchor = anchorLocatedSpan({
          text: SOURCE_TEXT,
          locator: '她们不吃罐头',
          focus: '不',
          side: 'source',
        },);
        expect(anchor.anchored,).toBe(true,);
        if (!anchor.anchored)
          return;

        expect(anchor.focus
          .text,).toBe('不',);
        expect(anchor.focus
          .start,).toBe(SOURCE_TEXT.indexOf('不吃',),);
        expect(anchor.focus
          .end,).toBe(anchor.focus
            .start + 1,);
        expect(anchor.locator
          .text,).toBe('她们不吃罐头',);
      },
    },),
    it({
      name:
        'ACCEPTS a focus far below any character floor, since a floor never measured whether a quote '
          + 'identifies anything: it measured length, and the padding it forced is what merged two '
          + 'different defects into one',
      fn: async () => {
        const anchor = anchorLocatedSpan({
          text: CANDIDATE_TEXT,
          locator: 'They eat canned food',
          focus: 'eat',
          side: 'candidate',
        },);
        expect(anchor.anchored,).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES a locator occurring twice, because it does not say which occurrence the claim is about',
      fn: async () => {
        // THE NEGATOR ALONE, which this passage carries twice. Offered as a
        // locator it identifies nothing, and it is exactly the quote a voice
        // reaches for when it is asked for the smallest span that changed.
        const anchor = anchorLocatedSpan({
          text: SOURCE_TEXT,
          locator: '不',
          focus: '不',
          side: 'source',
        },);
        expect(anchor.anchored,).toBe(false,);
        if (anchor.anchored)
          return;

        expect(anchor.reason,).toBe('ambiguous-locator (source)',);
      },
    },),
    it({
      name: 'REFUSES a locator no text carries, which is how a quote from some other translation falls',
      fn: async () => {
        const anchor = anchorLocatedSpan({
          text: SOURCE_TEXT,
          locator: '她们不吃鱼罐头',
          focus: '不',
          side: 'source',
        },);
        expect(anchor.anchored,).toBe(false,);
        if (anchor.anchored)
          return;

        expect(anchor.reason,).toBe('unanchored-locator (source)',);
      },
    },),
    it({
      name:
        'REFUSES a focus that sits in the document but OUTSIDE the locator, since a claim whose two '
          + 'spans point at different sentences has not said where it is',
      fn: async () => {
        const anchor = anchorLocatedSpan({
          text: SOURCE_TEXT,
          locator: '她们不吃罐头',
          focus: '凉牛奶',
          side: 'source',
        },);
        expect(anchor.anchored,).toBe(false,);
        if (anchor.anchored)
          return;

        expect(anchor.reason,).toBe('unanchored-focus (source)',);
      },
    },),
    it({
      name: 'REFUSES a focus repeating INSIDE its own locator, which again fails to say which one is meant',
      fn: async () => {
        const anchor = anchorLocatedSpan({
          text: SOURCE_TEXT,
          locator: '她们不吃罐头，也不喝凉牛奶',
          focus: '不',
          side: 'source',
        },);
        expect(anchor.anchored,).toBe(false,);
        if (anchor.anchored)
          return;

        expect(anchor.reason,).toBe('ambiguous-focus (source)',);
      },
    },),
    it({
      name: 'REFUSES an empty span on either field, naming which one was empty',
      fn: async () => {
        expect(
          anchorLocatedSpan({
            text: SOURCE_TEXT,
            locator: '',
            focus: '不',
            side: 'source',
          },),
        ).toEqual({
          anchored: false,
          reason: 'empty-locator (source)',
        },);
        expect(
          anchorLocatedSpan({
            text: SOURCE_TEXT,
            locator: '她们不吃罐头',
            focus: '',
            side: 'source',
          },),
        ).toEqual({
          anchored: false,
          reason: 'empty-focus (source)',
        },);
      },
    },),
    it({
      name:
        'REFUSES A SPAN THAT SHOWS A READER NOTHING as empty, though the text holds it exactly once: a locator '
          + 'of one ideographic space, and a focus of a zero-width space, a Hangul filler or one space inside a '
          + 'locator that holds it once',
      fn: async () => {
        /**
         Original carrying each invisible character once.
         */
        const spaced = '三只猫住在书店的阁楼里。\u{3000}她们不吃罐头\u{200B}，\u{3164}也不喝凉牛奶。';
        expect([
          anchorLocatedSpan({
            text: spaced,
            locator: '\u{3000}',
            focus: '\u{3000}',
            side: 'source',
          },),
          anchorLocatedSpan({
            text: spaced,
            locator: '她们不吃罐头\u{200B}',
            focus: '\u{200B}',
            side: 'source',
          },),
          anchorLocatedSpan({
            text: spaced,
            locator: '\u{3164}也不喝',
            focus: '\u{3164}',
            side: 'source',
          },),
          anchorLocatedSpan({
            text: CANDIDATE_TEXT,
            locator: 'canned food',
            focus: ' ',
            side: 'candidate',
          },),
        ],).toEqual([
          {
            anchored: false,
            reason: 'empty-locator (source)',
          },
          {
            anchored: false,
            reason: 'empty-focus (source)',
          },
          {
            anchored: false,
            reason: 'empty-focus (source)',
          },
          {
            anchored: false,
            reason: 'empty-focus (candidate)',
          },
        ],);
      },
    },),
    it({
      name:
        'ANCHORS ACROSS A SOFT WRAP and returns the DOCUMENT`s own characters rather than the quote`s, '
          + 'so a report never quotes a text back with wording it does not carry',
      fn: async () => {
        /**
         Candidate wrapped mid-sentence, as a stored document is.
         */
        const wrapped = 'Three cats live in the bookshop attic.\nThey eat canned food, and they nap.';

        const anchor = anchorLocatedSpan({
          text: wrapped,
          locator: 'bookshop attic. They eat canned food',
          focus: 'eat canned food',
          side: 'candidate',
        },);
        expect(anchor.anchored,).toBe(true,);
        if (!anchor.anchored)
          return;

        // THE NEWLINE COMES BACK, because evidence is sliced from the stored
        // text at offsets found in the canonical one.
        expect(anchor.locator
          .text,).toBe('bookshop attic.\nThey eat canned food',);
      },
    },),
    it({
      name:
        'ANCHORS A QUOTE TYPED WITH ASCII PUNCTUATION against a document holding the curly form, which '
          + 'is the failure that discarded real evidence before the punctuation fold existed',
      fn: async () => {
        /**
         Candidate holding a curly apostrophe, as the editing guide requires.
         */
        const curly = 'The bookshop’s three cats sleep in the attic.';

        const anchor = anchorLocatedSpan({
          text: curly,
          locator: "The bookshop's three cats",
          focus: "bookshop's",
          side: 'candidate',
        },);
        expect(anchor.anchored,).toBe(true,);
        if (!anchor.anchored)
          return;

        expect(anchor.focus
          .text,).toBe('bookshop’s',);
      },
    },),
  ],
},);

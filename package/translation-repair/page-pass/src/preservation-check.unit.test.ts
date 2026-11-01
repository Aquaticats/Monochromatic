/**
 Tests for the deterministic gate against edits that delete what nobody
 complained about.

 Every threshold this guards was calibrated on 50 real graded repairs rather
 than chosen, and two of the cases in this file are bugs that calibration run caught
 in earlier drafts of the gate. Both would have rejected repairs a human
 graded sound, which for a gate is the expensive direction.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { checkPreservation, } from '../dist/final/node/index.mjs';

await describe({
  name: checkPreservation.name,
  children: [
    it({
      name: 'lets the QUOTED defect disappear, since changing it is the entire '
        + 'point of the edit and a gate that refused would refuse every repair',
      fn: async () => {
        /**
         Edit that removes exactly what the issue complained about.
         */
        const verdict = checkPreservation({
          before: 'Mittens napped on the sill and often shared her opinions loudly.',
          after: 'Mittens napped on the sill.',
          licensedQuotes: ['and often shared her opinions loudly',],
        },);

        expect(verdict.preserved,).toBe(true,);
      },
    },),

    it({
      name: 'REJECTS an edit that drops a name nobody asked it to touch, which '
        + 'is the contributor-credit failure: the issue quoted a punctuation '
        + 'mark, the editor fixed it and deleted a person from the line',
      fn: async () => {
        /**
         Colon fixed, name silently removed.
         */
        const verdict = checkPreservation({
          before: 'Contributor for this entry：Whiskers - the Cat Archive',
          after: 'Contributor for this entry: the Cat Archive',
          licensedQuotes: ['Contributor for this entry：',],
        },);

        expect(verdict.preserved,).toBe(false,);
        expect(verdict.lostDistinctive,).toStrictEqual(['whiskers',],);
      },
    },),

    it({
      name: 'REJECTS an edit that replaces several sentences with one and takes '
        + 'unrelated content with it, which is the shape that deleted a whole '
        + 'clause the source supported',
      fn: async () => {
        /**
         Four sentences collapsed into one.
         */
        const verdict = checkPreservation({
          before: 'Mittens was not merely a napper but a climber. '
            + 'She knew every branch of the garden oak, from root to crown, '
            + 'and often shared her opinions loudly.',
          after: 'Mittens also had considerable talent for climbing.',
          licensedQuotes: ['and often shared her opinions loudly',],
        },);

        expect(verdict.preserved,).toBe(false,);
        expect(verdict.lossFraction,).toBeGreaterThan(0.8,);
      },
    },),

    it({
      name: 'ALLOWS rewording, because a reworded sentence is still there. This '
        + 'gate cannot see that "reminiscing" became "pleading", and pretending '
        + 'otherwise would mean tuning it until it rejected sound repairs too',
      fn: async () => {
        /**
         Same content, different words.
         */
        const verdict = checkPreservation({
          before: 'Her purr was faint, as though she were dozing and dreaming at once.',
          after: 'Her purr came in soft, broken waves, as though she were dozing and hoping at once.',
          licensedQuotes: ['Her purr was faint',],
        },);

        expect(verdict.preserved,).toBe(true,);
      },
    },),

    it({
      name: 'does not mistake a SENTENCE-INITIAL capital for a name. An earlier '
        + 'draft rejected an edit for losing "Yet" and "Moreover", which are '
        + 'ordinary words wearing a capital because of where they sit',
      fn: async () => {
        /**
         Sentence-opening capitals that the edit drops.
         */
        const verdict = checkPreservation({
          before: 'The cat slept. Yet the garden stayed loud. Moreover the birds returned.',
          after: 'The cat slept while the garden stayed loud and the birds returned.',
          licensedQuotes: [],
        },);

        expect(verdict.lostDistinctive,).toStrictEqual([],);
        expect(verdict.preserved,).toBe(true,);
      },
    },),

    it({
      name: 'treats "17th" becoming "17" as FORMATTING rather than a lost '
        + 'number. An earlier draft called any token starting with a digit a '
        + 'number and rejected a repair a human graded sound over exactly this',
      fn: async () => {
        /**
         Ordinal rewritten as a bare numeral.
         */
        const verdict = checkPreservation({
          before: 'On the morning of March 17th, she climbed the tallest fence.',
          after: 'It was the dawn of March 17, at the tallest fence.',
          licensedQuotes: ['she climbed the tallest fence',],
        },);

        expect(verdict.lostDistinctive,).toStrictEqual([],);
        expect(verdict.preserved,).toBe(true,);
      },
    },),

    it({
      name: 'still protects a REAL number, so the ordinal formatting tolerance '
        + 'does not amount to ignoring digits',
      fn: async () => {
        /**
         Catalogue number silently dropped.
         */
        const verdict = checkPreservation({
          before: 'Contributor for this entry：the Cat Archive (catalogue 611)',
          after: 'Contributor for this entry: the Cat Archive',
          licensedQuotes: ['Contributor for this entry：',],
        },);

        expect(verdict.preserved,).toBe(false,);
        expect(verdict.lostDistinctive,).toStrictEqual(['611',],);
      },
    },),

    it({
      name: 'REJECTS a wholesale deletion, where the edit wrote nothing at all',
      fn: async () => {
        /**
         Envelope emptied rather than rewritten.
         */
        const verdict = checkPreservation({
          before: 'Mittens knew every branch of the garden oak, from root to crown.',
          after: '',
          licensedQuotes: [],
        },);

        expect(verdict.preserved,).toBe(false,);
        expect(verdict.lossFraction,).toBe(1,);
      },
    },),

    it({
      name: 'does not apply the BULK rule to a residual too small to measure, '
        + 'since one substituted word out of two reads as total loss and says '
        + 'nothing about whether anything was deleted',
      fn: async () => {
        /**
         Two-token residual, one token changed.
         */
        const verdict = checkPreservation({
          before: 'She purred softly.',
          after: 'She rumbled.',
          licensedQuotes: [],
        },);

        expect(verdict.residualTokens,).toBeLessThan(5,);
        expect(verdict.preserved,).toBe(true,);
      },
    },),

    it({
      name: 'blanks the LONGEST quote first, so a short quote nested inside a '
        + 'longer one cannot leave the rest of the longer one looking '
        + 'unlicensed and trip the gate on its own licence',
      fn: async () => {
        /**
         One quote contained within another.
         */
        const verdict = checkPreservation({
          before: 'She knew every branch of the garden oak, from root to crown.',
          after: 'She knew the garden oak well.',
          licensedQuotes: [
            'every branch',
            'knew every branch of the garden oak, from root to crown',
          ],
        },);

        expect(verdict.preserved,).toBe(true,);
      },
    },),

    it({
      name: 'BLANKS NOTHING for an empty licensed quote, since splitting on an empty quote would take '
        + 'the text apart character by character',
      fn: async () => {
        const verdict = checkPreservation({
          before: 'Mittens napped on the sill and often shared her opinions loudly.',
          after: 'Mittens napped on the sill.',
          licensedQuotes: ['and often shared her opinions loudly', '',],
        },);

        // The three residual tokens are the content words the non-empty quote
        // leaves unlicensed; an empty quote split like any other would blank
        // them too, and the gate would pass with nothing left to measure.
        expect(verdict,).toEqual({
          preserved: true,
          lostDistinctive: [],
          lossFraction: 0,
          residualTokens: 3,
        },);
      },
    },),
  ],
},);

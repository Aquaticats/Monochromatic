/**
 Tests for the sentence a non-translation-blocked document carries.

 WHAT THESE PIN is a claim about the READER rather than about arithmetic. The
 two numbers are sums over the prepared slices, not over the translation, and
 a reader who takes them for the document draws the opposite conclusion from
 the true one: that most of the entry is not a translation, when it may be
 that most of what could be examined was not. Question 7 answer B kept the
 slice denominator and required the wording to say so.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonTranslationDominanceFinding, } from '../dist/final/node/index.mjs';

await describe({
  name: nonTranslationDominanceFinding.name,
  children: [
    it({
      name: 'names the population the ratio was decided over, so a reader cannot take the '
        + 'denominator for the whole translation: both numbers are sums over prepared slices, and '
        + 'a section the aligner refused to pair appears in neither',
      fn: async () => {
        /**
         Finding for a document blocked on most of what it examined.
         */
        const finding = nonTranslationDominanceFinding({
          standingChars: 900,
          totalChars: 1_100,
        },);
        expect(finding.includes('examined slices',),).toBe(true,);
        expect(finding.includes('not the whole translation',),).toBe(true,);
      },
    },),
    it({
      name: 'does NOT call the denominator target characters, which is the wording question 7 '
        + 'answer B removed and the reason this file exists',
      fn: async () => {
        expect(nonTranslationDominanceFinding({
          standingChars: 900,
          totalChars: 1_100,
        },)
          .includes('target chars',),).toBe(false,);
      },
    },),
    it({
      name: 'carries both counts verbatim, since a reader checking a log line against an artifact '
        + 'finding is comparing two renderings of one fact and they have to agree',
      fn: async () => {
        /**
         Finding whose numbers are distinctive enough to spot a swapped pair.
         */
        const finding = nonTranslationDominanceFinding({
          standingChars: 417,
          totalChars: 823,
        },);
        expect(finding.includes('417',),).toBe(true,);
        expect(finding.includes('823',),).toBe(true,);
        expect(finding.indexOf('417',) < finding.indexOf('823',),).toBe(true,);
      },
    },),
    it({
      name: 'ACCEPTS EXTRA FIELDS, because both call sites pass the whole dominance assessment '
        + 'rather than picking two numbers off it, and picking them off by hand at two call sites '
        + 'is how the two wordings drifted apart in the first place',
      fn: async () => {
        expect(nonTranslationDominanceFinding({
          standingChars: 900,
          totalChars: 1_100,
          blocked: true,
        } as Parameters<typeof nonTranslationDominanceFinding>[0],),).toBe(
          nonTranslationDominanceFinding({
            standingChars: 900,
            totalChars: 1_100,
          },),
        );
      },
    },),
  ],
},);

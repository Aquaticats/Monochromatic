/**
 Guards the texts a prober reads: the baseline, the existing translation next
 door and both texts of each region come without the notes the archive's
 translators left each other, while the original's own comments stay, and the
 regions keep their number and identity.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { proberShownTexts, } from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 大家看一下 -->';

/**
 Comment the original itself carries.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

await describe({
  name: proberShownTexts.name,
  children: [
    it({
      name: 'CUTS THE NOTE from the baseline, the window and both texts of every region, in region order',
      fn: async () => {
        /**
         Regions the caller passed.
         */
        const regions = [
          {
            envelopeId: 'envelope/first',
            issueIds: [],
            before: `The cat slept. ${NOTE}`,
            editorAfter: `The cat napped. ${NOTE}`,
          },
          {
            envelopeId: 'envelope/second',
            issueIds: [],
            before: 'The kitten purred.',
            editorAfter: 'The kitten hummed.',
          },
        ];
        const shown = proberShownTexts({
          sourceText: `猫睡了。${ORIGINAL_COMMENT}`,
          baselineText: `The cat slept. ${ORIGINAL_COMMENT} ${NOTE}`,
          neighbouringIncumbentText: `The kitten purred. ${NOTE}`,
          regions,
        },);
        expect(shown.baseline,).toContain(ORIGINAL_COMMENT,);
        expect(shown.baseline,).not.toContain('大家看一下',);
        expect(shown.incumbent,).toBe('The kitten purred.',);
        expect(shown.regions.map(function toId(region,) {
          return region.region.envelopeId;
        },),).toEqual(['envelope/first', 'envelope/second',],);
        expect(shown.regions.map(function toBefore(region,) {
          return region.before;
        },),).toEqual(['The cat slept.', 'The kitten purred.',],);
        expect(shown.regions.map(function toAfter(region,) {
          return region.editorAfter;
        },),).toEqual(['The cat napped.', 'The kitten hummed.',],);
      },
    },),
    it({
      name: 'KEEPS A COMMENT THE ORIGINAL NEXT DOOR CARRIES and shows nothing for a window that is not there',
      fn: async () => {
        const shown = proberShownTexts({
          sourceText: '猫睡了。',
          neighbouringSourceText: `小猫呼噜了。${ORIGINAL_COMMENT}`,
          baselineText: 'The cat slept.',
          regions: [],
        },);
        expect(shown.incumbent,).toBe('',);
        expect(shown.regions,).toEqual([],);
        /**
         The same comment in an existing translation next door is the original's own there.
         */
        const next = proberShownTexts({
          sourceText: '猫睡了。',
          neighbouringSourceText: `小猫呼噜了。${ORIGINAL_COMMENT}`,
          baselineText: 'The cat slept.',
          neighbouringIncumbentText: `The kitten purred. ${ORIGINAL_COMMENT}`,
          regions: [],
        },);
        expect(next.incumbent,).toContain(ORIGINAL_COMMENT,);
      },
    },),
  ],
},);

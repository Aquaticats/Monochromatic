/**
 A title reference the page renders between marks, title brackets or
 quotes: one span is the reference, and several are settled only by the one
 span that carries the heading's rendering. Cat-themed invention throughout;
 no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { unifyTitleReferences, } from '../../dist/final/node/index.mjs';
import {
  pair,
  textsOf,
} from './title-reference.test-fixture.ts';

/**
 Heading the references point at.
 */
const HEADING = pair({
  sliceIndex: 0,
  source: '### 午后猫语',
  target: '',
},);

/**
 Rendering of that heading.
 */
const HEADING_ROW = {
  sliceIndex: 0,
  replacementText: '### Afternoon Cat Murmurs',
} as const;

await describe({
  name: `${unifyTitleReferences.name} over marked spans`,
  children: [
    it({
      name: 'STANDS ASIDE where two quoted spans both open with the heading\'s rendering, since either could be the title',
      fn: async () => {
        /**
         Credit quoting the title and a longer title that opens with it.
         */
        const credit = '—— Yunmao “Afternoon Cat Murmurs”, from “Afternoon Cat Murmurs Live”';
        /**
         Pass over a heading and that credit.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '—— 云猫《午后猫语》，出自《午后猫语现场》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: credit,
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          credit,
        ],);
        expect(unified.findings,).toEqual([
          'title-reference-ambiguous (slice 1: 「午后猫语」 rendered by the heading of slice 0 as "Afternoon Cat Murmurs", '
            + 'but the slice offers more than one span to read)',
        ],);
      },
    },),
    it({
      name: 'KEEPS A QUOTED SPAN\'S TRAILING PERIOD OUTSIDE THE REWRITE, where the page closes its quote after it',
      fn: async () => {
        /**
         Pass over a credit whose quote closes after a period.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '—— 云猫《午后猫语》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: '—— Yunmao \u{201C}Afternoon Cat Talk.\u{201D}',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('—— Yunmao \u{201C}Afternoon Cat Murmurs.\u{201D}',);
        expect(unified.findings,).toEqual([
          'title-reference-unified (slice 1: "Afternoon Cat Talk" to "Afternoon Cat Murmurs"; 「午后猫语」 rendered by the '
            + 'heading of slice 0 as "Afternoon Cat Murmurs")',
        ],);
      },
    },),
  ],
},);

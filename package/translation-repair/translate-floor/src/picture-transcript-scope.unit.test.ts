/**
 Guards class one hundred sixteen (zheermao11, 2026-09-24): the translator
 sheet shows WHAT THE PICTURES HERE SAY for the pictures in and beside a
 passage, and said nothing about what the block is for, so the writers
 rendered a neighbouring picture's transcript into a passage whose ORIGINAL
 is a picture component or a sentence introducing the picture. Every such
 candidate ran past the produced volume bound and was cut (22 of the run's
 translate streams, 31 to 43 on every earlier zheermao run), and on a run
 with the reasoning seats reachable each cut came after 60 to 170 seconds
 of thinking, so the translate lane took 616 seconds against 77. The sheet
 now says the block is context and scopes a transcript to the passages that
 already carry one.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildTranslateMessages, } from '../dist/final/node/index.mjs';

/**
 Clause the translator sheet carries whenever it shows the pictures block,
 read off the rule verbatim so a rewording that drops the scope fails here.
 */
const PICTURE_SCOPE =
  'WHAT THE PICTURES HERE SAY transcribes the pictures in and beside this passage so that words about them read right; '
    + 'it is not part of the ORIGINAL passage. Render a picture\'s words only where the ORIGINAL passage itself writes them out '
    + 'or the EXISTING TRANSLATION of this passage already carries them. Render a picture component (a tag naming picture files) '
    + 'exactly as the ORIGINAL has it, with no transcript added beside it: a transcript neither the passage nor its existing '
    + 'translation carries is an addition, and where the page translates that picture in a block of its own it would stand '
    + 'on the page twice.';

/**
 Original passage: a picture component and nothing else.
 */
const SOURCE = `<PhotoScroll photos={[\n    '\${path}/photos/cat-note.webp',\n]} />`;

/**
 What the readers transcribed from the picture.
 */
const PICTURES = 'PICTURE cat-note.webp\nDear Whiskers, the windowsill is yours this afternoon.';

/**
 Reads the system message of a translator sheet.

 @param pictureContext - pictures block the sheet shows, empty for none

 @returns System message text

 @example
 ```ts
 const system = systemOf({ pictureContext: '', },);
 ```
 */
function systemOf({ pictureContext, }: { readonly pictureContext: string; },): string {
  /**
   First message of the sheet, the system instructions.
   */
  const [first,] = buildTranslateMessages({
    sourceText: SOURCE,
    existingText: SOURCE,
    pictureContext,
  },).messages;
  return first?.content ?? '';
}

await describe({
  name: 'the pictures block is context the translators never render beyond the passage (class one hundred sixteen, zheermao11)',
  children: [
    it({
      name: 'a sheet showing the pictures block carries the scope clause',
      fn: async () => {
        expect(systemOf({ pictureContext: PICTURES, },),).toContain(PICTURE_SCOPE,);
      },
    },),
    it({
      name: 'a sheet showing no pictures carries no pictures clause',
      fn: async () => {
        expect(systemOf({ pictureContext: '', },).includes(PICTURE_SCOPE,),).toBe(false,);
      },
    },),
  ],
},);

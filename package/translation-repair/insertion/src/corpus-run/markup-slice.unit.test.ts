/**
 Tests for the markup-only slice reading.

 This class was named by hand: a slice that is entirely a photo component
 sits near ratio 1.00 whatever the translator did, so it is below baseline for
 a reason unrelated to giving a passage up and pairs with any high neighbour.

 The cases that matter are the two NULLS. A screen that called ordinary prose
 markup would suppress real relocation candidates, which is a worse failure
 than the one it was built to fix, because a suppressed candidate leaves no
 trace to audit.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isMarkupOnly,
  markupFraction,
} from '../../dist/final/node/index.mjs';

/**
 A photo component of the shape the corpus actually carries, which is the
 class this exists for.
 */
const PHOTO_BLOCK = `<PhotoScroll photos={[
    '\${path}/photos/tabby1.webp',
    '\${path}/photos/tabby2.webp',
    '\${path}/photos/tabby3.webp',
]}/>`;

/**
 The same component as four source pages at pin `a41fc607` write it, with
 double-quoted paths. The photo reference reader missed this spelling until
 2026-09-04, and so did this screen.
 */
const DOUBLE_QUOTED_PHOTO_BLOCK = `<PhotoScroll photos={[
    "\${path}/photos/tabby1.webp",
    "\${path}/photos/tabby2.webp",
    "\${path}/photos/tabby3.webp"
]} />`;

/**
 Ordinary prose, which must never read as markup.
 */
const PROSE = `毛毛跳上窗台，看着外面的雨。
她等了很久，直到天黑。
邻居的猫在墙上叫了一声。`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: markupFraction.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IGNORES BLANK LINES on both sides of the fraction, because a slice broken into '
            + 'paragraphs would otherwise read as more structural the more readable it is, which '
            + 'inverts the measurement',
          fn: async () => {
            expect(markupFraction({ sourceText: PROSE, },),).toBe(0,);
            expect(
              markupFraction({ sourceText: `${PROSE}\n\n\n\n${PROSE}`, },),
            ).toBe(0,);
          },
        },),

        it({
          name: 'CALLS AN EMPTY SLICE FULLY STRUCTURAL, since a slice with nothing in it has no '
            + 'prose to give up either and must not be read as a donor',
          fn: async () => {
            expect(markupFraction({ sourceText: '', },),).toBe(1,);
            expect(markupFraction({ sourceText: '\n\n   \n', },),).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: isMarkupOnly.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECOGNISES A PHOTO COMPONENT, which is the class identified by hand as a false '
            + 'donor: the same markup appears verbatim on both sides, so the slice sits near ratio '
            + 'one whatever the translator did',
          fn: async () => {
            expect(isMarkupOnly({ sourceText: PHOTO_BLOCK, },),).toBe(true,);
          },
        },),

        it({
          name: 'RECOGNISES THE DOUBLE-QUOTED SPELLING OF THE SAME COMPONENT, which four source pages '
            + 'write: a screen that knew one quote mark read three of these five lines as prose and '
            + 'left the block open to being named a relocation candidate',
          fn: async () => {
            expect(markupFraction({ sourceText: DOUBLE_QUOTED_PHOTO_BLOCK, },),).toBe(1,);
            expect(isMarkupOnly({ sourceText: DOUBLE_QUOTED_PHOTO_BLOCK, },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES ORDINARY PROSE, which is the null that matters most: calling prose markup '
            + 'would suppress real relocation candidates, and a suppressed candidate leaves nothing '
            + 'to audit',
          fn: async () => {
            expect(isMarkupOnly({ sourceText: PROSE, },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A PHOTO BLOCK THAT CARRIES REAL PROSE WITH IT, since a slice with several '
            + 'sentences beside its markup did have something to give up',
          fn: async () => {
            expect(
              isMarkupOnly({ sourceText: `${PHOTO_BLOCK}\n\n${PROSE}`, },),
            ).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS A PHOTO BLOCK WITH ONE CAPTION, because the threshold is deliberately below '
            + 'one: a block that is nine parts component to one part caption still cannot expand '
            + 'enough to be a donor',
          fn: async () => {
            expect(
              isMarkupOnly({ sourceText: `${PHOTO_BLOCK}\n毛毛在窗台上。`, },),
            ).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

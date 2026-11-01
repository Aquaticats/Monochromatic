/**
 Tests the assembly guard on a page that reads as no document although every
 replacement reads in its own slice.

 THE SHAPE UNDER TEST. The nesting scan is stateful across lines: a fence one
 slice opens and never closes is closed by the fence line of the slice after
 it, so that slice's text, skipped as code on its own, is a paragraph the parser
 nests on the page. Each replacement passes every check of its own slice, and
 the page they make is refused by the plain grammar. The guard reads that as the
 strongest structural regression there is and withdraws the replacement that
 caused it, as it does any other, where it once let the refusal end the document.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  guardFootnoteAssembly,
  introducedStructuralRegressions,
} from '../dist/final/node/index.mjs';

//region Unreadable assembly tests

/**
 Original of the two-slice document: a nap under one heading, a bowl under
 another.
 */
const SOURCE_TEXT = '## 小睡\n\n小猫在窗台上打盹。\n\n## 碗\n\n碗是满的。\n';

/**
 Archive English of the document, the bowl mistranslated.
 */
const TARGET_TEXT = '## Nap\n\nThe kitten dozes on the windowsill.\n\n## Bowl\n\nThe bowl is empty.\n';

/**
 First slice rewritten to open a fence and never close it, which reads as a
 slice of text and code.
 */
const OPENING_REPLACEMENT = '## Nap\n\n```\nThe kitten naps on the windowsill.';

/**
 Second slice rewritten to open a fence and hold 300 open brackets, which reads
 as code on its own and as a paragraph nested 300 deep after the first slice's
 fence.
 */
const CLOSING_REPLACEMENT = `## Bowl\n\n\`\`\`\n${'['.repeat(300,)}The bowl is full.`;

/**
 What the page reads as with both replacements written.
 */
const BOTH_WRITTEN = '## Nap\n\n```\nThe kitten naps on the windowsill.\n\n## Bowl\n\n```\n'
  + `${'['.repeat(300,)}The bowl is full.\n`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: introducedStructuralRegressions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a page no grammar reads as a regression of its own, and an ordinary page as none',
          fn: async () => {
            expect(introducedStructuralRegressions({
              incumbentText: TARGET_TEXT,
              assembledText: BOTH_WRITTEN,
            },),).toEqual(['unreadable-page',],);
            expect(introducedStructuralRegressions({
              incumbentText: TARGET_TEXT,
              assembledText: TARGET_TEXT.replace('empty', 'full',),
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: guardFootnoteAssembly.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WITHDRAWS the first replacement of two whose page reads as no document, and keeps the second, '
            + 'which reads on its own',
          fn: async () => {
            /**
             Slices of the document, the nap first.
             */
            const { slices, } = prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },);
            const [nap, bowl,] = slices;
            if ((nap === undefined) || (bowl === undefined))
              throw new Error('the fixture produced fewer than two slices',);

            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                { sliceIndex: nap.target.sliceIndex, replacementText: OPENING_REPLACEMENT, },
                { sliceIndex: bowl.target.sliceIndex, replacementText: CLOSING_REPLACEMENT, },
              ],
            },);

            expect(guarded.revertedChunkIndices,).toEqual([nap.target.sliceIndex,],);
            expect(guarded.replacements,).toEqual([
              { sliceIndex: bowl.target.sliceIndex, replacementText: CLOSING_REPLACEMENT, },
            ],);
            expect(guarded.assembledText,).toBe(
              `## Nap\n\nThe kitten dozes on the windowsill.\n\n${CLOSING_REPLACEMENT}\n`,
            );
            expect(guarded.findings,).toEqual([
              'assembly-structure-single-withdrawal slice 0: reverting this replacement leaves no introduced '
                + 'structural or footnote defect',
              'assembly-structure-observed unreadable-page (round 1)',
            ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Unreadable assembly tests

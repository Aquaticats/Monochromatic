/**
 Tests for invisible-line masking.

 The fixtures use the shape that OCCURS: a line holding only a byte-order
 mark, with ordinary sentences directly above and below and no blank line
 anywhere near it. An earlier attempt at this fix was written against a
 hypothesis instead, a lone mark surrounded by blank lines, and it passed
 while leaving the corpus case untouched.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maskInvisibleLines, } from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';

/**
 Byte-order mark, the character the corpus actually carries.
 */
const MARK = '\u{FEFF}';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: maskInvisibleLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'blanks a line holding only a byte-order mark, which is what lets '
            + 'the paragraph break return. Such a line is NOT blank to CommonMark, '
            + 'so it welds the paragraphs either side of it into one block: the '
            + 'corpus translation carrying three of them parses to 29 blocks '
            + 'welded and 32 masked, measured through parseDocument at the pin',
          fn: async () => {
            expect(maskInvisibleLines({ text: `Alpha.\n${MARK}\nBeta.\n`, },).masked,)
              .toBe('Alpha.\n \nBeta.\n',);
          },
        },),

        it({
          name: 'blanks the same line on a CRLF page, keeping every carriage return where it was: the '
            + 'return is part of the line ending rather than of what a reader sees, so it neither makes '
            + 'the line visible nor gets blanked, and the text stays exactly as long',
          fn: async () => {
            /**
             The byte-order-mark page, as a CRLF file carries it.
             */
            const text = `Alpha.\r\n${MARK}\r\nBeta.\r\n`;

            /**
             What the mask made of it.
             */
            const { masked, regions, } = maskInvisibleLines({ text, },);
            expect(masked,).toBe('Alpha.\r\n \r\nBeta.\r\n',);
            expect(masked.length,).toBe(text.length,);
            expect(regions.length,).toBe(1,);
            // The region covers the mark alone, not the return after it.
            expect(regions[0]?.endOffset,).toBe((regions[0]?.startOffset ?? 0) + MARK.length,);
          },
        },),
        it({
          name: 'preserves LENGTH exactly, because node text, quotes, hashes and '
            + 'every claim anchor are sliced from the body by absolute offset, so '
            + 'removing the character rather than replacing it would move every '
            + 'anchor after it',
          fn: async () => {
            /**
             Body carrying two marked lines and one ordinary blank line.
             */
            const text = `Alpha.\n${MARK}\nBeta.\n\nGamma.\n${MARK}${MARK}\nDelta.\n`;

            expect(maskInvisibleLines({ text, },).masked.length,).toBe(text.length,);
          },
        },),

        it({
          name: 'leaves an ordinary blank line untouched and keeps a line whose '
            + 'text merely CONTAINS a mark, since only a line that shows nothing '
            + 'yet is not blank does the welding',
          fn: async () => {
            expect(maskInvisibleLines({ text: 'Alpha.\n\nBeta.\n', },).masked,)
              .toBe('Alpha.\n\nBeta.\n',);
            expect(maskInvisibleLines({ text: `Al${MARK}pha.\n`, },).masked,)
              .toBe(`Al${MARK}pha.\n`,);
          },
        },),

        it({
          name: 'restores the block that the mark had welded, measured through the '
            + 'parser rather than the masker, because the masker returning the '
            + 'right string proves nothing about how CommonMark then reads it',
          fn: async () => {
            expect(parseDocument({ text: `Alpha.\n${MARK}\nBeta.\n`, },).nodes.length,)
              .toBe(parseDocument({ text: 'Alpha.\n\nBeta.\n', },).nodes.length,);
          },
        },),

        it({
          name: 'blanks a line of NON-ASCII space, which welds exactly as the mark '
            + 'does. CommonMark counts only U+0020 and U+0009 as blank, so a line '
            + 'holding one of these is a paragraph continuation however much it '
            + 'looks like whitespace',
          fn: async () => {
            for (const space of [
              '\u{00A0}',
              '\u{202F}',
              '\u{3000}',
              '\u{2007}',
            ]) {
              expect(maskInvisibleLines({ text: `Alpha.\n${space}\nBeta.\n`, },).masked,)
                .toBe('Alpha.\n \nBeta.\n',);
            }
          },
        },),

        it({
          name: 'leaves the CONDITIONALLY invisible characters alone, since masking '
            + 'one would be a judgement about rendering rather than the restoration '
            + 'of a lost paragraph break. A soft hyphen renders wherever a line '
            + 'breaks, and the line and paragraph separators carry meaning of their '
            + 'own; none has been observed welding anything',
          fn: async () => {
            for (const character of [
              '\u{00AD}',
              '\u{2028}',
              '\u{2029}',
            ]) {
              expect(maskInvisibleLines({ text: `Alpha.\n${character}\nBeta.\n`, },).masked,)
                .toBe(`Alpha.\n${character}\nBeta.\n`,);
            }
          },
        },),

        it({
          name: 'catches those spaces despite every one of them being ECMAScript '
            + 'whitespace, which is the trap that broke the first draft of this '
            + 'file: any check phrased with trim() calls them empty and passes over '
            + 'the very characters it exists to catch',
          fn: async () => {
            for (const space of [
              '\u{00A0}',
              '\u{202F}',
              '\u{3000}',
            ])
              expect(space.trim(),).toBe('',);

            expect(parseDocument({ text: 'Alpha.\n\u{00A0}\nBeta.\n', },).nodes.length,)
              .toBe(parseDocument({ text: 'Alpha.\n\nBeta.\n', },).nodes.length,);
          },
        },),

        it({
          name: 'blanks a line mixing ordinary spaces with an invisible one, since '
            + 'the ordinary spaces alone would already have been blank and it is '
            + 'the invisible character that keeps the line from ending a paragraph',
          fn: async () => {
            expect(maskInvisibleLines({ text: `Alpha.\n  ${MARK} \nBeta.\n`, },).masked,)
              .toBe('Alpha.\n    \nBeta.\n',);
          },
        },),

        it({
          name: 'reports each blanked line as a region naming its offsets and the '
            + 'code points it carried, so the tolerance is never silent. Both '
            + 'parser defects this pipeline has hit were found by accident rather '
            + 'than from an artifact, and a line that vanishes with nothing '
            + 'recording it is the shape that hides the third one',
          fn: async () => {
            /**
             Two blanked lines with an untouched blank line between them.
             */
            const { regions, } = maskInvisibleLines({
              text: `Alpha.\n${MARK}\nBeta.\n\nGamma.\n\u{00A0}\nDelta.\n`,
            },);

            expect(regions.length,).toBe(2,);
            expect(regions[0]?.codePoints,).toEqual(['U+FEFF',],);
            expect(regions[1]?.codePoints,).toEqual(['U+00A0',],);
          },
        },),

        it({
          name: 'anchors those regions at offsets that slice the ORIGINAL text back '
            + 'out, which is what makes them usable as findings at all: every '
            + 'anchor downstream indexes the body by absolute offset',
          fn: async () => {
            /**
             Body whose blanked line sits at a known place.
             */
            const text = `Alpha.\n${MARK}\nBeta.\n`;

            /**
             Region for that line.
             */
            const { regions, } = maskInvisibleLines({ text, },);

            /**
             First region, present because the mark stands alone on its line.
             */
            const [region,] = regions;

            expect(text.slice(
              region?.startOffset,
              region?.endOffset,
            ),).toBe(MARK,);
          },
        },),

        it({
          name: 'reports NOTHING for a body it left alone, so a run whose documents '
            + 'carry no such line reads as no evidence rather than as unexamined',
          fn: async () => {
            /**
             Ordinary body with a blank line and a mark inside a word.
             */
            const { regions, } = maskInvisibleLines({
              text: `Alpha.\n\nBe${MARK}ta.\n`,
            },);

            expect(regions.length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: 'parseDocument invisible-line findings',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'surfaces the masking as a parse finding carrying the code point, '
            + 'so a welded document is diagnosable from its artifact instead of '
            + 'from someone happening to count blocks',
          fn: async () => {
            /**
             Document whose paragraphs a mark had welded.
             */
            const { parseFindings, } = parseDocument({
              text: `Alpha.\n${MARK}\nBeta.\n`,
            },);

            expect(parseFindings.length,).toBe(1,);
            expect(parseFindings[0]?.kind,).toBe('invisible-line-masked',);
            expect(parseFindings[0]?.detail,).toContain('U+FEFF',);
          },
        },),

        it({
          name: 'keeps findings in SOURCE order across all three kinds, which the '
            + 'type promises and the assembly order does not give: a downgrade '
            + 'finding starts at the body offset, so it would sort ahead of a '
            + 'comment appearing much later in the text',
          fn: async () => {
            /**
             Document carrying a masked line and a comment after it.
             */
            const { parseFindings, } = parseDocument({
              text: `Alpha.\n${MARK}\nBeta.\n\n<!-- note -->\n\nGamma.\n`,
            },);

            /**
             Offsets in the order they are reported.
             */
            const offsets = parseFindings.map(function toOffset(finding,) {
              return finding.startOffset;
            },);

            expect(offsets,).toEqual(offsets.toSorted(function ascending(left, right,) {
              return left - right;
            },),);
            expect(parseFindings.length,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);

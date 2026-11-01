/**
 Tests for the guard that stops an edit deleting a quoted passage.

 WHY THIS QUESTION AND NOT THE OTHER ONE. The harm is that a lane writing from
 the source alone deletes English the source cannot account for, and the
 obvious response is to work out which passages those are. That is not cheaply
 decidable: translation changes bytes by construction, so no exact match
 separates an unpaired passage from an ordinary translated one. Deletion, on
 the other hand, is decidable from the two texts alone, and measured over both
 settled pools it caught four real losses and nothing else in sixty-nine
 natural rows.

 THE COUNT IS THE PARSER'S (ledger B42). It was the number of
 blank-line-separated chunks opening with `>`, which refused a replacement
 keeping every quote where a quote opened on the line after a paragraph's,
 and saw no quote inside a container tag. The cases for both read the counts
 the old splitter got wrong.

 THE CARRIAGE-RETURN CASE IS NOT DEFENSIVENESS. One of the 184 markdown files
 in the pinned corpus uses CRLF throughout. A splitter looking for two bytes
 `\n\n` finds no boundary there and reads the whole document as one block.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MdxParseError,
  parsedTopLevelBlocks,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  countQuotedPassages,
  dropsQuotedPassage,
  isQuotedPassages,
  quoteLossRefusalFinding,
} from '../dist/final/node/index.mjs';

/**
 A text neither grammar reads: plain markdown accepts any text, and refuses
 only when reading exhausts the parser, as 16,000 nested quotation markers
 do (`translate-validate.unit.test.ts`).
 */
const UNREADABLE_TEXT = `${'>'.repeat(16_000,)} cat`;

/**
 Counts one text's quoted passages the way the guard counts the archive's.

 @param text - text to count

 @returns Its count, or the unreadable mark

 @example
 ```ts
 const count = archiveCount({ text: '> She said so.', },);
 ```
 */
function archiveCount({ text, }: { readonly text: string; },): number | 'unreadable' {
  return countQuotedPassages({
    incumbentText: text,
    shippedText: '',
  },).archive;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parsedTopLevelBlocks.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EACH TOP-LEVEL BLOCK with its bytes, kind and place, no block for a run of blank '
            + 'lines and none for an empty text, which is what the target-only run splices by',
          fn: async () => {
            expect(parsedTopLevelBlocks({ text: 'One.\n\n> Two.\n\n\n\nThree.', },),).toStrictEqual([
              { text: 'One.', kind: 'paragraph', startOffset: 0, endOffset: 4, },
              { text: '> Two.', kind: 'blockquote', startOffset: 6, endOffset: 12, },
              { text: 'Three.', kind: 'paragraph', startOffset: 16, endOffset: 22, },
            ],);
            expect(parsedTopLevelBlocks({ text: '', },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'READS A CARRIAGE-RETURN FILE block by block at offsets into the text as written, which '
            + 'one corpus file is: a splitter looking for two bytes of newline read it as ONE block',
          fn: async () => {
            expect(parsedTopLevelBlocks({ text: 'One.\r\n\r\nTwo.\r\n\r\nThree.', },),).toStrictEqual([
              { text: 'One.', kind: 'paragraph', startOffset: 0, endOffset: 4, },
              { text: 'Two.', kind: 'paragraph', startOffset: 8, endOffset: 12, },
              { text: 'Three.', kind: 'paragraph', startOffset: 16, endOffset: 22, },
            ],);
          },
        },),

        it({
          name: 'THROWS THE GRAMMAR\'S REFUSAL for a text the slice grammar cannot read, which the '
            + 'target-only run reads as nothing to protect (ledger B68)',
          fn: async () => {
            /**
             What reading a text opening on an unclosed component throws.
             */
            const refusal = caught(function readUnclosed(): void {
              parsedTopLevelBlocks({ text: '<Cat unclosed\n\nThe cat naps.', },);
            },);

            expect(refusal,).toBeInstanceOf(MdxParseError,);
            expect((refusal as Error).message,).toBe(
              'MDX body refused to parse at 3:13 (micromark-extension-mdx-jsx/unexpected-character); corpus '
                + 'documents compile as MDX upstream, so failure signals corruption or an unsupported construct.',
            );
          },
        },),
      ],
    },),

    describe({
      name: countQuotedPassages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS BLOCKS RATHER THAN LINES, so a lane may reflow a quotation freely and only losing a whole '
            + 'quoted passage counts against it',
          fn: async () => {
            expect(archiveCount({ text: '> The cat sat.\n> Then she left.\n> Then she came back.', },),).toBe(1,);
            expect(archiveCount({ text: '> The cat sat. Then she left. Then she came back.', },),).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS EACH SEPARATE QUOTATION, since a passage carrying two is a passage a reader would miss '
            + 'either of',
          fn: async () => {
            expect(archiveCount({ text: '> Mittens spoke.\n\nShe paused.\n\n> Then Whiskers did.', },),).toBe(2,);
          },
        },),

        it({
          name: 'COUNTS BOTH SIDES, each from its own text, so neither count stands in for the other',
          fn: async () => {
            expect(countQuotedPassages({
              incumbentText: '> Mittens spoke.\n\n> Then Whiskers did.',
              shippedText: 'Mittens spoke.\n\n> Then Whiskers did.',
            },),).toEqual({
              archive: 2,
              replacement: 1,
            },);
          },
        },),

        it({
          name: 'READS A CARRIAGE-RETURN FILE, where the blank-line splitter the guard once counted with found one '
            + 'block in the whole document',
          fn: async () => {
            expect(archiveCount({ text: '> She said so.\r\n\r\nAnd then left.\r\n\r\n> So she did.', },),).toBe(2,);
          },
        },),

        it({
          name: 'COUNTS A QUOTE OPENING ON THE LINE AFTER A PARAGRAPH\'S, which the parser and the floor read as a '
            + 'blockquote and the blank-line splitter counted as none, refusing a replacement that kept it (ledger B42, '
            + 'hulicaijia24 slice 2)',
          fn: async () => {
            expect(archiveCount({ text: 'Her notes read:\n> Name: Mittens.', },),).toBe(1,);
            expect(archiveCount({ text: '- The cat left a note.\n  > Feed me.', },),).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS A QUOTE INSIDE A CONTAINER TAG AND ONE INSIDE ANOTHER QUOTE, which the floor\'s top-level '
            + 'blocks do not reach, so the guard is the only check that sees them go (ledger B42)',
          fn: async () => {
            expect(archiveCount({ text: '<details>\n\n> Name: Mittens.\n\n</details>', },),).toBe(1,);
            expect(archiveCount({ text: '> She wrote:\n>\n> > Feed the cat.', },),).toBe(2,);
          },
        },),

        it({
          name: 'READS A TEXT THE STRICT GRAMMAR REFUSES UNDER PLAIN MARKDOWN, as the floor reads such a page, rather '
            + 'than calling it unreadable',
          fn: async () => {
            expect(archiveCount({ text: '> The bowl holds {treats.', },),).toBe(1,);
          },
        },),

        it({
          name: 'MARKS A TEXT NEITHER GRAMMAR READS rather than counting it as none, since none is what a guard '
            + 'passes a replacement on',
          fn: async () => {
            expect(archiveCount({ text: UNREADABLE_TEXT, },),).toBe('unreadable',);
          },
        },),
      ],
    },),

    describe({
      name: dropsQuotedPassage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A REPLACEMENT THAT DELETES A QUOTED PASSAGE, which is the whole point: the lost blocks in '
            + 'the measured cases were transcripts of images, written by a person, with no original to regenerate '
            + 'them from',
          fn: async () => {
            expect(dropsQuotedPassage({
              quotedPassages: countQuotedPassages({
                incumbentText: 'Her notes read:\n\n> Name: Mittens.\n> Likes: sunbeams.',
                shippedText: 'Her notes read as follows.',
              },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES A REPLACEMENT THAT DELETES A QUOTE FROM INSIDE A CONTAINER TAG, which the floor passes '
            + 'because the container itself is still there (ledger B42)',
          fn: async () => {
            expect(dropsQuotedPassage({
              quotedPassages: countQuotedPassages({
                incumbentText: '<details>\n\n> Name: Mittens.\n\n</details>',
                shippedText: '<details>\n\nHer name was Mittens.\n\n</details>',
              },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'ACCEPTS A REPLACEMENT THAT KEEPS THE QUOTATION, however much it rewords the prose around it and '
            + 'whether or not a blank line opens it, since rewording is what this lane is for (ledger B42)',
          fn: async () => {
            expect(dropsQuotedPassage({
              quotedPassages: countQuotedPassages({
                incumbentText: 'Her notes read:\n\n> Name: Mittens.',
                shippedText: 'What she wrote was this:\n> Name is Mittens.',
              },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS A REPLACEMENT THAT ADDS ONE, because a passage the archive never quoted and the original '
            + 'does is exactly the gap this lane exists to close',
          fn: async () => {
            expect(dropsQuotedPassage({
              quotedPassages: countQuotedPassages({
                incumbentText: 'She left a note.',
                shippedText: 'She left a note:\n\n> Feed the cat.',
              },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS PROSE FOR PROSE, so an edit that removes an invented paragraph is untouched by this guard: '
            + 'one lane correctly cut a paragraph of translator invention with nine accepted findings against it, '
            + 'and that cut was not a quotation',
          fn: async () => {
            expect(dropsQuotedPassage({
              quotedPassages: countQuotedPassages({
                incumbentText: 'She naps.\n\nA florid paragraph nobody wrote.',
                shippedText: 'She naps.',
              },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES WHERE EITHER SIDE COULD NOT BE READ, since a check that could not run has not shown the '
            + 'quotes survive',
          fn: async () => {
            expect(dropsQuotedPassage({
              quotedPassages: {
                archive: 'unreadable',
                replacement: 3,
              },
            },),).toBe(true,);
            expect(dropsQuotedPassage({
              quotedPassages: {
                archive: 0,
                replacement: 'unreadable',
              },
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: isQuotedPassages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS whole counts of none or more and the unreadable mark on both sides, which is every pair the '
            + 'guard stores',
          fn: async () => {
            expect(isQuotedPassages({
              archive: 0,
              replacement: 'unreadable',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a missing side, a negative count, a fractional count, another mark and a value that is '
            + 'no pair at all, none of which the guard writes',
          fn: async () => {
            expect([
              { archive: 1, },
              {
                archive: -1,
                replacement: 0,
              },
              {
                archive: 1.5,
                replacement: 0,
              },
              {
                archive: 1,
                replacement: 'unread',
              },
              '2 to 1',
            ].map(function accepted(value,): boolean {
              return isQuotedPassages(value,);
            },),).toEqual([false, false, false, false, false,],);
          },
        },),
      ],
    },),

    describe({
      name: quoteLossRefusalFinding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE SLICE AND BOTH COUNTS, each in the number it takes, so a corpus-wide reading can separate '
            + 'this refusal from the alignment one rather than counting them together',
          fn: async () => {
            expect(quoteLossRefusalFinding({
              sliceIndex: 4,
              quotedPassages: {
                archive: 2,
                replacement: 1,
              },
            },),).toBe('translate-refused-quote-loss (slice 4: archive carries 2 quoted passages, replacement carries '
              + '1 quoted passage)',);
            expect(quoteLossRefusalFinding({
              sliceIndex: 4,
              quotedPassages: {
                archive: 1,
                replacement: 0,
              },
            },),).toBe('translate-refused-quote-loss (slice 4: archive carries 1 quoted passage, replacement carries '
              + '0 quoted passages)',);
          },
        },),

        it({
          name: 'SAYS WHICH SIDE NO GRAMMAR COULD READ rather than printing a count nobody measured',
          fn: async () => {
            expect(quoteLossRefusalFinding({
              sliceIndex: 4,
              quotedPassages: {
                archive: 2,
                replacement: 'unreadable',
              },
            },),).toBe('translate-refused-quote-loss (slice 4: archive carries 2 quoted passages, replacement could '
              + 'not be read to count its quoted passages)',);
            expect(quoteLossRefusalFinding({
              sliceIndex: 4,
              quotedPassages: {
                archive: 'unreadable',
                replacement: 1,
              },
            },),).toBe('translate-refused-quote-loss (slice 4: archive could not be read to count its quoted passages, '
              + 'replacement carries 1 quoted passage)',);
          },
        },),
      ],
    },),
  ],
},);

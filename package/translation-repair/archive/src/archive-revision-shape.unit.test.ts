/**
 Tests for the shape floor on an archive block's revision (class
 seventy-seven), read directly rather than through the review stage.

 LEDGER T8, EIGHTEENTH BATCH: no case gave the floor an archive block the
 slice grammar refuses, nor a revision it refuses. Writing them found the
 refusals saying "the block is paragraph", a list of shapes where a count
 belongs; they now count each side's blocks as the translate lanes' block
 floor does ("1 block (paragraph)").

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { revisionShapeFindings, } from '../dist/final/node/index.mjs';

/**
 A box component opened and never closed, which the slice grammar refuses.
 */
const UNCLOSED = '<CatBox an unclosed box\n\nThe cat naps.';

/**
 Archive block of one paragraph.
 */
const PARAGRAPH = 'The cat naps by the stove.';

/**
 Reviewer named in every refusal.
 */
const REVIEWER = 'hf:cat/Cat-A';

await describe({
  name: revisionShapeFindings.name,
  children: [
    it({
      name: 'PASSES any revision of an archive block the slice grammar cannot read, which has no shape to hold one to',
      fn: async () => {
        expect(revisionShapeFindings({
          modelId: REVIEWER,
          blockText: UNCLOSED,
          replacementText: PARAGRAPH,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'WITHHOLDS a revision the slice grammar cannot read, carrying the parser\'s account and counting the '
        + 'block\'s blocks',
      fn: async () => {
        expect(revisionShapeFindings({
          modelId: REVIEWER,
          blockText: PARAGRAPH,
          replacementText: UNCLOSED,
        },),).toEqual([
          `archive-revision-refused (${REVIEWER}): the revision does not parse (MdxParseError: MDX body refused to `
            + 'parse at 3:13 (micromark-extension-mdx-jsx/unexpected-character); corpus documents compile as MDX '
            + 'upstream, so failure signals corruption or an unsupported construct.); the block is 1 block '
            + '(paragraph)',
        ],);
      },
    },),
    it({
      name: 'WITHHOLDS a revision of another shape, counting each side\'s blocks, and passes a removal and a '
        + 'revision of the same shape',
      fn: async () => {
        expect(revisionShapeFindings({
          modelId: REVIEWER,
          blockText: PARAGRAPH,
          replacementText: 'The cat naps.\n\nShe dreams of fish.',
        },),).toEqual([
          `archive-revision-refused (${REVIEWER}): the block is 1 block (paragraph) and the revision is 2 blocks `
            + '(paragraph, paragraph); a revision keeps the block\'s own shape',
        ],);
        expect([
          revisionShapeFindings({
            modelId: REVIEWER,
            blockText: PARAGRAPH,
            replacementText: '',
          },),
          revisionShapeFindings({
            modelId: REVIEWER,
            blockText: PARAGRAPH,
            replacementText: 'The cat dozes by the stove.',
          },),
        ],).toEqual([
          [],
          [],
        ],);
      },
    },),
    it({
      name: 'WITHHOLDS a paragraph written for an archive block that holds only a comment, naming the block as '
        + '"nothing"',
      fn: async () => {
        expect(revisionShapeFindings({
          modelId: REVIEWER,
          blockText: '<!-- the cat left no words here -->',
          replacementText: PARAGRAPH,
        },),).toEqual([
          `archive-revision-refused (${REVIEWER}): the block is nothing and the revision is 1 block (paragraph); `
            + 'a revision keeps the block\'s own shape',
        ],);
      },
    },),
    it({
      name: 'WITHHOLDS a revision that keeps the block\'s own shape but drops a quote nested inside a container '
        + 'tag, which the shape check cannot see, and PASSES one that keeps the nested quote (ledger B110)',
      fn: async () => {
        /**
         Block of one container tag, one quote nested inside it.
         */
        const block = '<CatBox>\n\n> Feed me at noon.\n\n</CatBox>';

        // A revision of the same container with the quote made prose:
        // `sameShape` sees one `mdxJsxFlowElement` on each side.
        expect(revisionShapeFindings({
          modelId: REVIEWER,
          blockText: block,
          replacementText: '<CatBox>\n\nShe was asked to be fed at noon.\n\n</CatBox>',
        },),).toEqual([
          `archive-revision-refused (${REVIEWER}): the block carries 1 quoted passage and the revision carries 0; a `
            + 'revision keeps every quoted passage the block carries, including one nested inside a container tag, '
            + 'a list, a footnote, or another blockquote',
        ],);

        // Control: the same container tag, the nested quote kept rather
        // than dropped, which must still pass.
        expect(revisionShapeFindings({
          modelId: REVIEWER,
          blockText: block,
          replacementText: '<CatBox>\n\n> She was asked to be fed at noon.\n\n</CatBox>',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'THROWS on a revision that shows a reader nothing and is not empty, which the review stage reads as the '
        + 'empty removal before any revision reaches this floor, rather than holding it to a shape a second way: '
        + 'a zero-width space, a Hangul filler, spaces',
      fn: async () => {
        /**
         Revisions that show a reader nothing.
         */
        const blank = [
          '\u{200B}',
          '\u{3164}',
          '   ',
        ];
        expect(blank.map(function refusalOf(replacementText,): string {
          return String(caught(function check(): unknown {
            return revisionShapeFindings({
              modelId: REVIEWER,
              blockText: PARAGRAPH,
              replacementText,
            },);
          },),);
        },),).toEqual(blank.map(function messageFor(): string {
          return `Error: unreachable: ${REVIEWER}'s revision shows a reader nothing and is not empty, though the `
            + 'review stage reads such a revision as the empty removal before the slate is built';
        },),);
      },
    },),
  ],
},);

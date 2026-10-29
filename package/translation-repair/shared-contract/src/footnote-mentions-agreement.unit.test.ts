/**
 Tests that the footnote mention scan reads the same GFM references as the
 footnote graph wherever a fragment's own text decides the answer (ledger
 B212). Both readers run on every text on the current build, so the test
 fails when either parser or either reader moves, and no expectation is
 stored. A text where the scan counts more than the graph reads is accepted
 only for a shape named here with its reason, since a wrongly counted marker
 costs a refused good candidate and a wrongly dropped one defeats the guards
 that read mentions. The texts are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  maskHtmlComments,
  MdxParseError,
  parseMarkdownBody,
  parseMdxBody,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  EDGE_TEXTS,
  generatedTexts,
  precederTexts,
  readBoth,
  type Reading,
  standingOf,
} from './footnote-mentions-agreement.test-fixture.ts';

/**
 Whether a node of the parse is one the scan counts a marker inside though the
 graph reads none: a code block, or a raw HTML node.

 @param node - mdast node under the walk

 @returns Reason a fragment cannot settle the marker, empty for any other node
 */
function blockReasonOf({ node, }: { readonly node: { readonly type: string; }; },): string {
  if (node.type === 'code')
    return 'a code block, which a fragment may open or close for the page';
  if (node.type === 'html')
    return 'a raw HTML node, which a fragment may open or close for the page';
  return '';
}

/**
 Parses a text with its comments masked as the page's parse reads it, under
 the strict MDX grammar and then under plain markdown, the two grammars a
 page may be read under.

 @param text - text to parse

 @returns Strict grammar's tree where it accepts the text, then the plain tree
 */
function treesOf({ text, }: { readonly text: string; },): readonly { readonly type: string; readonly children?: readonly { readonly type: string; }[]; }[] {
  /**
   The text as the parse reads it.
   */
  const { masked, } = maskHtmlComments({ text, },);
  /**
   Plain markdown's tree, which accepts every text.
   */
  const plain = parseMarkdownBody({ body: masked, },);
  try {
    return [parseMdxBody({ body: masked, },), plain,];
  }
  catch (refusal) {
    // The strict grammar refuses what it cannot read; the page is then read as plain markdown.
    expect(refusal,).toBeInstanceOf(MdxParseError,);
    return [plain,];
  }
}

/**
 The reasons the scan may count a marker the graph does not read in this text,
 read off the parse of the text under either grammar.

 @param text - text read

 @returns Reasons the text holds, each once, empty when it holds none
 */
function countingReasonsOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Nodes still to walk.
   */
  const work = [...treesOf({ text, },),];
  /**
   Reasons found, each once.
   */
  const reasons = new Set<string>();
  for (let node = work.pop(); node !== undefined; node = work.pop()) {
    /**
     Reason this node gives.
     */
    const reason = blockReasonOf({ node, },);
    if (reason !== '')
      reasons.add(reason,);
    work.push(...(node.children ?? []),);
  }
  return [...reasons,];
}

/**
 Readings that break the agreement: the scan lacks a reference the graph
 reads, or counts one the graph does not for a shape no name excuses.

 @param texts - texts to read with both readers

 @returns Breaking readings, each with its text and both answers
 */
function breaking({ texts, }: { readonly texts: readonly string[]; },): readonly (Reading & { readonly standing: string; })[] {
  return texts
    .map(function read(text,): Reading {
      return readBoth({ text, },);
    },)
    .flatMap(function breakingOnes(reading,): (Reading & { readonly standing: string; })[] {
      /**
       How the scan stands to the graph here.
       */
      const standing = standingOf({ reading, },);
      if (standing === 'agree')
        return [];
      if ((standing === 'counting') && (countingReasonsOf({ text: reading.text, },).length > 0))
        return [];
      return [{
        ...reading,
        standing,
      },];
    },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'footnote mention scan against the footnote graph',
      concurrency: 1,
      children: [
        it({
          name: 'AGREES WITH THE GRAPH on every preceder and URL body of the lead\'s table',
          fn: async () => {
            expect(breaking({ texts: precederTexts(), },),).toEqual([],);
          },
        },),
        it({
          name: 'AGREES WITH THE GRAPH on each edge text of a URL, a destination, an angle autolink, a label and a code span',
          fn: async () => {
            expect(breaking({ texts: EDGE_TEXTS, },),).toEqual([],);
          },
        },),
        it({
          name: 'AGREES WITH THE GRAPH on three thousand seeded URL-shaped texts with marker shapes in and beside them',
          fn: async () => {
            expect(breaking({
              texts: generatedTexts({
                seed: 20_261_005,
                count: 3_000,
                maxPieces: 6,
              },),
            },),).toEqual([],);
          },
        },),
        it({
          name: 'COUNTS A MARKER BESIDE A URL AND NONE INSIDE IT',
          fn: async () => {
            expect([
              readBoth({ text: 'A cat https://cat.example/[^9]x naps [^3].', },),
              readBoth({ text: 'A cat https://cat.example/ naps [^9] [^3].', },),
            ].map(function standings(reading,): readonly string[] {
              return reading.scan;
            },),).toEqual([['3',], ['3', '9',],],);
          },
        },),
        it({
          name: 'COUNTS MORE THAN THE GRAPH READS, and names why, in a code block and in a raw HTML node',
          fn: async () => {
            /**
             Texts with a marker shape the scan counts and the graph does not read.
             */
            const texts = ['```\n[^9]\n```\nA cat [^3].', 'A <span title="[^9]">cat</span> [^3].',];
            expect(texts.map(function standings(text,): readonly unknown[] {
              return [standingOf({ reading: readBoth({ text, },), },), countingReasonsOf({ text, },),];
            },),).toEqual([
              ['counting', ['a code block, which a fragment may open or close for the page',],],
              ['counting', ['a raw HTML node, which a fragment may open or close for the page',],],
            ],);
          },
        },),
      ],
    },),
  ],
},);

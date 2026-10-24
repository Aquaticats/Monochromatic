/**
 Tests for how a block that belongs to a footnote definition is read: as the body of the footnote is, with the blocks
 the footnote reaches into taken off the nodes still to read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  footnoteDefinitionRangesOf,
  parseMarkdownMathBody,
} from '@monochromatic-dev/translation-repair-document-structure';

import {
  footnoteRead,
  rangeOf,
  spansOf,
  topLevelPending,
} from '../../dist/final/node/index.mjs';

/**
 The state of a container whose last block is the first line of a footnote definition.
 */
const OPEN = {
  open: true,
  endLine: 1,
  endsInLiteral: false,
};

/**
 Reads the first block of a text as part of a footnote definition, as the spans reader does at the top.

 @param source - text

 @param continuesFootnote - whether the block is indented code under a footnote definition

 @returns What the reading answers, and how many nodes are left to read

 @example
 ```ts
 const { read, left, } = readFirst({ source: '[^1]: {\n', continuesFootnote: false, },);
 ```
 */
function readFirst({ source, continuesFootnote, }: { readonly source: string; readonly continuesFootnote: boolean; },) {
  /**
   The root of the parse.
   */
  const root = parseMarkdownMathBody({ body: source, },);
  const [first, ...rest] = root.children;
  if (first === undefined)
    throw new Error('the text holds no block',);
  /**
   The nodes still to read, the next one on top.
   */
  const pending = topLevelPending({ root, },).slice(
    0,
    rest.length,
  );
  return {
    read: footnoteRead({
      source,
      readable: source,
      readSpans: spansOf,
      footnotes: footnoteDefinitionRangesOf({ body: source, },),
      pending,
      node: first,
      range: rangeOf({ node: first, },),
      isRootChild: true,
      continuesFootnote,
      state: OPEN,
    },),
    left: pending.length,
  };
}

await describe({
  name: footnoteRead.name,
  children: [
    it({
      name: 'ANSWERS THAT A BLOCK IS NO PART OF A FOOTNOTE for a paragraph that opens none and holds none, and leaves '
        + 'the nodes alone',
      fn: async () => {
        expect(readFirst({
          source: '猫 {a}\n\n[^1]: 注\n',
          continuesFootnote: false,
        },),).toEqual({
          read: { kind: 'none', },
          left: 1,
        },);
      },
    },),

    it({
      name: 'READS A PARAGRAPH THAT OPENS A FOOTNOTE AS THE FOOTNOTE\'S BODY and takes the blocks the footnote reaches '
        + 'into off the nodes still to read, and closes the top',
      fn: async () => {
        /**
         A footnote whose paragraph holds a line of equals signs and a definition line.
         */
        const source = '[^1]: x {\ny\n===\n[r]: u{b}\n';
        const { read, left, } = readFirst({
          source,
          continuesFootnote: false,
        },);
        expect(read.kind,).toBe('read',);
        expect(left,).toBe(0,);
        if (read.kind !== 'read')
          throw new Error('the block is part of a footnote',);
        expect(read.state.open,).toBe(false,);
        expect(read.spans.text.some(function holdsBrace({
          start,
          end,
        },): boolean {
          return (start <= source.indexOf('{b}',)) && (source.indexOf('{b}',) < end);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'READS A PARAGRAPH IN WHICH A FOOTNOTE STARTS ON A LATER LINE AS TWO TEXTS, the lines before the '
        + 'definition and the footnote',
      fn: async () => {
        /**
         A paragraph whose code span would run across the definition's line.
         */
        const source = 'a `b\n[^1]: c {d}\ne` f\n';
        const { read, } = readFirst({
          source,
          continuesFootnote: false,
        },);
        if (read.kind !== 'read')
          throw new Error('the block is part of a footnote',);
        expect(read.spans.text.some(function holdsBrace({
          start,
          end,
        },): boolean {
          return (start <= source.indexOf('{d}',)) && (source.indexOf('{d}',) < end);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'READS A BLOCK TOLD TO CONTINUE A FOOTNOTE WHOLE, indented code the plain reading finds under it, and '
        + 'leaves the state given',
      fn: async () => {
        /**
         Indented code, as the plain reading reads the lines under a footnote.
         */
        const source = '    {a}\n';
        const { read, } = readFirst({
          source,
          continuesFootnote: true,
        },);
        if (read.kind !== 'read')
          throw new Error('the block is part of a footnote',);
        expect(read.state,).toEqual(OPEN,);
      },
    },),
  ],
},);

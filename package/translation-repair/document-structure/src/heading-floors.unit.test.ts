/**
 Tests for where the lines of a setext heading start when the parser started the heading at the definitions before it.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  headingFloorOf,
  parseMarkdownBody,
  parseMdxBody,
} from '../dist/final/node/index.mjs';

/**
 The first heading among the blocks at the top of a tree.

 @param root - tree to look in

 @returns Heading

 @throws {@link Error} when the tree holds no heading at the top

 @example
 ```ts
 const heading = firstHeading({ root: parseMdxBody({ body: '猫\n---\n', },), },);
 ```
 */
function firstHeading({ root, }: { readonly root: ReturnType<typeof parseMdxBody>; },) {
  /**
   Heading found.
   */
  const heading = root.children.find(function isHeading({ type, },): boolean {
    return type === 'heading';
  },);
  if (heading === undefined)
    throw new Error('the tree holds no heading',);
  return heading;
}

await describe({
  name: headingFloorOf.name,
  children: [
    it({
      name: 'GIVES THE OFFSET AFTER THE LAST DEFINITION THE PARSER STARTED A SETEXT HEADING AT, in both grammars',
      fn: async () => {
        /**
         Two definitions, then a heading of two lines.
         */
        const body = '[a]: u\n[b]: v\n猫 [ x\n睡\n---\n';
        for (const root of [parseMdxBody({ body, },), parseMarkdownBody({ body, },),])
          expect(headingFloorOf({ heading: firstHeading({ root, },), },),).toBe('[a]: u\n[b]: v'.length,);
      },
    },),

    it({
      name: 'GIVES ZERO FOR A HEADING NO DEFINITION OVERLAPS, an ATX heading after a definition and a setext heading '
        + 'alone',
      fn: async () => {
        expect(headingFloorOf({ heading: firstHeading({ root: parseMdxBody({ body: '[a]: u\n# 猫\n', },), },), },),)
          .toBe(0,);
        expect(headingFloorOf({ heading: firstHeading({ root: parseMdxBody({ body: '猫\n---\n', },), },), },),).toBe(0,);
        expect(headingFloorOf({ heading: firstHeading({ root: parseMdxBody({ body: '[a]: u\n\n猫\n---\n', },), },), },),)
          .toBe(0,);
      },
    },),

    it({
      name: 'GIVES THE FLOOR OF A HEADING IN A BLOCK QUOTE, which the walk reaches below the top',
      fn: async () => {
        /**
         A block quote holding a definition and a heading.
         */
        const root = parseMdxBody({ body: '> [a]: u\n> 猫\n> ---\n', },);
        const [quote,] = root.children;
        const heading = (quote !== undefined) && ('children' in quote)
          ? quote.children.find(function isHeading({ type, },): boolean {
            return type === 'heading';
          },)
          : undefined;
        if (heading === undefined)
          throw new Error('the block quote holds no heading',);
        expect(headingFloorOf({ heading, },),).toBe('> [a]: u'.length,);
      },
    },),
  ],
},);

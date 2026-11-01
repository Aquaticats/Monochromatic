/**
 Tests for the runs of text the autolink-literal transform rebuilds without a position, and the raw region each is
 read in. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseMdxBody,
  type TreeNode,
  unpositionedRuns,
} from '../dist/final/node/index.mjs';

/**
 The heading of a text, which the text holds once at the top.

 @param body - text, parsed as the strict grammar parses a page

 @returns Text's tree, and its heading

 @throws {@link Error} when the text holds no heading at the top

 @example
 ```ts
 const { root, heading, } = headingOf({ body: '猫\n---\n', },);
 ```
 */
function headingOf({ body, }: { readonly body: string; },): { readonly root: TreeNode; readonly heading: TreeNode; } {
  /**
   Tree of the text.
   */
  const root = parseMdxBody({ body, },);
  /**
   First heading among the blocks at the top.
   */
  const heading = root.children.find(function isHeading({ type, },): boolean {
    return type === 'heading';
  },);
  if (heading === undefined)
    throw new Error('the text holds no heading',);
  return {
    root,
    heading,
  };
}

await describe({
  name: unpositionedRuns.name,
  children: [
    it({
      name: 'PLACES THE RUN OF A SETEXT HEADING THAT FOLLOWS A LINK REFERENCE DEFINITION IN THE HEADING\'S OWN LINES, '
        + 'since the parser starts the heading at the definition and the definition holds a bracket of its own',
      fn: async () => {
        /**
         A definition, then a heading whose text is an open bracket and a bare address.
         */
        const body = '[a]: u\n[ https://example.net/a\n---\n';
        const { heading, } = headingOf({ body, },);
        if (!('children' in heading))
          throw new Error('the heading holds no children',);
        const [run,] = unpositionedRuns({
          parent: heading,
          text: body,
        },);
        expect(run?.regionStart,).toBeGreaterThanOrEqual(body.indexOf('\n',),);
        expect(body.slice(run?.regionStart,).startsWith('\n[ https',),).toBe(true,);
      },
    },),

    it({
      name: 'PLACES THE RUN OF A HEADING AFTER TWO DEFINITIONS, and of a heading after a definition in a block quote',
      fn: async () => {
        /**
         Two definitions before the heading.
         */
        const two = '[a]: u\n[b]: v\n[ https://example.net/a\n---\n';
        const { heading, } = headingOf({ body: two, },);
        if (!('children' in heading))
          throw new Error('the heading holds no children',);
        const [first,] = unpositionedRuns({
          parent: heading,
          text: two,
        },);
        expect(two.slice(first?.regionStart,).startsWith('\n[ https',),).toBe(true,);
        /**
         The same in a block quote.
         */
        const quoted = '> [a]: u\n> [ https://example.net/a\n> ---\n';
        const [quote,] = parseMdxBody({ body: quoted, },).children;
        const inner = (quote !== undefined) && ('children' in quote)
          ? quote.children.find(function isHeading({ type, },): boolean {
            return type === 'heading';
          },)
          : undefined;
        if ((inner === undefined) || (!('children' in inner)))
          throw new Error('the block quote holds no heading',);
        const [second,] = unpositionedRuns({
          parent: inner,
          text: quoted,
        },);
        expect(quoted.slice(second?.regionStart,).includes('[a]',),).toBe(false,);
      },
    },),

    it({
      name: 'KEEPS THE START OF A HEADING THAT NO DEFINITION PRECEDES, a paragraph after a definition and an ATX '
        + 'heading',
      fn: async () => {
        /**
         A heading alone.
         */
        const alone = '[ https://example.net/a\n---\n';
        const { heading, } = headingOf({ body: alone, },);
        if (!('children' in heading))
          throw new Error('the heading holds no children',);
        expect(unpositionedRuns({
          parent: heading,
          text: alone,
        },)[0]?.regionStart,).toBe(0,);
        /**
         An ATX heading after a definition.
         */
        const atx = '[a]: u\n# [ https://example.net/a\n';
        const { heading: atxHeading, } = headingOf({ body: atx, },);
        if (!('children' in atxHeading))
          throw new Error('the heading holds no children',);
        expect(atx.slice(unpositionedRuns({
          parent: atxHeading,
          text: atx,
        },)[0]?.regionStart,).startsWith('# [ https',),).toBe(true,);
      },
    },),
  ],
},);

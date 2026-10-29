/**
 Tests the reading of where a parse opens a footnote definition as a top-level
 block, the role the footnote graph gives a label: a definition in a
 container tag counts, one in a list item, in a block quote or inside another
 definition does not. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  definitionStartsOf,
  parseMarkdownBody,
  parseMdxBody,
} from '../dist/final/node/index.mjs';

await describe({
  name: definitionStartsOf.name,
  children: [
    it({
      name: 'READS EACH DEFINITION OPENING A TOP-LEVEL BLOCK at the offset of its bracket, under either grammar',
      fn: async () => {
        /**
         Text with two definitions, the second indented three spaces.
         */
        const body = 'The cat naps[^1] and[^2].\n\n[^1]: A note.\n\n   [^2]: Another.\n';
        expect([...definitionStartsOf({ root: parseMarkdownBody({ body, },), },),],).toEqual([27, 45,],);
        expect([...definitionStartsOf({ root: parseMdxBody({ body, },), },),],).toEqual([27, 45,],);
      },
    },),
    it({
      name: 'READS A DEFINITION IN A CONTAINER TAG as a top-level block, the container dissolved as the graph dissolves it',
      fn: async () => {
        /**
         Text with a definition between a container's tags.
         */
        const body = '<Box>\n\n[^1]: A note.\n\n</Box>\n\nThe cat naps[^1].\n';
        expect([...definitionStartsOf({ root: parseMdxBody({ body, },), },),],).toEqual([7,],);
      },
    },),
    it({
      name: 'READS NO DEFINITION nested in a list item, a block quote or another definition, which the graph does '
        + 'not read as one',
      fn: async () => {
        expect([...definitionStartsOf({ root: parseMdxBody({ body: '- a cat\n\n  [^1]: A note.\n', }), },),],).toEqual([],);
        expect([...definitionStartsOf({ root: parseMdxBody({ body: '> [^1]: A note.\n', }), },),],).toEqual([],);
        expect([...definitionStartsOf({ root: parseMdxBody({ body: '[^2]: First.\n\n    [^1]: A note.\n', }), },),],).toEqual([
          0,
        ],);
      },
    },),
  ],
},);

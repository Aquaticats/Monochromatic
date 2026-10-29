/**
 Tests for the store of where the lines of a setext heading start when the parser started the heading earlier.
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
  noteHeadingFloor,
  parseMarkdownBody,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'heading floor store',
  children: [
    describe({
      name: noteHeadingFloor.name,
      children: [
        it({
          name: 'KEEPS THE FLOOR BY THE HEADING NODE, and the floor of a node never noted is zero',
          fn: async () => {
            /**
             Two trees, so two heading nodes that are not the same object.
             */
            const [heading,] = parseMarkdownBody({ body: '猫\n---\n', },).children;
            const [other,] = parseMarkdownBody({ body: '虎\n---\n', },).children;
            if ((heading === undefined) || (other === undefined))
              throw new Error('the texts hold no heading',);
            expect(headingFloorOf({ heading, },),).toBe(0,);
            noteHeadingFloor({
              heading,
              floor: 6,
            },);
            expect(headingFloorOf({ heading, },),).toBe(6,);
            expect(headingFloorOf({ heading: other, },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);

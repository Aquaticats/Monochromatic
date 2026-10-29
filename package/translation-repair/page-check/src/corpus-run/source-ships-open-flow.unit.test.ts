/**
 Tests for the closing line of a fence or display math that a text leaves open at its end.

 The strict grammar ends such a flow where the page ends, so the opening line of the next run closes it and the text of
 that run after the line is outside it. Written as literal text, an original is closed at its end.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  closedWith,
  closerOfWritten,
  NO_MARK,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'source ships open flow',
  children: [
    describe({
      name: closerOfWritten.name,
      children: [
        it({
          name: 'NAMES THE MARK OF A FENCE OR DISPLAY MATH THE TEXT ENDS INSIDE, as long as the mark that opened it',
          fn: async () => {
            for (
              const [text, closer,] of [
                ['```\n{a}\n', '```',],
                ['~~~~\n{a}', '~~~~',],
                ['  ```js\n{a}\n', '```',],
                ['$$\n{x\n', '$$',],
                ['$$$\nx', '$$$',],
                ['```\n', '```',],
                ['x\n\n```\n    ```\n', '```',],
                ['```\n{a}\n    ```\n', '```',],
              ] as const
            ) {
              expect(closerOfWritten({ text, },),).toBe(closer,);
            }
          },
        },),

        it({
          name: 'NAMES NO MARK FOR A TEXT THAT CLOSES ITS FLOW, ENDS IN A CONTAINER, OR HOLDS NO FLOW AT ITS END',
          fn: async () => {
            for (
              const text of [
                '```\n{a}\n```\n',
                '````\n{a}\n`````  \n',
                '$$\nx\n$$',
                '```\n{a}\n```\n\n{b}\n',
                '- ```\n  {a}\n',
                '> $$\n> x\n',
                '[^1]: x\n    ```\n    {a}\n',
                '    ```\n',
                '她说 {\n',
                '',
              ] as const
            ) {
              expect(closerOfWritten({ text, },),).toBe(NO_MARK,);
            }
          },
        },),

        it({
          name: 'NAMES NO MARK FOR A TEXT NESTED PAST THE BOUND OF THE PLAIN READING, which reads it as holding no '
            + 'block',
          fn: async () => {
            expect(closerOfWritten({ text: `${'> '.repeat(300,)}\`\`\`\n`, },),).toBe(NO_MARK,);
          },
        },),
      ],
    },),

    describe({
      name: closedWith.name,
      children: [
        it({
          name: 'PUTS THE MARK ON A LINE OF ITS OWN AFTER THE TEXT, in the line ending the text uses',
          fn: async () => {
            expect(closedWith({ text: '```\nx', closer: '```', },),).toBe('```\nx\n```',);
            expect(closedWith({ text: '```\nx\n', closer: '```', },),).toBe('```\nx\n```',);
            expect(closedWith({ text: '```\r\nx', closer: '```', },),).toBe('```\r\nx\r\n```',);
            expect(closedWith({ text: '```\r\nx\r\n', closer: '```', },),).toBe('```\r\nx\r\n```',);
          },
        },),

        it({
          name: 'RETURNS THE TEXT ITSELF FOR NO MARK',
          fn: async () => {
            expect(closedWith({ text: '她说\n', closer: NO_MARK, },),).toBe('她说\n',);
          },
        },),
      ],
    },),
  ],
},);

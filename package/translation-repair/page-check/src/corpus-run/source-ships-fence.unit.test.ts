/**
 Tests for the fence marks only the strict grammar reads: a mark indented four spaces or more, which MDX opens a code
 block at and the plain reading keeps as text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  holdsIndentedFenceMark,
  indentedFenceMarkEnd,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the fence marks only the strict grammar reads',
  children: [
    describe({
      name: indentedFenceMarkEnd.name,
      children: [
        it({
          name: 'ENDS A MARK OF THREE OR MORE BACKTICKS OR TILDES AFTER FOUR SPACES, a tab, or a block quote mark and '
            + 'four spaces',
          fn: async () => {
            expect(indentedFenceMarkEnd({
              source: 'a\n    ```\n',
              at: 6,
            },),).toBe(9,);
            expect(indentedFenceMarkEnd({
              source: 'a\n    ~~~~~\n',
              at: 6,
            },),).toBe(11,);
            expect(indentedFenceMarkEnd({
              source: 'a\n\t```js\n',
              at: 3,
            },),).toBe(6,);
            expect(indentedFenceMarkEnd({
              source: '>     ```\n',
              at: 6,
            },),).toBe(9,);
          },
        },),

        it({
          name: 'ENDS NOTHING FOR A MARK THAT BOTH GRAMMARS READ ALIKE, a mark after three spaces, after a block quote '
            + 'mark and one space, or at the start of the line',
          fn: async () => {
            expect(indentedFenceMarkEnd({
              source: '   ```\n',
              at: 3,
            },),).toBe(3,);
            expect(indentedFenceMarkEnd({
              source: '> ```\n',
              at: 2,
            },),).toBe(2,);
            expect(indentedFenceMarkEnd({
              source: '```\n',
              at: 0,
            },),).toBe(0,);
          },
        },),

        it({
          name: 'ENDS NOTHING FOR WHAT IS NO FENCE MARK, two characters, a mark after other text, another character, '
            + 'the end of the text, or a mark inside a run of the character',
          fn: async () => {
            expect(indentedFenceMarkEnd({
              source: '    ``x\n',
              at: 4,
            },),).toBe(4,);
            expect(indentedFenceMarkEnd({
              source: '    a ```\n',
              at: 6,
            },),).toBe(6,);
            expect(indentedFenceMarkEnd({
              source: '    ***\n',
              at: 4,
            },),).toBe(4,);
            expect(indentedFenceMarkEnd({
              source: '    ',
              at: 4,
            },),).toBe(4,);
            expect(indentedFenceMarkEnd({
              source: '    ```\n',
              at: 5,
            },),).toBe(5,);
          },
        },),
      ],
    },),

    describe({
      name: holdsIndentedFenceMark.name,
      children: [
        it({
          name: 'FINDS A MARK ON ANY LINE OF A TEXT, in a block quote and after a tab among them',
          fn: async () => {
            expect(holdsIndentedFenceMark({ text: '    ```\n', },),).toBe(true,);
            expect(holdsIndentedFenceMark({ text: '她说\n\n    ~~~\n{\n', },),).toBe(true,);
            expect(holdsIndentedFenceMark({ text: '> a\n>     ```\n', },),).toBe(true,);
            expect(holdsIndentedFenceMark({ text: 'a\n\t```\n', },),).toBe(true,);
          },
        },),

        it({
          name: 'FINDS NOTHING IN A TEXT WITH ONLY MARKS BOTH GRAMMARS READ ALIKE, or none at all',
          fn: async () => {
            expect(holdsIndentedFenceMark({ text: '', },),).toBe(false,);
            expect(holdsIndentedFenceMark({ text: '```\n{\n```\n', },),).toBe(false,);
            expect(holdsIndentedFenceMark({ text: '   ```\n', },),).toBe(false,);
            expect(holdsIndentedFenceMark({ text: '她说 ``` 然后\n    走了。\n', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

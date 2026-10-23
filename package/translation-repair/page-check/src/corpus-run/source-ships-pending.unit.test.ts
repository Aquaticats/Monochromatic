/**
 Tests for the state the spans reader keeps for each container it walks.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import { footnoteStateOf, } from '../../dist/final/node/index.mjs';

/**
 The state of each container met, as the function under test takes it.
 */
type Footnotes = Parameters<typeof footnoteStateOf>[0]['footnotes'];

await describe({
  name: footnoteStateOf.name,
  children: [
    it({
      name: 'GIVES A CONTAINER NOTHING READ IN IT THE STATE OF NO FOOTNOTE DEFINITION',
      fn: async () => {
        /**
         A text of a block quote and a paragraph.
         */
        const root = parseMarkdownMathBody({ body: '> a\n\nb\n', },);
        expect(footnoteStateOf({
          footnotes: new Map(),
          parent: root,
        },),).toEqual({
          open: false,
          endLine: 0,
          endsInLiteral: false,
        },);
      },
    },),

    it({
      name: 'GIVES THE SAME STATE FOR A CONTAINER EVERY TIME, so what the walk records in it is there the next time',
      fn: async () => {
        /**
         A text of a block quote and a paragraph.
         */
        const root = parseMarkdownMathBody({ body: '> a\n\nb\n', },);
        /**
         State of each container met.
         */
        const footnotes: Footnotes = new Map();
        /**
         State the first look made.
         */
        const first = footnoteStateOf({
          footnotes,
          parent: root,
        },);
        first.open = true;
        expect(footnoteStateOf({
          footnotes,
          parent: root,
        },),).toBe(first,);
        expect(footnoteStateOf({
          footnotes,
          parent: root,
        },).open,).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS THE STATE OF TWO CONTAINERS APART, the root and a block quote in it',
      fn: async () => {
        /**
         A text of a block quote and a paragraph.
         */
        const root = parseMarkdownMathBody({ body: '> a\n\nb\n', },);
        /**
         The block quote.
         */
        const [quote,] = root.children;
        if (quote === undefined)
          throw new Error('the text holds no block',);
        /**
         State of each container met.
         */
        const footnotes: Footnotes = new Map();
        footnoteStateOf({
          footnotes,
          parent: root,
        },).open = true;
        expect(footnoteStateOf({
          footnotes,
          parent: quote,
        },).open,).toBe(false,);
      },
    },),
  ],
},);

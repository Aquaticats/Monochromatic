/**
 Tests the plugin that reads raw HTML as text, the way the page has it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { withoutRawHtml, } from '../dist/final/node/index.mjs';

/**
 The types of the blocks a text has, read with raw HTML on and then off.

 @param body - text to parse

 @returns Block types with raw HTML on, and with it off

 @example
 ```ts
 const { on, off, } = blockTypesOf({ body: 'cat\n<div>\npurr\n', },);
 ```
 */
function blockTypesOf({ body, }: { readonly body: string; },): {
  readonly on: readonly string[];
  readonly off: readonly string[];
} {
  return {
    on: unified()
      .use(remarkParse,)
      .parse(body,).children
      .map(function typeOf({ type, },): string {
        return type;
      },),
    off: unified()
      .use(remarkParse,)
      .use(withoutRawHtml(),)
      .parse(body,).children
      .map(function typeOf({ type, },): string {
        return type;
      },),
  };
}

await describe({
  name: withoutRawHtml.name,
  children: [
    it({
      name: 'READS A LINE OF RAW HTML AS A LINE OF THE PARAGRAPH ABOVE IT, where the plain reading ends the paragraph',
      fn: async () => {
        expect(blockTypesOf({ body: 'cat\n<div>\npurr\n', },),).toEqual({
          on: [
            'paragraph',
            'html',
          ],
          off: ['paragraph',],
        },);
      },
    },),
    it({
      name: 'READS A TAG INSIDE A LINE AS TEXT, with no inline HTML node',
      fn: async () => {
        /**
         The one paragraph of the text, read with raw HTML off.
         */
        const [paragraph,] = unified()
          .use(remarkParse,)
          .use(withoutRawHtml(),)
          .parse('cat <b>purr</b>\n',).children;
        expect(paragraph?.type,).toBe('paragraph',);
        expect(JSON.stringify(paragraph,),).not.toContain('"html"',);
      },
    },),
    it({
      name: 'LEAVES THE OTHER BLOCKS OF THE PLAIN READING AS THEY ARE, such as a code block and a heading',
      fn: async () => {
        expect(blockTypesOf({ body: '# cat\n\n```js\npurr\n```\n', },),).toEqual({
          on: [
            'heading',
            'code',
          ],
          off: [
            'heading',
            'code',
          ],
        },);
      },
    },),
  ],
},);

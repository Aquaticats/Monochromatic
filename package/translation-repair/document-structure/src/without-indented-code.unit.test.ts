/**
 Tests the plugin that reads no indented code, the way the page has it.

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

import { withoutIndentedCode, } from '../dist/final/node/index.mjs';

/**
 The types of the blocks a text has, read with indented code on and then off.

 @param body - text to parse

 @returns Block types with indented code on, and with it off

 @example
 ```ts
 const { on, off, } = blockTypesOf({ body: 'cat\n\n    purr\n', },);
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
      .use(withoutIndentedCode(),)
      .parse(body,).children
      .map(function typeOf({ type, },): string {
        return type;
      },),
  };
}

await describe({
  name: withoutIndentedCode.name,
  children: [
    it({
      name: 'READS A LINE INDENTED FOUR SPACES AFTER A BLANK LINE AS A PARAGRAPH, where the plain reading makes code of it',
      fn: async () => {
        expect(blockTypesOf({ body: 'cat\n\n    purr\n', },),).toEqual({
          on: [
            'paragraph',
            'code',
          ],
          off: [
            'paragraph',
            'paragraph',
          ],
        },);
      },
    },),
    it({
      name: 'READS A LIST AND A FENCE THE SAME, whatever the indent construct',
      fn: async () => {
        expect(blockTypesOf({ body: '- cat\n\n```js\npurr\n```\n', },),).toEqual({
          on: [
            'list',
            'code',
          ],
          off: [
            'list',
            'code',
          ],
        },);
      },
    },),
  ],
},);

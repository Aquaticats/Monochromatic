/**
 Tests for the reading of raw html the nesting scan keeps, so that a line
 which looks like a fence inside html is never taken for one: the kinds of
 block that end at a blank line, the kinds that end at a marker, and what a
 blank line does to each.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  htmlBlockAfter,
  htmlBlockAfterBlank,
} from '../dist/final/node/index.mjs';

//region Nesting html block tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: htmlBlockAfter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a line that does not begin with an angle bracket as opening no block',
          fn: async () => {
            expect(htmlBlockAfter({ before: 'none', rest: 'The cat naps.', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'none', rest: '', },),).toBe('none',);
          },
        },),
        it({
          name: 'READS any other line that begins with an angle bracket as a block that ends at a blank line',
          fn: async () => {
            expect(htmlBlockAfter({ before: 'none', rest: '<div>', },),).toBe('blank',);
            expect(htmlBlockAfter({ before: 'none', rest: '<https://cat.example/>', },),).toBe('blank',);
            expect(htmlBlockAfter({ before: 'none', rest: '<preview>', },),).toBe('blank',);
            expect(htmlBlockAfter({ before: 'blank', rest: 'The cat naps.', },),).toBe('blank',);
          },
        },),
        it({
          name: 'READS a line that is an angle bracket and an exclamation mark alone as a block that ends at a blank '
            + 'line, since no letter follows to open a declaration',
          fn: async () => {
            expect(htmlBlockAfter({ before: 'none', rest: '<!', },),).toBe('blank',);
          },
        },),
        it({
          name: 'READS the five kinds that end at a marker as open until the line that carries it',
          fn: async () => {
            expect(htmlBlockAfter({ before: 'none', rest: '<PRE class="cat">', },),).toBe('pre',);
            expect(htmlBlockAfter({ before: 'none', rest: '<script>', },),).toBe('pre',);
            expect(htmlBlockAfter({ before: 'none', rest: '<style', },),).toBe('pre',);
            expect(htmlBlockAfter({ before: 'none', rest: '<textarea>', },),).toBe('pre',);
            expect(htmlBlockAfter({ before: 'none', rest: '<!-- note', },),).toBe('comment',);
            expect(htmlBlockAfter({ before: 'none', rest: '<?cat', },),).toBe('instruction',);
            expect(htmlBlockAfter({ before: 'none', rest: '<![CDATA[', },),).toBe('cdata',);
            expect(htmlBlockAfter({ before: 'none', rest: '<!DOCTYPE cat', },),).toBe('declaration',);
          },
        },),
        it({
          name: 'LEAVES each of those kinds on a line that carries its end marker, and keeps it on any other',
          fn: async () => {
            expect(htmlBlockAfter({ before: 'comment', rest: 'still a note', },),).toBe('comment',);
            expect(htmlBlockAfter({ before: 'comment', rest: 'end of note -->', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'instruction', rest: 'cat ?>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'cdata', rest: 'cat ]]>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'declaration', rest: 'cat>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'pre', rest: 'cat', },),).toBe('pre',);
            expect(htmlBlockAfter({ before: 'pre', rest: '</Pre>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'pre', rest: '</textarea> after', },),).toBe('none',);
          },
        },),
        it({
          name: 'LEAVES a kind on its own opening line only when its marker comes after the opener',
          fn: async () => {
            expect(htmlBlockAfter({ before: 'none', rest: '<!-- note -->', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'none', rest: '<!-->', },),).toBe('comment',);
            expect(htmlBlockAfter({ before: 'none', rest: '<?cat?>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'none', rest: '<![CDATA[cat]]>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'none', rest: '<!DOCTYPE cat>', },),).toBe('none',);
            expect(htmlBlockAfter({ before: 'none', rest: '<pre>cat</pre>', },),).toBe('none',);
          },
        },),
      ],
    },),

    describe({
      name: htmlBlockAfterBlank.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ENDS a block that ends at a blank line and keeps every block that ends at a marker',
          fn: async () => {
            expect(htmlBlockAfterBlank({ before: 'blank', },),).toBe('none',);
            expect(htmlBlockAfterBlank({ before: 'none', },),).toBe('none',);
            expect(htmlBlockAfterBlank({ before: 'comment', },),).toBe('comment',);
            expect(htmlBlockAfterBlank({ before: 'pre', },),).toBe('pre',);
          },
        },),
      ],
    },),
  ],
},);

//endregion Nesting html block tests

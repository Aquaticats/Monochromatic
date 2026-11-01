/**
 Tests the handle-edge match the link-name floor and the page-name note share
 (ledger B23): which characters continue a handle, a needle standing whole at
 an offset, and a text carrying one somewhere. Cat-themed invention
 throughout.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carriesHandleToken,
  isHandleCharacter,
  standsAsHandle,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isHandleCharacter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES ASCII LETTERS, DIGITS, THE HYPHEN AND THE UNDERSCORE, and nothing else',
          fn: async () => {
            for (const character of ['a', 'Z', '7', '-', '_',]) {
              expect(isHandleCharacter({ character, },),).toBe(true,);
            }
            for (const character of ['', ' ', '@', '.', '\'', '\u{732B}', '\u{00E9}',]) {
              expect(isHandleCharacter({ character, },),).toBe(false,);
            }
          },
        },),
      ],
    },),

    describe({
      name: standsAsHandle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A NEEDLE A HANDLE RUNS ON FROM, at either end',
          fn: async () => {
            expect(standsAsHandle({ text: 'Tomcat', at: 0, needle: 'Tom', },),).toBe(false,);
            expect(standsAsHandle({ text: 'BigTom', at: 3, needle: 'Tom', },),).toBe(false,);
            expect(standsAsHandle({ text: 'Tom_Cat', at: 0, needle: 'Tom', },),).toBe(false,);
            expect(standsAsHandle({ text: 'mi-Tom', at: 3, needle: 'Tom', },),).toBe(false,);
          },
        },),
        it({
          name: 'TAKES A NEEDLE WITH NO HANDLE CHARACTER AT AN EDGE, and one whose edge needs none',
          fn: async () => {
            expect(standsAsHandle({ text: 'Tom\'s nap', at: 0, needle: 'Tom', },),).toBe(true,);
            expect(standsAsHandle({ text: 'a@Tom', at: 2, needle: 'Tom', },),).toBe(true,);
            expect(standsAsHandle({ text: 'x@mi-mi-42 naps', at: 1, needle: '@mi-mi-42', },),).toBe(true,);
            expect(standsAsHandle({ text: 'Tom\u{732B}\u{732B}', at: 3, needle: '\u{732B}\u{732B}', },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: carriesHandleToken.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A WHOLE TOKEN AT ANY OCCURRENCE, and none inside a longer handle',
          fn: async () => {
            expect(carriesHandleToken({ text: '@mi-mi-420', needle: '@mi-mi-42', },),).toBe(false,);
            expect(carriesHandleToken({ text: '@mi-mi-420 and @mi-mi-42', needle: '@mi-mi-42', },),).toBe(true,);
            expect(carriesHandleToken({ text: 'Tomcat', needle: 'Tom', },),).toBe(false,);
          },
        },),
        it({
          name: 'CARRIES NO EMPTY NEEDLE rather than one at every offset',
          fn: async () => {
            expect(carriesHandleToken({ text: 'Tom', needle: '', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

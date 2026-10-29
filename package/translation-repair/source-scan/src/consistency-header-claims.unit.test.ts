/**
 Guards the two sentences of the shown-text headers that said something the code does not do (n160 finding Y6).

 `consistency-shown-edge.ts` said the first or last character of the quoted words "is a Latin letter or digit" while the code
 reads any character of a script that sets its words apart by spaces. `consistency-unread-units.ts` said the inline-code floor
 refuses a word of code beside a word of text, which the floor does not do: it refuses only a place inside the code; the word
 test now reads the content of inline code as shown, which is what refuses it.

 @module
 */

import { readFileSync, } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sourcePathOf, } from './family-source.test-fixture.ts';

/**
 The text of a source file of the family.

 @param name - package directory and file name inside its `src`

 @returns File's text

 @example
 ```ts
 const header = sourceOf({ name: 'consistency/consistency-shown-edge.ts', },);
 ```
 */
function sourceOf({ name, }: { readonly name: string; },): string {
  return readFileSync(
    sourcePathOf({ path: name, },),
    'utf8',
  );
}

await describe({
  name: 'headers of the shown-text word test',
  concurrency: 1,
  children: [
    it({
      name: 'NO LONGER SAYS an edge of the quoted words is a Latin letter or digit, which the code reads as any spaced script',
      fn: async () => {
        /**
         The header and code of the shown-edge test.
         */
        const source = sourceOf({ name: 'consistency/consistency-shown-edge.ts', },);
        expect(source.includes('is a Latin letter or digit',),).toBe(false,);
        expect(source,).toContain('word character of a script that sets its words apart by spaces',);
      },
    },),

    it({
      name: 'NO LONGER SAYS the inline-code floor refuses a word of code beside a word of text',
      fn: async () => {
        /**
         The header and code of the unread-units reader.
         */
        const source = sourceOf({ name: 'consistency/consistency-unread-units.ts', },);
        expect(source.includes('which the inline-code floor refuses by its own rule',),).toBe(false,);
        expect(source,).toContain('the content of inline code is shown as the code word it is',);
      },
    },),
  ],
},);

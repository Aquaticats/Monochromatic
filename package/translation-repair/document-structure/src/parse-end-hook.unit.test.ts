/**
 Tests for the hook the grammar calls last with the tree it is about to return: what it does with nothing set, with a
 function set under the registry symbol, and with anything else standing there. The cases of
 `parse-end-seam.unit.test.ts` read it through the two grammar functions.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  callParseEndHook,
  PARSE_END_HOOK_KEY,
} from '../dist/final/node/index.mjs';

/**
 The kind of tree the hook is given.
 */
type Tree = Parameters<typeof callParseEndHook>[0]['root'];

/**
 A tree of one paragraph, enough for the hook to be given something.
 */
const TREE: Tree = {
  type: 'root',
  children: [
    {
      type: 'paragraph',
      children: [
        {
          type: 'text',
          value: 'A tabby naps.',
        },
      ],
    },
  ],
};

/**
 Sets what stands under the key until the returned value is disposed of, and puts back what stood there.

 @param value - what to put under the key

 @returns A disposable that removes it

 @example
 ```ts
 using _set = standing({ value: function keep(): void {}, },);
 ```
 */
function standing({ value, }: { readonly value: unknown; },): Disposable {
  Reflect.set(
    Symbol,
    PARSE_END_HOOK_KEY,
    value,
  );
  return {
    [Symbol.dispose]: function removeIt(): void {
      Reflect.deleteProperty(
        Symbol,
        PARSE_END_HOOK_KEY,
      );
    },
  };
}

await describe({
  name: callParseEndHook.name,
  children: [
    it({
      name: 'DOES NOTHING WHEN NO HOOK STANDS UNDER THE KEY: the call returns and the tree is as it was',
      fn: async () => {
        expect(callParseEndHook({ root: TREE, },),).toBe(undefined,);
        expect(TREE.children.length,).toBe(1,);
      },
    },),

    it({
      name: 'CALLS A FUNCTION THAT STANDS UNDER THE KEY ONCE, WITH THE TREE IT WAS GIVEN, in one object',
      fn: async () => {
        /**
         What the hook was called with.
         */
        const given: unknown[] = [];
        using _standing = standing({
          value: function keep(arguments_: unknown,): void {
            given.push(arguments_,);
          },
        },);
        callParseEndHook({ root: TREE, },);
        expect(given.length,).toBe(1,);
        expect(given[0],).toEqual({ root: TREE, },);
      },
    },),

    it({
      name: 'IGNORES A VALUE THAT STANDS UNDER THE KEY AND IS NO FUNCTION: a hook is a function or nothing',
      fn: async () => {
        for (const value of ['hook', 1, {}, null,]) {
          using _standing = standing({ value, },);
          expect(callParseEndHook({ root: TREE, },),).toBe(undefined,);
        }
      },
    },),

    it({
      name: 'LOOKS THE HOOK UP UNDER THE REGISTRY SYMBOL OF THE PACKAGE: the key is the symbol a fixture in another '
        + 'package finds by the same text',
      fn: async () => {
        expect(PARSE_END_HOOK_KEY,).toBe(Symbol.for(
          '@monochromatic-dev/translation-repair-document-structure/parse-end-hook',
        ),);
      },
    },),
  ],
},);

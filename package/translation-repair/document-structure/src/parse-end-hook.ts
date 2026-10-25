import type { Root, } from 'mdast';

//region Parse end hook
// A SEAM FOR A TEST THAT MAKES THE TREE OF A PARSE RAISE WHEN THE CALLER READS IT. `parseMdxBody` and
// `parseMarkdownBody` read the tree they have just built (they count the dropped characters and note the setext
// heading floors) before they return it, and a test that stands in for the tree needs to answer those reads and raise
// at every read after. It once told them apart by looking for the grammar function's name in the call stack, which a
// call through an object property, the module namespace or `Function.prototype.call` hides, since V8 writes the frame
// of a function called with an object as its receiver as `Object.parseMdxBody`. The grammar now says when it is done:
// it calls the hook last, with the tree it is about to return.
//
// THE HOOK STANDS ON THE `Symbol` FUNCTION UNDER A REGISTRY SYMBOL, not in an export, so that the package holding the
// test fixture needs no dependency on this package (this package's tests already depend on that one) and so that a
// source copy and a built copy of this module find the same hook: `Symbol` is the one object every copy shares, and
// the registry symbol is the same symbol in each. The global object is not read, since a read of it that names no
// member is a place where code can hide from the scans of the package source.
//
// NOTHING IS STANDING THERE IN PRODUCTION, and the call then costs one property lookup.

/**
 The key the hook is looked up under on the `Symbol` function.

 @internal
 */
export const PARSE_END_HOOK_KEY: symbol = Symbol.for(
  '@monochromatic-dev/translation-repair-document-structure/parse-end-hook',
);

/**
 What a hook is.
 */
type ParseEndHook = (arguments_: { readonly root: Root; },) => void;

/**
 Whether a value is a function, which is all a hook is.

 @param value - what stands under the key

 @returns Whether it can be called

 @example
 ```ts
 const callable = isParseEndHook(() => {},); // true
 ```
 */
function isParseEndHook(value: unknown,): value is ParseEndHook {
  return (typeof value) === 'function';
}

/**
 Calls the hook for a tree the grammar is about to return, when a test has set one.

 @internal

 @param root - tree, final once the grammar has made its own reads of it

 @example
 ```ts
 callParseEndHook({ root, },);
 ```
 */
export function callParseEndHook({ root, }: { readonly root: Root; },): void {
  /**
   What stands under the key, nothing outside a test.
   */
  const hook: unknown = Reflect.get(
    Symbol,
    PARSE_END_HOOK_KEY,
  );
  if (!isParseEndHook(hook,))
    return;
  hook({ root, },);
}

//endregion Parse end hook

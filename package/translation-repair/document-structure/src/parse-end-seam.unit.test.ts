/**
 Tests for the seam the walk fixture of `parse-exhaustion.test-fixture.ts` uses to tell the grammar's own reads of the
 tree it has just built from the reads of the code that called the grammar.

 `parseMdxBody` and `parseMarkdownBody` read the tree they build (they count the dropped characters and note the setext
 heading floors) before they return it, and the fixture answers those reads from the real tree and raises the engine's
 stack exhaustion at every read after. It once told the two apart by looking for the grammar function's name in the
 call stack, which a call through an object property, the module namespace or `Function.prototype.call` hides, so the
 grammar's own read raised and the parse call itself failed. The grammar now calls a hook last, once the tree is final,
 which defaults to nothing and which the fixture sets for the span of one call; these cases call the grammar the ways a
 caller can reach it and read the tree afterward. V8 writes a frame called with an object as its receiver as
 `Object.parseMdxBody`, which is why a property of an object, the module namespace and `Function.prototype.call` with an
 object hid the name the stack test looked for.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { Root, } from 'mdast';

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  parseMarkdownBody,
  parseMdxBody,
} from '../dist/final/node/index.mjs';
import {
  ENGINE_EXHAUSTION_MESSAGE,
  withExhaustionFromWalk,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/parse-exhaustion.test-fixture.ts';

/**
 The module namespace of the built grammar, which the call that goes through it is given.
 */
type GrammarNamespace = {
  readonly parseMarkdownBody: (arguments_: { readonly body: string; },) => Root;
  readonly parseMdxBody: (arguments_: { readonly body: string; },) => Root;
};

/**
 Whether a loaded module holds the two grammar functions.

 @param value - the namespace the loader gave

 @returns Whether both are functions on it

 @example
 ```ts
 const holds = isGrammarNamespace({ value: await importGrammar(), },);
 ```
 */
function isGrammarNamespace(value: unknown,): value is GrammarNamespace {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('parseMarkdownBody' in value)
    && ('parseMdxBody' in value)
    && ((typeof value.parseMarkdownBody) === 'function')
    && ((typeof value.parseMdxBody) === 'function');
}

/**
 Where the built grammar stands, held in a constant so the dynamic import of `importGrammar` names no relative path: a
 namespace import and a literal relative dynamic import are forms the scan of dead test declarations does not follow,
 and the names this file takes from the namespace are the ones it imports by name from the same module.
 */
const GRAMMAR_PATH = '../dist/final/node/index.mjs';

/**
 Loads the module namespace of the built grammar, for the call that goes through it.

 @returns The namespace

 @throws {@link Error} when the module lacks either function

 @example
 ```ts
 const grammar = await importGrammar();
 ```
 */
async function importGrammar(): Promise<GrammarNamespace> {
  /**
   What the loader gave.
   */
  const loaded: unknown = await import(GRAMMAR_PATH);
  if (!isGrammarNamespace(loaded,))
    throw new Error('unreachable: the built grammar holds both parse functions',);
  return loaded;
}

/**
 The key the grammar looks its hook up under, the registry symbol the fixture sets it under.
 */
const PARSE_END_HOOK_KEY = Symbol.for('@monochromatic-dev/translation-repair-document-structure/parse-end-hook',);

/**
 A body with a heading, a footnote reference and its definition, so the grammar's own reads have a tree to walk.
 */
const BODY = '# Cat\n\nA tabby[^1] naps.\n\n[^1]: A note.\n';

/**
 A way to call a grammar function.
 */
type Call = (arguments_: { readonly body: string; },) => Root;

/**
 The ways a caller reaches the grammar, for one function of it.
 */
type Shape = {
  /**
   What the shape is, for the case name.
   */
  readonly name: string;

  /**
   Calls the function the shape's way.
   */
  readonly call: (
    arguments_: {
      readonly name: 'parseMarkdownBody' | 'parseMdxBody';
      readonly body: string;
      readonly grammar: GrammarNamespace;
    },
  ) => Root;
};

/**
 Picks one of the two grammar functions by name.

 @param name - function wanted

 @returns Function

 @example
 ```ts
 const parse = grammarFunction({ name: 'parseMdxBody', },);
 ```
 */
function grammarFunction({ name, }: { readonly name: 'parseMarkdownBody' | 'parseMdxBody'; },): Call {
  return (name === 'parseMdxBody') ? parseMdxBody : parseMarkdownBody;
}

/**
 Every way a caller reaches a function of the grammar: a plain call, an arrow function that wraps it, a method of an
 object, the module namespace, `Function.prototype.call` and `Reflect.apply`.
 */
const SHAPES: readonly Shape[] = [
  {
    name: 'a plain call',
    call: function plain({ name, body, },): Root {
      return grammarFunction({ name, },)({ body, },);
    },
  },
  {
    name: 'an arrow function that wraps the function',
    call: function wrapped({ name, body, },): Root {
      return ((arguments_: { readonly body: string; },): Root => grammarFunction({ name, },)(arguments_,))({ body, },);
    },
  },
  {
    name: 'a property of an object',
    call: function property({ name, body, },): Root {
      /**
       An object holding the function under another name, as a caller that keeps a reader in a table does.
       */
      const holder = { parse: grammarFunction({ name, },), };
      return holder.parse({ body, },);
    },
  },
  {
    name: 'the module namespace',
    call: function namespace({ name, body, grammar, },): Root {
      return grammar[name]({ body, },);
    },
  },
  {
    name: 'Function.prototype.call with an object as the receiver',
    call: function viaCall({ name, body, },): Root {
      return grammarFunction({ name, },).call(
        {},
        { body, },
      );
    },
  },
  {
    name: 'Reflect.apply with no receiver',
    call: function viaReflect({ name, body, },): Root {
      return Reflect.apply(
        grammarFunction({ name, },),
        undefined,
        [{ body, },],
      );
    },
  },
];

/**
 Sets a hook under the registry symbol until the returned value is disposed of.

 @param hook - what the grammar calls with the final tree

 @returns Disposable that takes the hook away

 @example
 ```ts
 using _set = hookSet({ hook: function keep(): void {}, },);
 ```
 */
function hookSet(
  { hook, }: { readonly hook: (arguments_: { readonly root: Root; },) => void; },
): Disposable {
  Reflect.set(
    Symbol,
    PARSE_END_HOOK_KEY,
    hook,
  );
  return {
    [Symbol.dispose]: function takeAway(): void {
      Reflect.deleteProperty(
        Symbol,
        PARSE_END_HOOK_KEY,
      );
    },
  };
}

/**
 Whether a thrown value is the engine's stack exhaustion, unconverted.

 @param value - what a call threw

 @returns Whether it is a `RangeError` with the engine's message

 @example
 ```ts
 const raw = isRawExhaustion({ value: new RangeError('Maximum call stack size exceeded',), }); // true
 ```
 */
function isRawExhaustion({ value, }: { readonly value: unknown; }): boolean {
  return (value instanceof RangeError) && (value.message === ENGINE_EXHAUSTION_MESSAGE);
}

await describe({
  name: 'the seam between the grammar\'s own reads of its tree and the reads of its caller',
  children: [
    ...(['parseMdxBody', 'parseMarkdownBody',] as const).map(function ofFunction(name,) {
      return describe({
        name,
        children: SHAPES.map(function ofShape({ name: shape, call, },) {
          return it({
            name: `ANSWERS THE GRAMMAR'S OWN READS AND RAISES AT THE FIRST READ AFTER IT FOR ${shape.toUpperCase()}: `
              + 'the call returns the tree and the caller\'s read of it raises the engine\'s exhaustion',
            fn: async () => {
              /**
               The module namespace of the built grammar.
               */
              const grammar = await importGrammar();
              /**
               The tree the call returned inside the walk, read after the call ended.
               */
              const root = withExhaustionFromWalk({
                run: function parse(): Root {
                  return call({
                    name,
                    body: BODY,
                    grammar,
                  },);
                },
              },);
              expect(isRawExhaustion({
                value: caught(function read(): number {
                  return root.children.length;
                },),
              },),).toBe(true,);
            },
          },);
        },),
      },);
    },),

    describe({
      name: 'the hook',
      children: [
        it({
          name: 'CALLS NOTHING BY DEFAULT: the grammar returns an ordinary tree a caller reads freely',
          fn: async () => {
            expect(parseMdxBody({ body: BODY, },).children.length > 0,).toBe(true,);
            expect(parseMarkdownBody({ body: BODY, },).children.length > 0,).toBe(true,);
          },
        },),

        it({
          name: 'CALLS A HOOK SET UNDER THE REGISTRY SYMBOL ONCE, WITH THE FINAL TREE, AT THE END OF EACH FUNCTION: '
            + 'the tree it is given is the one the call returns, with its heading floors noted',
          fn: async () => {
            /**
             The trees the hook was given.
             */
            const seen: unknown[] = [];
            using _set = hookSet({
              hook: function keep({ root, }: { readonly root: Root; },): void {
                seen.push(root,);
              },
            },);
            /**
             The trees the two functions returned.
             */
            const returned = [parseMdxBody({ body: BODY, },), parseMarkdownBody({ body: BODY, },),];
            expect(seen.length,).toBe(2,);
            expect(seen.every(function sameTree(tree, index,): boolean {
              return tree === returned[index];
            },),).toBe(true,);
          },
        },),

        it({
          name: 'IS PUT BACK WHEN THE WALK ENDS: a parse after the walk returns an ordinary tree',
          fn: async () => {
            withExhaustionFromWalk({
              run: function parse(): Root {
                return parseMdxBody({ body: BODY, },);
              },
            },);
            expect(Reflect.get(
              Symbol,
              PARSE_END_HOOK_KEY,
            ),).toBe(undefined,);
            expect(parseMdxBody({ body: BODY, },).children.length > 0,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

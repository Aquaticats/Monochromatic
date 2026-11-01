import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Default suffix allowlist. A class passes the rule when either its direct
 superclass identifier or its own class name ends with one of these
 suffixes.
 
 `Error` matches both the global `Error` itself (direct subclasses) and
 transitive chains where the child's own name ends in `Error`.
 
 `Element` matches `HTMLElement`, `LitElement`, and any downstream
 `*Element` base class a project defines.
 */
const DEFAULT_SUFFIXES: readonly string[] = [
  'Error',
  'Element',
];

/**
 Narrowed runtime shape of the first options-array element. `suffixes` is
 left as `readonly unknown[]` because the JSON schema only validates at
 config-load time; the rule re-checks each entry.
 */
type FirstOption = {
  readonly suffixes?: readonly unknown[];
};

/**
 Tests whether a value satisfies the `FirstOption` runtime shape: a
 non-null, non-array object that optionally carries a `suffixes` field.
 
 @param value - candidate options entry to inspect
 
 @returns true when the value can be read as `FirstOption`
 */
function isFirstOption(value: unknown,): value is FirstOption {
  if ((value === null) || (value === undefined))
    return false;
  if ((typeof value) !== 'object')
    return false;
  if (Array.isArray(value,))
    return false;
  return true;
}

/**
 Extracts the configured suffix list from rule options, falling back to
 {@link DEFAULT_SUFFIXES} when no option object is provided or the shape,
 narrowed via {@link isFirstOption}, is unrecognised. Non-string entries in
 a user-supplied `suffixes` array are filtered out.
 
 @param options - rule options array; first element optionally carries `suffixes`
 
 @returns resolved suffix list, never empty when defaults apply
 */
function readSuffixes(
  options: readonly unknown[],
): readonly string[] {
  if (!Array.isArray(options,))
    return DEFAULT_SUFFIXES;
  /**
   First element of the options array; ESLint convention places the
   option object here. Read as `unknown` (the user-facing array is
   `unknown[]`); narrowed to `FirstOption` via the type guard.
   */
  // oxlint-disable-next-line typescript/no-unsafe-assignment -- Array.isArray on readonly unknown[] | undefined narrows to any[]; we re-check the element shape via isFirstOption below
  const [first,] = options;
  if (!isFirstOption(first,))
    return DEFAULT_SUFFIXES;
  /**
   Configured suffix array; may be undefined when the user passes a partial object.
   */
  const { suffixes, } = first;
  if (!Array.isArray(suffixes,))
    return DEFAULT_SUFFIXES;
  return suffixes.filter(function keepStrings(value,): value is string {
    return (typeof value) === 'string';
  },);
}

/**
 Bans `class` declarations unless the direct superclass identifier or the
 class's own name ends with one of the configured suffixes.
 
 Long-lived stateful objects should be expressed as a factory function
 returning a frozen object literal. Closure variables provide the same
 privacy guarantees as `#private` (the captured variables are unreachable
 from outside the factory's scope when the return type omits them), with
 no `this`-binding gotchas, no `new` keyword, no prototype confusion, and
 no accidental `extends` ladder.
 
 Default suffixes are `Error` and `Element`. They cover direct subclasses
 of `Error`, `HTMLElement`, and `LitElement` (each base name ends with a
 suffix), plus transitive `*Error` and `*Element` chains across files.
 
 Override via rule options when a different convention applies:
 `["error", { "suffixes": ["Error", "Element", "EventTarget"] }]`; options
 are resolved once per file via {@link readSuffixes} and tested per class
 via {@link matchesSuffix}.
 
 `Symbol.dispose` and `Symbol.asyncDispose` are not an excuse: place them
 as keys in the returned object literal instead of declaring `implements Disposable` on a class.
 
 @example
 ```ts
 // Bad
 class HashCache {
   readonly #map = new Map();
   get(k) { return this.#map.get(k); }
   set(k, v) { this.#map.set(k, v); }
 }
 
 // Good
 function createHashCache() {
   const map = new Map();
   function get(k) { return map.get(k); }
   function set(k, v) { map.set(k, v); }
   return Object.freeze({ get, set, },);
 }
 
 // Good; Disposable resource
 function createServer() {
   const server = startServer();
   return {
     handle: server.handle,
     [Symbol.dispose]() { server.close(); },
   };
 }
 
 // Good; Error subclass (direct or transitive)
 class MyError extends Error {}
 class ChildError extends MyError {}
 
 // Good; Web Component (direct or transitive)
 class MyButton extends HTMLElement {}
 class MyDialog extends LitElement {}
 ```
 */
export const noClass: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow class declarations except when the direct superclass or the class\'s own name ends with a configured suffix (default: Error, Element). Use factory functions returning frozen objects instead.',
      recommended: true,
    },
    messages: {
      forbidden:
        'Classes are banned unless the direct superclass identifier or the class\'s own name ends with a configured suffix (default: `Error`, `Element`). Replace with a factory function returning a frozen object; `Symbol.dispose` belongs in the returned literal, not as a class member.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          suffixes: {
            type: 'array',
            items: {
              type: 'string',
            },
          },
        },
        additionalProperties: false,
      },
    ],
    defaultOptions: [
      {
        suffixes: [...DEFAULT_SUFFIXES,],
      },
    ],
  },
  /**
   Handles foreign Oxlint callback.
   
   @param context - Foreign rule context receiving diagnostics.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Tests whether `name` ends with any configured allowlist suffix.
     
     Reads `context.options` per call rather than once in `createOnce`. Oxlint leaves
     `options` null until it is about to lint a file (`apps/oxlint/src-js/plugins/context.ts`
     documents it as "Initially `null` during `createOnce`, set to options object before
     linting a file"), so hoisting the read silently pins every file to the defaults and
     makes the rule look unconfigurable.
     
     @param name - identifier name to test against the suffix list
     
     @returns true when the name ends with any configured suffix
     */
    function matchesSuffix(name: string,): boolean {
      /**
       Raw rule options for the file being linted; oxlint leaves this null outside a file.
       */
      const { options, } = context;
      return readSuffixes(options ?? [],)
        .some(function endsWith(suffix,): boolean {
          return name.endsWith(suffix,);
        },);
    }

    /**
     Reports a class node unless its direct superclass identifier or its
     own name matches one of the configured suffixes via
     {@link matchesSuffix}. Classes with no `extends` clause are always
     reported. Ambient `declare class` declarations are skipped because
     they emit no runtime.
     
     @param node - class declaration or class expression to check
     */
    function checkClass(node: ForeignBorrowed<ESTree.Class>,): void {
      // `declare class` in ambient `.d.ts` files describes external types
      // and emits no runtime; the rule targets emitted classes only.
      if (node.declare
        === true)
        return;

      /**
       Parent expression following `extends`, when present.
       */
      const { superClass, } = node;
      if ((superClass === null) || (superClass === undefined)) {
        context.report({
          node,
          messageId: 'forbidden',
        },);
        return;
      }

      if (
        (superClass.type
          === 'Identifier')
          && matchesSuffix(superClass.name,)
      ) {
        return;
      }

      /**
       Class identifier; null on anonymous class expressions.
       */
      const { id, } = node;
      if (
        (id !== null)
          && (id !== undefined)
          && matchesSuffix(id.name,)
      ) {
        return;
      }

      context.report({
        node,
        messageId: 'forbidden',
      },);
    }

    return {
      ClassDeclaration: checkClass,
      ClassExpression: checkClass,
    };
  },
};

import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Bans `let` declarations at module root scope.
 
 Module-scope `let` is mutable across the entire file. Every importer
 and every function in the module shares the same binding; reasoning
 about when the value can change requires reading the whole module.
 
 Reports `VariableDeclaration` with `kind === 'let'` as a direct child
 of `Program.body`, including the form `export let x = 1` (parsed as
 `ExportNamedDeclaration` wrapping a `VariableDeclaration`).
 
 No allowlist heuristics: modules are not called and do not return.
 The remediation paths are a concrete data structure (`Map`, `WeakMap`,
 `Set`, `WeakSet` for mutable containers), the {@link memoize} helper from
 `@monochromatic-dev/kv-store-memoize` for cached computations, or wrapping
 initialization in an IIFE assigned to const.
 
 @example
 ```ts
 // Bad; module-scope mutable state
 let cache: Map<string, string> | null = null;
 
 // Good; Map container is the mutable surface
 const cache = new Map<string, string>();
 
 // Good; IIFE-into-const for lazy initialization
 const cache = (function init (): Map<string, string> {
   const m = new Map<string, string>();
   m.set('seed', 'value',);
   return m;
 })();
 
 // Good; memoize() holds its cache internally, no module-root let
 const fetchValue = memoize({
   fn: function compute (key: string): string {
     return expensiveLookup(key,);
   },
   keyFn: function identity (key: string): string {
     return key;
   },
 },);
 const value = fetchValue({ args: ['k',], salt: 'v1', },);
 ```
 */
export const noModuleRootLet: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow `let` at module root scope. Use Map/WeakMap, memoize(), or an IIFE-into-const.',
      recommended: true,
    },
    messages: {
      forbidden: '`let` at module-root scope is mutable across the entire module. '
        + 'Replace with a `Map`/`WeakMap`/`Set`/`WeakSet` for mutable containers, '
        + '`memoize()` from `@monochromatic-dev/kv-store-memoize` for cached computations, '
        + 'or an IIFE assigned to const `const cached = (function init () { let v = null; /* compute */ return v; })()`. '
        + 'If genuinely unavoidable, add '
        + '`oxlint-disable-next-line no-restricted-syntax/no-module-root-let` with a justification.',
    },
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
     Reports a node if it is a non-ambient `let` VariableDeclaration.
     
     @param decl - Statement-level node from `Program.body` or
     `ExportNamedDeclaration.declaration`. Typed `ESTree.Node` rather than the
     narrower `Directive | Statement` because an inline union of external alias
     types flattens and cannot be allow-listed for prefer-readonly.
     */
    function reportIfLet(decl: ForeignBorrowed<ESTree.Node>,): void {
      if (decl.type
        !== 'VariableDeclaration')
        return;
      if (decl.kind
        !== 'let')
        return;
      if (decl.declare
        === true)
        return;
      context.report({
        node: decl,
        messageId: 'forbidden',
      },);
    }

    return {
      Program(node: ForeignBorrowed<ESTree.Program>,): void {
        for (const stmt of node.body) {
          if (stmt.type
            === 'ExportNamedDeclaration') {
            if (stmt.declaration
              !== null)
              reportIfLet(stmt.declaration,);
            continue;
          }
          reportIfLet(stmt,);
        }
      },
    };
  },
};

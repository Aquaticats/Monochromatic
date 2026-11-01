import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { reportMissing, } from './node-extraction.ts';
import { shouldSkipIgnoredFile, } from './tsdoc-visitors.ts';

/**
 Requires TSDoc comments on every documentable declaration, including
 local variables inside function bodies and block scopes.
 
 FunctionExpression and ArrowFunctionExpression are excluded because
 their TSDoc is owned by the enclosing VariableDeclaration or
 MethodDefinition node.
 
 For-loop bindings (`for (const x of arr)`, `for (let loopIndex = 0; ...)`)
 are excluded because they have no natural site for TSDoc.
 
 Each visitor reports through {@link reportMissing}, and
 {@link shouldSkipIgnoredFile} skips files excluded from this rule.
 
 @example
 ```ts
 // Bad; missing TSDoc on local
 /\** Doubles a value. *\/
 function double(n: number): number {
   const result = n * 2;
   return result;
 }
 
 // Good
 /\** Doubles a value. *\/
 function double(n: number): number {
   /\** Computed double of the input. *\/
   const result = n * 2;
   return result;
 }
 ```
 */
export const requireTsdoc: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require TSDoc comments on all documentable declarations.',
      recommended: true,
    },
    messages: {
      missing: 'Missing TSDoc comment on {{kind}} "{{name}}".',
    },
  },
  /**
   Handles effectful plugin callback.
   
   @param context - Foreign callback value carrying diagnostic capability.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Mutable visitor traversal state shared across AST callbacks.
     
     AGENTS.md bans function-root `let` for cleanliness; an object with
     a mutable property carries the same state in a single `const` binding.
     */
    const state = {
      /**
       True when the next VariableDeclaration is a for-loop binding (for/for-of/for-in init).
       */
      inForLoopInit: false,
    };

    return {
      before() {
        if (shouldSkipIgnoredFile({ context, }))
          return false;
        return undefined;
      },
      FunctionDeclaration(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      ClassDeclaration(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      MethodDefinition(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      TSInterfaceDeclaration(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      TSTypeAliasDeclaration(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      TSEnumDeclaration(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      ForStatement(): void {
        state.inForLoopInit = true;
      },
      ForOfStatement(): void {
        state.inForLoopInit = true;
      },
      ForInStatement(): void {
        state.inForLoopInit = true;
      },
      VariableDeclaration(node: ForeignBorrowed<ESTree.Node>,): void {
        if (state.inForLoopInit) {
          state.inForLoopInit = false;
          return;
        }
        reportMissing({
          node,
          context,
        },);
      },
      PropertyDefinition(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      TSEnumMember(node: ForeignBorrowed<ESTree.Node>,): void {
        reportMissing({
          node,
          context,
        },);
      },
      Property(node: ForeignBorrowed<
        | ESTree.AssignmentTargetProperty
          | ESTree.BindingProperty
          | ESTree.ObjectProperty
        >,): void {
        if ((node.kind
          === 'get') || (node.kind
            === 'set')) {
          reportMissing({
            node,
            context,
          },);
        }
      },
    } as VisitorWithHooks;
  },
};

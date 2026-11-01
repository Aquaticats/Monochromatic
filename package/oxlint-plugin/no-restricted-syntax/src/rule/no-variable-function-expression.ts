import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Bans function expressions assigned to variables.
 
 Patterns like `const myFn = function myFn() {}` are redundant:
 the name appears twice and the binding adds noise for no benefit.
 Use a `function` declaration instead, which is compatible with
 TSDoc, supports function overloading, and is easier to scan.
 
 Function expressions passed as arguments (callbacks) are **not** affected
 by this rule because they have no variable binding.
 
 @example
 ```ts
 // Bad
 const greet = function greet(name: string): void { };
 const greet = function(name: string): void { };
 
 // Good
 function greet(name: string): void { }
 ```
 */
export const noVariableFunctionExpression: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow function expressions assigned to variables. Use function declarations instead.',
      recommended: true,
    },
    messages: {
      forbidden:
        'Function expressions assigned to variables are banned. Use a function declaration instead.',
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
    return {
      VariableDeclaration(node: ForeignBorrowed<ESTree.VariableDeclaration>,): void {
        for (const declarator of node.declarations) {
          if ((declarator.init
            !== null)
            && (declarator.init
              .type
              === 'FunctionExpression'))
            {
            context.report({
              node,
              messageId: 'forbidden',
            },);
          }
        }
      },
    };
  },
};

import type {
  Context,
  CreateOnceRule,
  ESTree,
  Fixer,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { readsEnclosingBinding, } from './arrow-enclosing-binding.ts';
import { extractParamsText, } from './arrow-function-params.ts';

/**
 Bans arrow function expressions in favor of named function declarations
 and named function expressions.
 
 Arrow functions produce anonymous entries in stack traces, making debugging
 harder. Named functions provide clear trace names, are hoisted for flexible
 ordering, and signal intent more explicitly.
 
 Auto-fix handles the common `const name = (...) => ...` pattern by converting
 it to a `function name(...) { ... }` declaration, extracting the parameter
 list via {@link extractParamsText}. `export const` is similarly converted
 to `export function`. A type-annotated `const name: Type = (...) => ...` keeps
 its declaration and annotation and only the arrow becomes a named function
 expression, so contextual parameter types survive; so does a `const` outside a
 statement list such as a `for` head. Arrows reading an enclosing `this`, `arguments`, `super`,
 or `new.target`, and `let`/`var` bindings, are reported without a fix (see {@link readsEnclosingBinding}).
 Other contexts (callbacks, object properties)
 are reported but not auto-fixed because naming them requires human judgment.
 
 @example
 ```ts
 // Bad
 const double = (x: number): number => x * 2;
 items.map((item) => item.value);
 
 // Good
 function double(x: number): number { return x * 2; }
 items.map(function getValue(item) { return item.value; });
 ```
 */
export const noArrowFunction: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description:
        'Disallow arrow function expressions. Use named function declarations or named function expressions instead.',
      recommended: true,
    },
    messages: {
      forbidden:
        'Arrow functions are banned. Use named function declarations or named function expressions instead.',
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
      ArrowFunctionExpression(
        node: ForeignBorrowed<ESTree.ArrowFunctionExpression>,
      ): void {
        /**
         Only auto-fix when the arrow is the direct initializer of a variable declaration:
         `const name = (...) => ...` or `export const name = (...) => ...`.
         
         Callbacks, object properties, and other contexts are too context-dependent
         to auto-fix reliably.
         */
        const { parent, } = node;
        if (
          (parent.type
            !== 'VariableDeclarator')
            || (parent.id
              .type
              !== 'Identifier')
            || (parent.parent
              .type
              !== 'VariableDeclaration')
            || (parent.parent
              .declarations
              .length
              !== 1)
        ) {
          context.report({
            node,
            messageId: 'forbidden',
          },);
          return;
        }

        /**
         Arrows that read an enclosing `this`, `arguments`, `super`, or `new.target`, and
         `let`/`var` bindings, have no behavior-preserving rewrite: a `function` rebinds the
         former, and a named function expression binds its own name, which changes
         calls through a reassigned `let`/`var`. They are reported without a fix.
         */
        if (
          readsEnclosingBinding(node,)
            || (parent.parent
              .kind
              !== 'const')
        ) {
          context.report({
            node,
            messageId: 'forbidden',
          },);
          return;
        }

        /**
         Variable name from the declarator (e.g. `foo` in `const foo = ...`).
         */
        const { name, } = parent.id;

        /**
         Node containing the `VariableDeclaration`; inspected to detect an `export` wrapper.
         */
        const grandparent = parent.parent
          .parent;
        /**
         True when the declaration is exported, so the replacement keeps the `export` prefix.
         */
        const isExported = grandparent.type
          === 'ExportNamedDeclaration';

        /**
         The full declaration node to replace (including `export` if present).
         */
        const replaceNode = isExported ? grandparent : parent.parent;

        /**
         Async prefix if the arrow is async.
         */
        const asyncPrefix = node.async ? 'async ' : '';

        /**
         Generic type parameters if present.
         */
        const typeParamsText =
          (node.typeParameters
            !== null) && (node.typeParameters
              !== undefined)
            ? context.sourceCode
              .getText(node.typeParameters,)
            : '';

        /**
         Source text of the parameter list including parentheses: verbatim
         from the source for parenthesized lists, rebuilt from parameter texts
         for bare parameters.
         */
        const paramsText = extractParamsText({
          fullText: context.sourceCode
            .getText(node,),
          node,
          paramTexts: node.params
            .map(function paramText(param,): string {
              return context.sourceCode
                .getText(param,);
            },),
        },);

        /**
         Return type annotation if present.
         */
        const returnTypeText =
          (node.returnType
            !== null) && (node.returnType
              !== undefined)
            ? context.sourceCode
              .getText(node.returnType,)
            : '';

        /**
         Body text, wrapping expression bodies in `{ return ...; }`.
         */
        const bodyText = node.expression
          ? `{ return ${context.sourceCode
            .getText(node.body,)} }`
          : context.sourceCode
            .getText(node.body,);

        /**
         Function head and body shared by the declaration and the named function expression forms.
         */
        const functionText =
          `${asyncPrefix}function ${name}${typeParamsText}${paramsText}${returnTypeText} ${bodyText}`;

        /**
         True when the variable carries a type annotation (`const name: Type = ...`).

         A function declaration cannot hold that annotation, and dropping it would
         strip the contextual types of unannotated parameters, so the arrow alone is
         swapped for a named function expression and the declaration stays intact.
         */
        const isAnnotated = (parent.id
          .typeAnnotation
          !== null)
          && (parent.id
            .typeAnnotation
            !== undefined);

        /**
         Node that lists the statements holding the declaration, past an `export` wrapper.

         A function declaration is only valid in a statement list, so a `const` in a
         `for` head keeps its declaration and receives a named function expression too.
         */
        const statementListOwner = isExported ? grandparent.parent : grandparent;

        /**
         True when the declaration sits where a function declaration may stand.
         */
        const isInStatementList = (statementListOwner.type === 'Program')
          || (statementListOwner.type === 'BlockStatement')
          || (statementListOwner.type === 'StaticBlock')
          || (statementListOwner.type === 'TSModuleBlock');

        /**
         True when the fix keeps the surrounding declaration and swaps only the arrow.
         */
        const replacesArrowOnly = isAnnotated || (!isInStatementList);

        /**
         Export keyword prefix.
         */
        const exportPrefix = isExported ? 'export ' : '';

        context.report({
          node,
          messageId: 'forbidden',
          fix(fixer: ForeignBorrowed<Fixer>,) {
            return replacesArrowOnly
              ? fixer.replaceText(
                node,
                functionText,
              )
              : fixer.replaceText(
                replaceNode,
                `${exportPrefix}${functionText}`,
              );
          },
        },);
      },
    };
  },
};

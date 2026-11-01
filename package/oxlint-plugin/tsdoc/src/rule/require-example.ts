/**
 TSDoc require-example rule for exported functions.
 
 Reports when an exported function's TSDoc comment lacks an `\@example` tag.
 Functions with `\@inheritDoc` or `\@internal` are exempt since their
 documentation is inherited or intentionally hidden.
 
 Handles both direct exports (`export function foo()`) and
 specifier-list exports (`export { foo }`).
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  CreateOnceRule,
  Span,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import {
  isRecord,
  isRecordArray,
} from '../ast-access.ts';
import {
  findTsdocComment,
  NO_TSDOC,
  parseTsdocForNode,
  type TsdocParseResult,
} from '../tsdoc-utils.ts';

import { extractNodeName, } from './node-extraction.ts';
import {
  commentReportLoc,
  shouldSkipIgnoredFile,
} from './tsdoc-visitors.ts';

/**
 Checks whether a node is directly exported via inline `export` keyword.
 
 Detects `export function`, `export const`, and `export default`
 by inspecting the parent node type.
 
 @param node - AST node to check
 
 @returns true when the node's parent is an export declaration
 */
function isDirectlyExported(node: ForeignBorrowed<Span>,): boolean {
  /**
   Narrowed view of `node` that exposes the untyped `parent` property added by the host.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped
  const typed = node as Span & Record<string, unknown>;
  /**
   Parent node, used to detect whether the function sits under an `export` declaration.
   */
  const { parent, } = typed;
  if (!isRecord(parent,))
    return false;
  return (parent.type
    === 'ExportNamedDeclaration')
    || (parent.type
      === 'ExportDefaultDeclaration');
}

/**
 Checks whether a TSDoc comment contains an `\@example` block tag.
 
 @param result - parsed TSDoc result
 
 @returns true when at least one `\@example` block exists
 */
function hasExampleTag(result: ReadonlyDeep<TsdocParseResult>,): boolean {
  return result.docComment
    .hasExampleTag;
}

/**
 Checks whether a TSDoc comment is exempt from requiring `\@example`.
 
 Exempt when the comment uses `\@inheritDoc` (documentation comes from
 the referenced symbol) or `\@internal` (not part of public API docs).
 
 @param result - parsed TSDoc result
 
 @returns true when the comment should be skipped
 */
function isExempt(result: ReadonlyDeep<TsdocParseResult>,): boolean {
  /**
   Scanned doc model carrying the `@inheritDoc`/`@internal` exemption flags.
   */
  const { docComment, } = result;
  return docComment.hasInheritDocTag || docComment.hasInternalModifier;
}

/**
 Requires `\@example` tags on exported functions.
 
 Targets both directly exported functions (`export function foo()`)
 and functions exported via specifier lists (`export { foo }`), identified
 via {@link extractNodeName} and parsed via {@link parseTsdocForNode}.
 Class methods, non-exported functions, type aliases, and constants
 are not checked.
 
 @example
 ```ts
 // Bad; exported function without \@example
 /\** Adds two numbers. *\/
 export function add(a: number, b: number): number { return a + b; }
 
 // Good
 /\**
  Adds two numbers.
  
  \@example
  ```ts
  add(1, 2); // => 3
  ```
 *\/
 export function add(a: number, b: number): number { return a + b; }
 ```
 */
export const requireExample: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Require @example tag on exported functions.',
      recommended: true,
    },
    messages: {
      missing: 'Exported function is missing an @example tag in its TSDoc comment.',
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
     Deferred checks for functions that may be exported via specifier lists.
     Maps declaration name to the AST node and its TSDoc comment.
     */
    const functionNodes = new Map<string, {
      node: Span;
      comment: Comment;
    }>();

    /**
     Names exported via `export { name }` specifier lists.
     */
    const specifierExportedNames = new Set<string>();

    /**
     Reports a missing `\@example` tag for a given node and its TSDoc comment.
     */
    function reportMissingExample({
      node,
      comment,
    }: {
      /**
       Function-like AST node.
       */
      readonly node: Span;
      /**
       TSDoc comment AST node for error location.
       */
      readonly comment: ReadonlyDeep<Comment>;
    },): void {
      /**
       Parsed TSDoc result; absent when the comment cannot be parsed at all.
       */
      const result = parseTsdocForNode({
        node,
        context,
      },);
      if (result === NO_TSDOC)
        return;
      if (isExempt(result,))
        return;
      if (!hasExampleTag(result,)) {
        context.report({
          loc: commentReportLoc(comment,),
          messageId: 'missing',
        },);
      }
    }

    /**
     Checks a function-like node: reports immediately for direct exports,
     or defers to after-pass for specifier-list exports.
     
     @param node - AST node to check
     */
    function checkFunction(node: ForeignBorrowed<Span>,): void {
      /**
       Attached TSDoc comment, when present; absent means nothing to validate.
       */
      const comment = findTsdocComment({
        node,
        context,
      },);
      if (comment === NO_TSDOC)
        return;

      if (isDirectlyExported(node,)) {
        reportMissingExample({
          node,
          comment,
        },);
        return;
      }

      // Defer: might be exported via `export { name }` seen later
      /**
       Identifier of the function, used to match later `export { name }` specifiers.
       */
      const name = extractNodeName(node,);
      if (name !== 'anonymous') {
        functionNodes.set(
          name,
          {
            node,
            comment,
          },
        );
      }
    }

    /**
     Checks a VariableDeclaration whose init is a function expression
     or arrow function. Only the variable name is registered since the
     TSDoc comment is attached to the VariableDeclaration, not the inner
     function expression.
     
     @param node - VariableDeclaration AST node
     */
    function checkVariable(node: ForeignBorrowed<Span>,): void {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped. */
      /**
       Narrowed view of the VariableDeclaration so untyped `declarations` is reachable.
       */
      const typed = node as Span & Record<string, unknown>;
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      /**
       Declarator list; for `const a = ..., b = ...` only the first is inspected.
       */
      const { declarations, } = typed;
      if ((!isRecordArray(declarations,)) || (declarations.length
        === 0))
        return;

      /**
       First declarator; inspected so its `init` can be checked below.
       */
      const [decl,] = declarations;
      if (!isRecord(decl,))
        return;

      // Only check variables whose init is a function expression or arrow.
      // AST uses null (not undefined) for missing initializers (e.g. for-of bindings);
      // both fail the isRecord guard, so a non-record init is skipped here.
      /**
       Initializer of the first declarator; a non-record value means no function init.
       */
      const { init, } = decl;
      if (!isRecord(init,))
        return;
      if ((init.type
        !== 'FunctionExpression')
        && (init.type
          !== 'ArrowFunctionExpression'))
        {
        return;
      }

      /**
       Attached TSDoc comment for the variable; absent means nothing to validate.
       */
      const comment = findTsdocComment({
        node,
        context,
      },);
      if (comment === NO_TSDOC)
        return;

      if (isDirectlyExported(node,)) {
        reportMissingExample({
          node,
          comment,
        },);
        return;
      }

      // Defer for specifier-list exports
      /**
       Declared variable name, used to match later `export { name }` specifiers.
       */
      const name = extractNodeName(node,);
      if (name !== 'anonymous') {
        functionNodes.set(
          name,
          {
            node,
            comment,
          },
        );
      }
    }

    return {
      before() {
        if (shouldSkipIgnoredFile({ context, }))
          return false;
        // createOnce persists this visitor across all files in the lint run.
        // Without clearing, function names from file A leak into file B's
        // specifier-export check, causing false positives that vary with
        // thread ordering.
        functionNodes.clear();
        specifierExportedNames.clear();
        return undefined;
      },
      FunctionDeclaration: checkFunction,
      VariableDeclaration: checkVariable,
      ExportNamedDeclaration(node: ForeignBorrowed<Span>,): void {
        /* oxlint-disable typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped. */
        /**
         Narrowed view of the ExportNamedDeclaration so untyped `specifiers` is reachable.
         */
        const typed = node as Span & Record<string, unknown>;
        /* oxlint-enable typescript/no-unsafe-type-assertion */
        /**
         Specifier list; populated only for `export { a, b }` form, absent for inline exports.
         */
        const { specifiers, } = typed;
        if (!isRecordArray(specifiers,))
          return;

        for (const spec of specifiers) {
          /**
           Local-name node of the specifier (`a` in `export { a as b }`).
           */
          const { local, } = spec;
          if (!isRecord(local,))
            continue;
          /**
           Exported local identifier text; only string names are tracked.
           */
          const { name, } = local;
          if ((typeof name)
            === 'string')
            specifierExportedNames.add(name,);
        }
      },
      after(): void {
        // Check deferred function nodes against specifier-exported names
        for (const [name, entry,] of functionNodes) {
          if (specifierExportedNames.has(name,)) {
            reportMissingExample({
              node: entry.node,
              comment: entry.comment,
            },);
          }
        }
      },
    } as VisitorWithHooks;
  },
};

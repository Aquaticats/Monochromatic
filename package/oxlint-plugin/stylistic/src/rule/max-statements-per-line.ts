import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Fixer,
  Span,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { baseIndentAt, } from '../utility/indent.ts';
import { lineAt, } from '../utility/line-at.ts';
import {
  at,
  rangeOf,
} from '../utility/range.ts';
import { isOnlyWhitespaceOrSeparator, } from '../utility/source-filler.ts';

//region Statement tallies
// Which nodes count as a statement, and which parents absorb their single child
// into the parent's tally.

/**
 Parent node types that allow a single inline child statement
 (the inner statement does not count toward this line's tally).
 
 Mirrors the upstream `@stylistic/max-statements-per-line` regex:
 `if (a) foo();` counts as 1 statement (the `IfStatement`), the inner
 `foo()` is exempt; `if (a) foo(); else bar();` counts as 2 because the
 alternate branch of `IfStatement` is not exempt.
 */
const SINGLE_CHILD_ALLOWED = new Set([
  'DoWhileStatement',
  'ForStatement',
  'ForInStatement',
  'ForOfStatement',
  'IfStatement',
  'LabeledStatement',
  'WhileStatement',
  'ExportDefaultDeclaration',
  'ExportNamedDeclaration',
],);

/**
 Statement-shaped AST nodes that contribute to the per-line tally.
 */
const STATEMENT_TYPES = [
  'BreakStatement',
  'ClassDeclaration',
  'ContinueStatement',
  'DebuggerStatement',
  'DoWhileStatement',
  'ExpressionStatement',
  'ForInStatement',
  'ForOfStatement',
  'ForStatement',
  'FunctionDeclaration',
  'IfStatement',
  'ImportDeclaration',
  'LabeledStatement',
  'ReturnStatement',
  'SwitchStatement',
  'ThrowStatement',
  'TryStatement',
  'VariableDeclaration',
  'WhileStatement',
  'WithStatement',
  'ExportNamedDeclaration',
  'ExportDefaultDeclaration',
  'ExportAllDeclaration',
];
//endregion Statement tallies

//region Rule
// The exported rule: its metadata, its two callbacks, and the visitor that
// wires every statement type to them.

/**
 Enforces at most one statement per source line.
 
 Mirrors the upstream `@stylistic/max-statements-per-line` semantics with
 a hard-coded `max` of `1`. Single-child container parents
 (`if`, `while`, `do-while`, `for`, `for-in`, `for-of`, `labeled`, and
 `export-default`/`export-named` declarations) exempt their inner
 statement, so `if (a) foo();` reports zero violations. The alternate
 branch of an `if`/`else` is not exempt: `if (a) foo(); else bar();`
 counts as 2 statements and reports `bar()`.
 
 The autofix inserts `\n<indent>` between same-line statements. When the
 inter-statement source slice contains anything beyond whitespace and `;`
 (a comment, a label, anything non-trivial), the fix is suppressed and
 the violation is still reported. This preserves inline comments like
 `foo(); /* note *\/ bar();` rather than deleting them.
 
 @example
 ```ts
 // Bad
 const a = 1; const b = 2;
 if (true) foo(); else bar();
 
 // Good
 const a = 1;
 const b = 2;
 if (true) foo();
 else bar();
 ```
 */
export const maxStatementsPerLine: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description: 'Require at most one statement per source line.',
      recommended: true,
    },
    messages: {
      exceed: 'This line has {{count}} statements. Maximum allowed is 1.',
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
    //region Collecting statements per line
    // Every statement type lands in a bucket keyed by its start line.

    /**
     Statements counted per start-line, in source order.
     */
    const perLine = new Map<number, Span[]>();

    /**
     Records a statement under its start-line bucket unless its parent is
     a single-child container that absorbs it into the parent's tally.
     
     @param node - statement AST node
     */
    function trackStatement(node: ForeignBorrowed<Span>,): void {
      /**
       Statement node narrowed to the parent link used for exemptions.
       */
      const { parent, } = node as Span & {
        readonly parent?: {
          readonly type: string;
          readonly alternate?: Span;
        };
      };
      if ((parent !== undefined) && SINGLE_CHILD_ALLOWED
        .has(parent.type,)) {
        /**
         Alternate branch of an `if`/`else` is not exempt; it counts toward the line tally.
         */
        const isIfAlternate = (parent.type
          === 'IfStatement')
          && (parent.alternate
            === node);
        if (!isIfAlternate)
          return;
      }

      /**
       Source text is needed to map the node's start offset to a line number.
       */
      const sourceText = context.sourceCode
        .getText();
      /**
       Line number of the statement's start offset; bucket key.
       */
      const line = lineAt({
        sourceText,
        offset: rangeOf(node,)[0],
      },);
      /**
       Per-line bucket of statements seen so far; created on demand.
       */
      const bucket = perLine.get(line,)
        ?? [];
      bucket.push(node,);
      perLine.set(
        line,
        bucket,
      );
    }
    //endregion Collecting statements per line

    //region Reporting excess statements
    // One report per statement past the first on a line, fixed by inserting a
    // newline and the line's own indent unless a comment or nesting intervenes.

    /**
     On program exit, walks every line bucket and reports each statement
     past the first.
     */
    function reportExceeding(): void {
      /**
       Source text is needed for indent lookup and inter-statement slices.
       */
      const sourceText = context.sourceCode
        .getText();
      for (const stmts of perLine.values()) {
        if (stmts.length
          <= 1)
          continue;

        /**
         Range of the first statement on this line; its leading whitespace defines the indent for the fix.
         */
        const firstRange = rangeOf(at({
          arr: stmts,
          index: 0,
        },),);
        /**
         Indent applied to each split-out statement so continuations align with the original line.
         */
        const indent = baseIndentAt({
          sourceText,
          offset: firstRange[0],
        },);

        for (let loopIndex = 1; loopIndex < stmts
          .length; loopIndex++) {
          /**
           Previous statement; its end offset is the cut point for the inter-statement slice.
           */
          const prev = at({
            arr: stmts,
            index: loopIndex - 1,
          },);
          /**
           Current statement; its start offset is the other cut point and the reported node.
           */
          const curr = at({
            arr: stmts,
            index: loopIndex,
          },);
          /**
           End offset of the previous statement; queried once and reused below.
           */
          const [, prevEnd,] = rangeOf(prev,);
          /**
           Start offset of the current statement; queried once and reused below.
           */
          const [currStart,] = rangeOf(curr,);
          // When `curr` is nested inside `prev` (e.g. the alternate of an
          // `IfStatement` whose own range covers the whole `if/else`), the
          // slice `[prevEnd, currStart]` is negative. The fix shape is
          // wrong for this case anyway; splitting `if (a) foo(); else bar();`
          // requires inserting before `else`, not before `bar()`. Skip the
          // fix and still report.
          /**
           Whether the current statement is nested inside the previous (e.g. `if/else` alternate); blocks the autofix.
           */
          const nested = currStart <= prevEnd;
          /**
           Source slice between the two statements; comments here block the autofix.
           */
          const between = nested ? '' : sourceText.slice(
            prevEnd,
            currStart,
          );
          /**
           Whether the inter-statement slice is trivially replaceable (no nested span, only whitespace/semicolons).
           */
          const canFix = (!nested) && isOnlyWhitespaceOrSeparator({
            text: between,
            separator: ';',
          },);

          context.report({
            node: curr,
            messageId: 'exceed',
            data: { count: stmts.length, },
            ...canFix
              ? {
                fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['replaceTextRange']> {
                  return fixer.replaceTextRange(
                    [
                      prevEnd,
                      currStart,
                    ],
                    `\n${indent}`,
                  );
                },
              }
              : {},
          },);
        }
      }
      perLine.clear();
    }
    //endregion Reporting excess statements

    //region Visitor wiring
    // Every statement type is tracked, and the tally is reported on program exit.

    /**
     Build the visitor: each statement type plus the Program:exit hook.
     */
    const visitor: VisitorWithHooks = {
      'Program:exit': reportExceeding,
      ...Object.fromEntries(STATEMENT_TYPES.map(
        function asEntry(type,): [
          string,
          typeof trackStatement,
        ] {
          return [
            type,
            trackStatement,
          ];
        },
      ),),
    };
    return visitor;
  },
};
//endregion Visitor wiring
//endregion Rule

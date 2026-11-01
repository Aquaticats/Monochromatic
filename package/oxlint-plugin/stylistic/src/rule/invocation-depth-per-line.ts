import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Fixer,
  Span,
  VisitorWithHooks,
} from '@oxlint/plugins';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { buildSplitFix, } from '../utility/invocation-depth-fix.ts';
import {
  collectSpine,
  isSpineRoot,
  type SpineNode,
} from '../utility/invocation-spine.ts';

/**
 Maximum counted invocation heads allowed to start on one source line.
 */
const MAX_INVOCATIONS_PER_LINE = 2;

/**
 Enforces at most two counted invocation heads per source line on an operand spine.
 
 A counted invocation is a `CallExpression` (including optional calls),
 `NewExpression`, or `ImportExpression`. The rule walks each operand spine, the
 chain of single-argument invocations threaded through transparent wrappers
 (`await`, unary, `yield`, spread, `!`, `as`, `satisfies`, type assertion,
 optional-chaining), and counts how many invocation heads begin on each source
 line. A line carrying three or more heads fails; the highest invocation on
 that line is reported, and the fix splits its single operand onto its own
 line. The rule is threshold-only: an already-split layout passes when every
 line stays within the limit. Multi-argument calls belong to {@link argumentPerLine}
 and callee chains to {@link chainPerLine}, so this rule descends only the single
 operand and leaves those axes alone; the fix may overlap theirs and converges
 over repeated `oxlint --fix` passes.
 
 @example
 ```ts
 // Bad
 const value = a(b(c()));
 
 // Good
 const value = a(
   b(c()),
 );
 
 // Good: depth two stays on one line
 const value = a(b());
 ```
 */
export const invocationDepthPerLine: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description:
        'Allow at most two nested invocation heads on one source line; split deeper operand spines.',
      recommended: true,
    },
    messages: {
      invocationDepth:
        'No more than two nested invocations may start on one line; split the operand onto its own line.',
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
     Visitor entry for every counted invocation. Bails unless {@link isSpineRoot}
     says the node is a spine root, collects the spine via {@link collectSpine},
     then reports each source line whose spine carries more than two invocation
     heads, building each fix with {@link buildSplitFix}.
     
     @param node - candidate `CallExpression`, `NewExpression`, or `ImportExpression`
     */
    function check(node: ForeignBorrowed<Span>,): void {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- visitor nodes carry the callee/arguments/source/options/expression/argument/parent fields SpineNode reads; oxlint types them only as bare Span */
      /**
       Node narrowed to the spine view the walk reads.
       */
      const owner = node as SpineNode;
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      if (!isSpineRoot(owner,))
        return;
      /**
       Counted invocations from the root down, outermost first.
       */
      const spine = collectSpine(owner,);
      if (spine.length
        <= MAX_INVOCATIONS_PER_LINE)
        return;
      /**
       Outermost invocation per source line; the first seen is the highest.
       */
      const lineOwners = new Map<number, SpineNode>();
      /**
       Count of invocation heads per source line.
       */
      const lineCounts = new Map<number, number>();
      spine.forEach(function tally(spineNode: ForeignBorrowed<SpineNode>,): void {
        /**
         Source location of this invocation's head.
         */
        const { loc, } = spineNode;
        /**
         1-indexed source line where the invocation head begins.
         */
        const { line, } = loc.start;
        lineCounts.set(
          line,
          (lineCounts.get(line,) ?? 0)
            + 1,
        );
        if (!lineOwners.has(line,))
          lineOwners.set(
            line,
            spineNode,
          );
      },);
      lineCounts.forEach(function reportLine(
        count,
        line,
      ): void {
        if (count <= MAX_INVOCATIONS_PER_LINE)
          return;
        /**
         Highest invocation on the violating line; the fix splits its operand.
         */
        const lineOwner = nonNullishOrThrow(lineOwners.get(line,),);
        context.report({
          node: lineOwner,
          messageId: 'invocationDepth',
          fix(fixer: ForeignBorrowed<Fixer>,) {
            return buildSplitFix({
              context,
              fixer,
              owner: lineOwner,
            },);
          },
        },);
      },);
    }

    return {
      CallExpression: check,
      NewExpression: check,
      ImportExpression: check,
    };
  },
};

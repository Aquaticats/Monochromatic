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

/**
 Parent types under which a multi-declarator declaration is allowed inline.
 */
const FOR_PARENT_TYPES = new Set([
  'ForStatement',
  'ForInStatement',
  'ForOfStatement',
],);

/**
 Enforces one declarator per line in `var`/`let`/`const`/`using` declarations.
 
 Operates in `'always'` mode: every multi-declarator declaration is flagged
 whenever two consecutive declarators share a source line, regardless of
 whether either has an initializer.
 
 Declarations inside `for`/`for-in`/`for-of` init positions are skipped
 because the for-statement init slot is not a top-level statement and the
 one-per-line shape would be syntactically meaningless there.
 
 The autofix inserts `,\n<indent>` between same-line declarators. When the
 inter-declarator source slice contains a comment (anything beyond
 whitespace and `,`), the fix is suppressed so the comment is preserved;
 the violation is still reported.
 
 @example
 ```ts
 // Bad
 const a = 1, b = 2;
 let x, y;
 
 // Good
 const a = 1,
   b = 2;
 let x,
   y;
 ```
 */
export const oneVarDeclarationPerLine: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'whitespace',
    docs: {
      description:
        'Require each variable declarator to be on its own line when two or more share a declaration.',
      recommended: true,
    },
    messages: {
      expectVarOnNewline: 'Expected variable declaration to be on a new line.',
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
     Checks consecutive declarator pairs and reports those that share a line.
     
     @param node - VariableDeclaration AST node
     */
    function checkDeclaration(node: ForeignBorrowed<Span>,): void {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- oxlint Span omits declaration fields exposed by this visitor node */
      /**
       Declaration node narrowed to declarator and parent fields.
       */
      const {
        declarations,
        parent,
      } = node as Span & {
        readonly declarations: readonly Span[];
        readonly parent?: { readonly type: string; };
      };
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      if (
        (parent !== undefined)
          && FOR_PARENT_TYPES
            .has(parent.type,)
      ) {
        return;
      }

      if (declarations.length
        < 2)
        return;

      /**
       Source text is needed for line-number lookups and inter-declarator slices.
       */
      const sourceText = context.sourceCode
        .getText();
      /**
       Indentation of the declaration keyword; the fix aligns continuations relative to it.
       */
      const baseIndent = baseIndentAt({
        sourceText,
        offset: rangeOf(node,)[0],
      },);
      /**
       Continuation indent for declarators after the first; two-space convention matches the rest of the package.
       */
      const childIndent = `${baseIndent}  `;

      for (let loopIndex = 1; loopIndex < declarations
        .length; loopIndex++) {
        /**
         Previous declarator; its end offset is the cut point for the inter-declarator slice.
         */
        const prev = at({
          arr: declarations,
          index: loopIndex - 1,
        },);
        /**
         Current declarator; its start offset is the other cut point.
         */
        const curr = at({
          arr: declarations,
          index: loopIndex,
        },);
        /**
         Source range of the previous declarator, queried once.
         */
        const prevRange = rangeOf(prev,);
        /**
         Source range of the current declarator, queried once.
         */
        const currRange = rangeOf(curr,);

        if (
          lineAt({
            sourceText,
            offset: prevRange[1],
          },)
            !== lineAt({
              sourceText,
              offset: currRange[0],
            },)
        ) {
          continue;
        }

        /**
         Source slice between the two declarators; comments here block the autofix.
         */
        const between = sourceText.slice(
          prevRange[1],
          currRange[0],
        );
        /**
         Whether the inter-declarator slice contains only whitespace and commas (i.e. no comments to preserve).
         */
        const canFix = isOnlyWhitespaceOrSeparator({
          text: between,
          separator: ',',
        },);

        context.report({
          node: curr,
          messageId: 'expectVarOnNewline',
          ...canFix
            ? {
              fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['replaceTextRange']> {
                return fixer.replaceTextRange(
                  [
                    prevRange[1],
                    currRange[0],
                  ],
                  `,\n${childIndent}`,
                );
              },
            }
            : {},
        },);
      }
    }

    return {
      VariableDeclaration: checkDeclaration,
    };
  },
};

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Span, } from '@oxlint/plugins';

/**
 Parameters for {@link hasParens}.
 */
export type HasParensParams = {
  /**
   AST node whose surrounding source is checked.
   */
  readonly child: Span;
  /**
   Full file source text.
   */
  readonly sourceText: string;
};

/**
 Determines whether an AST node's source text is wrapped in matching
 parentheses, allowing whitespace between the parens and the node span.
 
 oxc/ESTree expressions do not include surrounding parens in their
 `start`/`end` range, so this helper peeks at the source text immediately
 before `start` and after `end` and tolerates intermediate whitespace
 (e.g. `( a + b )`).
 
 `String.prototype.trimEnd` / `trimStart` strip exactly the `\s*` runs the
 prior regex anchored against; using them keeps the check strictly linear
 with no regex backtracking surface.
 
 @returns true if the node is wrapped in `( ... )` at its boundary
 
 @example
 ```ts
 // For source `(a + b) * c`, with child = BinaryExpression `a + b`:
 hasParens({ child, sourceText, }); // true
 ```
 */
export function hasParens({
  child,
  sourceText,
}: ForeignBorrowed<HasParensParams>,): boolean {
  /**
   Source slice before the operand; trailing whitespace stripped so the `(` lands at the end.
   */
  const before = sourceText
    .slice(
    0,
    child.start,
    )
    .trimEnd();
  /**
   Source slice after the operand; leading whitespace stripped so the `)` lands at the start.
   */
  const after = sourceText
    .slice(child.end,)
    .trimStart();
  return before.endsWith('(',)
    && after
      .startsWith(')',);
}

/**
 What one node invokes, whichever syntax spells the invocation.
 
 The activation walk matched `CallExpression` and nothing else, so two other invoking syntaxes left
 their callee unactivated and its body unscanned. That is the cost, and it is not about what either
 syntax receives: a store inside an unactivated closure is attributed to nobody.
 
 Measured, each against the same body written as an ordinary call, which recorded `opaque=[0]`:
 
 ```ts
 const storingTag = (_strings: TemplateStringsArray,): void => { holder.kept = gotten.row; };
 storingTag``;
 
 class BodyStorer {
   constructor() {
     holder.kept = gotten.row;
   }
 }
 void new BodyStorer();
 ```
 
 Both recorded nothing. A field initializer in the construction's position is charged already, so the
 gap there is the constructor body rather than the class.
 
 Named as its own module because the question is one concept and the activation walk had reached its
 line budget. Adding a fourth invoking syntax is a clause here rather than an edit at four call sites,
 which is the shape the tagged-template fix was written for and the construction fix then reused.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isCallExpression,
  isDecorator,
  isNewExpression,
  isTaggedTemplateExpression,
  isTemplateExpression,
} from 'typescript/ast/is';

/**
 Sentinel for a node that invokes nothing.
 */
export const INVOKES_NOTHING: unique symbol = Symbol('node invokes nothing',);

/**
 Names what one node invokes and what it hands over, whichever syntax spells the invocation.
 
 A tagged template and a construction are both calls, and this walk saw only `CallExpression`, so
 neither the tag nor the constructor was activated and neither body was scanned. Measured, with the same closure written both ways:
 
 ```ts
 const storingTag = (_strings: TemplateStringsArray,): void => { holder.kept = gotten.row; };
 storingTag``;
 ```
 
 recorded nothing for `gotten`, while an identical closure invoked as `storingCall()` recorded
 `opaque=[0]`. So the defect is not about what a tag receives; it is that an unseen invocation leaves
 the closure unactivated, and an unactivated body's store is attributed to nobody.
 
 Interpolated values answer as actuals, because a tag receives them exactly as a call receives
 arguments. The strings array is not among them: it is built at the call site rather than handed in.
 
 @param node - Node that may invoke something.
 
 @returns callee and actuals, or the sentinel when nothing is invoked.
 
 @example
 ```ts
 invokedParts({ node });
 ```
 */
export function invokedParts({ node, }: { readonly node: Node; },):
  | {
    readonly callee: Node;
    readonly actuals: readonly Node[];
  }
    | typeof INVOKES_NOTHING
  {
  if (isCallExpression(node,))
    return {
      callee: node.expression,
      actuals: node.arguments,
    };
  /* A construction invokes a constructor, and this walk saw neither it nor a tag. Measured: a class
   * declared inside the callable, constructed with no arguments, whose constructor stored the caller's
   * row outward, recorded nothing at all and read identically to a control storing a fresh row. A field
   * initializer in the same position is charged already, so the gap is the constructor body rather than
   * the class. */
  /* A decorator invokes its expression against the decorated declaration, and the bare form writes no
   * call, so nothing recognised it. Recognition alone is not enough and neither is the ancestry repair
   * beside it: each was measured as a no-op on its own, because a decorator is gated out by the
   * declaration it decorates and, once past the gate, still needs to be seen as an invocation. Both
   * together charge the subject.
   *
   * No actuals. A decorator receives the decorated target and a context object, neither of which the
   * enclosing callable handed over, so treating them as actuals would claim a handoff nobody wrote. The
   * factory form `@factory(value,)` needs nothing here: its expression is a call the clause above answers
   * when the walk reaches it. */
  if (isDecorator(node,))
    return {
      callee: node.expression,
      actuals: [],
    };
  if (isNewExpression(node,))
    return {
      callee: node.expression,
      actuals: node.arguments ?? [],
    };
  if (!isTaggedTemplateExpression(node,))
    return INVOKES_NOTHING;
  return {
    callee: node.tag,
    actuals: isTemplateExpression(node.template,)
      ? node.template
        .templateSpans
        .map(function spanValue(span,): Node {
          return span.expression;
        },)
      : [],
  };
}

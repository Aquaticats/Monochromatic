/**
 Whether a name a walk stopped at still holds what its declaration says.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isBinaryExpression,
  isIdentifier,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { collectAstNodes, } from './effect-summary-model.ts';

/**
 Assignment operators that replace what a name holds rather than reading it.
 
 Compound forms are included even though none of them can produce a container: a walk
 asking whether a name still holds its declared value is answered by any operator that
 writes the name, and enumerating only `=` would make the answer depend on which spelling
 an unrelated future edit chose.
 */
const BINDING_WRITING_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.EqualsToken,
  SyntaxKind.PlusEqualsToken,
  SyntaxKind.MinusEqualsToken,
  SyntaxKind.AsteriskEqualsToken,
  SyntaxKind.SlashEqualsToken,
  SyntaxKind.PercentEqualsToken,
  SyntaxKind.AmpersandAmpersandEqualsToken,
  SyntaxKind.BarBarEqualsToken,
  SyntaxKind.QuestionQuestionEqualsToken,
],);

/**
 Tests whether a name is written anywhere inside a body.
 
 The endpoint test a walk proving absence needs, and the one a declaration cannot supply.
 `bindingIsReassignable` answers for a `let` because the declaration itself carries the
 answer, and a parameter carries none: it is declared once and may still be pointed
 somewhere else by any statement in the body.
 
 That matters because the ownership marker does not stop it. `ForeignBorrowed<Value>` is
 `Value & { readonly [MARKER]?: true; }` and the marker property is optional, so a
 `ForeignBorrowed<Row[]>` value assigns to a plain `Row[]` parameter with no error at all:
 
 ```ts
 function expose(owned: Row[], foreign: ForeignBorrowed<Row[]>,): Row[] {
   owned = foreign;
   return owned.slice(0,);
 }
 ```
 
 Classifying `owned` reads the declared type and finds nothing foreign, while the value
 reaching `slice` is the foreign one. The type system permits the assignment, so refusing
 a written name is what has to catch it.
 
 Scanned rather than resolved through the checker, because the question is about this body
 and a symbol's write sites are not something a declaration reports.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param body - Body the walk is reasoning within.
 
 @param node - Name the walk stopped at.
 
 @returns whether any statement in that body writes this name, unresolved counting as yes.
 
 @example
 ```ts
 bindingAssignedWithin({ project, body, node: base });
 ```
 */
export function bindingAssignedWithin({
  project,
  body,
  node,
}: {
  readonly project: Project;
  readonly body: Node;
  readonly node: Node;
},): boolean {
  if (!isIdentifier(node,))
    /* Not a name, so nothing can be assigned to it and the caller's own classification
     * stands. A property access is answered by the classifier reading what it accesses. */
    return false;
  /**
   Binding this name stands for.
   */
  const symbol = project.checker
    .getSymbolAtLocation(node,);
  if (symbol === undefined)
    /* An unresolved name is one nothing can be proven about, which for a walk proving
     * absence is the same as knowing it is written. */
    return true;
  return collectAstNodes(body,)
    .some(function writesBinding(candidate,): boolean {
      if (!isBinaryExpression(candidate,))
        return false;
      if (!BINDING_WRITING_OPERATORS.has(candidate.operatorToken
        .kind,))
        return false;
      /**
       Name being written, when this assignment targets one.
       */
      const target = candidate.left;
      if (!isIdentifier(target,))
        return false;
      /**
       Binding this assignment writes.
       */
      const targetSymbol = project.checker
        .getSymbolAtLocation(target,);
      return targetSymbol?.id === symbol.id;
    },);
}

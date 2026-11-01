/**
 Which value a local binding was declared with, and whether it can still move.
 
 @module
 */

import {
  type Node,
  NodeFlags,
  type VariableDeclaration,
} from 'typescript/ast';
import {
  isIdentifier,
  isVariableDeclaration,
  isVariableDeclarationList,
} from 'typescript/ast/is';
import type {
  Checker,
  Project,
} from 'typescript/sync';

/**
 Sentinel when a name stands for no local declaration carrying a value.
 
 A sentinel rather than `undefined`, since this repo models absence without nullish
 unions.
 */
export const NO_BINDING_INITIALIZER: unique symbol = Symbol(
  'name declares no local initializer',
);

/**
 Resolves the value a name was declared with, when it names one local declaration.
 
 One definition of the declaration hop, shared rather than repeated. Both walks that need
 it ask a different question afterwards, and that is exactly why the step itself has to be
 the same: `containerElementReceiver` follows it looking for a call carrying a container
 relation, while `expressionElementOrigins` follows it looking for anywhere a relation
 might be, including inside a selector. When the hop lived only in the first, the two could
 not compose, and `const copy = cond ? rows.slice() : [];` reached neither answer.
 
 Only a declaration initializer is resolved, never a later assignment. That is the
 over-attributing direction and deliberate: a reassigned local keeps answering for the
 container it was declared with, which costs precision and never an offer.
 
 That reasoning holds only while the hop is used to *add* origins. A step proving something
 absent reads it backwards and needs `constBindingInitializer` instead.
 
 @param project - TypeScript project resolving declarations.
 
 @param checker - TypeScript checker resolving binding symbols.
 
 @param node - Name whose declared value is wanted.
 
 @returns declared initializer, or sentinel when there is no single local one.
 
 @example
 ```ts
 bindingDeclarationInitializer({ project, checker, node: receiver });
 ```
 */
export function bindingDeclarationInitializer({
  project,
  checker,
  node,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly node: Node;
},): Node | typeof NO_BINDING_INITIALIZER {
  /**
   Declaration this name was introduced by, when it names one local variable.
   */
  const declaration = declaredVariable({
    project,
    checker,
    node,
  },);
  if (declaration === NO_BINDING_INITIALIZER)
    return NO_BINDING_INITIALIZER;
  return declaration.initializer ?? NO_BINDING_INITIALIZER;
}

/**
 Resolves the single local variable declaration a name stands for.
 
 Split out so the plain hop and the `const`-only hop share one resolution and differ in one
 test, rather than each restating symbol lookup, declaration count and node kind.
 
 A binding with more than one declaration answers the sentinel. Merged declarations mean the
 name does not stand for one value, and picking one of them would assert an identity nothing
 here proves.
 
 @param project - TypeScript project resolving declarations.
 
 @param checker - TypeScript checker resolving binding symbols.
 
 @param node - Name whose declaration is wanted.
 
 @returns resolved declaration, or sentinel when there is no single local one.
 
 @example
 ```ts
 declaredVariable({ project, checker, node: receiver });
 ```
 */
function declaredVariable({
  project,
  checker,
  node,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly node: Node;
},): VariableDeclaration | typeof NO_BINDING_INITIALIZER {
  if (!isIdentifier(node,))
    return NO_BINDING_INITIALIZER;
  /**
   Binding this name stands for.
   */
  const symbol = checker.getSymbolAtLocation(node,);
  if ((symbol === undefined)
    || (symbol.declarations
      .length
      !== 1))
    return NO_BINDING_INITIALIZER;
  /**
   Declaration the binding was introduced by, resolved into this project.
   */
  const declaration = symbol.declarations[0]
    ?.resolve(project,);
  if ((declaration === undefined)
    || (!isVariableDeclaration(declaration,)))
    return NO_BINDING_INITIALIZER;
  return declaration;
}

/**
 Tests whether a declaration cannot be pointed somewhere else after it is written.
 
 The guard that makes a declaration answer for every later use of the name rather than only
 for the moment it was written. `let stack = [root,]; stack = config.rows; stack.pop();`
 holds a container this callable built at its declaration and one it was given by the time
 the member runs, so reading the declaration as the answer loses that write. A `const`
 cannot move.
 
 Shared rather than repeated, because two steps need exactly this and for the same reason:
 the container record consults it before suppressing a mutation charge, and the
 returned-result discharge consults it before accepting a declaration initializer as proof
 that a receiver holds no foreign-owned state.
 
 @param declaration - Variable declaration under test.
 
 @returns whether the binding is declared `const`.
 
 @example
 ```ts
 declaredConst({ declaration });
 ```
 */
export function declaredConst({
  declaration,
}: {
  readonly declaration: VariableDeclaration;
},): boolean {
  /**
   List the declaration belongs to, carrying the `const` flag for every name in it.
   */
  const list = declaration.parent;
  if (!isVariableDeclarationList(list,))
    return false;
  return (list.flags & NodeFlags.Const) !== 0;
}

/**
 Tests whether a name stands for a local binding that can still be pointed elsewhere.
 
 Asked as its own question rather than folded into the hop, because the hop's sentinel
 cannot carry the answer. `bindingDeclarationInitializer` reports "no initializer to
 follow" for a parameter and for a reassignable local alike, and those need opposite
 treatment in a walk proving absence: a parameter is the ownership-bearing node the walk
 exists to classify, while a reassignable local names a value that may not be the one it
 was declared with. A hop that merged them would stop at both and then classify both,
 which is the unsound half restated rather than fixed.
 
 `let held = owned; held = foreign; return held.filter(keep,);` is the program. Its type is
 the one `owned` gave it, so classifying the name finds nothing foreign, and following its
 initializer finds nothing foreign either. Only refusing it outright is correct.
 
 @param project - TypeScript project resolving declarations.
 
 @param checker - TypeScript checker resolving binding symbols.
 
 @param node - Name under test.
 
 @returns whether this name is a local variable not declared `const`.
 
 @example
 ```ts
 bindingIsReassignable({ project, checker, node: receiver });
 ```
 */
export function bindingIsReassignable({
  project,
  checker,
  node,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly node: Node;
},): boolean {
  /**
   Declaration this name was introduced by, when it names one local variable.
   */
  const declaration = declaredVariable({
    project,
    checker,
    node,
  },);
  if (declaration === NO_BINDING_INITIALIZER)
    return false;
  return !declaredConst({ declaration, },);
}

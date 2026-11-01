/**
 Which local bindings hold a container this callable built rather than one it was given.
 
 @module
 */

import type {
  Node,
  VariableDeclaration,
} from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isIdentifier,
  isObjectLiteralExpression,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { declaredConst, } from './effect-binding-initializer.ts';

/**
 Passes the alias walk takes before it stops, a backstop rather than the terminator.
 
 The walk settles on its own, because the holder set only grows and a pass that adds
 nothing ends it. This bounds a pathological chain rather than deciding the answer.
 */
const HOLDER_PASS_LIMIT_MARGIN = 1;

/**
 Sentinel for a name that declares no symbol this walk can identify.
 */
const NO_BINDING_SYMBOL: unique symbol = Symbol(
  'name declares no resolvable binding symbol',
);

/**
 Reads the symbol id a binding name declares.
 
 @param project - TypeScript project resolving the binding symbol.
 
 @param name - Binding name being identified.
 
 @returns symbol id, or sentinel when the name declares no resolvable symbol.
 
 @example
 ```ts
 bindingSymbolId({ project, name: declaration.name });
 ```
 */
function bindingSymbolId({
  project,
  name,
}: {
  readonly project: Project;
  readonly name: Node;
},): number | typeof NO_BINDING_SYMBOL {
  if (!isIdentifier(name,))
    return NO_BINDING_SYMBOL;
  /**
   Symbol the name declares, absent when the checker resolves none.
   */
  const symbol = project.checker
    .getSymbolAtLocation(name,);
  if (symbol === undefined)
    return NO_BINDING_SYMBOL;
  return symbol.id;
}

/**
 Tests whether a name already stands for a recorded container binding.
 
 @param project - TypeScript project resolving the binding symbol.
 
 @param holders - Bindings recorded so far.
 
 @param name - Name being tested.
 
 @returns whether the name is a recorded holder.
 
 @example
 ```ts
 holderAlready({ project, holders, name: initializer });
 ```
 */
function holderAlready({
  project,
  holders,
  name,
}: {
  readonly project: Project;
  readonly holders: ReadonlySet<number>;
  readonly name: Node;
},): boolean {
  /**
   Symbol the name stands for, absent when it resolves to none.
   */
  const symbolId = bindingSymbolId({
    project,
    name,
  },);
  if (symbolId === NO_BINDING_SYMBOL)
    return false;
  return holders.has(symbolId,);
}

/**
 Collects local bindings whose value is a container this callable constructed.
 
 Separate from the origin map on purpose, and additive to it. `expressionValueOrigins`
 answers which caller parameters an expression's value can reach, and `[root,]` really
 can reach `root`, so recording that is correct for every consumer asking that question.
 One consumer asks a different one. `recordCollectionMemberEffect` charges the receiver's
 origins when a member restructures its receiver, and `stack.pop()` restructures the
 fresh array rather than the parameter inside it: a container reaching the parameter is
 not the parameter reaching the container.
 
 Rather than emptying the origins, which would lose every write made *through* the
 container and is costed in `doc/planning/prefer-readonly-container-value-provenance.md`,
 this records the provenance shape beside them and lets exactly one charge consult it.
 Nothing else changes, so `const stack = [root,]; stack[0].label = x` keeps its
 attribution through the element path.
 
 Three conditions, each load-bearing. The name must be a plain identifier, because a
 destructuring pattern binds what a container holds rather than the container. The
 declaration must be `const`, so the value cannot be replaced by one this callable did
 not build. The initializer must be an array or object literal written here, or an
 identifier already recorded, which is what carries the record across
 `const alias = stack;` the way origins already cross it.
 
 A binding initialized from a property, a parameter, or a call is deliberately absent.
 `const inner = config.rows; inner.push(row,);` writes a container the caller owns, and
 the charge on it has to stand.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param variableDeclarations - Body declarations eligible for the record.
 
 @returns symbol ids of bindings holding a container built here.
 
 @example
 ```ts
 containerLiteralHolderSymbolIds({ project, variableDeclarations });
 ```
 */
export function containerLiteralHolderSymbolIds({
  project,
  variableDeclarations,
}: {
  readonly project: Project;
  readonly variableDeclarations: readonly VariableDeclaration[];
},): ReadonlySet<number> {
  /**
   Bindings recorded so far, growing until a pass adds nothing.
   */
  const holders = new Set<number>();
  /**
   Convergence state, so an alias chain in reverse source order still settles.
   */
  const walk = {
    changed: true,
    pass: 0,
  };
  while (walk.changed
    && (walk.pass <= (variableDeclarations.length + HOLDER_PASS_LIMIT_MARGIN))) {
    walk.changed = false;
    walk.pass++;
    for (const declaration of variableDeclarations) {
      /**
       Value this declaration binds, absent for a declaration without one.
       */
      const { initializer, } = declaration;
      if ((initializer === undefined) || (!declaredConst({ declaration, },)))
        continue;
      /**
       Whether the bound value is a container built at this declaration or aliased.
       */
      const holdsContainer = isArrayLiteralExpression(initializer,)
        || isObjectLiteralExpression(initializer,)
        || (isIdentifier(initializer,)
          && holderAlready({
            project,
            holders,
            name: initializer,
          },));
      if (!holdsContainer)
        continue;
      /**
       Binding this declaration introduces, when it names one symbol.
       */
      const symbolId = bindingSymbolId({
        project,
        name: declaration.name,
      },);
      if ((symbolId === NO_BINDING_SYMBOL) || holders.has(symbolId,))
        continue;
      holders.add(symbolId,);
      walk.changed = true;
    }
  }
  return holders;
}

/**
 Tests whether an expression names a binding holding a container built here.
 
 @param project - TypeScript project resolving the binding symbol.
 
 @param containerLiteralHolders - Bindings recorded as holding a constructed container.
 
 @param node - Receiver expression under test.
 
 @returns whether the receiver is such a binding.
 
 @example
 ```ts
 receiverHoldsConstructedContainer({ project, containerLiteralHolders, node: receiver });
 ```
 */
export function receiverHoldsConstructedContainer({
  project,
  containerLiteralHolders,
  node,
}: {
  readonly project: Project;
  readonly containerLiteralHolders: ReadonlySet<number>;
  readonly node: Node;
},): boolean {
  /**
   Symbol the receiver names, absent when it is not a plain binding reference.
   */
  const symbolId = bindingSymbolId({
    project,
    name: node,
  },);
  if (symbolId === NO_BINDING_SYMBOL)
    return false;
  return containerLiteralHolders.has(symbolId,);
}

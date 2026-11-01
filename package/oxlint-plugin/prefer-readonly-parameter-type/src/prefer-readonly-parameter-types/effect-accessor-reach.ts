/**
 Callables a body reaches by reading a property rather than by calling anything.
 
 The reach walk follows calls, so it answers for a closure that calls a sibling and misses one that
 reads an accessor. Measured, with the configuration offered:
 
 ```ts
 const holder = {
   get row(): Row {
     return gottenOut.row;
   },
 };
 registry.keep((): Row => holder.row,);
 ```
 
 The handed closure names `holder` and nothing else. Resolving `holder` finds no parameter origin,
 because it is a local rather than a parameter, and the call walk finds no call, because a property
 read is not one. So the reach walk answered empty while reading `holder.row` runs a body that
 hands back the caller's row.
 
 A getter is the sharp case and the walk does not stop there. Any callable a literal declares is
 collected, accessors and methods alike, because a property read can hand a method onward as a
 value just as it can run a getter. Which property was read is not tracked, exactly as the
 aggregate descent elsewhere declines to track keys: narrowing to one member would need a claim
 this walk cannot support, and taking all of them can only add an origin.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isClassDeclaration,
  isClassExpression,
  isElementAccessExpression,
  isIdentifier,
  isObjectBindingPattern,
  isNewExpression,
  isObjectLiteralExpression,
  isParameterDeclaration,
  isPropertyAccessExpression,
  isSpreadAssignment,
  isSpreadElement,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { reachableValueSources, } from './effect-result-reach.ts';
import {
  collectAstNodes,
  isEffectCallableDeclaration,
} from './effect-summary-model.ts';

/**
 Collects every callable a body can reach through a property read.
 
 @param project - TypeScript project resolving what a receiver holds.
 
 @param within - Body scanned for property reads.
 
 @param fileName - File a reached callable must share, bounding work rather than deciding an
 answer, exactly as the call walk's bound does.
 
 @returns callables declared by the aggregates those reads name.
 
 @example
 ```ts
 accessedCallables({ project, within, fileName });
 ```
 */
export function accessedCallables({
  project,
  within,
  fileName,
}: {
  readonly project: Project;
  readonly within: Node;
  readonly fileName: string;
},): readonly Node[] {
  return collectAstNodes(within,)
    .flatMap(function declaredCallables(node,): readonly Node[] {
      return readReceivers({ node, },)
        .flatMap(function callablesOfReceiver(receiver,): readonly Node[] {
          return reachableValueSources({
            project,
            node: receiver,
          },)
            .flatMap(function membersOfSource(source,): readonly Node[] {
              return aggregateCallables({
                project,
                source,
              },);
            },);
        },);
    },)
    .filter(function sameFile(callable,): boolean {
      /**
       File the reached callable is written in.
       */
      const { fileName: reachedFileName, } = callable.getSourceFile();
      return reachedFileName === fileName;
    },);
}

/**
 Names the expressions a node reads a property off, whichever way it spells the read.
 
 Plain property access was the only recognised form, and three others run a getter just as surely.
 Measured, each offering the configuration its getter hands out while the plain form charged it:
 
 ```ts
 registry.keep((): Row => holder['row'],);
 registry.keep((): Row => { const { row, } = holder; return row; },);
 ```
 
 Both spread kinds are included, though not on identical grounds: an object spread runs every getter,
 while an array or argument spread runs `Symbol.iterator` instead. What they share is what this walk
 asks about, reaching a callable the aggregate declares without writing a call.
 
 Which property was read is not tracked here, exactly as the aggregate descent declines to track keys,
 so an element access with a computed key needs no special handling: the receiver is what matters and
 taking every member can only add an origin.
 
 @param node - Node that may read a property off something.
 
 @returns receiver expressions whose getters that read can run.
 
 @example
 ```ts
 readReceivers({ node });
 ```
 */
function readReceivers({ node, }: { readonly node: Node; },): readonly Node[] {
  if (isPropertyAccessExpression(node,) || isElementAccessExpression(node,))
    return [node.expression,];
  /* Both spread node kinds, for two different measured reasons rather than one shared one. An object
   * spread runs every getter the source declares; an array or argument spread runs its `Symbol.iterator`
   * and no getter at all. Measured directly: `{ ...holder }` logged `getter`, `[...holder]` logged
   * `iterator`. Either way the read reaches a callable the aggregate declares without writing a call,
   * which is the relation this walk exists to answer, so both kinds name their receiver. */
  if (isSpreadAssignment(node,) || isSpreadElement(node,))
    return [node.expression,];
  /* A pattern runs a getter for every name it binds, and the receiver is what the pattern was filled
   * from. Both spellings answer here, `const { row, } = holder` and a parameter defaulting to one,
   * because each is a declaration whose name is a pattern and whose initializer is the receiver. */
  if ((isVariableDeclaration(node,) || isParameterDeclaration(node,))
    && isObjectBindingPattern(node.name,)
    && (node.initializer !== undefined))
    return [node.initializer,];
  return [];
}

/**
 Names every callable one authored aggregate declares.
 
 Only an authored literal or class answers. A value read out of something else is reached by the
 layers the normalisation walk already strips, and guessing at the members of anything else would
 claim a declaration the source never wrote.
 
 A class declaration answers beside a class expression. Excluding it was arbitrary rather than
 principled: a class declared inside the callable being summarised closes over its parameters exactly
 as an expression form does, and measured, a getter on such a class handed the caller's row out while
 the offer stood.
 
 @param source - Expression that may be an authored aggregate.
 
 @returns callables it declares.
 
 @example
 ```ts
 aggregateCallables({ source });
 ```
 */
function aggregateCallables({
  project,
  source,
}: {
  readonly project: Project;
  readonly source: Node;
},): readonly Node[] {
  /* A construction names its class rather than being one. Reading a property off an instance runs a
   * getter the class declares, so following the construction to that class is the same relation this
   * answers for a literal. Measured: a getter on a class declared inside the callable handed the
   * caller's row out while the offer stood, because the receiver resolved to `new Holder()` and a
   * construction is not an aggregate. */
  if (isNewExpression(source,))
    return constructedClassMembers({
      project,
      constructed: source.expression,
    },);
  if (!(isObjectLiteralExpression(source,)
    || isClassExpression(source,)
    || isClassDeclaration(source,)))
    return [];
  return collectAstNodes(source,)
    .filter(function isCallable(node,): boolean {
      return isEffectCallableDeclaration(node,);
    },);
}

/**
 Names every callable the class one construction refers to declares.
 
 @param project - TypeScript project resolving the constructed name.
 
 @param constructed - Expression naming the class being constructed.
 
 @returns callables that class declares, empty when nothing authored answers.
 
 @example
 ```ts
 constructedClassMembers({ project, constructed });
 ```
 */
function constructedClassMembers({
  project,
  constructed,
}: {
  readonly project: Project;
  readonly constructed: Node;
},): readonly Node[] {
  /**
   Symbol the constructed name resolves to.
   */
  const symbol = isIdentifier(constructed,)
    ? project.checker
      .getResolvedSymbol(constructed,)
    : project.checker
      .getSymbolAtLocation(constructed,);
  /**
   First declaration that symbol names.
   */
  const first = symbol
    ?.declarations
    .at(0,);
  /**
   Declaration the symbol names, preferring the one carrying its value.
   */
  const declared = symbol
    ?.valueDeclaration
    ?? first;
  /**
   Declaration resolved into the project owning it.
   */
  const declaration = declared
    ?.resolve(project,);
  if ((declaration === undefined)
    || (!(isClassDeclaration(declaration,) || isClassExpression(declaration,))))
    return [];
  return collectAstNodes(declaration,)
    .filter(function isCallable(node,): boolean {
      return isEffectCallableDeclaration(node,);
    },);
}

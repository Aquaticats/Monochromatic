/**
 Active nested-closure selection for callable effect scans.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isBinaryExpression,
  isDecorator,
  isIdentifier,
  isReturnStatement,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  expressionHasParameterOrigin,
} from './effect-binding-origins.ts';
import {
  INVOKES_NOTHING,
  invokedParts,
} from './effect-invoked-parts.ts';
import { possibleValueNodes, } from './effect-possible-values.ts';
import {
  callableKey,
  type EffectCallableDeclaration,
  collectAstNodes,
  isEffectCallableDeclaration,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Activates callables reachable through escaped expression containers and aliases.
 
 @param project - TypeScript project resolving variable aliases.
 
 @param node - Passed or returned expression whose callables escape.
 
 @param nestedKeys - Callable declarations nested under outer body.
 
 @param activeKeys - Accumulator receiving escaped callable keys.
 
 @mutates activeKeys - Adds every reachable nested callable key.
 */
function activateEscapedCallables({
  project,
  node,
  nestedKeys,
  activeKeys,
}: {
  readonly project: Project;
  readonly node: Node;
  readonly nestedKeys: ReadonlySet<string>;
  readonly activeKeys: Set<string>;
}): void {
  /**
   Explicit work stack for container and alias traversal.
   */
  const stack: Node[] = [node,];
  /**
   Stable node keys prevent cyclic object and variable aliases.
   */
  const visited = new Set<string>();
  while (stack.length > 0) {
    /**
     Next escaped expression node.
     */
    const current = stack.pop();
    if (current === undefined)
      continue;
    /**
     Stable source span for cycle detection.
     */
    const currentKey = `${current.getSourceFile()
      .fileName}:${String(current.pos,)}:${String(current.end,)}`;
    if (visited.has(currentKey,))
      continue;
    visited.add(currentKey,);
    if (isEffectCallableDeclaration(current,)) {
      /**
       Stable callable key activated as escaped value.
       */
      const key = callableKey(current,);
      if (nestedKeys.has(key,))
        activeKeys.add(key,);
      continue;
    }
    if (isIdentifier(current,)) {
      /**
       Symbol reached through identifier alias.
       */
      const symbol = project.checker
        .getResolvedSymbol(current,);
      /**
       Declaration reached through identifier alias.
       */
      const declaration = (symbol?.valueDeclaration
        ?? symbol?.declarations
          .at(0,))?.resolve(project,);
      if ((declaration !== undefined)
        && isEffectCallableDeclaration(declaration,)) {
        stack.push(declaration,);
        continue;
      }
      if ((declaration !== undefined)
        && isVariableDeclaration(declaration,)
        && (declaration.initializer !== undefined)) {
        stack.push(declaration.initializer,);
        continue;
      }
    }
    current.forEachChild(function addEscapedChild(child,): undefined {
      stack.push(child,);
      return undefined;
    },);
  }
}

/**
 Tests that node is enclosed only by active nested closures.
 
 @param node - Descendant whose closure ancestry is checked.
 
 @param boundary - Declaration owning the scan, which parameter initializers ascend to rather
 than to the body.
 
 @param activeKeys - Nested callables proven active.
 
 @returns whether every enclosing nested closure is active.
 */
function insideOnlyActiveClosures({
  node,
  boundary,
  activeKeys,
}: {
  readonly node: Node;
  readonly boundary: Node;
  readonly activeKeys: ReadonlySet<string>;
}): boolean {
  /**
   Parent cursor ascending through every nested closure.
   */
  const cursor: { current: Node; } = { current: node.parent, };
  /* A decorator is lexically inside the declaration it decorates and does not run inside it. It runs
   * when the class is defined, whether or not the decorated member is ever called, so gating it on that
   * member's activity is wrong. Measured: a decorator storing the caller's row recorded nothing, because
   * the ascent reached the undecorated method, found it inactive and stopped.
   *
   * So the ascent skips exactly one declaration after passing out of a decorator, and keeps gating on
   * everything above it, which is what still excludes a decorator written inside an inactive closure. */
  /**
   Whether the ascent has just left a decorator, whose next declaration it decorates rather than runs in.
   */
  const leftDecorator: { pending: boolean; } = { pending: isDecorator(node,), };
  while (cursor.current !== boundary) {
    if (isDecorator(cursor.current,))
      leftDecorator.pending = true;
    else if (isEffectCallableDeclaration(cursor.current,)) {
      if (leftDecorator.pending)
        leftDecorator.pending = false;
      else if (!activeKeys.has(callableKey(cursor.current,),))
        return false;
    }
    if (cursor.current
      .parent
      === cursor.current)
      return false;
    cursor.current = cursor.current
      .parent;
  }
  return true;
}

/**
 Selects outer body nodes plus nested closures that may execute or escape.
 
 Direct invocation,
 direct callback arguments,
 and direct returns activate nested closure body.
 Unreferenced local function declarations remain outside outer callable effect.
 
 @param project - TypeScript project resolving callable references.
 
 @param body - Outer callable body.
 
 @param boundary - Declaration owning the scan, which parameter initializers ascend to rather
 than to the body.
 
 @param parameterInitializerNodes - Nodes of every parameter default, joining the same universe
 the body contributes so one ancestry filter gates both.
 
 @param bindingOriginBySymbolId - Binding symbols mapped to source parameters.
 
 @returns effect-relevant body nodes.
 
 @example
 ```ts
 const nodes = activeCallableBodyNodes({
   project,
   body,
   boundary,
   parameterInitializerNodes,
   bindingOriginBySymbolId,
 });
 ```
 */
export function activeCallableBodyNodes({
  project,
  body,
  boundary,
  parameterInitializerNodes,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly body: Node;
  readonly boundary: Node;
  readonly parameterInitializerNodes: readonly Node[];
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
}): readonly Node[] {
  /* Parameter initializers join the universe rather than bypassing it. They used to be added to the
   * selected set unconditionally, on the correct grounds that an initializer expression runs on
   * entry whenever the argument is omitted, and that took every callable packaged inside one with
   * them. So a default closure nothing ever invokes had its write attributed, which is a fact about
   * a body that never runs.
   *
   * The ancestry filter answers both halves without a special case. An initializer's own expression
   * has no callable between it and the declaration, so it passes. A callable packaged inside one is
   * gated on being activated, and a call to the parameter now resolves through its default, so
   * `closureDefaultInvoked` keeps the write it genuinely performs while
   * `closureDefaultNeverInvoked` loses the one it never reaches. */
  /**
   Complete descendants used to discover nested declarations and activations.
   */
  const allNodes = [
    ...parameterInitializerNodes,
    ...collectAstNodes(body,),
  ];
  /**
   Stable keys for every nested callable declaration.
   */
  const nestedKeys = new Set(
    allNodes
      .filter(function nestedCallable(node,): node is EffectCallableDeclaration {
        return isEffectCallableDeclaration(node,);
      },)
      .map(function nestedKey(declaration,): string {
        return callableKey(declaration,);
      },),
  );
  /**
   Nested callable keys proven invoked or escaped.
   */
  const activeKeys = new Set<string>();
  /* Gated on ancestry and iterated to a fixed point, rather than scanned once over every node.
   * The scan visited every node in the body, so a call written inside a closure that never runs
   * activated its target anyway, and the target's own body was then read as though the enclosing
   * callable had run it. Measured: `declareWritingSiblingUncalled` recorded `mutated=[0]` for a
   * write the callable never reaches, and `storeCallingDeclaredSibling` recorded a returned
   * origin it never returns.
   *
   * Two forms behave differently and the first probe of this used the wrong one. A sibling bound
   * to a `const` arrow did not reproduce it, because overload resolution answers with the arrow
   * and its key matched nothing the scan had reached; a sibling written as a function declaration
   * did. So the earlier note recording that the consequence did not reproduce was true of the
   * shape it tested and false of the defect.
   *
   * Termination: `activeKeys` only grows, and it is bounded by `nestedKeys`, which is fixed. */
  /**
   Whether the last pass activated anything, so the loop knows to look again.
   */
  const state: { changed: boolean; } = { changed: true, };
  while (state.changed) {
    state.changed = false;
    /**
     Keys already active before this pass, so growth is what ends the loop.
     */
    const knownBefore = activeKeys.size;
    activationSites({
      project,
      bindingOriginBySymbolId,
      allNodes,
      boundary,
      nestedKeys,
      activeKeys,
    },);
    state.changed = activeKeys.size !== knownBefore;
  }
  return allNodes.filter(function activeNode(node,): boolean {
    return insideOnlyActiveClosures({
      node,
      boundary,
      activeKeys,
    },);
  },);
}

/**
 Activates every callable reachable from a site the enclosing callable can actually run.
 
 @param project - TypeScript project resolving call targets.
 
 @param bindingOriginBySymbolId - Parameter and alias origins of the callable being summarised.
 
 @param allNodes - Every node of the body.
 
 @param boundary - Declaration owning the scan, which parameter initializers ascend to rather
 than to the body.
 
 @param nestedKeys - Keys of every callable nested under the body.
 
 @param activeKeys - Accumulator receiving newly activated keys.
 
 @mutates activeKeys - Adds every key reachable from a runnable site.
 
 @example
 ```ts
 activationSites({
   project,
   bindingOriginBySymbolId,
   allNodes,
   boundary,
   nestedKeys,
   activeKeys,
 });
 ```
 */
function activationSites({
  project,
  bindingOriginBySymbolId,
  allNodes,
  boundary,
  nestedKeys,
  activeKeys,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly allNodes: readonly Node[];
  readonly boundary: Node;
  readonly nestedKeys: ReadonlySet<string>;
  readonly activeKeys: Set<string>;
},): void {
  allNodes.forEach(function findActivation(node,): void {
    /* The gate. A site inside a closure nothing has proven active cannot activate anything,
     * because the enclosing callable does not reach it. */
    if (!insideOnlyActiveClosures({
      node,
      boundary,
      activeKeys,
    },))
      return;
    if (isBinaryExpression(node,)
      && (node.operatorToken
        .kind
        === SyntaxKind.EqualsToken)
      && expressionHasParameterOrigin({
        project,
        bindingOriginBySymbolId,
        node: node.left,
      },)) {
      activateEscapedCallables({
        project,
        node: node.right,
        nestedKeys,
        activeKeys,
      },);
      return;
    }
    /**
     Callee and interpolated or written actuals, absent when this node invokes nothing.
     */
    const invoked = invokedParts({ node, },);
    if (invoked !== INVOKES_NOTHING) {
      /**
       Callable selected by overload resolution.
       */
      const signatureDeclaration = project.checker
        .getResolvedSignature(node,)
        ?.declaration
        ?.resolve(project,);
      if ((signatureDeclaration !== undefined)
        && isEffectCallableDeclaration(signatureDeclaration,)) {
        /**
         Stable declaration key for selected nested call target.
         */
        const key = callableKey(signatureDeclaration,);
        if (nestedKeys.has(key,))
          activeKeys.add(key,);
      }
      /* Overload resolution answers with the declared type's signature, so a binding declared
       * without an initializer and filled afterwards activates nothing: `local()` resolves to the
       * function type rather than to the arrow assigned into `local`, and the write inside that
       * arrow was filtered out. Measured before this: the declaration form recorded `mutated=[0]`
       * and the assignment form recorded nothing at all.
       *
       * Every assignment to the binding counts, whichever one a given call actually reaches. That
       * over-activates when a binding holds different closures at different points, which
       * attributes more and therefore withholds more, and reaching-definition filtering is what
       * would narrow it. Activating too much is the safe direction here; activating too little
       * loses a write. */
      [
        ...assignedValues({
          allNodes,
          project,
          target: invoked.callee,
        },),
        /* And whatever the called binding was declared holding, which covers a parameter with a
         * callable default. Overload resolution answers with the declared function type there too,
         * so `callback()` where `callback` defaults to an arrow activated nothing. */
        ...possibleValueNodes({
          project,
          node: invoked.callee,
        },),
      ]
        .forEach(function activateResolved(resolved,): void {
          activateEscapedCallables({
            project,
            node: resolved,
            nestedKeys,
            activeKeys,
          },);
        },);
      invoked.actuals
        .forEach(function callbackArgument(argument,): void {
          activateEscapedCallables({
            project,
            node: argument,
            nestedKeys,
            activeKeys,
          },);
        },);
      return;
    }
    if ((!isReturnStatement(node,)) || (node.expression === undefined))
      return;
    activateEscapedCallables({
      project,
      node: node.expression,
      nestedKeys,
      activeKeys,
    },);
  },);
}

/**
 Names every value assigned to one binding anywhere in the scanned body.
 
 @param allNodes - Every node of the body.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param target - Expression naming the binding whose assignments are wanted.
 
 @returns right operands of assignments to that binding.
 
 @example
 ```ts
 assignedValues({ allNodes, project, target });
 ```
 */
function assignedValues({
  allNodes,
  project,
  target,
}: {
  readonly allNodes: readonly Node[];
  readonly project: Project;
  readonly target: Node;
},): readonly Node[] {
  if (!isIdentifier(target,))
    return [];
  /**
   Symbol the called binding resolves to.
   */
  const symbol = project.checker
    .getResolvedSymbol(target,);
  if (symbol === undefined)
    return [];
  return allNodes.flatMap(function assignedTo(node,): readonly Node[] {
    if (!isBinaryExpression(node,))
      return [];
    if (node.operatorToken
      .kind
      !== SyntaxKind.EqualsToken)
      return [];
    if (!isIdentifier(node.left,))
      return [];
    /**
     Symbol the assignment target resolves to.
     */
    const assigned = project.checker
      .getResolvedSymbol(node.left,);
    return assigned?.id === symbol.id ? [node.right,] : [];
  },);
}

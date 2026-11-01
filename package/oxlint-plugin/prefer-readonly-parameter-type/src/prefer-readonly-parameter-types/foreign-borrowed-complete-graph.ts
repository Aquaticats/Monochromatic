/**
 Demand-bounded owned inbound graph for declaration-global foreign provenance.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { SourceFile, } from 'typescript/ast';
import { isCallExpression, } from 'typescript/ast/is';

import { callersAreEnumerable, } from './effect-caller-enumeration.ts';
import type {
  Project,
  SignatureUsage,
} from 'typescript/sync';

import type { EffectAnalysisBudget, } from './effect-analysis-budget.ts';
import { parameterSlotTable, } from './effect-parameter-slots.ts';
import type {
  EffectSlot,
  ParameterIndex,
} from './effect-slot-identity.ts';
import {
  callableKey,
  type EffectCallableDeclaration,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';
import { addForeignBorrowedCallEdge, } from './foreign-borrowed-call-edge.ts';
import {
  addForeignObserverCaller,
  nearestForeignOwnedCallable,
} from './foreign-borrowed-caller.ts';
import { foreignBorrowedOwnershipSeed, } from './foreign-borrowed-direct-summary.ts';
import {
  FOREIGN_OBSERVER_CALL_UNAVAILABLE,
  foreignObserverCall,
} from './foreign-borrowed-observer-edge.ts';
import { propagateForeignBorrowed, } from './foreign-borrowed-propagation.ts';

/**
 Foreign inbound graph logger.
 */
const l = tagged({ tag: 'foreign-borrowed-complete-graph', },);

/**
 Sentinel when TypeScript cannot enumerate signature usage.
 */
const SIGNATURE_USAGE_UNAVAILABLE: unique symbol = Symbol(
  'TypeScript signature usages could not be enumerated',
);

/**
 Reads project-wide usages for one exact callable signature.
 
 @param project - TypeScript semantic project snapshot.
 
 @param declaration - Callable whose inbound references are required.
 
 @returns signature usages or unavailable sentinel after logged failure.
 */
function signatureUsages({
  project,
  declaration,
}: {
  readonly project: Project;
  readonly declaration: EffectCallableDeclaration;
}): readonly SignatureUsage[] | typeof SIGNATURE_USAGE_UNAVAILABLE {
  try {
    return project.languageService
      .getSignatureUsage(declaration,);
  }
  catch (error) {
    l.error(
      `signature usage unavailable for ${callableKey(declaration,)}: ${String(error,)}`,
    );
    return SIGNATURE_USAGE_UNAVAILABLE;
  }
}

/**
 Creates ordinary unknown inbound edge that removes inferred provenance.
 
 @param declaration - Callee whose unknown inbound must fail closed.
 
 @returns synthetic caller summary carrying ordinary argument origins.
 */
function unknownInboundSummary(
  declaration: EffectCallableDeclaration,
): MutableEffectSummary {
  /**
   Callee slots this synthetic edge answers for, all of them with no origins.
   
   Sized by the callee's slots rather than its formals, because that is what propagation
   indexes this array with. A callee that destructures has strictly more of the first.
   */
  const emptyBySlot = parameterSlotTable({ declaration, },)
    .parameterOfSlot
    .map(function emptyOrigins(): readonly EffectSlot[] {
      return [];
    },);
  /**
   Foreign origins per formal, empty for the same reason ordinary origins are.
   */
  const foreignOrigins = declaration.parameters
    .map(function emptyForeignOrigins(): readonly ParameterIndex[] {
      return [];
    },);
  return {
    /* This summary is a caller standing in for an inbound nothing could name, not the
     * callee. It declares no parameters of its own, so it owns no slots either; giving it
     * the callee's table would claim the callee's parameters as this caller's. */
    slots: {
      parameterCount: 0,
      slotCount: 0,
      parameterOfSlot: [],
      slotsByParameter: [],
    },
    bindingOriginBySymbolId: new Map(),
    directReturned: new Set(),
    returned: new Set(),
    directMutated: new Set(),
    directInvoked: new Set(),
    directOpaque: new Set(),
    opaqueProvenanceBySlot: new Map(),
    mutated: new Set(),
    invoked: new Set(),
    opaque: new Set(),
    directForeignBorrowed: new Set(),
    relations: [],
    elementApplications: [],
    resultApplications: [],
    calls: [{
      /* Stands for an inbound foreign relation rather than a syntactic call, so it names
       * itself as one. Nothing looks this key up: only a deferred result use does, and
       * this edge has no call whose result a caller could consume. */
      callSiteKey: `foreign-inbound:${callableKey(declaration,)}`,
      calleeKey: callableKey(declaration,),
      calleeFileName: declaration.getSourceFile()
        .fileName,
      originsByCalleeSlot: emptyBySlot,
      /* A foreign relation packages no callable, so there is nothing captured to carry, and
       * the length still has to match the formal-indexed list beside it. */
      capturedOriginsByFormal: foreignOrigins
        .map(function noCaptures(): readonly EffectSlot[] {
          return [];
        },),
      foreignOriginsByFormal: foreignOrigins,
      directForeignByFormal: declaration.parameters
        .map(function ordinaryArgument(): boolean {
          return false;
        },),
      foreignInbound: true,
      callbackKeysByCalleeSlot: emptyBySlot
        .map(function unavailableCallback() {
          return OWNED_CALLABLE_UNAVAILABLE;
        },),
      callbackFileNamesByCalleeSlot: emptyBySlot
        .map(function unavailableCallbackFile() {
          return OWNED_CALLABLE_UNAVAILABLE;
        },),
    },],
  };
}

/**
 Records unknown inbound once for one callee.
 
 @param summaries - Mutable ownership summaries.
 
 @param declaration - Callee whose inbound cannot be proven.
 */
function addUnknownInbound({
  summaries,
  declaration,
}: {
  readonly summaries: Map<string, MutableEffectSummary>;
  readonly declaration: EffectCallableDeclaration;
}): void {
  /**
   Synthetic caller identity unique to callee.
   */
  const syntheticKey = `\0unknown-inbound:${callableKey(declaration,)}`;
  if (summaries.has(syntheticKey,))
    return;
  summaries.set(
    syntheticKey,
    unknownInboundSummary(declaration,),
  );
}

/**
 Computes guaranteed foreign parameters through exact signature inbounds.
 
 TypeScript enumerates every reference to each demanded callable signature.
 Caller summaries are then added backwards until no new callable owner is
 reached. Non-call references, top-level calls, and unresolved owned edges
 add ordinary inbounds and therefore reject inferred foreign provenance.
 
 @param project - TypeScript project resolving signature usages.
 
 @param indexedSourceFiles - Complete source scope admitted by ownership policy.
 
 @param rootDeclaration - Reached candidate requiring complete inbound proof.
 
 @param analysisBudget - Shared fail-closed project analysis budget.
 
 @param analysisRoot - Optional external package root admitted as owned.
 
 @returns foreign parameter indexes for demanded backwards closure.
 
 @example
 ```ts
 completeForeignBorrowedGraph({
   project,
   indexedSourceFiles,
   rootDeclaration,
   analysisBudget,
 });
 ```
 */
export function completeForeignBorrowedGraph({
  project,
  indexedSourceFiles,
  rootDeclaration,
  analysisBudget,
  analysisRoot,
}: {
  readonly project: Project;
  readonly indexedSourceFiles: ReadonlyMap<string, SourceFile>;
  readonly rootDeclaration: EffectCallableDeclaration;
  readonly analysisBudget: EffectAnalysisBudget;
  readonly analysisRoot?: string;
}): ReadonlyMap<string, ReadonlySet<ParameterIndex>> {
  /**
   Ownership summaries in demanded backwards caller closure.
   */
  const summaries = new Map<string, MutableEffectSummary>();
  /**
   Callable declarations queued for exact inbound discovery.
   */
  const queue: EffectCallableDeclaration[] = [rootDeclaration,];
  /**
   Callable identities whose signature usages were enumerated.
   */
  const visited = new Set<string>();
  /**
   Queue cursor avoiding recursive graph traversal.
   */
  const cursor = { current: 0, };

  while (cursor.current < queue.length) {
    /**
     Current callable requiring direct facts and every inbound usage.
     */
    const declaration = queue[cursor.current];
    cursor.current++;
    if (declaration === undefined)
      throw new Error('Foreign inbound queue lost current declaration.',);
    /**
     Stable current callable identity.
     */
    const key = callableKey(declaration,);
    if (visited.has(key,))
      continue;
    visited.add(key,);
    analysisBudget.assertAvailable(`foreign signature usage ${key}`,);
    if (!summaries.has(key,)) {
      summaries.set(
        key,
        foreignBorrowedOwnershipSeed({
          project,
          declaration,
        },),
      );
    }
    /**
     Collection call directly containing inline observer declaration.
     */
    const inlineObserverCall = foreignObserverCall({ node: declaration, },);
    /**
     Whether inline observer received one exact position-aware inbound.
     */
    const inlineObserverAdded = inlineObserverCall === FOREIGN_OBSERVER_CALL_UNAVAILABLE
      ? false
      : addForeignObserverCaller({
        project,
        indexedSourceFiles,
        summaries,
        queue,
        call: inlineObserverCall,
        observerDeclaration: declaration,
      },);
    if ((inlineObserverCall !== FOREIGN_OBSERVER_CALL_UNAVAILABLE)
      && (!inlineObserverAdded)) {
      addUnknownInbound({
        summaries,
        declaration,
      },);
    }
    /* The same completeness question the returned-result discharge asks, and asked here for
     * the same reason: this walk proves an inbound closure, and a callable other files may
     * import has inbounds no enumeration reaches. Recorded as an unknown inbound rather than
     * abandoning the walk, since that is exactly this graph's existing way of saying an edge
     * could not be proven, and it already rejects inferred provenance.
     *
     * An inline observer's call syntax is its complete inbound identity and needs no exported
     * signature enumeration. */
    if ((inlineObserverCall === FOREIGN_OBSERVER_CALL_UNAVAILABLE)
      && (!callersAreEnumerable({
        project,
        declaration,
      },))) {
      addUnknownInbound({
        summaries,
        declaration,
      },);
    }
    /**
     Start time for exact TypeScript signature reference query.
     */
    const startedAt = analysisBudget.start();
    /**
     Every project usage of current callable signature.
     */
    const usages = signatureUsages({
      project,
      declaration,
    },);
    analysisBudget.record({
      startedAt,
      phase: `foreign signature usage ${key}`,
    },);
    if (usages === SIGNATURE_USAGE_UNAVAILABLE) {
      addUnknownInbound({
        summaries,
        declaration,
      },);
      continue;
    }
    usages.forEach(function addInboundCaller(usage,): void {
      /**
       Resolved call expression or non-call escape marker.
       */
      const call = usage.call
        ?.resolve(project,);
      if (call === undefined) {
        /**
         Signature reference passed as observer rather than directly invoked.
         */
        const { name: usageNameHandle, } = usage;
        /**
         Reference node resolved from signature usage handle.
         */
        const usageName = usageNameHandle.resolve(project,);
        /**
         Collection call containing reference as direct argument.
         */
        const observerCall = usageName === undefined
          ? FOREIGN_OBSERVER_CALL_UNAVAILABLE
          : foreignObserverCall({ node: usageName, },);
        if ((observerCall !== FOREIGN_OBSERVER_CALL_UNAVAILABLE)
          && addForeignObserverCaller({
            project,
            indexedSourceFiles,
            summaries,
            queue,
            call: observerCall,
            observerDeclaration: declaration,
          },))
          return;
        addUnknownInbound({
          summaries,
          declaration,
        },);
        return;
      }
      /**
       Nearest callable owner admitted as project-owned source.
       */
      const caller = nearestForeignOwnedCallable({
        node: call,
        indexedSourceFiles,
      },);
      if (caller === OWNED_CALLABLE_UNAVAILABLE) {
        addUnknownInbound({
          summaries,
          declaration,
        },);
        return;
      }
      /**
       Stable caller identity.
       */
      const callerKey = callableKey(caller,);
      /**
       Caller ownership seed receiving exact current usage edge.
       */
      const callerSummary = summaries.get(callerKey,)
        ?? foreignBorrowedOwnershipSeed({
          project,
          declaration: caller,
        },);
      summaries.set(
        callerKey,
        callerSummary,
      );
      queue.push(caller,);
      if (!isCallExpression(call,)) {
        addUnknownInbound({
          summaries,
          declaration,
        },);
        return;
      }
      /**
       Call count before exact usage edge is added.
       */
      const priorCallCount = callerSummary.calls
        .length;
      /**
       Whether exact usage resolved to owned callee.
       */
      const added = addForeignBorrowedCallEdge({
        project,
        declaration: caller,
        call,
        summary: callerSummary,
        ...(analysisRoot === undefined) ? {} : { analysisRoot, },
      },);
      /**
       Exact edge produced for current usage.
       */
      const exactEdge = callerSummary.calls[priorCallCount];
      if ((!added)
        || (callerSummary.calls
          .length
          !== (priorCallCount
            + 1))
        || (exactEdge === undefined)
        || (!exactEdge.foreignInbound)
        || (exactEdge.calleeKey !== key)) {
        callerSummary.calls
          .splice(priorCallCount,);
        addUnknownInbound({
          summaries,
          declaration,
        },);
      }
    },);
  }
  return propagateForeignBorrowed(summaries,);
}

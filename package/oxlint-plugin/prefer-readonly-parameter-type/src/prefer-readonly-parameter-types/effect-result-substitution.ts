/**
 Substituting a callee's returned state into the caller that uses the result.
 
 The gap this closes was measured rather than predicted, and is recorded in
 `doc/planning/prefer-readonly-return-substitution.md`, section "A second false offer,
 on the array path". A callee that hands its own parameter back lets a caller mutate
 caller-owned state through the returned value, and nothing attributed that write:
 `launderMutable(rows,).push(...)` was offered `readonly Row[]`, the applied annotation
 type-checked, and running it grew the caller's array.
 
 The syntax pass cannot close it alone. `directEffectSummary` sees one declaration and
 no other callable's summary, so asking what a callee returns while walking its caller
 would make the answer depend on which was analysed first, which `#29` forbids. The use
 is therefore recorded where it is visible, and the origins are resolved here, where the
 callee summary and the edge's formal-to-actual mapping sit together.
 
 This module only ever ADDS facts. Nothing discharges on the strength of a returned set
 being empty, and nothing may: `doc/decision/prefer-readonly-result-provenance.md` makes
 caller substitution the precondition for discharging receiver opacity, not a
 consequence of it, and an empty returned set is equally consistent with a fresh result
 and with a return shape this analysis does not model.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isAssertionExpression,
  isCallExpression,
  isAwaitExpression,
  isNonNullExpression,
  isParenthesizedExpression,
  isSatisfiesExpression,
} from 'typescript/ast/is';

import { capturedOriginsByCalleeSlot, } from './effect-captured-result-origins.ts';
import {
  addEffectSlot,
  type CallEdge,
  callSiteKey,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';

/**
 No call underlies this expression, so no result use can be deferred against one.
 */
export const NOT_A_DEFERRABLE_RESULT: unique symbol = Symbol(
  'expression is not the result of a call',
);

/**
 Names the call whose result an expression is, when it is one.
 
 Unwraps the wrappers that keep a value's identity, matching `transparentOperand` in
 `effect-expression-provenance.ts`. `await` is deliberately absent from both: thenable
 assimilation does not prove the awaited value is the one the callee returned, so an
 awaited result stays unmodelled rather than being claimed.
 
 @param node - Expression whose underlying call is sought.
 
 @returns call-site identity, or sentinel when no call underlies it.
 
 @example
 ```ts
 deferrableResultSite({ node: receiver });
 ```
 */
export function deferrableResultSite(
  { node, }: { readonly node: Node; },
): string | typeof NOT_A_DEFERRABLE_RESULT {
  /**
   Value this expression is, past every wrapper that keeps its identity.
   */
  const root = transparentValueRoot(node,);
  return isCallExpression(root,)
    ? callSiteKey(root,)
    : NOT_A_DEFERRABLE_RESULT;
}

/**
 Removes the wrappers that keep a value's identity.
 
 Exported because more than one question needs the same normalization and answering them
 with separate spellings produced a hole: `expressionResultSites` stripped access layers,
 asked `deferrableResultSite` about the result, and then tested the ORIGINAL node for
 being an identifier. So `const alias = local as Row;` reached the identifier inside the
 assertion, learned it was not a call, and then failed the identifier test against the
 assertion itself. Measured: that alias and a parenthesised one recorded no write while
 the bare alias recorded one.
 
 `await` is deliberately absent, matching `transparentOperand` in
 `effect-expression-provenance.ts`: thenable assimilation does not prove the awaited
 value is the one the callee returned.
 
 @param node - Expression whose identity-keeping wrappers are removed.
 
 @returns innermost expression holding the same value.
 
 @example
 ```ts
 transparentValueRoot(declaration.initializer);
 ```
 */
export function transparentValueRoot(node: Node,): Node {
  /**
   Cursor descending through wrappers that keep the value's identity.
   */
  const cursor: { current: Node; } = { current: node, };
  /* `await` joined the transparent forms because an async return was tracked at the callee and
   * lost at the caller. `rowAsync` records `returned=[0]` exactly as `rowSync` does, and a
   * caller writing through `(await rowAsync(config,)).label` recorded nothing while the
   * synchronous caller recorded `mutated=[0]`. The accepted decision permits returning caller
   * state on the condition that callers keep tracking it, so the condition was failing for
   * every async return.
   *
   * Transparent for this purpose rather than in general: what an await yields is whatever the
   * awaited promise resolves to, which is what the callee handed back, and that is the only
   * question this walk asks. */
  while (isParenthesizedExpression(cursor.current,)
    || isNonNullExpression(cursor.current,)
    || isAssertionExpression(cursor.current,)
    || isAwaitExpression(cursor.current,)
    || isSatisfiesExpression(cursor.current,))
    cursor.current = cursor.current
      .expression;
  return cursor.current;
}

/**
 Records that a caller uses one call's result in a way whose origins must be resolved.
 
 @param summary - Caller summary receiving the deferred use.
 
 @param node - Expression whose underlying call result is used.
 
 @param kind - Whether the result is mutated or handed back.
 
 @mutates summary - Appends one deferred result use when a call underlies the node.
 
 @example
 ```ts
 recordResultApplication({ summary, node: receiver, kind: 'mutated' });
 ```
 */
export function recordResultApplication({
  summary,
  node,
  kind,
}: {
  readonly summary: MutableEffectSummary;
  readonly node: Node;
  readonly kind: 'mutated' | 'returned';
},): void {
  /**
   Call whose result this use consumes, when one underlies the expression.
   */
  const site = deferrableResultSite({ node, },);
  if (site === NOT_A_DEFERRABLE_RESULT)
    return;
  recordResultApplicationSites({
    summary,
    sites: new Set([site,],),
    kind,
  },);
}

/**
 Records that a caller uses the results of named calls in a way needing resolution.
 
 Takes call sites rather than an expression because a use can reach more than one. A
 binding holds whatever call filled it, and following aliases can find several, so the
 node-to-site question and the site-to-record question are separate and only the first
 has one answer.
 
 @param summary - Caller summary receiving the deferred uses.
 
 @param sites - Call sites whose results this use consumes.
 
 @param kind - Whether the results are mutated or handed back.
 
 @mutates summary - Appends one deferred use per named call site.
 
 @example
 ```ts
 recordResultApplicationSites({ summary, sites, kind: 'mutated' });
 ```
 */
export function recordResultApplicationSites({
  summary,
  sites,
  kind,
}: {
  readonly summary: MutableEffectSummary;
  readonly sites: ReadonlySet<string>;
  readonly kind: 'mutated' | 'returned';
},): void {
  sites.forEach(function recordOne(site,): void {
    summary.resultApplications
      .push({
        callSiteKey: site,
        kind,
      },);
  },);
}

/**
 Records that a caller hands one call's result to something outliving the call.
 
 Separate from `recordResultApplication` rather than a third argument to it, because the
 retention channel carries provenance and the other two kinds have nothing to say. A
 parameter that is meaningful for one of three kinds is a parameter callers have to be
 told to omit.
 
 @param summary - Caller summary receiving the deferred retention.
 
 @param node - Expression whose underlying call result is handed outward.
 
 @param provenance - Retention provenance naming where the value went.
 
 @mutates summary - Appends one deferred retention when a call underlies the node.
 
 @example
 ```ts
 recordResultRetention({ summary, node: assignment.right, provenance });
 ```
 */
export function recordResultRetention({
  summary,
  node,
  provenance,
}: {
  readonly summary: MutableEffectSummary;
  readonly node: Node;
  readonly provenance: string;
},): void {
  /**
   Call whose result this retention consumes, when one underlies the expression.
   */
  const site = deferrableResultSite({ node, },);
  if (site === NOT_A_DEFERRABLE_RESULT)
    return;
  recordResultRetentionSites({
    summary,
    sites: new Set([site,],),
    provenance,
  },);
}

/**
 Records that a caller hands the results of named calls to something outliving them.
 
 {@inheritDoc recordResultApplicationSites}
 
 @param summary - Caller summary receiving the deferred retentions.
 
 @param sites - Call sites whose results are handed outward.
 
 @param provenance - Retention provenance naming where the value went.
 
 @mutates summary - Appends one deferred retention per named call site.
 
 @example
 ```ts
 recordResultRetentionSites({ summary, sites, provenance });
 ```
 */
export function recordResultRetentionSites({
  summary,
  sites,
  provenance,
}: {
  readonly summary: MutableEffectSummary;
  readonly sites: ReadonlySet<string>;
  readonly provenance: string;
},): void {
  sites.forEach(function recordOne(site,): void {
    summary.resultApplications
      .push({
        callSiteKey: site,
        kind: 'retained',
        provenance,
      },);
  },);
}

/**
 Adds opacity and provenance for every caller origin a retained result carries.
 
 Writes `summary.opaque` rather than `summary.directOpaque`, which is the difference
 between this and `addOpaqueEffect`. The direct set is seeded into the propagated one
 once, at the end of the syntactic pass, and this runs afterwards: an addition to the
 direct set here would land in the provenance map and never reach the set the verifier
 reads.
 
 @param summary - Caller summary receiving opacity and provenance.
 
 @param edge - Owned call edge carrying formal-to-actual origins.
 
 @param calleeReturned - Callee slots its result can carry.
 
 @param capturedByCalleeSlot - Captures of the formal owning each callee slot, which a result
 carries for the same reason an ordinary origin does.
 
 @param provenance - Retention provenance naming where the value went.
 
 @mutates summary - Adds each retained origin as an opaque slot with its provenance.
 
 @returns whether the caller gained an opaque slot.
 
 @example
 ```ts
 substituteRetainedOrigins({
   summary,
   edge,
   calleeReturned,
   capturedByCalleeSlot,
   provenance,
 });
 ```
 */
function substituteRetainedOrigins({
  summary,
  edge,
  calleeReturned,
  capturedByCalleeSlot,
  provenance,
}: {
  readonly summary: MutableEffectSummary;
  readonly edge: CallEdge;
  readonly calleeReturned: ReadonlySet<EffectSlot>;
  readonly capturedByCalleeSlot: readonly (readonly EffectSlot[])[];
  readonly provenance: string;
},): boolean {
  /**
   Whether retention added an origin the caller did not already carry.
   */
  const growth: { any: boolean; } = { any: false, };
  for (const calleeSlot of calleeReturned) {
    /**
     Caller origins the callee reaches through this slot, by receiving them and by invoking
     what it received.
     */
    const origins = resultCarriedOrigins({
      edge,
      capturedByCalleeSlot,
      calleeSlot,
    },);
    for (const origin of origins) {
      if (addEffectSlot({
        target: summary.opaque,
        value: origin,
      },))
        growth.any = true;
      /**
       Provenance already recorded for this slot, or a new accumulator.
       */
      const facts = summary.opaqueProvenanceBySlot
        .get(origin,)
        ?? new Set<string>();
      /**
       Fact count before this retention, deciding whether provenance grew.
       */
      const factsBefore = facts.size;
      facts.add(provenance,);
      /* Progress has to count the provenance and not only the slot, and reporting the slot
       * alone was wrong rather than merely conservative. A slot already opaque from some
       * other cause gains a new retention fact here while the set does not grow, so this
       * returned false having changed state. `propagateUncertaintyProvenance` reads a
       * callee's `opaqueProvenanceBySlot` DURING the fixed point, and summaries are walked
       * in map order, so a caller processed before its callee gained the fact would never
       * be revisited and the cause would never cross the call edge. */
      if (facts.size !== factsBefore)
        growth.any = true;
      summary.opaqueProvenanceBySlot
        .set(
        origin,
        facts,
        );
    }
  }
  return growth.any;
}

/**
 Adds every caller origin a callee's returned slots map to, through one edge.
 
 @param target - Caller effect set receiving substituted origins.
 
 @param edge - Owned call edge carrying formal-to-actual origins.
 
 @param calleeReturned - Callee slots its result can carry.
 
 @param capturedByCalleeSlot - Captures of the formal owning each callee slot, which a result
 carries for the same reason an ordinary origin does.
 
 @mutates target - Adds each caller origin behind a returned callee slot.
 
 @returns whether target gained an origin.
 
 @example
 ```ts
 substituteReturnedOrigins({
   target: summary.mutated,
   edge,
   calleeReturned,
   capturedByCalleeSlot,
 });
 ```
 */
function substituteReturnedOrigins({
  target,
  edge,
  calleeReturned,
  capturedByCalleeSlot,
}: {
  readonly target: Set<EffectSlot>;
  readonly edge: CallEdge;
  readonly calleeReturned: ReadonlySet<EffectSlot>;
  readonly capturedByCalleeSlot: readonly (readonly EffectSlot[])[];
},): boolean {
  /**
   Whether substitution added an origin the caller did not already carry.
   */
  const growth: { any: boolean; } = { any: false, };
  for (const calleeSlot of calleeReturned) {
    /**
     Caller origins the callee reaches through this slot, by receiving them and by invoking
     what it received.
     */
    const origins = resultCarriedOrigins({
      edge,
      capturedByCalleeSlot,
      calleeSlot,
    },);
    for (const origin of origins) {
      if (addEffectSlot({
        target,
        value: origin,
      },))
        growth.any = true;
    }
  }
  return growth.any;
}

/**
 Names every caller origin one returned callee slot can carry.
 
 Two contributions, unioned rather than chosen between. The ordinary origins are read from the
 edge rather than from the call's arguments, because `formalActualPositions` already resolved
 explicit `this`, rest formals and spread actuals into that mapping, and indexing arguments by
 parameter position would disagree with it on every one of those. The captures are what invoking
 a callable the caller packaged into that formal can reach, which the callee's result carries
 whether the callee handed the callable back or handed back what calling it produced.
 
 @param edge - Owned call edge carrying formal-to-actual origins.
 
 @param capturedByCalleeSlot - Captures of the formal owning each callee slot.
 
 @param calleeSlot - Slot the callee named in its returned set.
 
 @returns caller origins that slot's result can carry.
 
 @example
 ```ts
 resultCarriedOrigins({ edge, capturedByCalleeSlot, calleeSlot });
 ```
 */
function resultCarriedOrigins({
  edge,
  capturedByCalleeSlot,
  calleeSlot,
}: {
  readonly edge: CallEdge;
  readonly capturedByCalleeSlot: readonly (readonly EffectSlot[])[];
  readonly calleeSlot: EffectSlot;
},): readonly EffectSlot[] {
  return [
    ...edge.originsByCalleeSlot[calleeSlot] ?? [],
    ...capturedByCalleeSlot[calleeSlot] ?? [],
  ];
}

/**
 Edges shared by no call site, so a deferred use with no owned edge allocates none.
 */
const NO_EDGES: readonly CallEdge[] = [];

/**
 Resolves one caller's deferred result uses against its callees' returned state.
 
 @param summaries - Owned callable summaries by declaration key.
 
 @param summary - Caller summary whose result uses are resolved.
 
 @mutates summary - Adds mutation and returned origins behind used call results.
 
 @returns whether caller summary changed.
 
 @example
 ```ts
 propagateResultApplications({ summaries, summary });
 ```
 */
export function propagateResultApplications({
  summaries,
  summary,
}: {
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly summary: MutableEffectSummary;
},): boolean {
  if (summary.resultApplications
    .length
    === 0)
    return false;
  /**
   Edges of this caller by call site, so a deferred use finds every call it belongs to.
   
   Every edge rather than one. One call site carries several edges whenever the callee expression
   resolves to more than one callable, which a conditional default does, and this was built with
   `new Map(entries)`, which keeps the last pair and silently discarded the rest. The effect and
   capability passes iterate `summary.calls` directly and saw them all, so only a deferred result use
   lost anything, which is why no effect probe showed it.
   
   Demonstrated once the shared resolver reached a default naming an ordinary function: two
   conditional defaults differing only in which branch was written first answered differently, one
   charging its caller's configuration and the other offering it. An answer that flips with source
   order is the diagnosis.
   
   Unioned rather than merged into one edge. Different callees have different slot layouts, summaries,
   captures and formal-to-actual mappings, so a merged edge would state a relation neither callee
   has.
   */
  const edgesByCallSite = new Map<string, CallEdge[]>();
  summary.calls
    .forEach(function keyed(edge,): void {
      /**
       Edges already recorded for this call site.
       */
      const already = edgesByCallSite.get(edge.callSiteKey,);
      if (already === undefined) {
        edgesByCallSite.set(
          edge.callSiteKey,
          [edge,],
        );
        return;
      }
      already.push(edge,);
    },);
  /**
   Whether any deferred use contributed an origin this pass.
   */
  const growth: { any: boolean; } = { any: false, };
  for (const application of summary.resultApplications) {
    /**
     Edges for the call whose result this use consumes.
     
     Empty when the callee was never resolved as owned, in which case the call already
     took an opaque boundary of its own and this use needs no separate treatment.
     */
    const edges = edgesByCallSite.get(application.callSiteKey,) ?? NO_EDGES;
    for (const edge of edges) {
    /**
     Summary of the callee whose result this use consumes.
     
     Absent when the callee's summary could not be built. `propagateEffects` already
     turns that into opacity for every origin the edge packages, so nothing is added
     here and nothing is claimed.
     */
      const calleeSummary = summaries.get(edge.calleeKey,);
      if (calleeSummary === undefined)
        continue;
    /**
     Captures re-filed under the callee slots their formals own, so the substitution walk can
     read them with the same index it reads ordinary origins with.
     */
      const capturedByCalleeSlot = capturedOriginsByCalleeSlot({
        calleeSummary,
        edge,
      },);
      if (application.kind === 'retained') {
      /* No guard on the provenance, because the type carries the invariant now. A
       * retention with nothing to say would arrive as an unexplained opaque slot and be
       * reported as an unresolved effect, and the first spelling of this defended against
       * that with a runtime throw over an optional field. That left `{ kind: 'retained' }`
       * type-checking, so the throw was the only thing between a well-typed literal and a
       * crash inside the fixed point. Making `ResultApplication` a union proved the branch
       * unreachable: TypeScript narrowed the check to `never` and rejected it. */
        if (substituteRetainedOrigins({
          summary,
          edge,
          calleeReturned: calleeSummary.returned,
          capturedByCalleeSlot,
          provenance: application.provenance,
        },))
          growth.any = true;
        continue;
      }
      if (substituteReturnedOrigins({
        target: application.kind === 'mutated' ? summary.mutated : summary.returned,
        edge,
        calleeReturned: calleeSummary.returned,
        capturedByCalleeSlot,
      },))
        growth.any = true;
    }
  }
  return growth.any;
}

/**
 Seeds the propagated returned set from the directly recorded one.
 
 Separate from substitution because the direct facts are the base case of the fixed
 point: a callable returning its own parameter says so without any callee's help, and a
 callable returning another's result needs that base case to have been seeded first.
 
 @param summary - Summary whose returned set is seeded.
 
 @mutates summary - Copies direct returned slots into the propagated set.
 
 @returns whether the propagated set grew.
 
 @example
 ```ts
 seedReturnedSlots({ summary });
 ```
 */
export function seedReturnedSlots(
  { summary, }: { readonly summary: MutableEffectSummary; },
): boolean {
  /**
   Whether seeding added a slot the propagated set lacked.
   */
  const growth: { any: boolean; } = { any: false, };
  for (const slot of summary.directReturned) {
    if (addEffectSlot({
      target: summary.returned,
      value: slot,
    },))
      growth.any = true;
  }
  return growth.any;
}

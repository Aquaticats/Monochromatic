/**
 Anchoring for the foreign-ownership greatest fixed point.
 
 The fixed point in `foreign-borrowed-propagation.ts` answers safety: a parameter survives only
 while every inbound argument reaching it is foreign. It does not answer whether anything is
 foreign in the first place. `initializeCandidates` seeds every parameter of any callable holding
 an inbound, so a component whose inbounds all pass a surviving parameter straight through
 sustains itself with no marker anywhere, and self-recursion is enough to do that alone.
 
 Measured on `markerlessRecursion` in `readonly-recursive-ownership-invalid.ts`: identical
 parameter and identical read-only body as its non-recursive twin, and only the twin was offered
 as read-only. Foreign ownership suppresses that offer, so the recursive one lost an offer it
 deserved.
 
 This module supplies the missing half. A candidate is kept only when some derivation reaches a
 real marker, which is an existential reachability rather than the universal condition the fixed
 point already applied. The distinction is load-bearing: requiring every inbound to be grounded
 would discard a marker-fed recursive helper, whose self-edge is unsatisfied at the step its
 parameter would first enter, and that helper is exactly what a greatest fixed point exists to
 support.
 
 @module
 */

import type { ParameterIndex, } from './effect-slot-identity.ts';
import type {
  CallEdge,
  MutableEffectSummary,
} from './effect-summary-model.ts';

//region Inbound shape
// One inbound owned call carrying caller state into a callee.

/**
 One inbound owned call carrying caller state into callee.
 */
export type ForeignInbound = {
  readonly callerKey: string;
  readonly edge: CallEdge;
};

//endregion Inbound shape

//region Candidate identities
// The separator joining a callable identity to a parameter position, and the
// joined identity of one candidate parameter.

/**
 Separator joining a callable identity to a parameter position.
 
 A NUL, because `callableKey` builds its identity from a file path and a declaration position and
 neither can contain one, so no two distinct candidates can collide on a joined key.
 */
const CANDIDATE_KEY_SEPARATOR = '\0';

/**
 Builds the joined identity for one candidate parameter.
 
 @param callableKeyValue - Callable identity owning parameter.
 
 @param parameterIndex - Parameter position under test.
 
 @returns joined candidate identity.
 
 @example
 ```ts
 candidateKey({ callableKeyValue: 'file.ts:12', parameterIndex: asParameterIndex(0,), });
 ```
 */
function candidateKey({
  callableKeyValue,
  parameterIndex,
}: {
  readonly callableKeyValue: string;
  readonly parameterIndex: ParameterIndex;
},): string {
  return `${callableKeyValue}${CANDIDATE_KEY_SEPARATOR}${parameterIndex}`;
}

//endregion Candidate identities

//region Direct proof
// Whether one inbound proves its argument foreign without consulting any caller
// candidate.

/**
 Whether one inbound proves its argument foreign without consulting any caller candidate.
 
 `directForeignByFormal` records that the argument expression's own type carries a marker at this
 call site, which is a marker sighting rather than an inference, so it anchors a derivation the
 same way a marked parameter does.
 
 @param inbound - Caller identity and argument mapping.
 
 @param parameterIndex - Callee parameter position under test.
 
 @returns whether this inbound is anchored on its own.
 
 @example
 ```ts
 inboundIsAnchored({ inbound, parameterIndex: asParameterIndex(0,), });
 ```
 */
function inboundIsAnchored({
  inbound,
  parameterIndex,
}: {
  readonly inbound: ForeignInbound;
  readonly parameterIndex: ParameterIndex;
},): boolean {
  /**
   Whether the argument's own type carried a marker at this call site.
   */
  const directByFormal = inbound.edge
    .directForeignByFormal;
  return directByFormal[parameterIndex] === true;
}

//endregion Direct proof

//region Reachability
// The anchored candidates and support edges, then the removal of candidates no
// marker derivation reaches.

/**
 Collects the anchored candidates and the support edges between the rest.
 
 @param summaries - Summaries providing explicit markers.
 
 @param incomingByCallee - Inbound calls by callee key.
 
 @param candidates - Parameters surviving the greatest fixed point.
 
 @returns anchored candidate keys and caller-to-callee support adjacency.
 
 @example
 ```ts
 supportGraph({ summaries, incomingByCallee, candidates, });
 ```
 */
function supportGraph({
  summaries,
  incomingByCallee,
  candidates,
}: {
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly incomingByCallee: ReadonlyMap<string, readonly ForeignInbound[]>;
  readonly candidates: ReadonlyMap<string, ReadonlySet<ParameterIndex>>;
},): {
  readonly anchored: ReadonlySet<string>;
  readonly supportedBy: ReadonlyMap<string, readonly string[]>;
} {
  /**
   Candidates a marker sighting anchors directly.
   */
  const anchored = new Set<string>();
  /**
   Candidates each candidate can carry its provenance to.
   */
  const supportedBy = new Map<string, string[]>();
  for (const [key, parameterIndexes,] of candidates) {
    /**
     Explicit markers on this callable's own parameters.
     */
    const directForeignBorrowed = summaries.get(key,)
      ?.directForeignBorrowed;
    /**
     Every owned call reaching this callable.
     */
    const inbounds = incomingByCallee.get(key,) ?? [];
    for (const parameterIndex of parameterIndexes) {
      /**
       Joined identity for the candidate under test.
       */
      const target = candidateKey({
        callableKeyValue: key,
        parameterIndex,
      },);
      if (directForeignBorrowed?.has(parameterIndex,) === true) {
        anchored.add(target,);
        continue;
      }
      for (const inbound of inbounds) {
        if (inboundIsAnchored({
          inbound,
          parameterIndex,
        },)) {
          anchored.add(target,);
          continue;
        }
        /**
         Caller parameters this inbound packaged into the callee argument.
         */
        const callerIndexes = inbound.edge
          .foreignOriginsByFormal;
        (callerIndexes[parameterIndex] ?? [])
          .forEach(function addSupportEdge(callerIndex,): void {
            /**
             Caller candidate whose provenance reaches this one.
             */
            const source = candidateKey({
              callableKeyValue: inbound.callerKey,
              parameterIndex: callerIndex,
            },);
            supportedBy.set(
              source,
              [
                ...supportedBy.get(source,) ?? [],
                target,
              ],
            );
          },);
      }
    }
  }
  return {
    anchored,
    supportedBy,
  };
}

/**
 Removes foreign candidates no marker derivation reaches.
 
 Reachability over candidates the fixed point already accepted, so this can only remove. Every
 removal turns a suppressed read-only offer back into an offer, which is the direction that needs
 reading rather than the direction that risks unsoundness: nothing marked the parameter, so
 nothing foreign owns it.
 
 @param summaries - Summaries providing explicit markers.
 
 @param incomingByCallee - Inbound calls by callee key.
 
 @param candidates - Parameters surviving the greatest fixed point.
 
 @returns candidates a marker derivation reaches, by callable key.
 
 @example
 ```ts
 groundForeignCandidates({ summaries, incomingByCallee, candidates, });
 ```
 */
export function groundForeignCandidates({
  summaries,
  incomingByCallee,
  candidates,
}: {
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly incomingByCallee: ReadonlyMap<string, readonly ForeignInbound[]>;
  readonly candidates: ReadonlyMap<string, ReadonlySet<ParameterIndex>>;
},): ReadonlyMap<string, ReadonlySet<ParameterIndex>> {
  /**
   Marker anchors and the caller-to-callee edges provenance travels along.
   */
  const {
    anchored,
    supportedBy,
  } = supportGraph({
    summaries,
    incomingByCallee,
    candidates,
  },);
  /**
   Candidates a marker derivation reaches, grown from every anchor.
   */
  const grounded = new Set<string>(anchored,);
  /**
   Anchors and newly grounded candidates whose successors remain to walk.
   */
  const pending = [...anchored,];
  while (pending.length > 0) {
    /**
     Next grounded candidate whose successors inherit its provenance.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    (supportedBy.get(current,) ?? [])
      .forEach(function groundSuccessor(successor,): void {
        if (grounded.has(successor,))
          return;
        grounded.add(successor,);
        pending.push(successor,);
      },);
  }
  return new Map([...candidates,]
    .map(function retainGrounded(
      [key, parameterIndexes,],
    ): readonly [
      string,
      ReadonlySet<ParameterIndex>,
    ] {
      return [
        key,
        new Set([...parameterIndexes,]
          .filter(function isGrounded(parameterIndex,): boolean {
            return grounded.has(candidateKey({
              callableKeyValue: key,
              parameterIndex,
            },),);
          },),),
      ];
    },),);
}
//endregion Reachability

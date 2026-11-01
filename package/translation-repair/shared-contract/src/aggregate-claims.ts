import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  computeIssueClaimId,
  type IssueClaim,
  type SpanAnchor,
} from './issue-model.ts';
import type { ReadonlyEntry, } from './readonly-entry.ts';

//region Claim aggregation
// Cross-model dedupe and merge PROPOSAL only: identical claims collapse by
// deterministic identity, and overlapping claims cluster so an adjudicator
// can dispose the merge (settled architecture: clustering never decides;
// the LLM panel is part of the union algorithm, user directive 2026-07-23).
// Claims stay atomic inside clusters, and provenance stays outside
// the claim: the shell maps claim ids to proposers for calibration, never
// for judging, because a real defect can arrive with exactly one proposer
// (reference run: gpt-oss-120b was the sole finder of a planted seed).

/**
 Neighborhood in characters around zero-width insertion anchors when testing
 claim-to-claim overlap; models drop omission anchors at slightly different
 points around the same gap, and honest critics anchor on surrounding
 context. Two insertion anchors therefore cluster within twice this
 distance of each other, since both expand.
 */
export const CLUSTER_ANCHOR_TOLERANCE = 30;

/**
 One claim carried with its deterministic identity,
 so downstream stages and steering operations get stable handles
 without recomputing hashes.

 @example
 ```ts
 const member: AggregatedClaim = { claimId, claim, };
 ```
 */
export type AggregatedClaim = {
  /**
   Deterministic `issue/<hash>` identity from {@link computeIssueClaimId}.
   */
  readonly claimId: string;

  /**
   Atomic claim exactly as validated; aggregation never rewrites claims.
   */
  readonly claim: IssueClaim;
};

/**
 One proposed merge group.
 A single-member cluster proposes nothing; a multi-member cluster proposes
 that its members describe one defect, for an adjudicator to dispose.

 @example
 ```ts
 const merge: ClaimCluster = { clusterId, position: 42, members, };
 ```
 */
export type ClaimCluster = {
  /**
   Deterministic `cluster/<hash>` identity over sorted member claim ids.
   */
  readonly clusterId: string;

  /**
   Earliest span start across members, for document-order processing
   downstream (envelope derivation, adjudication prompts).
   */
  readonly position: number;

  /**
   Members sorted by claim id; atomic claims, never merged content.
   */
  readonly members: readonly AggregatedClaim[];
};

/**
 Complete partition of the deduplicated input claims:
 every claim belongs to exactly one cluster.

 @example
 ```ts
 const { clusters, }: ClaimAggregation = aggregateClaims({ claims, },);
 ```
 */
export type ClaimAggregation = {
  /**
   Clusters in document order (position, then cluster id).
   */
  readonly clusters: readonly ClaimCluster[];
};

/**
 Half-open offset interval used for overlap tests.
 */
type OffsetInterval = {
  /**
   Inclusive start.
   */
  readonly start: number;

  /**
   Exclusive end.
   */
  readonly end: number;
};

/**
 Interval a span occupies for overlap testing.
 Zero-width insertion anchors expand by {@link CLUSTER_ANCHOR_TOLERANCE} on
 both sides because they name a gap, not text; quoted spans stay exact.

 @param span - anchored evidence whose neighborhood overlap testing needs

 @returns Half-open interval, possibly extending below zero for expanded anchors

 @example
 ```ts
 const interval = expandedInterval({ span, },);
 ```
 */
function expandedInterval(
  { span, }: { readonly span: SpanAnchor; },
): OffsetInterval {
  if (span.startOffset === span.endOffset) {
    return {
      start: span.startOffset - CLUSTER_ANCHOR_TOLERANCE,
      end: span.endOffset + CLUSTER_ANCHOR_TOLERANCE,
    };
  }

  return {
    start: span.startOffset,
    end: span.endOffset,
  };
}

/**
 Whether two spans point at intersecting evidence.
 Offsets are absolute within full document source, so intersection needs no
 node identity check; node labels stay out of it so anchors at node
 boundaries still meet.

 @param left - one span under comparison

 @param right - other span under comparison

 @returns Whether both spans share a side and their intervals intersect

 @example
 ```ts
 spansOverlap({ left, right, },);
 ```
 */
function spansOverlap(
  {
    left,
    right,
  }: {
    readonly left: SpanAnchor;
    readonly right: SpanAnchor;
  },
): boolean {
  if (left.side !== right.side)
    return false;

  /**
   Left interval with anchor expansion applied.
   */
  const leftInterval = expandedInterval({ span: left, },);

  /**
   Right interval with anchor expansion applied.
   */
  const rightInterval = expandedInterval({ span: right, },);

  return (leftInterval.start < rightInterval.end)
    && (rightInterval.start < leftInterval.end);
}

/**
 Whether two claims plausibly describe one defect:
 evidence overlap on at least one same-side span pair.
 Neither category family nor severity participates: critics label one
 defect under different families (measured on real corpus artifacts:
 62 overlapping cross-family issue pairs the old family gate kept from
 panel judgment) and grade it differently, and the panel's sameDefect
 disposal is the union algorithm's judging half, so proposals maximize
 recall and the panel decides.

 @param left - one claim under comparison

 @param right - other claim under comparison

 @returns Whether a merge between the two is worth proposing

 @example
 ```ts
 claimsShareDefect({ left, right, },);
 ```
 */
function claimsShareDefect(
  {
    left,
    right,
  }: {
    readonly left: IssueClaim;
    readonly right: IssueClaim;
  },
): boolean {
  return left.spans
    .some(function anyPairOverlaps(leftSpan,) {
      return right.spans
        .some(function pairOverlaps(rightSpan,) {
          return spansOverlap({
            left: leftSpan,
            right: rightSpan,
          },);
        },);
    },);
}

/**
 Earliest span start across one claim's spans,
 for document-order cluster sorting.

 @param claim - claim whose leading position sorting needs

 @returns Minimum start offset across spans

 @example
 ```ts
 claimPosition({ claim, },);
 ```
 */
function claimPosition(
  { claim, }: { readonly claim: IssueClaim; },
): number {
  return claim
    .spans
    .reduce(
    function toMinStart(
      min,
      span,
    ) {
      return Math.min(
        min,
        span.startOffset,
      );
    },
    Number.POSITIVE_INFINITY,
    );
}

/**
 Partitions validated claims into merge-proposal clusters.
 Exact duplicates collapse first by deterministic identity; then claims
 sharing a defect (see {@link claimsShareDefect}) join one cluster
 transitively, walked iteratively with a work stack.
 Deterministic regardless of input order: members sort by claim id,
 cluster identity hashes the sorted member ids, and clusters sort by
 document position.

 @param claims - claims that already passed `validateIssueClaim`

 @returns Complete partition; multi-member clusters are the merge proposals

 @example
 ```ts
 const { clusters, } = aggregateClaims({ claims: validatedClaims, },);
 ```
 */
export function aggregateClaims(
  { claims, }: { readonly claims: readonly IssueClaim[]; },
): ClaimAggregation {
  /**
   Claims keyed by deterministic identity; first occurrence wins because
   identical ids mean structurally identical claims.
   */
  const byId = new Map<string, IssueClaim>();
  for (const claim of claims) {
    /**
     Stable identity of this claim.
     */
    const claimId = computeIssueClaimId({ claim, },);
    if (!byId.has(claimId,))
      byId.set(
        claimId,
        claim,
      );
  }

  /**
   Deduplicated members in claim-id order, so adjacency and cluster
   identity never depend on input order.
   */
  const members: readonly AggregatedClaim[] = [...byId.entries(),]
    .map(function toMember([claimId, claim,]: ReadonlyEntry<string, IssueClaim>,): AggregatedClaim {
      return {
        claimId,
        claim,
      };
    },)
    .toSorted(function byClaimId(
      left,
      right,
    ) {
      return compareCodePoints({
        left: left.claimId,
        right: right.claimId,
      },);
    },);

  /**
   Component seed per member index; absence means not yet walked.
   */
  const componentOf = new Map<number, number>();

  for (const [seed,] of members.entries()) {
    if (componentOf.has(seed,))
      continue;

    /**
     Work stack of member indices belonging to the component being walked.
     */
    const stack: number[] = [seed,];
    componentOf.set(
      seed,
      seed,
    );
    while (stack.length > 0) {
      /**
       Member index currently expanding, present while the stack is non-empty.
       */
      const current = nonNullishOrThrow(stack.pop(),);

      /**
       Claim currently expanding, present by index construction.
       */
      const currentMember = nonNullishOrThrow(members[current],);
      for (const [candidate, member,] of members.entries()) {
        if (componentOf.has(candidate,))
          continue;
        if (claimsShareDefect({
          left: currentMember.claim,
          right: member.claim,
        },)) {
          componentOf.set(
            candidate,
            seed,
          );
          stack.push(candidate,);
        }
      }
    }
  }

  /**
   Members grouped by component seed, each group in member order and the
   groups in the order their first members come.
   */
  const byComponent = Map.groupBy(
    members,
    function seedOf(
      _member,
      index,
    ): number {
      // Every index has a seed: the walk over every index assigned one.
      return nonNullishOrThrow(componentOf.get(index,),);
    },
  );

  /**
   Clusters with deterministic identity and document position.
   */
  const clusters = [...byComponent.values(),]
    .map(function toCluster(group: readonly AggregatedClaim[],): ClaimCluster {
      /**
       Member claim ids in already-sorted member order.
       */
      const memberIds = group.map(function toId(member,) {
        return member.claimId;
      },);

      return {
        clusterId: `cluster/${hashContent({ content: JSON.stringify(memberIds,), },)}`,
        position: group.reduce(
          function toMinPosition(
            min,
            member,
          ) {
            return Math.min(
              min,
              claimPosition({ claim: member.claim, },),
            );
          },
          Number.POSITIVE_INFINITY,
        ),
        members: group,
      };
    },)
    .toSorted(function byDocumentOrder(
      left,
      right,
    ) {
      if (left.position !== right.position)
        return left.position - right.position;
      return compareCodePoints({
        left: left.clusterId,
        right: right.clusterId,
      },);
    },);

  return { clusters, };
}

//endregion Claim aggregation

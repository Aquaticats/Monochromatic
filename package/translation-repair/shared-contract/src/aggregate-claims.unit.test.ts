/**
 Tests for cross-model claim aggregation into merge-proposal clusters.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  aggregateClaims,
  CLUSTER_ANCHOR_TOLERANCE,
  type IssueCategory,
  type IssueClaim,
  type IssueSeverity,
  type SpanAnchor,
} from '../dist/final/node/index.mjs';

/**
 Invented target-side quoted span at chosen offsets.
 */
function targetSpan(
  {
    startOffset,
    endOffset,
    quotedText,
  }: {
    readonly startOffset: number;
    readonly endOffset: number;
    readonly quotedText: string;
  },
): SpanAnchor {
  return {
    side: 'target',
    nodeId: 'block/1',
    nodeHash: hashContent({ content: 'The cat naps in the sun.', },),
    startOffset,
    endOffset,
    quotedText,
  };
}

/**
 Invented target-side zero-width insertion anchor at one offset.
 */
function insertionAnchor(
  { offset, }: { readonly offset: number; },
): SpanAnchor {
  return {
    side: 'target',
    nodeId: 'block/1',
    nodeHash: hashContent({ content: 'The cat naps in the sun.', },),
    startOffset: offset,
    endOffset: offset,
    quotedText: '',
  };
}

/**
 Invented claim around one span list.
 */
function claimWith(
  {
    category,
    severity,
    summary,
    spans,
  }: {
    readonly category: IssueCategory;
    readonly severity: IssueSeverity;
    readonly summary: string;
    readonly spans: readonly SpanAnchor[];
  },
): IssueClaim {
  return {
    category,
    severity,
    summary,
    spans,
  };
}

await describe({
  name: aggregateClaims.name,
  children: [
    it({
      name: 'collapses exact duplicate claims into one member',
      fn: async () => {
        /** One claim submitted twice, as identical critics would. */
        const duplicated = claimWith({
          category: 'accuracy/mistranslation',
          severity: 'major',
          summary: 'Sunbeam is rendered as moonbeam.',
          spans: [targetSpan({ startOffset: 10, endOffset: 20, quotedText: 'a moonbeam', },),],
        },);
        /** Aggregation over the duplicate pair. */
        const { clusters, } = aggregateClaims({ claims: [duplicated, { ...duplicated, },], },);
        expect(clusters,).toHaveLength(1,);
        expect(clusters[0]?.members,).toHaveLength(1,);
      },
    },),

    it({
      name: 'clusters overlapping same-family claims across differing severities',
      fn: async () => {
        /** Mistranslation claim over one region. */
        const mistranslation = claimWith({
          category: 'accuracy/mistranslation',
          severity: 'major',
          summary: 'Chasing is rendered as ignoring.',
          spans: [targetSpan({ startOffset: 40, endOffset: 70, quotedText: 'ignores the red butterfly now', },),],
        },);
        /** Omission claim intersecting the same region, graded differently. */
        const omission = claimWith({
          category: 'accuracy/omission',
          severity: 'minor',
          summary: 'The butterfly color is dropped.',
          spans: [targetSpan({ startOffset: 60, endOffset: 80, quotedText: 'butterfly now, purring', },),],
        },);
        /** Aggregation over both claims. */
        const { clusters, } = aggregateClaims({ claims: [mistranslation, omission,], },);
        expect(clusters,).toHaveLength(1,);
        expect(clusters[0]?.members,).toHaveLength(2,);
      },
    },),

    it({
      name: 'clusters overlapping cross-family claims for panel disposal',
      fn: async () => {
        /** Accuracy claim over one region. */
        const accuracy = claimWith({
          category: 'accuracy/mistranslation',
          severity: 'major',
          summary: 'Napping is rendered as hunting.',
          spans: [targetSpan({ startOffset: 100, endOffset: 120, quotedText: 'hunts through the', },),],
        },);
        /** Fluency claim over the same region. */
        const fluency = claimWith({
          category: 'fluency/grammar',
          severity: 'minor',
          summary: 'Verb agreement slips in the napping sentence.',
          spans: [targetSpan({ startOffset: 100, endOffset: 120, quotedText: 'hunts through the', },),],
        },);
        /** Aggregation over both claims. */
        const { clusters, } = aggregateClaims({ claims: [accuracy, fluency,], },);
        // One PROPOSED merge; the panel's sameDefect vote disposes it
        // (LLMs are part of the union algorithm, never the family gate).
        expect(clusters,).toHaveLength(1,);
        expect(clusters[0]?.members,).toHaveLength(2,);
      },
    },),

    it({
      name: 'keeps disjoint same-family claims apart and sorts clusters by document position',
      fn: async () => {
        /** Later claim, submitted first to exercise sorting. */
        const later = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: 'The final purr is untranslated.',
          spans: [targetSpan({ startOffset: 500, endOffset: 520, quotedText: 'purrs one last time', },),],
        },);
        /** Earlier claim. */
        const earlier = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: 'The first meow is untranslated.',
          spans: [targetSpan({ startOffset: 5, endOffset: 15, quotedText: 'meows once', },),],
        },);
        /** Aggregation with later claim first. */
        const { clusters, } = aggregateClaims({ claims: [later, earlier,], },);
        expect(clusters,).toHaveLength(2,);
        expect(clusters[0]?.position,).toBe(5,);
        expect(clusters[1]?.position,).toBe(500,);
      },
    },),

    it({
      name: 'clusters insertion anchors within the expanded neighborhood and not beyond',
      fn: async () => {
        /** Reference insertion anchor. */
        const reference = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: 'A sentence about the window perch is missing.',
          spans: [insertionAnchor({ offset: 200, },),],
        },);
        /** Anchor within twice the tolerance, since both anchors expand. */
        const near = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: 'The window perch sentence is dropped.',
          spans: [insertionAnchor({ offset: (200 + (2 * CLUSTER_ANCHOR_TOLERANCE)) - 1, },),],
        },);
        /** Anchor exactly at twice the tolerance, no longer intersecting. */
        const far = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: 'Something about a perch is missing somewhere.',
          spans: [insertionAnchor({ offset: 200 + (2 * CLUSTER_ANCHOR_TOLERANCE), },),],
        },);
        /** Aggregation over near pair. */
        const nearResult = aggregateClaims({ claims: [reference, near,], },);
        expect(nearResult.clusters,).toHaveLength(1,);
        /** Aggregation over far pair. */
        const farResult = aggregateClaims({ claims: [reference, far,], },);
        expect(farResult.clusters,).toHaveLength(2,);
      },
    },),

    it({
      name: 'merges transitively through a bridging claim',
      fn: async () => {
        /** Leftmost claim. */
        const left = claimWith({
          category: 'accuracy/mistranslation',
          severity: 'major',
          summary: 'Whisker count drifts.',
          spans: [targetSpan({ startOffset: 300, endOffset: 320, quotedText: 'twelve whiskers twitch', },),],
        },);
        /** Bridge overlapping both neighbors. */
        const bridge = claimWith({
          category: 'accuracy/mistranslation',
          severity: 'minor',
          summary: 'The whisker sentence drifts from the source.',
          spans: [targetSpan({ startOffset: 315, endOffset: 340, quotedText: 'twitch in the warm light', },),],
        },);
        /** Rightmost claim overlapping only the bridge. */
        const right = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: 'The warm-light clause is dropped.',
          spans: [targetSpan({ startOffset: 335, endOffset: 360, quotedText: 'light near the stove door', },),],
        },);
        /** Aggregation over the chain. */
        const { clusters, } = aggregateClaims({ claims: [right, left, bridge,], },);
        expect(clusters,).toHaveLength(1,);
        expect(clusters[0]?.members,).toHaveLength(3,);
      },
    },),

    it({
      name: 'clusters multi-span claims through any overlapping span pair',
      fn: async () => {
        /** Omission claim quoting source and anchoring the target gap. */
        const sourceAnchored = claimWith({
          category: 'accuracy/omission',
          severity: 'major',
          summary: '猫猫晒太阳的句子没有翻译。',
          spans: [
            {
              side: 'source',
              nodeId: 'block/3',
              nodeHash: hashContent({ content: '猫猫在窗台上晒太阳。', },),
              startOffset: 90,
              endOffset: 100,
              quotedText: '猫猫在窗台上晒太阳。',
            },
            insertionAnchor({ offset: 400, },),
          ],
        },);
        /** Second model's claim meeting only on the target anchor. */
        const targetAnchored = claimWith({
          category: 'accuracy/omission',
          severity: 'minor',
          summary: 'The sunbathing sentence is missing.',
          spans: [insertionAnchor({ offset: 410, },),],
        },);
        /** Aggregation over both claims. */
        const { clusters, } = aggregateClaims({ claims: [sourceAnchored, targetAnchored,], },);
        expect(clusters,).toHaveLength(1,);
        expect(clusters[0]?.members,).toHaveLength(2,);
      },
    },),

    it({
      name: 'keeps same-offset claims on different sides apart',
      fn: async () => {
        /** Source-side claim. */
        const sourceSide = claimWith({
          category: 'extension/suspected-source-error',
          severity: 'neutral',
          summary: '原文里的猫名疑似笔误。',
          spans: [
            {
              side: 'source',
              nodeId: 'block/2',
              nodeHash: hashContent({ content: '小猫咪叫「雪球」。', },),
              startOffset: 50,
              endOffset: 60,
              quotedText: '叫「雪球」。',
            },
          ],
        },);
        /** Target-side claim at identical offsets. */
        const targetSide = claimWith({
          category: 'extension/alignment-error',
          severity: 'neutral',
          summary: 'The naming paragraph pairs with the wrong section.',
          spans: [targetSpan({ startOffset: 50, endOffset: 60, quotedText: 'named Snow', },),],
        },);
        /** Aggregation over both claims. */
        const { clusters, } = aggregateClaims({ claims: [sourceSide, targetSide,], },);
        expect(clusters,).toHaveLength(2,);
      },
    },),

    it({
      name: 'produces identical clusters regardless of input order',
      fn: async () => {
        /** Overlapping pair plus a distant loner. */
        const claims = [
          claimWith({
            category: 'accuracy/mistranslation',
            severity: 'major',
            summary: 'Kibble becomes gravel.',
            spans: [targetSpan({ startOffset: 20, endOffset: 40, quotedText: 'a bowl of fresh gravel', },),],
          },),
          claimWith({
            category: 'accuracy/addition',
            severity: 'minor',
            summary: 'An invented brand name appears.',
            spans: [targetSpan({ startOffset: 30, endOffset: 55, quotedText: 'gravel from Meow Deluxe', },),],
          },),
          claimWith({
            category: 'style/register',
            severity: 'minor',
            summary: 'The closing farewell turns casual.',
            spans: [targetSpan({ startOffset: 900, endOffset: 920, quotedText: 'catch ya later, cat', },),],
          },),
        ];
        /** Aggregation in submission order. */
        const forward = aggregateClaims({ claims, },);
        /** Aggregation in reversed order. */
        const backward = aggregateClaims({ claims: [...claims,].toReversed(), },);
        expect(forward.clusters.map(function toId(cluster,) {
          return cluster.clusterId;
        },),).toEqual(backward.clusters.map(function toId(cluster,) {
          return cluster.clusterId;
        },),);
        expect(forward,).toStrictEqual(backward,);
      },
    },),
  ],
},);

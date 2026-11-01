import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  IssueCategory,
  IssueSeverity,
} from './issue-taxonomy.ts';

//region Issue claim model
// Atomic claims from critics: one claim asserts one defect, anchored to exact
// document evidence. Clustering may propose merges later; only an adjudicator
// disposes. Claims carry no proposer provenance because adjudication is
// provenance-blind; the shell tracks proposer identity outside the claim for
// scorecard calibration.

/**
 Which document of the repair pair a span points into:
 `source` names the original-language document,
 `target` names the translation under repair.

 @example
 ```ts
 const side: DocumentSide = 'target';
 ```
 */
export type DocumentSide = 'source' | 'target';

/**
 One anchored piece of evidence inside one document.
 Offsets are absolute within full document source and must fall inside the named
 node; `quotedText` must equal the offset slice byte-for-byte, so paraphrases and
 hallucinated quotes fail deterministic validation instead of reaching
 adjudication.
 Zero-width spans (`startOffset === endOffset`, empty `quotedText`) are insertion
 anchors: they name where omitted content belongs without quoting anything.

 @example
 ```ts
 const anchor: SpanAnchor = {
   side: 'target',
   nodeId: 'block/1',
   nodeHash: node.contentHash,
   startOffset: 42,
   endOffset: 42,
   quotedText: '',
 };
 ```
 */
export type SpanAnchor = {
  /**
   Document of the pair this span points into.
   */
  readonly side: DocumentSide;

  /**
   Block node claimed to contain this span.
   */
  readonly nodeId: string;

  /**
   Content hash of the claimed node at claim time;
   validation rejects spans whose base drifted.
   */
  readonly nodeHash: string;

  /**
   Absolute start offset within full document source.
   */
  readonly startOffset: number;

  /**
   Absolute end offset (exclusive) within full document source;
   equal to `startOffset` for insertion anchors.
   */
  readonly endOffset: number;

  /**
   Exact text claimed to occupy the span;
   empty for insertion anchors.
   */
  readonly quotedText: string;
};

/**
 One atomic issue claim.
 Multi-span by design: an omission quotes untranslated source and drops a
 zero-width insertion anchor in the target; an alignment error may span several
 blocks per side.

 @example
 ```ts
 const claim: IssueClaim = {
   category: 'accuracy/omission',
   severity: 'major',
   summary: 'Second clause of the sunbathing sentence is untranslated.',
   spans: [sourceSpan, targetInsertionAnchor,],
 };
 ```
 */
export type IssueClaim = {
  /**
   Category slug from the closed taxonomy.
   */
  readonly category: IssueCategory;

  /**
   Claimed severity; adjudication may re-grade it.
   */
  readonly severity: IssueSeverity;

  /**
   One-sentence statement of the single defect claimed.
   */
  readonly summary: string;

  /**
   Anchored evidence in claim-relevant order;
   validation rejects claims without any span.
   */
  readonly spans: readonly SpanAnchor[];
};

/**
 Computes deterministic identity for one claim:
 hash of canonical serialization with explicit field order,
 so identical claims deduplicate across critics and reruns,
 and steering operations (approve, strike) get stable handles.
 Span order participates in identity because span order carries claim-relevant
 pairing.

 @param claim - claim whose stable handle callers need

 @returns `issue/<sha256 hex>` identifier

 @example
 ```ts
 const id = computeIssueClaimId({ claim, },);
 ```
 */
export function computeIssueClaimId(
  { claim, }: { readonly claim: IssueClaim; },
): string {
  /**
   Canonical serialization: nested arrays with explicit field order,
   immune to object-key ordering differences between producers.
   */
  const canonical = JSON.stringify([
    claim.category,
    claim.severity,
    claim.summary,
    claim
      .spans
      .map(function toSpanTuple(span,) {
        return [
          span.side,
          span.nodeId,
          span.nodeHash,
          span.startOffset,
          span.endOffset,
          span.quotedText,
        ];
      },),
  ],);

  return `issue/${hashContent({ content: canonical, },)}`;
}

//endregion Issue claim model

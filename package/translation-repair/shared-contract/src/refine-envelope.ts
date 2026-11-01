import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RepairDocument, } from './parse-document.ts';
import type { EditableEnvelope, } from './patch-model.ts';
import { selectRefinableParagraphs, } from './refine-eligibility.ts';

//region Refinement envelopes
// Turns the eligible paragraphs of a repaired slice into the same
// `EditableEnvelope` shape the editor stage already uses, so refinement gets
// the whole deterministic apply gate without a second implementation of it:
// unknown region, duplicate claim, stale hash, drifted base, and no-op
// replacement are all already decided there and decided identically.
//
// Also collects the slice's link and footnote definitions, because a paragraph
// parsed alone cannot resolve a reference and the structural gate would then
// compare two paragraphs that both look like they reference nothing.

/**
 Eligible paragraphs as envelopes, plus what was skipped and why.

 @example
 ```ts
 const { envelopes, definitions, } = deriveRefinableEnvelopes({ document, },);
 ```
 */
export type RefinableSlice = {
  /**
   Paragraphs the lane may rewrite, in document order.
   */
  readonly envelopes: readonly EditableEnvelope[];

  /**
   Link and footnote definition lines of this slice, so references resolve
   while gating.
   */
  readonly definitions: string;

  /**
   Skip reasons in document order, so lane yield is explainable.
   */
  readonly findings: readonly string[];
};

/**
 mdast kinds whose source IS a definition the rest of the slice references.
 */
const DEFINITION_KINDS: ReadonlySet<string> = new Set([
  'definition',
  'footnoteDefinition',
],);

/**
 Collects the link and footnote definition lines of one document.

 Taken from the whole assembled `T1` rather than from one slice: a paragraph
 may reference a footnote defined in a different slice, and a reference whose
 definition is out of scope does not parse as a reference at all.

 @param document - parsed document to read definitions from

 @returns Definition lines joined as the parser would see them

 @example
 ```ts
 const definitions = collectDefinitions({ document, },);
 ```
 */
export function collectDefinitions(
  {
    document,
  }: {
    readonly document: RepairDocument;
  },
): string {
  return document.nodes
    .filter(function isDefinition(node,) {
      return DEFINITION_KINDS.has(node.kind,);
    },)
    .map(function toText(node,) {
      return node.text;
    },)
    .join('\n',);
}

/**
 Derives the refinable envelopes of one repaired slice.

 @param document - REPAIRED slice, parsed after accuracy edits landed

 @param minimumChars - shortest paragraph eligible in calling refinement role

 @returns Envelopes, definitions, and skip findings

 @example
 ```ts
 const slice = deriveRefinableEnvelopes({ document, },);
 ```
 */
export function deriveRefinableEnvelopes(
  {
    document,
    minimumChars,
  }: {
    readonly document: RepairDocument;
    readonly minimumChars?: number;
  },
): RefinableSlice {
  /**
   Eligibility verdict per block.
   */
  const verdicts = selectRefinableParagraphs({
    document,
    ...((minimumChars === undefined) ? {} : { minimumChars, }),
  },);

  return {
    definitions: collectDefinitions({ document, },),
    envelopes: verdicts
      .flatMap(function toEnvelope(verdict,): readonly EditableEnvelope[] {
        if (!verdict.eligible)
          return [];
        return [{
          envelopeId: `paragraph/${verdict.node
            .id}`,
          startOffset: verdict.node
            .startOffset,
          endOffset: verdict.node
            .endOffset,
          baseText: verdict.node
            .text,
          baseHash: hashContent({ content: verdict.node
            .text, },),
          issueIds: [],
        },];
      },),
    findings: verdicts
      .flatMap(function toFinding(verdict,) {
        if (verdict.eligible)
          return [];
        return [`refine-skip ${verdict.node
          .id} (${verdict.reason})`,];
      },),
  };
}

//endregion Refinement envelopes

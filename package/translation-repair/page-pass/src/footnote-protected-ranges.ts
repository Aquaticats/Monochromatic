import {
  activeFootnoteMarkers,
  isCommentFinding,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { archiveOriginalReadingOf, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  FootnoteRewriteError,
  normalizeFootnoteIdentifier,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { FootnoteLabelRewrite, } from './footnote-label-rewrite.ts';

//region Declared English-original protection
// Both the sealed bytes and their owning declaration must remain outside any moved region.

/**
 Immutable archive interval, including the declaration that establishes its authority.

 @example
 ```ts
 const ranges = footnoteProtectedRanges({ text: archiveText });
 ```
 */
export type FootnoteProtectedRange = {
  /**
   First protected text offset in the current coordinate space.
   */
  readonly startOffset: number;
  /**
   Exclusive protected end.
   */
  readonly endOffset: number;
};

/**
 Anchors protected spans to the exact comment occurrence that declared them.
 Whole-page originals protect the entire input, including apparatus.

 @param text - current archive bytes, reparsed after any allowed external rename

 @returns Protected intervals in the same coordinate space as the input

 @throws FootnoteRewriteError when a declared span cannot be tied to its comment occurrence

 @example
 ```ts
 const protectedRanges = footnoteProtectedRanges({ text });
 ```
 */
export function footnoteProtectedRanges({ text, }: { readonly text: string; },): readonly FootnoteProtectedRange[] {
  /**
   Parser findings retain exact masked-comment positions.
   */
  const document = parseDocument({ text, },);
  /**
   Existing archive-original policy, not a new language classifier.
   */
  const reading = archiveOriginalReadingOf({ document, },);
  if (reading.kind === 'none')
    return [];
  if (reading.kind === 'whole-page')
    return [{
      startOffset: 0,
      endOffset: text.length,
    },];
  return reading.spans
    .map(function anchored(span,): FootnoteProtectedRange {
    /**
     Position, not repeated wording, selects the owning declaration.
     */
      const comment = document.parseFindings
        .find(function owns(finding,): boolean {
          return isCommentFinding({ finding, },)
            && (finding.endOffset === span.startOffset);
        },);
      if (comment === undefined)
        throw new FootnoteRewriteError({ kind: 'position', },);
      return {
        startOffset: comment.startOffset,
        endOffset: span.endOffset,
      };
    },);
}

/**
 Tests whether a candidate edit region intersects protected archive bytes.

 @param startOffset - first potentially changed position

 @param endOffset - exclusive edit-region end

 @param protectedRanges - current-coordinate original-English intervals

 @returns Whether this operation must be withheld

 @example
 ```ts
 const blocked = overlapsFootnoteProtection({ startOffset, endOffset, protectedRanges });
 ```
 */
export function overlapsFootnoteProtection(
  {
    startOffset,
    endOffset,
    protectedRanges,
  }: {
    readonly startOffset: number;
    readonly endOffset: number;
    readonly protectedRanges: readonly FootnoteProtectedRange[];
  },
): boolean {
  return protectedRanges.some(function intersects(range,): boolean {
    return (startOffset < range.endOffset) && (endOffset > range.startOffset);
  },);
}

/**
 Checks actual marker changes, not every supplied correspondence, against original-English authority.

 @param text - archive before any rename

 @param map - closed simultaneous operational map

 @param protectedRanges - original-coordinate protected intervals

 @returns Whether even one rename would change protected bytes

 @example
 ```ts
 const blocked = footnoteRenameTouchesOriginal({ text, map, protectedRanges });
 ```
 */
export function footnoteRenameTouchesOriginal(
  {
    text,
    map,
    protectedRanges,
  }: {
    readonly text: string;
    readonly map: readonly FootnoteLabelRewrite[];
    readonly protectedRanges: readonly FootnoteProtectedRange[];
  },
): boolean {
  if ((protectedRanges.length === 0) || (map.length === 0))
    return false;
  /**
   Logical identities whose raw markers will actually move.
   */
  const changed = new Set(map.filter(function changes(move,): boolean {
    return normalizeFootnoteIdentifier({ identifier: move.from, },) !== normalizeFootnoteIdentifier({ identifier: move.to, },);
  },)
    .map(function key(move,): string { return normalizeFootnoteIdentifier({ identifier: move.from, },); },),);
  return activeFootnoteMarkers({ text, },)
    .some(function touches(marker,): boolean {
      return changed.has(marker.identifier,) && overlapsFootnoteProtection({
        ...marker,
        protectedRanges,
      },);
    },);
}

//endregion Declared English-original protection

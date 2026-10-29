import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ActiveFootnoteMarker, } from './active-footnote-markers.ts';
import type { DocumentChunk, } from './chunk-placement.ts';
import { FootnoteRewriteError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Full-document marker projection
// Prepared fragments may contain container halves. Their evidence comes from positioned whole-document syntax.

/**
 Finds the first marker whose end crosses the requested boundary, or the append position.
 Sorted disjoint marker spans make this a binary search rather than a full scan per slice.

 @param markers - source-ordered active reference markers

 @param offset - slice opening boundary

 @returns First potentially intersecting marker index, including the valid end position

 @example
 ```ts
 const start = markerBoundaryIndex({ markers, offset: chunk.startOffset });
 ```
 */
function markerBoundaryIndex(
  {
    markers,
    offset,
  }: {
    readonly markers: readonly ActiveFootnoteMarker[];
    readonly offset: number
  },
): number {
  /**
   Owned half-open search interval, narrowed together as one algorithmic state.
   */
  const bounds = {
    start: 0,
    end: markers.length,
  };
  while (bounds.start < bounds.end) {
    /**
     Midpoint remains inside the current nonempty search interval.
     */
    const middle = Math.floor((bounds.start + bounds.end) / 2,);
    /**
     Current boundary comparison never needs decoded label lengths.
     */
    const marker = nonNullishOrThrow(markers[middle],);
    if (marker.endOffset <= offset)
      bounds.start = middle + 1;
    else
      bounds.end = middle;
  }
  return bounds.start;
}

/**
 Projects full-document reference evidence into an exact current prepared range.
 A fragment is never reparsed as a complete MDX document or rescued by masking container halves.

 @param chunk - prepared canonical range and text

 @param documentText - complete current source or archive backing the range

 @param markers - active references parsed from that complete document

 @returns Distinct raw reference labels in occurrence order

 @throws FootnoteRewriteError when text, range bounds or marker containment disagree with the document

 @example
 ```ts
 const labels = sliceFootnoteLabels({ chunk: slice.source, documentText: prepared.sourceText, markers });
 ```
 */
export function sliceFootnoteLabels(
  {
    chunk,
    documentText,
    markers,
  }: {
    readonly chunk: Pick<DocumentChunk, 'text' | 'startOffset' | 'endOffset'>;
    readonly documentText: string;
    readonly markers: readonly ActiveFootnoteMarker[];
  },
): readonly string[] {
  /**
   Both range endpoints belong to the supplied current document.
   */
  const {
    startOffset,
    endOffset,
  } = chunk;
  if ((!Number.isSafeInteger(startOffset,)) || (!Number.isSafeInteger(endOffset,))
    || (startOffset < 0)
    || (endOffset < startOffset)
    || (endOffset > documentText.length)
    || (chunk.text !== documentText.slice(
      startOffset,
      endOffset,
    )))
    throw new FootnoteRewriteError({ kind: 'slice-scope', },);
  /**
   First spelling per normalized identity within this slice.
   */
  const labels = new Map<string, string>();
  for (let index = markerBoundaryIndex({
    markers,
    offset: startOffset,
  },); index < markers.length; index += 1) {
    /**
     Only intersecting markers are inspected after the initial binary search.
     */
    const marker = nonNullishOrThrow(markers[index],);
    if (marker.startOffset >= endOffset)
      break;
    if ((marker.startOffset < startOffset) || (marker.endOffset > endOffset))
      throw new FootnoteRewriteError({ kind: 'slice-scope', },);
    if (!labels.has(marker.identifier,))
      labels.set(
        marker.identifier,
        marker.rawLabel,
      );
  }
  return [...labels.values(),];
}

//endregion Full-document marker projection

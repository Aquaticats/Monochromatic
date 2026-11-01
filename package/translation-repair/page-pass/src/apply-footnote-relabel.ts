import { activeFootnoteMarkers, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  FootnoteRewriteError,
  normalizeFootnoteIdentifier,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { FootnoteLabelRewrite, } from './footnote-label-rewrite.ts';
import { footnoteRewriteMap, } from './footnote-rewrite-map.ts';

//region Simultaneous syntax-positioned rewriting
// Bytes outside active marker spans are copied unchanged; no later replacement sees an earlier replacement's output.

/**
 Applies an injective simultaneous rename to active references and definition openers only.
 Code, metadata, attributes, URLs, comments and escaped openings remain byte-identical.
 Reparsed marker roles and identities must equal the intended renamed graph before text is returned.

 @param text - canonical archive or slice bytes

 @param map - operational rewrites whose changing source keys occur in the current document

 @returns Rewritten text with every unrelated byte retained

 @throws FootnoteRewriteError when syntax, map domains, labels, injectivity or the resulting graph cannot be verified

 @example
 ```ts
 const rewritten = applyFootnoteRelabel({ text, map: [{ from: '1', to: '2' }, { from: '2', to: '1' }] });
 ```
 */
export function applyFootnoteRelabel(
  {
    text,
    map,
  }: {
    readonly text: string;
    readonly map: readonly FootnoteLabelRewrite[]
  },
): string {
  if (map.length === 0)
    return text;
  /**
   Exact input spans and logical identities.
   */
  const markers = activeFootnoteMarkers({ text, },);
  /**
   Validated destinations, including checks against unmoved occupants.
   */
  const lookup = footnoteRewriteMap({
    map,
    markers,
  },);
  /**
   Disjoint copied prefixes and marker replacements, joined only once.
   */
  const pieces = markers.map(function piece(
    marker,
    index,
  ): string {
    /**
     End of the preceding copied marker, or start of the document.
     */
    const start = markers[index - 1]
      ?.endOffset
      ?? 0;
    /**
     Raw destination, absent for an untouched identity.
     */
    const destination = lookup.get(marker.identifier,);
    /**
     Normalization-equivalent identities preserve their original spelling.
     */
    const replacement = (destination === undefined) || (normalizeFootnoteIdentifier({ identifier: destination, },) === marker.identifier)
      ? text.slice(
        marker.startOffset,
        marker.endOffset,
      )
      : `[^${destination}]`;
    return `${text.slice(
      start,
      marker.startOffset,
    )}${replacement}`;
  },);
  /**
   Trailing bytes include every definition body and final line ending.
   */
  const suffix = text.slice(markers.at(-1,)
    ?.endOffset
    ?? 0,);
  /**
   Candidate remains local until the resulting syntax graph is verified.
   */
  const rewritten = [
    ...pieces,
    suffix,
  ].join('',);
  if (rewritten === text)
    return text;
  /**
   Actual output marker sequence after parsing, not inferred from the replacement plan.
   */
  const after = activeFootnoteMarkers({ text: rewritten, },);
  if ((after.length !== markers.length) || markers.some(function differs(
    marker,
    index,
  ): boolean {
    /**
     Output role and normalized destination must preserve this original occurrence.
     */
    const actual = after[index];
    return (actual?.kind !== marker.kind) || (actual.identifier !== normalizeFootnoteIdentifier({ identifier: lookup.get(marker.identifier,) ?? marker.identifier, },));
  },))
    throw new FootnoteRewriteError({ kind: 'graph', },);
  return rewritten;
}

//endregion Simultaneous syntax-positioned rewriting

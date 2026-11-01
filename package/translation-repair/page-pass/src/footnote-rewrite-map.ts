import type { ActiveFootnoteMarker, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  FootnoteRewriteError,
  gfmMarkerAt,
  normalizeFootnoteIdentifier,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { FootnoteLabelRewrite, } from './footnote-label-rewrite.ts';

//region Operational rewrite validation
// Original correspondence is established elsewhere; this boundary proves only grammatical injective renaming.

/**
 Builds normalized lookups only for destinations encoding exactly one valid marker label.

 @param map - simultaneous operational rewrites

 @param markers - active input namespace, including unresolved references

 @returns Normalized source keys and raw destination spellings

 @throws FootnoteRewriteError when map domains are stale, destinations cross syntax boundaries or identifiers would merge

 @example
 ```ts
 const lookup = footnoteRewriteMap({ map, markers });
 ```
 */
export function footnoteRewriteMap(
  {
    map,
    markers,
  }: {
    readonly map: readonly FootnoteLabelRewrite[];
    readonly markers: readonly ActiveFootnoteMarker[]
  },
): ReadonlyMap<string, string> {
  /**
   First supplied raw destination per normalized key.
   */
  const lookup = new Map<string, string>();
  for (const move of map) {
    /**
     Destination interpolation must occupy exactly one marker lexeme.
     */
    const syntax = `[^${move.to}]`;
    /**
     Bounded grammar check at the actual output boundary.
     */
    const marker = gfmMarkerAt({
      text: syntax,
      offset: 0,
    },);
    if (((typeof marker) === 'symbol') || (marker.endOffset !== syntax.length)
      || (marker.rawLabel !== move.to))
      throw new FootnoteRewriteError({ kind: 'label', },);
    /**
     From-side matching uses logical identity, never raw case or label length.
     */
    const key = normalizeFootnoteIdentifier({ identifier: move.from, },);
    /**
     Earlier destination, when this logical key was supplied more than once.
     */
    const earlier = lookup.get(key,);
    if ((key === '') || ((earlier !== undefined) && (normalizeFootnoteIdentifier({ identifier: earlier, },)
      !== normalizeFootnoteIdentifier({ identifier: move.to, },))))
      throw new FootnoteRewriteError({ kind: 'mapping', },);
    if (earlier === undefined)
      lookup.set(
        key,
        move.to,
      );
  }
  /**
   Existing identities include unmoved occupants, not just map destinations.
   */
  const before = new Set(markers.map(function identity(marker,): string {
    return marker.identifier;
  },),);
  for (const [identifier, destination,] of lookup) {
    if ((normalizeFootnoteIdentifier({ identifier: destination, },) !== identifier) && (!before.has(identifier,)))
      throw new FootnoteRewriteError({ kind: 'missing-source', },);
  }
  /**
   Distinct output identities must have the same cardinality.
   */
  const after = new Set([...before,].map(function destination(identifier,): string {
    return normalizeFootnoteIdentifier({ identifier: lookup.get(identifier,) ?? identifier, },);
  },),);
  if (before.size !== after.size)
    throw new FootnoteRewriteError({ kind: 'collision', },);
  return lookup;
}

//endregion Operational rewrite validation

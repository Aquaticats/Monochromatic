import {
  type FootnoteRelabelReading,
  type LabelCorrespondence,
  mapLabels,
} from './archive-footnote-relabel.ts';

//region Archive footnote relabel widening
// CLASS NINETY-FIVE (hulicaijia14, 2026-09-23). The roster paired 14 of 15
// blocks and left one original definition unpaired, so the map read off the
// definitions alone landed on an archive label while one original label stood
// unaccounted for; the archive's labels stood, the lanes wrote the original's
// labels into a page keyed by the archive's, the class ninety-two floor
// refused the archive's own paragraph for "dropping" the original's marker
// and the entry stopped. The paired slices carried the missing relation the
// whole time (the paragraph citing the note pairs with its rendering, marker
// beside marker). WHERE THE DEFINITIONS ALONE DO NOT CLOSE, THE SLICES ARE
// READ BESIDE THEM: both readings' relations go through the one consistency
// check, so a slice that contradicts a definition pair still leaves the
// archive standing, and a slice that only confirms or completes one closes
// the map.

/**
 What a reading's relations were read off, for the ambiguity detail, which
 names the earlier claim's place as well as the later one's.
 */
const DEFINITION_WHERE = 'a definition the roster paired';

/**
 What a reading's relations were read off, for the ambiguity detail.
 */
const SLICE_WHERE = 'a paired slice';

/**
 Relations of one reading as claims the consistency check can name.

 @param reading - reading whose relations are claimed; never ambiguous

 @param where - what the relations were read off

 @returns One claim per relation, in the reading's order

 @example
 ```ts
 const claims = claimsOf({ reading, where: 'a paired slice', },);
 ```
 */
function claimsOf(
  {
    reading,
    where,
  }: {
    readonly reading: Exclude<FootnoteRelabelReading, { readonly kind: 'ambiguous'; }>;
    readonly where: string;
  },
): readonly LabelCorrespondence[] {
  return reading.correspondences
    .map(function toClaim(relation,): LabelCorrespondence {
      return {
        from: relation.from,
        to: relation.to,
        where,
      };
    },);
}

/**
 Reads the definitions the roster paired and the paired slices as one body
 of evidence: the definitions' relations first, the slices' after, every one
 through the same consistency check.

 @param definitions - reading off the definitions the roster paired

 @param slices - reading off the paired slices

 @returns The widened reading; an ambiguous input reading as it is

 @example
 ```ts
 const widened = widenFootnoteRelabel({ definitions, slices, },);
 ```
 */
export function widenFootnoteRelabel(
  {
    definitions,
    slices,
  }: {
    readonly definitions: FootnoteRelabelReading;
    readonly slices: FootnoteRelabelReading;
  },
): FootnoteRelabelReading {
  if (definitions.kind === 'ambiguous')
    return definitions;
  if (slices.kind === 'ambiguous')
    return slices;
  return mapLabels({
    correspondences: [
      ...claimsOf({
        reading: definitions,
        where: DEFINITION_WHERE,
      },),
      ...claimsOf({
        reading: slices,
        where: SLICE_WHERE,
      },),
    ],
    skipped: [
      ...definitions.skipped,
      ...slices.skipped,
    ],
  },);
}

//endregion Archive footnote relabel widening
